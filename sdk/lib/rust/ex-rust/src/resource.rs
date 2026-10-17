use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::{KSPIN_LOCK, DbgBreakPoint, KeBugCheckEx};
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _ERESOURCE, PERESOURCE, ERESOURCE_THREAD, _OWNER_ENTRY, POWNER_ENTRY,
    PKEVENT, PKSEMAPHORE, _KEVENT, _KSEMAPHORE,
    _KLOCK_QUEUE_HANDLE, PKLOCK_QUEUE_HANDLE,
    KeAcquireInStackQueuedSpinLock, KeReleaseInStackQueuedSpinLock,
    KeInitializeEvent, KeInitializeSemaphore, KeInitializeSpinLock,
    KeEnterCriticalRegion, KeLeaveCriticalRegion,
    KeDelayExecutionThread, KeWaitForSingleObject, KeReleaseSemaphore,
    KeSetEventBoostPriority, KiAcquireDispatcherLock, KiReleaseDispatcherLock,
    KiAcquireThreadLock, KiReleaseThreadLock, KiSetPriorityThread,
    LARGE_INTEGER, LIST_ENTRY, PKTHREAD, PETHREAD,
    PsGetCurrentThread, _KWAIT_REASON_WrResource,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};


const TAG_RESOURCE_EVENT: u32 = u32::from_be_bytes(*b"EvtR");
const TAG_RESOURCE_SEMAPHORE: u32 = u32::from_be_bytes(*b"SemR");
const TAG_RESOURCE_TABLE: u32 = u32::from_be_bytes(*b"TblR");

const ResourceNeverExclusive: u16 = 0x10;
const ResourceOwnedExclusive: u16 = 0x80;
const ResourceHasDisabledPriorityBoost: u16 = 0x08;

type KLOCK_QUEUE_HANDLE = _KLOCK_QUEUE_HANDLE;

#[inline]
unsafe fn ExGetCurrentResourceThread() -> ERESOURCE_THREAD {
    PsGetCurrentThread() as ERESOURCE_THREAD
}

/// Helper to get current thread as PKTHREAD for field access
#[inline]
unsafe fn ExpGetCurrentKThread() -> PKTHREAD {
    PsGetCurrentThread() as PKTHREAD
}

/// Helper: read OwnerCount bitfield from _OWNER_ENTRY
#[inline]
unsafe fn owner_get_count(entry: *const _OWNER_ENTRY) -> u32 {
    (*entry).__bindgen_anon_1.__bindgen_anon_1.OwnerCount()
}

/// Helper: set OwnerCount bitfield on _OWNER_ENTRY
#[inline]
unsafe fn owner_set_count(entry: *mut _OWNER_ENTRY, val: u32) {
    (*entry).__bindgen_anon_1.__bindgen_anon_1.set_OwnerCount(val);
}

/// Helper: read TableSize from _OWNER_ENTRY (union field)
#[inline]
unsafe fn owner_get_table_size(entry: *const _OWNER_ENTRY) -> u32 {
    (*entry).__bindgen_anon_1.TableSize
}

/// Helper: set TableSize on _OWNER_ENTRY (union field)
#[inline]
unsafe fn owner_set_table_size(entry: *mut _OWNER_ENTRY, val: u32) {
    (*entry).__bindgen_anon_1.TableSize = val;
}

fn IsExclusiveWaiting(r: PERESOURCE) -> bool {
    unsafe { (*r).NumberOfExclusiveWaiters > 0 }
}

fn IsSharedWaiting(r: PERESOURCE) -> bool {
    unsafe { (*r).NumberOfSharedWaiters > 0 }
}

fn IsOwnedExclusive(r: PERESOURCE) -> bool {
    unsafe { ((*r).Flag & ResourceOwnedExclusive) != 0 }
}

fn IsBoostAllowed(r: PERESOURCE) -> bool {
    unsafe { ((*r).Flag & ResourceHasDisabledPriorityBoost) == 0 }
}

#[cfg(all(not(feature = "smp"), not(debug_assertions)))]
unsafe fn ExAcquireResourceLock(_Resource: PERESOURCE, _LockHandle: PKLOCK_QUEUE_HANDLE) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    core::arch::asm!("cli");
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("msr daifset, #2");
}

#[cfg(all(not(feature = "smp"), not(debug_assertions)))]
unsafe fn ExReleaseResourceLock(_Resource: PERESOURCE, _LockHandle: PKLOCK_QUEUE_HANDLE) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    core::arch::asm!("sti");
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("msr daifclr, #2");
}

#[cfg(any(feature = "smp", debug_assertions))]
unsafe fn ExAcquireResourceLock(Resource: PERESOURCE, LockHandle: PKLOCK_QUEUE_HANDLE) {
    KeAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*Resource).SpinLock), LockHandle);
}

#[cfg(any(feature = "smp", debug_assertions))]
unsafe fn ExReleaseResourceLock(_Resource: PERESOURCE, LockHandle: PKLOCK_QUEUE_HANDLE) {
    KeReleaseInStackQueuedSpinLock(LockHandle);
}

static mut ExShortTime: LARGE_INTEGER = unsafe { core::mem::transmute(-100000i64) };
static mut ExpTimeout: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut ExpResourceTimeoutCount: ULONG = 90 * 3600 / 2;

// Registry-controlled values owned by ex/resource. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static RESOURCE_CONTROL_VECTOR: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector({ [
        ntkrnl_sys::cv!("Session Manager", "ResourceTimeoutCount",
            core::ptr::addr_of_mut!(ExpResourceTimeoutCount)),
    ] });

// KSPIN_LOCK — must match ddk ULONG_PTR: u64 on amd64/arm64, u32 on i386
#[cfg(target_pointer_width = "64")]
static mut ExpResourceSpinLock: u64 = 0;
#[cfg(target_pointer_width = "32")]
static mut ExpResourceSpinLock: u32 = 0;
#[no_mangle]
static mut ExpSystemResourcesList: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut ExResourceStrict: BOOLEAN = 1u8;

// ===========================================================================
// KTHREAD field accessor helpers — bindgen anonymous union numbering differs
// between i386 and amd64/arm64.
// ===========================================================================

/// CombinedApcDisable: amd64 __bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1, i386 __bindgen_anon_4
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn kthread_combined_apc_disable(t: PKTHREAD) -> ULONG {
    (*t).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kthread_combined_apc_disable(t: PKTHREAD) -> ULONG {
    (*t).__bindgen_anon_4.CombinedApcDisable
}

/// ResourceIndex: amd64 __bindgen_anon_5.__bindgen_anon_3, i386 __bindgen_anon_7.__bindgen_anon_3
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn kthread_resource_index(t: PKTHREAD) -> UCHAR {
    (*t).__bindgen_anon_5.__bindgen_anon_3.ResourceIndex
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kthread_resource_index(t: PKTHREAD) -> UCHAR {
    (*t).__bindgen_anon_7.__bindgen_anon_3.ResourceIndex
}

#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn kthread_set_resource_index(t: PKTHREAD, val: UCHAR) {
    (*t).__bindgen_anon_5.__bindgen_anon_3.ResourceIndex = val;
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kthread_set_resource_index(t: PKTHREAD, val: UCHAR) {
    (*t).__bindgen_anon_7.__bindgen_anon_3.ResourceIndex = val;
}

/// Quantum: amd64 __bindgen_anon_7.__bindgen_anon_1, i386 __bindgen_anon_9.__bindgen_anon_1
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn kthread_quantum(t: PKTHREAD) -> i8 {
    (*t).__bindgen_anon_7.__bindgen_anon_1.Quantum
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kthread_quantum(t: PKTHREAD) -> i8 {
    (*t).__bindgen_anon_9.__bindgen_anon_1.Quantum
}

/// QuantumReset: amd64 __bindgen_anon_7.__bindgen_anon_2, i386 __bindgen_anon_9.__bindgen_anon_2
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn kthread_quantum_reset(t: PKTHREAD) -> UCHAR {
    (*t).__bindgen_anon_7.__bindgen_anon_2.QuantumReset
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kthread_quantum_reset(t: PKTHREAD) -> UCHAR {
    (*t).__bindgen_anon_9.__bindgen_anon_2.QuantumReset
}

/// WaitIrql: amd64 __bindgen_anon_5.__bindgen_anon_6, i386 direct field
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn kthread_wait_irql(t: PKTHREAD) -> UCHAR {
    (*t).__bindgen_anon_5.__bindgen_anon_6.WaitIrql
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kthread_wait_irql(t: PKTHREAD) -> UCHAR {
    (*t).WaitIrql as UCHAR
}

#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn kthread_set_wait_irql(t: PKTHREAD, val: UCHAR) {
    (*t).__bindgen_anon_5.__bindgen_anon_6.WaitIrql = val;
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn kthread_set_wait_irql(t: PKTHREAD, val: UCHAR) {
    (*t).WaitIrql = val as _;
}

#[cfg(debug_assertions)]
unsafe fn ExpVerifyResource(Resource: PERESOURCE) {
}

#[cfg(debug_assertions)]
unsafe fn ExpCheckForApcsDisabled(Irql: KIRQL, Resource: PERESOURCE, Thread: PKTHREAD) {
    if ExResourceStrict != 0
        && Irql < APC_LEVEL
        && (*(Thread as PETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() == 0
        && kthread_combined_apc_disable(Thread) == 0
    {
        DbgBreakPoint();
    }
}

#[cfg(not(debug_assertions))]
unsafe fn ExpVerifyResource(_Resource: PERESOURCE) {}

#[cfg(not(debug_assertions))]
unsafe fn ExpCheckForApcsDisabled(_Irql: KIRQL, _Resource: PERESOURCE, _Thread: PKTHREAD) {}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpResourceInitialization() {
    ExpTimeout = core::mem::transmute((4i32 as i64) * (-10000000i64));
    InitializeListHead(core::ptr::addr_of_mut!(ExpSystemResourcesList) as PLIST_ENTRY);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(ExpResourceSpinLock));
}

unsafe fn ExpAllocateExclusiveWaiterEvent(Resource: PERESOURCE, LockHandle: PKLOCK_QUEUE_HANDLE) {
    ExReleaseResourceLock(Resource, LockHandle);

    loop {
        let Event = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_KEVENT>(),
            TAG_RESOURCE_EVENT,
        ) as PKEVENT;

        if !Event.is_null() {
            KeInitializeEvent(Event, 1u32, 0u8);

            if !InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*Resource).ExclusiveWaiters) as *mut PVOID,
                Event as PVOID,
                core::ptr::null_mut(),
            )
            .is_null()
            {
                ExFreePoolWithTag(Event as PVOID, TAG_RESOURCE_EVENT);
            }

            break;
        }

        KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of_mut!(ExShortTime));
    }

    ExAcquireResourceLock(Resource, LockHandle);
}

unsafe fn ExpAllocateSharedWaiterSemaphore(
    Resource: PERESOURCE,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    ExReleaseResourceLock(Resource, LockHandle);

    loop {
        let Semaphore = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_KSEMAPHORE>(),
            TAG_RESOURCE_SEMAPHORE,
        ) as PKSEMAPHORE;

        if !Semaphore.is_null() {
            KeInitializeSemaphore(Semaphore, 0, 0x7fffffff);

            if !InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*Resource).SharedWaiters) as *mut PVOID,
                Semaphore as PVOID,
                core::ptr::null_mut(),
            )
            .is_null()
            {
                ExFreePoolWithTag(Semaphore as PVOID, TAG_RESOURCE_SEMAPHORE);
            }

            break;
        }

        KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of_mut!(ExShortTime));
    }

    ExAcquireResourceLock(Resource, LockHandle);
}

unsafe fn ExpExpandResourceOwnerTable(Resource: PERESOURCE, LockHandle: PKLOCK_QUEUE_HANDLE) {
    let Owner = (*Resource).OwnerTable;
    let (OldSize, NewSize) = if Owner.is_null() {
        (0u32, 3u32)
    } else {
        let old = owner_get_table_size(Owner);
        (old, old + 4)
    };

    ExReleaseResourceLock(Resource, LockHandle);

    let Table = ExAllocatePoolWithTag(
        NonPagedPool,
        (NewSize as usize) * core::mem::size_of::<_OWNER_ENTRY>(),
        TAG_RESOURCE_TABLE,
    ) as POWNER_ENTRY;

    core::ptr::write_bytes(
        (Table as *mut u8).add((OldSize as usize) * core::mem::size_of::<_OWNER_ENTRY>()),
        0,
        ((NewSize - OldSize) as usize) * core::mem::size_of::<_OWNER_ENTRY>(),
    );

    ExAcquireResourceLock(Resource, LockHandle);

    if Owner != (*Resource).OwnerTable
        || (!Owner.is_null() && OldSize != owner_get_table_size(Owner))
    {
        ExReleaseResourceLock(Resource, LockHandle);
        ExFreePoolWithTag(Table as PVOID, TAG_RESOURCE_TABLE);
    } else {
        if !Owner.is_null() {
            core::ptr::copy_nonoverlapping(
                Owner as *const u8,
                Table as *mut u8,
                (OldSize as usize) * core::mem::size_of::<_OWNER_ENTRY>(),
            );
        }

        let OldIrql = KiAcquireDispatcherLock();

        owner_set_table_size(Table, NewSize);
        (*Resource).OwnerTable = Table;

        KiReleaseDispatcherLock(OldIrql);

        ExpVerifyResource(Resource);

        ExReleaseResourceLock(Resource, LockHandle);

        if !Owner.is_null() {
            ExFreePoolWithTag(Owner as PVOID, TAG_RESOURCE_TABLE);
        }

    }

    let idx = if OldSize == 0 { 1u32 } else { OldSize };
    kthread_set_resource_index(ExpGetCurrentKThread(), idx as u8);

    ExAcquireResourceLock(Resource, LockHandle);
}

unsafe fn ExpFindFreeEntry(
    Resource: PERESOURCE,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) -> POWNER_ENTRY {
    let mut Owner = (*Resource).OwnerTable;

    if !Owner.is_null() {
        let Limit = Owner.add(owner_get_table_size(Owner) as usize);
        Owner = Owner.add(1);

        while (*Owner).OwnerThread != 0 {
            Owner = Owner.add(1);
            if Owner == Limit {
                ExpExpandResourceOwnerTable(Resource, LockHandle);
                return core::ptr::null_mut();
            }
        }

        kthread_set_resource_index(ExpGetCurrentKThread(),
            ((Owner as usize - (*Resource).OwnerTable as usize)
                / core::mem::size_of::<_OWNER_ENTRY>()) as u8);
        Owner
    } else {
        ExpExpandResourceOwnerTable(Resource, LockHandle);
        core::ptr::null_mut()
    }
}

unsafe fn ExpFindEntryForThread(
    Resource: PERESOURCE,
    Thread: ERESOURCE_THREAD,
    LockHandle: PKLOCK_QUEUE_HANDLE,
    FirstEntryInelligible: BOOLEAN,
) -> POWNER_ENTRY {
    let mut Owner = core::ptr::addr_of_mut!((*Resource).OwnerEntry);

    if (*Owner).OwnerThread == Thread {
        return Owner;
    }

    let mut FreeEntry = if FirstEntryInelligible != 0 || (*Owner).OwnerThread != 0 {
        core::ptr::null_mut()
    } else {
        Owner
    };

    Owner = (*Resource).OwnerTable;
    if !Owner.is_null() {
        let Limit = Owner.add(owner_get_table_size(Owner) as usize);
        Owner = Owner.add(1);

        while (*Owner).OwnerThread != Thread {
            if FreeEntry.is_null() && (*Owner).OwnerThread == 0 {
                FreeEntry = Owner;
            }

            Owner = Owner.add(1);
            if Owner == Limit {
                if LockHandle.is_null() {
                    return core::ptr::null_mut();
                }

                if !FreeEntry.is_null() {
                    kthread_set_resource_index(ExpGetCurrentKThread(),
                        ((FreeEntry as usize - (*Resource).OwnerTable as usize)
                            / core::mem::size_of::<_OWNER_ENTRY>())
                            as u8);
                    return FreeEntry;
                }

                ExpExpandResourceOwnerTable(Resource, LockHandle);
                return core::ptr::null_mut();
            }
        }

        kthread_set_resource_index(ExpGetCurrentKThread(),
            ((Owner as usize - (*Resource).OwnerTable as usize)
                / core::mem::size_of::<_OWNER_ENTRY>()) as u8);
        return Owner;
    } else {
        if LockHandle.is_null() {
            return core::ptr::null_mut();
        }

        if !FreeEntry.is_null() {
            kthread_set_resource_index(ExpGetCurrentKThread(),
                ((FreeEntry as usize - (*Resource).OwnerTable as usize)
                    / core::mem::size_of::<_OWNER_ENTRY>()) as u8);
            return FreeEntry;
        }

        ExpExpandResourceOwnerTable(Resource, LockHandle);
        core::ptr::null_mut()
    }
}

unsafe fn ExpBoostOwnerThread(Thread: PKTHREAD, OwnerThread: PKTHREAD) {
    if (OwnerThread as ULONG_PTR) & 0x3 == 0 {
        if ((*OwnerThread).Priority as i32) < ((*Thread).Priority as i32) && ((*OwnerThread).Priority as i32) < 14 {
            // The WAITER's lock does not protect the OWNER's dispatcher state, and
            // everything below mutates OwnerThread -- PriorityDecrement, Quantum,
            // KiSetPriorityThread. Live on every contended ERESOURCE acquisition
            // where the owner runs at lower priority than a waiter.
            // Upstream-Commit: e6d2efe90708
            KiAcquireThreadLock(OwnerThread);

            (*OwnerThread).PriorityDecrement += (14 - (*OwnerThread).Priority as i32) as i8;
            // Quantum and QuantumReset are at different __bindgen_anon_N per arch
            let qr = kthread_quantum_reset(OwnerThread);
            // Write quantum directly via arch-specific path
            #[cfg(not(target_arch = "x86"))]
            { (*OwnerThread).__bindgen_anon_7.__bindgen_anon_1.Quantum = qr as i8; }
            #[cfg(target_arch = "x86")]
            { (*OwnerThread).__bindgen_anon_9.__bindgen_anon_1.Quantum = qr as i8; }
            KiSetPriorityThread(OwnerThread, 14);

            KiReleaseThreadLock(OwnerThread);
        }
    }
}

unsafe fn ExpWaitForResource(Resource: PERESOURCE, Object: PVOID) {
    (*Resource).ContentionCount += 1;
    let mut Timeout = core::mem::transmute(500i64 * -10000i64);
    let mut WaitCount: ULONG = 0;

    loop {
        let Status = KeWaitForSingleObject(Object, _KWAIT_REASON_WrResource, 0i8, 0u8, &mut Timeout);
        if Status != STATUS_TIMEOUT {
            break;
        }

        WaitCount += 1;
        Timeout = ExpTimeout;

        if WaitCount > ExpResourceTimeoutCount {
            WaitCount = 0;
            #[cfg(debug_assertions)]
            {
                let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
                ExAcquireResourceLock(Resource, &mut LockHandle);
                DbgBreakPoint();
                ExReleaseResourceLock(Resource, &mut LockHandle);
            }
        }

        if IsBoostAllowed(Resource) {
            let Thread = ExpGetCurrentKThread();
            kthread_set_wait_irql(Thread, KiAcquireDispatcherLock());
            (*Thread).WaitNext = 1u8;

            let OwnerThread = (*Resource).OwnerEntry.OwnerThread as PKTHREAD;
            if !OwnerThread.is_null() {
                ExpBoostOwnerThread(Thread, OwnerThread);
            }

            if !IsOwnedExclusive(Resource) {
                let Owner = (*Resource).OwnerTable;
                if !Owner.is_null() {
                    let Size = owner_get_table_size(Owner);
                    for i in 1..Size {
                        let entry = Owner.add(i as usize);
                        let OwnerThread = (*entry).OwnerThread as PKTHREAD;
                        if !OwnerThread.is_null() {
                            ExpBoostOwnerThread(Thread, OwnerThread);
                        }
                    }
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExAcquireResourceExclusiveLite(
    Resource: PERESOURCE,
    Wait: BOOLEAN,
) -> BOOLEAN {
    let Thread = ExGetCurrentResourceThread();

    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);
    ExpCheckForApcsDisabled(LockHandle.OldIrql, Resource, Thread as PKTHREAD);

    let mut Success: BOOLEAN;

    loop {
        if (*Resource).ActiveEntries != 0 {
            if IsOwnedExclusive(Resource) && (*Resource).OwnerEntry.OwnerThread == Thread {
                owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)) + 1);
                Success = 1u8;
                break;
            } else {
                if Wait == 0 {
                    Success = 0u8;
                    break;
                } else {
                    if (*Resource).ExclusiveWaiters.is_null() {
                        ExpAllocateExclusiveWaiterEvent(Resource, &mut LockHandle);
                        continue;
                    }

                    (*Resource).NumberOfExclusiveWaiters += 1;
                    ExReleaseResourceLock(Resource, &mut LockHandle);
                    ExpWaitForResource(Resource, (*Resource).ExclusiveWaiters as PVOID);

                    (*Resource).OwnerEntry.OwnerThread = ExGetCurrentResourceThread();
                    return 1u8;
                }
            }
        } else {
            (*Resource).Flag |= ResourceOwnedExclusive;
            (*Resource).ActiveEntries = 1;
            (*Resource).ActiveCount = 1;
            (*Resource).OwnerEntry.OwnerThread = Thread;
            owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 1);
            Success = 1u8;
            break;
        }
    }

    ExReleaseResourceLock(Resource, &mut LockHandle);
    Success
}

#[no_mangle]
pub unsafe extern "system" fn ExAcquireResourceSharedLite(
    Resource: PERESOURCE,
    Wait: BOOLEAN,
) -> BOOLEAN {
    let Thread = ExGetCurrentResourceThread();

    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);
    ExpCheckForApcsDisabled(LockHandle.OldIrql, Resource, Thread as PKTHREAD);

    let mut Owner: POWNER_ENTRY = core::ptr::null_mut();

    while (*Resource).ActiveEntries != 0 {
        if IsOwnedExclusive(Resource) {
            if (*Resource).OwnerEntry.OwnerThread == Thread {
                owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)) + 1);
                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            }

            Owner = ExpFindFreeEntry(Resource, &mut LockHandle);
            if Owner.is_null() {
                continue;
            }
        } else {
            let FirstEntryBusy = if IsExclusiveWaiting(Resource) { 1u8 } else { 0u8 };
            Owner = ExpFindEntryForThread(Resource, Thread, &mut LockHandle, FirstEntryBusy);
            if Owner.is_null() {
                continue;
            }

            if (*Owner).OwnerThread == Thread {
                owner_set_count(Owner, owner_get_count(Owner) + 1);
                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            }

            if FirstEntryBusy == 0 {
                (*Owner).OwnerThread = Thread;
                owner_set_count(Owner, 1);

                if (*Resource).ActiveEntries == 0 {
                    (*Resource).ActiveEntries = 1;
                    (*Resource).ActiveCount = 1;
                } else {
                    (*Resource).ActiveEntries += 1;
                }

                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            }
        }

        if Wait == 0 {
            ExReleaseResourceLock(Resource, &mut LockHandle);
            return 0u8;
        }

        if (*Resource).SharedWaiters.is_null() {
            ExpAllocateSharedWaiterSemaphore(Resource, &mut LockHandle);
        } else {
            break;
        }
    }

    if (*Resource).ActiveEntries == 0 {
        (*Resource).ActiveEntries = 1;
        (*Resource).ActiveCount = 1;
        (*Resource).OwnerEntry.OwnerThread = Thread;
        owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 1);

        ExReleaseResourceLock(Resource, &mut LockHandle);
        return 1u8;
    }

    (*Owner).OwnerThread = Thread;
    owner_set_count(Owner, 1);
    (*Resource).NumberOfSharedWaiters += 1;

    ExReleaseResourceLock(Resource, &mut LockHandle);
    ExpWaitForResource(Resource, (*Resource).SharedWaiters as PVOID);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ExAcquireSharedStarveExclusive(
    Resource: PERESOURCE,
    Wait: BOOLEAN,
) -> BOOLEAN {
    let Thread = ExGetCurrentResourceThread();

    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);

    let mut Owner: POWNER_ENTRY;

    loop {
        if (*Resource).ActiveEntries == 0 {
            (*Resource).ActiveCount = 1;
            (*Resource).ActiveEntries = 1;
            (*Resource).OwnerEntry.OwnerThread = Thread;
            owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 1);

            ExReleaseResourceLock(Resource, &mut LockHandle);
            return 1u8;
        }

        if IsOwnedExclusive(Resource) {
            if (*Resource).OwnerEntry.OwnerThread == Thread {
                owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)) + 1);
                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            }

            Owner = ExpFindFreeEntry(Resource, &mut LockHandle);
            if Owner.is_null() {
                continue;
            }
        } else {
            Owner = ExpFindEntryForThread(Resource, Thread, &mut LockHandle, 0u8);
            if Owner.is_null() {
                continue;
            }

            if (*Owner).OwnerThread == Thread {
                owner_set_count(Owner, owner_get_count(Owner) + 1);
                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            }

            (*Owner).OwnerThread = Thread;
            owner_set_count(Owner, 1);

            if (*Resource).ActiveEntries == 0 {
                (*Resource).ActiveEntries = 1;
                (*Resource).ActiveCount = 1;
            } else {
                (*Resource).ActiveEntries += 1;
            }

            ExReleaseResourceLock(Resource, &mut LockHandle);
            return 1u8;
        }

        if Wait == 0 {
            ExReleaseResourceLock(Resource, &mut LockHandle);
            return 0u8;
        }

        if (*Resource).SharedWaiters.is_null() {
            ExpAllocateSharedWaiterSemaphore(Resource, &mut LockHandle);
            continue;
        }

        break;
    }

    (*Owner).OwnerThread = Thread;
    owner_set_count(Owner, 1);
    (*Resource).NumberOfSharedWaiters += 1;

    ExReleaseResourceLock(Resource, &mut LockHandle);
    ExpWaitForResource(Resource, (*Resource).SharedWaiters as PVOID);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ExAcquireSharedWaitForExclusive(
    Resource: PERESOURCE,
    Wait: BOOLEAN,
) -> BOOLEAN {
    let Thread = ExGetCurrentResourceThread();

    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);

    let mut Owner: POWNER_ENTRY;

    loop {
        if (*Resource).ActiveEntries == 0 {
            (*Resource).ActiveCount = 1;
            (*Resource).ActiveEntries = 1;
            (*Resource).OwnerEntry.OwnerThread = Thread;
            owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 1);

            ExReleaseResourceLock(Resource, &mut LockHandle);
            return 1u8;
        }

        if IsOwnedExclusive(Resource) {
            if (*Resource).OwnerEntry.OwnerThread == Thread {
                owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)) + 1);
                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            }

            Owner = ExpFindFreeEntry(Resource, &mut LockHandle);
            if Owner.is_null() {
                continue;
            }
        } else {
            if IsExclusiveWaiting(Resource) {
                if Wait == 0 {
                    ExReleaseResourceLock(Resource, &mut LockHandle);
                    return 0u8;
                }

                if (*Resource).SharedWaiters.is_null() {
                    ExpAllocateSharedWaiterSemaphore(Resource, &mut LockHandle);
                    continue;
                }

                (*Resource).NumberOfSharedWaiters += 1;
                ExReleaseResourceLock(Resource, &mut LockHandle);
                ExpWaitForResource(Resource, (*Resource).SharedWaiters as PVOID);

                ExAcquireResourceLock(Resource, &mut LockHandle);

                loop {
                    Owner = ExpFindEntryForThread(Resource, Thread, &mut LockHandle, 1u8);
                    if !Owner.is_null() {
                        break;
                    }
                }

                (*Owner).OwnerThread = Thread;
                owner_set_count(Owner, 1);

                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            } else {
                Owner = ExpFindEntryForThread(Resource, Thread, &mut LockHandle, 0u8);
                if Owner.is_null() {
                    continue;
                }

                if (*Owner).OwnerThread == Thread {
                    owner_set_count(Owner, owner_get_count(Owner) + 1);
                    ExReleaseResourceLock(Resource, &mut LockHandle);
                    return 1u8;
                }

                (*Owner).OwnerThread = Thread;
                owner_set_count(Owner, 1);

                if (*Resource).ActiveEntries == 0 {
                    (*Resource).ActiveEntries = 1;
                    (*Resource).ActiveCount = 1;
                } else {
                    (*Resource).ActiveEntries += 1;
                }

                ExReleaseResourceLock(Resource, &mut LockHandle);
                return 1u8;
            }
        }

        if Wait == 0 {
            ExReleaseResourceLock(Resource, &mut LockHandle);
            return 0u8;
        }

        if (*Resource).SharedWaiters.is_null() {
            ExpAllocateSharedWaiterSemaphore(Resource, &mut LockHandle);
            continue;
        }

        break;
    }

    (*Owner).OwnerThread = Thread;
    owner_set_count(Owner, 1);
    (*Resource).NumberOfSharedWaiters += 1;

    ExReleaseResourceLock(Resource, &mut LockHandle);
    ExpWaitForResource(Resource, (*Resource).SharedWaiters as PVOID);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ExConvertExclusiveToSharedLite(Resource: PERESOURCE) {
    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);

    (*Resource).Flag &= !ResourceOwnedExclusive;

    if IsSharedWaiting(Resource) {
        let OldWaiters = (*Resource).NumberOfSharedWaiters;
        (*Resource).ActiveEntries += OldWaiters;
        (*Resource).NumberOfSharedWaiters = 0;

        ExReleaseResourceLock(Resource, &mut LockHandle);
        KeReleaseSemaphore((*Resource).SharedWaiters, 0, OldWaiters as LONG, 0u8);
    } else {
        ExReleaseResourceLock(Resource, &mut LockHandle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExDeleteResourceLite(Resource: PERESOURCE) -> NTSTATUS {
    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!(ExpResourceSpinLock),
        &mut LockHandle,
    );

    RemoveEntryList(core::ptr::addr_of_mut!((*Resource).SystemResourcesList) as PLIST_ENTRY);

    KeReleaseInStackQueuedSpinLock(&mut LockHandle);

    if !(*Resource).OwnerTable.is_null() {
        ExFreePoolWithTag((*Resource).OwnerTable as PVOID, TAG_RESOURCE_TABLE);
    }
    if !(*Resource).SharedWaiters.is_null() {
        ExFreePoolWithTag((*Resource).SharedWaiters as PVOID, TAG_RESOURCE_SEMAPHORE);
    }
    if !(*Resource).ExclusiveWaiters.is_null() {
        ExFreePoolWithTag((*Resource).ExclusiveWaiters as PVOID, TAG_RESOURCE_EVENT);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ExDisableResourceBoostLite(Resource: PERESOURCE) {
    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);

    (*Resource).Flag |= ResourceHasDisabledPriorityBoost;

    ExReleaseResourceLock(Resource, &mut LockHandle);
}

#[no_mangle]
pub unsafe extern "system" fn ExGetExclusiveWaiterCount(Resource: PERESOURCE) -> ULONG {
    (*Resource).NumberOfExclusiveWaiters
}

#[no_mangle]
pub unsafe extern "system" fn ExGetSharedWaiterCount(Resource: PERESOURCE) -> ULONG {
    (*Resource).NumberOfSharedWaiters
}

#[no_mangle]
pub unsafe extern "system" fn ExInitializeResourceLite(Resource: PERESOURCE) -> NTSTATUS {
    // The C's RtlZeroMemory, i.e. memset, on purpose: ExInitializeResourceLite(NULL)
    // must FAULT and reach the caller's handler as STATUS_ACCESS_VIOLATION (kmtest
    // ExResource checks it). write_bytes' UB check would panic the kernel instead.
    extern "C" {
        fn memset(dst: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
    }
    memset(Resource as *mut core::ffi::c_void, 0, core::mem::size_of::<_ERESOURCE>());

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Resource).SpinLock));

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!(ExpResourceSpinLock),
        &mut LockHandle,
    );
    InsertTailList(
        core::ptr::addr_of_mut!(ExpSystemResourcesList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Resource).SystemResourcesList) as PLIST_ENTRY,
    );
    KeReleaseInStackQueuedSpinLock(&mut LockHandle);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ExIsResourceAcquiredExclusiveLite(Resource: PERESOURCE) -> BOOLEAN {
    ExpVerifyResource(Resource);

    let IsAcquired = if IsOwnedExclusive(Resource)
        && (*Resource).OwnerEntry.OwnerThread == ExGetCurrentResourceThread()
    {
        1u8
    } else {
        0u8
    };

    IsAcquired
}

#[no_mangle]
pub unsafe extern "system" fn ExIsResourceAcquiredSharedLite(Resource: PERESOURCE) -> ULONG {
    ExpVerifyResource(Resource);

    if (*Resource).ActiveEntries == 0 {
        return 0;
    }

    let Thread = ExGetCurrentResourceThread();

    if (*Resource).OwnerEntry.OwnerThread == Thread {
        return owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry));
    } else {
        if IsOwnedExclusive(Resource) {
            return 0;
        }

        let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
        ExAcquireResourceLock(Resource, &mut LockHandle);

        let Owner = (*Resource).OwnerTable;
        let mut Count: ULONG = 0;

        if !Owner.is_null() {
            let mut i = kthread_resource_index(Thread as PKTHREAD) as ULONG;
            let Size = owner_get_table_size(Owner);

            if i >= Size || (*Owner.add(i as usize)).OwnerThread != Thread {
                for idx in 1..Size {
                    let entry = Owner.add(idx as usize);
                    if (*entry).OwnerThread == Thread {
                        Count = owner_get_count(entry);
                        break;
                    }
                }
            } else {
                Count = owner_get_count(Owner.add(i as usize));
            }
        }

        ExReleaseResourceLock(Resource, &mut LockHandle);
        Count
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExReinitializeResourceLite(Resource: PERESOURCE) -> NTSTATUS {
    let Owner = (*Resource).OwnerTable;
    if !Owner.is_null() {
        let Size = owner_get_table_size(Owner);
        for i in 0..Size {
            let entry = Owner.add(i as usize);
            (*entry).OwnerThread = 0;
            owner_set_count(entry, 0);
        }
    }

    (*Resource).Flag = 0;
    (*Resource).ActiveCount = 0;
    (*Resource).ActiveEntries = 0;

    let Semaphore = (*Resource).SharedWaiters;
    if !Semaphore.is_null() {
        KeInitializeSemaphore(Semaphore, 0, 0x7fffffff);
    }

    let Event = (*Resource).ExclusiveWaiters;
    if !Event.is_null() {
        KeInitializeEvent(Event, 1u32, 0u8);
    }

    (*Resource).OwnerEntry.OwnerThread = 0;
    owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 0);
    (*Resource).ContentionCount = 0;
    (*Resource).NumberOfSharedWaiters = 0;
    (*Resource).NumberOfExclusiveWaiters = 0;
    // The arm64 last-releaser slots (ExReleaseResourceForThreadLite).
    #[cfg(target_arch = "aarch64")]
    {
        (*Resource).Reserved2 = core::ptr::null_mut();
        (*Resource).__bindgen_anon_1.Address = core::ptr::null_mut();
    }

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExReleaseResourceLite(Resource: PERESOURCE) {
    ExReleaseResourceForThreadLite(Resource, ExGetCurrentResourceThread());
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExReleaseResourceLite(Resource: PERESOURCE) {
    ExReleaseResourceForThreadLite(Resource, ExGetCurrentResourceThread());
}

#[no_mangle]
pub unsafe extern "system" fn ExReleaseResourceForThreadLite(
    Resource: PERESOURCE,
    Thread: ERESOURCE_THREAD,
) {
    // Captured FIRST, like ExSetResourceOwnerPointer's: the 0xE3 below names
    // the resource and thread but never the releaser, and #43's captures all
    // come from here. LR is the direct caller (often ExReleaseResourceLite),
    // and the frame record chain gives the ones above it.
    #[cfg(target_arch = "aarch64")]
    let (CallerPc, CallerFp): (usize, usize) = {
        let lr: usize;
        let fp: usize;
        core::arch::asm!("mov {}, x30", "mov {}, x29", out(reg) lr, out(reg) fp,
                         options(nomem, nostack, preserves_flags));
        (lr, fp)
    };
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);

    ExpVerifyResource(Resource);
    ExpCheckForApcsDisabled(LockHandle.OldIrql, Resource, ExpGetCurrentKThread());

    // Who released it LAST (#43 hunt): an 0xE3 finds the resource already idle,
    // so the evidence is the release BEFORE it. Reserved2 and Address are debug
    // slots nothing else here uses. Read before being overwritten below.
    #[cfg(target_arch = "aarch64")]
    let (PrevPc, PrevThr) = ((*Resource).Reserved2 as usize, (*Resource).__bindgen_anon_1.Address as usize);
    #[cfg(target_arch = "aarch64")]
    {
        (*Resource).Reserved2 = CallerPc as PVOID;
        (*Resource).__bindgen_anon_1.Address = Thread as PVOID;
    }

    if IsOwnedExclusive(Resource) {
        owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)) - 1);
        if owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)) != 0 {
            ExReleaseResourceLock(Resource, &mut LockHandle);
            return;
        }

        (*Resource).OwnerEntry.OwnerThread = 0;
        (*Resource).ActiveEntries -= 1;

        if IsSharedWaiting(Resource) {
            (*Resource).Flag &= !ResourceOwnedExclusive;

            let Count = (*Resource).NumberOfSharedWaiters;
            (*Resource).ActiveEntries = Count;
            (*Resource).NumberOfSharedWaiters = 0;

            ExReleaseResourceLock(Resource, &mut LockHandle);
            KeReleaseSemaphore((*Resource).SharedWaiters, 0, Count as LONG, 0u8);
            return;
        } else if IsExclusiveWaiting(Resource) {
            (*Resource).OwnerEntry.OwnerThread = 1;
            owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 1);
            (*Resource).ActiveEntries = 1;
            (*Resource).NumberOfExclusiveWaiters -= 1;

            ExReleaseResourceLock(Resource, &mut LockHandle);
            KeSetEventBoostPriority(
                (*Resource).ExclusiveWaiters,
                core::ptr::addr_of_mut!((*Resource).OwnerEntry.OwnerThread) as *mut PKTHREAD,
            );
            return;
        }

        (*Resource).Flag &= !ResourceOwnedExclusive;
        (*Resource).ActiveCount = 0;
    } else {
        let Owner: POWNER_ENTRY;

        if (*Resource).OwnerEntry.OwnerThread == Thread {
            Owner = core::ptr::addr_of_mut!((*Resource).OwnerEntry);
        } else {
            let mut i: ULONG = 1;
            if (Thread as ULONG_PTR) & 3 == 0 {
                i = kthread_resource_index(Thread as PKTHREAD) as ULONG;
            }

            let OwnerTable = (*Resource).OwnerTable;

            if i >= owner_get_table_size(OwnerTable) || (*OwnerTable.add(i as usize)).OwnerThread != Thread
            {
                let Limit = OwnerTable.add(owner_get_table_size(OwnerTable) as usize);
                let mut entry = OwnerTable.add(1);

                loop {
                    if entry >= Limit {
                        #[cfg(target_arch = "aarch64")]
                        {
                            let mut up = [0usize; 3];
                            let mut fp = CallerFp;
                            for slot in up.iter_mut() {
                                if fp < 0xFFFF_0000_0000_0000 || fp & 7 != 0 {
                                    break;
                                }
                                *slot = *((fp + 8) as *const usize);
                                fp = *(fp as *const usize);
                            }
                            {
                                extern "system" {
                                    fn KiPcToFileHeader(
                                        Pc: PVOID,
                                        LdrEntry: *mut ntkrnl_sys::ddk::PLDR_DATA_TABLE_ENTRY,
                                        DriversOnly: BOOLEAN,
                                        InKernel: *mut BOOLEAN,
                                    ) -> PVOID;
                                }
                                let mut le: ntkrnl_sys::ddk::PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                                let mut ink: BOOLEAN = 0;
                                let base = KiPcToFileHeader(PrevPc as PVOID, &mut le, 0, &mut ink);
                                if !le.is_null() {
                                    ntkrnl_sys::driver_ddk::DbgPrint(
                                        c"[ERESOURCE] prev.caller = %wZ+0x%p\n".as_ptr(),
                                        core::ptr::addr_of!((*le).BaseDllName),
                                        (PrevPc - base as usize) as *const core::ffi::c_void,
                                    );
                                }
                            }
                            ntkrnl_sys::driver_ddk::DbgPrint(
                                c"[ERESOURCE] 0xE3 release caller=%p up=%p %p %p prev.caller=%p prev.thr=%p res=%p thr=%p cur=%p inline.thr=%p inline.cnt=%lu active=%d entries=%lu flag=%04lx shw=%lu exw=%lu\n".as_ptr(),
                                CallerPc as *const core::ffi::c_void,
                                up[0] as *const core::ffi::c_void,
                                up[1] as *const core::ffi::c_void,
                                up[2] as *const core::ffi::c_void,
                                PrevPc as *const core::ffi::c_void,
                                PrevThr as *const core::ffi::c_void,
                                Resource as *const core::ffi::c_void,
                                Thread as *const core::ffi::c_void,
                                ExGetCurrentResourceThread() as *const core::ffi::c_void,
                                (*Resource).OwnerEntry.OwnerThread as *const core::ffi::c_void,
                                owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)),
                                (*Resource).ActiveCount as i32,
                                (*Resource).ActiveEntries as u32,
                                (*Resource).Flag as u32,
                                (*Resource).NumberOfSharedWaiters as u32,
                                (*Resource).NumberOfExclusiveWaiters as u32,
                            );
                        }
                        KeBugCheckEx(
                            ntkrnl_sys::bugcodes::RESOURCE_NOT_OWNED,
                            Resource as ULONG_PTR,
                            Thread as ULONG_PTR,
                            (*Resource).OwnerTable as ULONG_PTR,
                            3,
                        );
                    }

                    if (*entry).OwnerThread == Thread {
                        Owner = entry;
                        break;
                    }

                    entry = entry.add(1);
                }
            } else {
                Owner = OwnerTable.add(i as usize);
            }
        }

        owner_set_count(Owner, owner_get_count(Owner) - 1);
        if owner_get_count(Owner) != 0 {
            ExReleaseResourceLock(Resource, &mut LockHandle);
            return;
        }

        (*Owner).OwnerThread = 0;

        (*Resource).ActiveEntries -= 1;
        if (*Resource).ActiveEntries == 0 {
            if IsExclusiveWaiting(Resource) {
                (*Resource).Flag |= ResourceOwnedExclusive;
                (*Resource).OwnerEntry.OwnerThread = 1;
                owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 1);
                (*Resource).ActiveEntries = 1;
                (*Resource).NumberOfExclusiveWaiters -= 1;

                ExReleaseResourceLock(Resource, &mut LockHandle);
                KeSetEventBoostPriority(
                    (*Resource).ExclusiveWaiters,
                    core::ptr::addr_of_mut!((*Resource).OwnerEntry.OwnerThread) as *mut PKTHREAD,
                );
                return;
            }

            (*Resource).ActiveCount = 0;
        }
    }

    ExReleaseResourceLock(Resource, &mut LockHandle);
}

#[no_mangle]
pub unsafe extern "system" fn ExSetResourceOwnerPointer(
    Resource: PERESOURCE,
    OwnerPointer: PVOID,
) {
    // Captured FIRST, before any call clobbers the link register: the 0xE3 below
    // reports which resource and thread, but never who asked for the transfer,
    // and the two captures we have came from entirely different call contexts.
    // arm64 only — that is the only arch the stop has ever been seen on, and the
    // amd64 stack-scan heuristic (rtl-diag's RtlRaiseStatus) is too noisy to earn
    // its keep here.
    #[cfg(target_arch = "aarch64")]
    let CallerPc: usize = {
        let lr: usize;
        core::arch::asm!("mov {}, x30", out(reg) lr, options(nomem, nostack, preserves_flags));
        lr
    };
    #[cfg(not(target_arch = "aarch64"))]
    let CallerPc: usize = 0;

    let OwnerPointer = opaque_ptr(OwnerPointer);
    let Thread = ExGetCurrentResourceThread();

    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);

    if IsOwnedExclusive(Resource) {
        (*Resource).OwnerEntry.OwnerThread = OwnerPointer as ERESOURCE_THREAD;
    } else {
        let ThisOwner = ExpFindEntryForThread(
            Resource,
            OwnerPointer as ERESOURCE_THREAD,
            core::ptr::null_mut(),
            0u8,
        );
        let Owner = ExpFindEntryForThread(Resource, Thread, core::ptr::null_mut(), 0u8);

        if Owner.is_null() {
            // Name the culprit before the stop. The discriminator is inline.thr:
            // if it already equals ownerptr, ownership was transferred to this
            // same owner once before and we are re-running a transfer for a
            // reused context — the current thread legitimately owns nothing.
            // Any other value means a genuinely foreign caller, which is a
            // different bug. Everything else here is the resource's shape, which
            // the four bugcheck parameters cannot carry.
            ntkrnl_sys::driver_ddk::DbgPrint(
                c"[ERESOURCE] 0xE3 caller=%p res=%p thr=%p ownerptr=%p inline.thr=%p inline.cnt=%lu thisowner=%p active=%d entries=%lu flag=%04lx\n".as_ptr(),
                CallerPc as *const core::ffi::c_void,
                Resource as *const core::ffi::c_void,
                Thread as *const core::ffi::c_void,
                OwnerPointer as *const core::ffi::c_void,
                (*Resource).OwnerEntry.OwnerThread as *const core::ffi::c_void,
                owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)),
                ThisOwner as *const core::ffi::c_void,
                (*Resource).ActiveCount as i32,
                (*Resource).ActiveEntries as u32,
                (*Resource).Flag as u32,
            );
            KeBugCheckEx(
                ntkrnl_sys::bugcodes::RESOURCE_NOT_OWNED,
                Resource as ULONG_PTR,
                Thread as ULONG_PTR,
                (*Resource).OwnerTable as ULONG_PTR,
                3,
            );
        }

        if !ThisOwner.is_null() {
            owner_set_count(ThisOwner, owner_get_count(ThisOwner) + owner_get_count(Owner));
            owner_set_count(Owner, 0);
            (*Owner).OwnerThread = 0;
            (*Resource).ActiveEntries -= 1;
        } else {
            (*Owner).OwnerThread = OwnerPointer as ERESOURCE_THREAD;
        }
    }

    ExReleaseResourceLock(Resource, &mut LockHandle);
}

#[no_mangle]
pub unsafe extern "system" fn ExTryToAcquireResourceExclusiveLite(
    Resource: PERESOURCE,
) -> BOOLEAN {
    let Thread = ExGetCurrentResourceThread();

    ExpVerifyResource(Resource);

    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ExAcquireResourceLock(Resource, &mut LockHandle);

    let mut Acquired: BOOLEAN = 0u8;

    if (*Resource).ActiveCount == 0 {
        (*Resource).Flag |= ResourceOwnedExclusive;
        (*Resource).OwnerEntry.OwnerThread = Thread;
        owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), 1);
        (*Resource).ActiveCount = 1;
        (*Resource).ActiveEntries = 1;
        Acquired = 1u8;
    } else if IsOwnedExclusive(Resource) && (*Resource).OwnerEntry.OwnerThread == Thread {
        owner_set_count(core::ptr::addr_of_mut!((*Resource).OwnerEntry), owner_get_count(core::ptr::addr_of!((*Resource).OwnerEntry)) + 1);
        Acquired = 1u8;
    }

    ExReleaseResourceLock(Resource, &mut LockHandle);
    Acquired
}

#[no_mangle]
pub unsafe extern "system" fn ExEnterCriticalRegionAndAcquireResourceExclusive(
    Resource: PERESOURCE,
) -> PVOID {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(Resource, 1u8);
    (*ExpGetCurrentKThread()).Win32Thread
}

#[no_mangle]
pub unsafe extern "system" fn ExEnterCriticalRegionAndAcquireResourceShared(
    Resource: PERESOURCE,
) -> PVOID {
    KeEnterCriticalRegion();
    ExAcquireResourceSharedLite(Resource, 1u8);
    (*ExpGetCurrentKThread()).Win32Thread
}

#[no_mangle]
pub unsafe extern "system" fn ExEnterCriticalRegionAndAcquireSharedWaitForExclusive(
    Resource: PERESOURCE,
) -> PVOID {
    KeEnterCriticalRegion();
    ExAcquireSharedWaitForExclusive(Resource, 1u8);
    (*ExpGetCurrentKThread()).Win32Thread
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExReleaseResourceAndLeaveCriticalRegion(Resource: PERESOURCE) {
    ExReleaseResourceLite(Resource);
    KeLeaveCriticalRegion();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExReleaseResourceAndLeaveCriticalRegion(Resource: PERESOURCE) {
    ExReleaseResourceLite(Resource);
    KeLeaveCriticalRegion();
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/resource.c
//  * PURPOSE:         Executive Resource Implementation
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Macros for reading resource flags */
// #define IsExclusiveWaiting(r)   (r->NumberOfExclusiveWaiters > 0)
// #define IsSharedWaiting(r)      (r->NumberOfSharedWaiters > 0)
// #define IsOwnedExclusive(r)     (r->Flag & ResourceOwnedExclusive)
// #define IsBoostAllowed(r)       (!(r->Flag & ResourceHasDisabledPriorityBoost))
//
// #if !defined(CONFIG_SMP) && !DBG
//
// FORCEINLINE
// VOID
// ExAcquireResourceLock(IN PERESOURCE Resource,
//                       IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     UNREFERENCED_PARAMETER(Resource);
//     UNREFERENCED_PARAMETER(LockHandle);
//
//     /* Simply disable interrupts */
//     _disable();
// }
//
// FORCEINLINE
// VOID
// ExReleaseResourceLock(IN PERESOURCE Resource,
//                       IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     UNREFERENCED_PARAMETER(Resource);
//     UNREFERENCED_PARAMETER(LockHandle);
//
//     /* Simply enable interrupts */
//     _enable();
// }
//
// #else
//
// FORCEINLINE
// VOID
// ExAcquireResourceLock(IN PERESOURCE Resource,
//                       IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     /* Acquire the lock */
//     KeAcquireInStackQueuedSpinLock(&Resource->SpinLock, LockHandle);
// }
//
// FORCEINLINE
// VOID
// ExReleaseResourceLock(IN PERESOURCE Resource,
//                       IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     UNREFERENCED_PARAMETER(Resource);
//
//     /* Release the lock */
//     KeReleaseInStackQueuedSpinLock(LockHandle);
// }
//
// #endif // !defined(CONFIG_SMP) && !DBG
//
// /* DATA***********************************************************************/
//
// LARGE_INTEGER ExShortTime = {{-100000, -1}};
// LARGE_INTEGER ExpTimeout;
//
// /* Timeout value for resources in 4-second units (7 days) */
// ULONG ExpResourceTimeoutCount = 90 * 3600 / 2; // NT value: 648000 (30 days)
//
// KSPIN_LOCK ExpResourceSpinLock;
// LIST_ENTRY ExpSystemResourcesList;
// BOOLEAN ExResourceStrict = TRUE;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// #if DBG
// /*++
//  * @name ExpVerifyResource
//  *
//  *     The ExpVerifyResource routine verifies the correctness of an ERESOURCE
//  *
//  * @param Resource
//  *        Pointer to the resource being verified.
//  *
//  * @return None.
//  *
//  * @remarks Only present on DBG builds.
//  *
//  *--*/
// VOID
// NTAPI
// ExpVerifyResource(IN PERESOURCE Resource)
// {
//     /* Verify the resource data */
//     ASSERT((((ULONG_PTR)Resource) & (sizeof(ULONG_PTR) - 1)) == 0);
//     ASSERT(!Resource->SharedWaiters ||
//             Resource->SharedWaiters->Header.Type == SemaphoreObject);
//     ASSERT(!Resource->SharedWaiters ||
//             Resource->SharedWaiters->Header.Size == (sizeof(KSEMAPHORE) / sizeof(ULONG)));
//     ASSERT(!Resource->ExclusiveWaiters ||
//             Resource->ExclusiveWaiters->Header.Type == SynchronizationEvent);
//     ASSERT(!Resource->ExclusiveWaiters ||
//             Resource->ExclusiveWaiters->Header.Size == (sizeof(KEVENT) / sizeof(ULONG)));
// }
//
// /*++
//  * @name ExpCheckForApcsDisabled
//  *
//  *     The ExpCheckForApcsDisabled routine checks if Kernel APCs are still
//  *     enabled when they should be disabled, and optionally breakpoints.
//  *
//  * @param Irql
//  *        Specifies the IRQL during the acquire attempt.
//  *
//  * @param Resource
//  *        Pointer to the resource being checked.
//  *
//  * @param Thread
//  *        Pointer to the thread being checked.
//  *
//  * @return None.
//  *
//  * @remarks Only present on DBG builds. Depends on ExResourceStrict value.
//  *
//  *--*/
// VOID
// NTAPI
// ExpCheckForApcsDisabled(IN KIRQL Irql,
//                         IN PERESOURCE Resource,
//                         IN PKTHREAD Thread)
// {
//     /* Check if we should care and check if we should break */
//     if ((ExResourceStrict) &&
//         (Irql < APC_LEVEL) &&
//         !(((PETHREAD)Thread)->SystemThread) &&
//         !(Thread->CombinedApcDisable))
//     {
//         /* Bad! */
//         DPRINT1("EX: resource: APCs still enabled before resource %p acquire/release "
//                 "!!!\n", Resource);
//         DbgBreakPoint();
//     }
// }
// #else
// #define ExpVerifyResource(r)
// #define ExpCheckForApcsDisabled(b,r,t)
// #endif
//
// /*++
//  * @name ExpResourceInitialization
//  *
//  *     The ExpResourceInitialization routine initializes resources for use.
//  *
//  * @param None.
//  *
//  * @return None.
//  *
//  * @remarks This routine should only be called once, during system startup.
//  *
//  *--*/
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpResourceInitialization(VOID)
// {
//     /* Setup the timeout */
//     ExpTimeout.QuadPart = Int32x32To64(4, -10000000);
//     InitializeListHead(&ExpSystemResourcesList);
//     KeInitializeSpinLock(&ExpResourceSpinLock);
// }
//
// /*++
//  * @name ExpAllocateExclusiveWaiterEvent
//  *
//  *     The ExpAllocateExclusiveWaiterEvent routine creates the event that will
//  *     be used by exclusive waiters on the resource.
//  *
//  * @param Resource
//  *        Pointer to the resource.
//  *
//  * @param LockHandle
//  *        Pointer to in-stack queued spinlock.
//  *
//  * @return None.
//  *
//  * @remarks The pointer to the event must be atomically set.
//  *
//  *--*/
// VOID
// NTAPI
// ExpAllocateExclusiveWaiterEvent(IN PERESOURCE Resource,
//                                 IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     PKEVENT Event;
//
//     /* Release the lock */
//     ExReleaseResourceLock(Resource, LockHandle);
//
//     /* Loop as long as we keep running out of memory */
//     do
//     {
//         /* Allocate the event */
//         Event = ExAllocatePoolWithTag(NonPagedPool,
//                                       sizeof(KEVENT),
//                                       TAG_RESOURCE_EVENT);
//         if (Event)
//         {
//             /* Initialize it */
//             KeInitializeEvent(Event, SynchronizationEvent, FALSE);
//
//             /* Set it */
//             if (InterlockedCompareExchangePointer((PVOID*)&Resource->ExclusiveWaiters,
//                                                   Event,
//                                                   NULL))
//             {
//                 /* Someone already set it, free our event */
//                 DPRINT1("WARNING: Handling race condition\n");
//                 ExFreePoolWithTag(Event, TAG_RESOURCE_EVENT);
//             }
//
//             break;
//         }
//
//         /* Wait a bit before trying again */
//         KeDelayExecutionThread(KernelMode, FALSE, &ExShortTime);
//     } while (TRUE);
//
//     /* Re-acquire the lock */
//     ExAcquireResourceLock(Resource, LockHandle);
// }
//
// /*++
//  * @name ExpAllocateSharedWaiterSemaphore
//  *
//  *     The ExpAllocateSharedWaiterSemaphore routine creates the semaphore that
//  *     will be used by shared waiters on the resource.
//  *
//  * @param Resource
//  *        Pointer to the resource.
//  *
//  * @param LockHandle
//  *        Pointer to in-stack queued spinlock.
//  *
//  * @return None.
//  *
//  * @remarks The pointer to the semaphore must be atomically set.
//  *
//  *--*/
// VOID
// NTAPI
// ExpAllocateSharedWaiterSemaphore(IN PERESOURCE Resource,
//                                  IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     PKSEMAPHORE Semaphore;
//
//     /* Release the lock */
//     ExReleaseResourceLock(Resource, LockHandle);
//
//     /* Loop as long as we keep running out of memory */
//     do
//     {
//         /* Allocate the semaphore */
//         Semaphore = ExAllocatePoolWithTag(NonPagedPool,
//                                           sizeof(KSEMAPHORE),
//                                           TAG_RESOURCE_SEMAPHORE);
//         if (Semaphore)
//         {
//             /* Initialize it */
//             KeInitializeSemaphore(Semaphore, 0, MAXLONG);
//
//             /* Set it */
//             if (InterlockedCompareExchangePointer((PVOID*)&Resource->SharedWaiters,
//                                                   Semaphore,
//                                                   NULL))
//             {
//                 /* Someone already set it, free our semaphore */
//                 DPRINT1("WARNING: Handling race condition\n");
//                 ExFreePoolWithTag(Semaphore, TAG_RESOURCE_SEMAPHORE);
//             }
//
//             break;
//         }
//
//         /* Wait a bit before trying again */
//         KeDelayExecutionThread(KernelMode, FALSE, &ExShortTime);
//     } while (TRUE);
//
//     /* Re-acquire the lock */
//     ExAcquireResourceLock(Resource, LockHandle);
// }
//
// /*++
//  * @name ExpExpandResourceOwnerTable
//  *
//  *     The ExpExpandResourceOwnerTable routine expands the owner table of the
//  *     specified resource.
//  *
//  * @param Resource
//  *        Pointer to the resource.
//  *
//  * @param LockHandle
//  *        Pointer to in-stack queued spinlock.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// ExpExpandResourceOwnerTable(IN PERESOURCE Resource,
//                             IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     POWNER_ENTRY Owner, Table;
//     KIRQL OldIrql;
//     ULONG NewSize, OldSize;
//
//     /* Get the owner table */
//     Owner = Resource->OwnerTable;
//     if (!Owner)
//     {
//         /* Start with the default size of 3 */
//         OldSize = 0;
//         NewSize = 3;
//     }
//     else
//     {
//         /* Add 4 more entries */
//         OldSize = Owner->TableSize;
//         NewSize = OldSize + 4;
//     }
//
//     /* Release the lock */
//     ExReleaseResourceLock(Resource, LockHandle);
//
//     /* Allocate memory for the table */
//     Table = ExAllocatePoolWithTag(NonPagedPool,
//                                   NewSize * sizeof(OWNER_ENTRY),
//                                   TAG_RESOURCE_TABLE);
//
//     /* Zero the table */
//     RtlZeroMemory(Table + OldSize,
//                   (NewSize - OldSize) * sizeof(OWNER_ENTRY));
//
//     /* Lock the resource */
//     ExAcquireResourceLock(Resource, LockHandle);
//
//     /* Make sure nothing has changed */
//     if ((Owner != Resource->OwnerTable) ||
//         ((Owner) && (OldSize != Owner->TableSize)))
//     {
//         /* Resource changed while we weren't holding the lock; bail out */
//         ExReleaseResourceLock(Resource, LockHandle);
//         ExFreePoolWithTag(Table, TAG_RESOURCE_TABLE);
//     }
//     else
//     {
//         /* Copy the table */
//         if (Owner) RtlCopyMemory(Table, Owner, OldSize * sizeof(OWNER_ENTRY));
//
//         /* Acquire dispatcher lock to prevent thread boosting */
//         OldIrql = KiAcquireDispatcherLock();
//
//         /* Set the new table data */
//         Table->TableSize = NewSize;
//         Resource->OwnerTable = Table;
//
//         /* Release dispatcher lock */
//         KiReleaseDispatcherLock(OldIrql);
//
//         /* Sanity check */
//         ExpVerifyResource(Resource);
//
//         /* Release lock */
//         ExReleaseResourceLock(Resource, LockHandle);
//
//         /* Free the old table */
//         if (Owner) ExFreePoolWithTag(Owner, TAG_RESOURCE_TABLE);
//
//         /* Set the resource index */
//         if (!OldSize) OldSize = 1;
//     }
//
//     /* Set the resource index */
//     KeGetCurrentThread()->ResourceIndex = (UCHAR)OldSize;
//
//     /* Lock the resource again */
//     ExAcquireResourceLock(Resource, LockHandle);
// }
//
// /*++
//  * @name ExpFindFreeEntry
//  *
//  *     The ExpFindFreeEntry routine locates an empty owner entry in the
//  *     specified resource. If none was found, then the owner table is
//  *     expanded.
//  *
//  * @param Resource
//  *        Pointer to the resource.
//  *
//  * @param LockHandle
//  *        Pointer to in-stack queued spinlock.
//  *
//  * @return Pointer to an empty OWNER_ENTRY structure.
//  *
//  * @remarks None.
//  *
//  *--*/
// POWNER_ENTRY
// FASTCALL
// ExpFindFreeEntry(IN PERESOURCE Resource,
//                  IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     POWNER_ENTRY Owner, Limit;
//
//     /* Sanity check */
//     ASSERT(LockHandle != 0);
//     ASSERT(Resource->OwnerEntry.OwnerThread != 0);
//
//     /* Get the current table pointer */
//     Owner = Resource->OwnerTable;
//     if (Owner)
//     {
//         /* Set the limit, move to the next owner and loop owner entries */
//         Limit = &Owner[Owner->TableSize];
//         Owner++;
//         while (Owner->OwnerThread)
//         {
//             /* Move to the next one */
//             Owner++;
//
//             /* Check if the entry is free */
//             if (Owner == Limit) goto Expand;
//         }
//
//         /* Update the resource entry */
//         KeGetCurrentThread()->ResourceIndex = (UCHAR)(Owner - Resource->OwnerTable);
//     }
//     else
//     {
// Expand:
//         /* No free entry, expand the table */
//         ExpExpandResourceOwnerTable(Resource, LockHandle);
//         Owner = NULL;
//     }
//
//     /* Return the entry found */
//     return Owner;
// }
//
// /*++
//  * @name ExpFindEntryForThread
//  *
//  *     The ExpFindEntryForThread routine locates the owner entry associated with
//  *     the specified thread in the given resource. If none was found, then the
//  *     owner table is expanded.
//  *
//  * @param Resource
//  *        Pointer to the resource.
//  *
//  * @param Thread
//  *        Pointer to the thread to find.
//  *
//  * @param LockHandle
//  *        Pointer to in-stack queued spinlock.
//  *
//  * @return Pointer to an empty OWNER_ENTRY structure.
//  *
//  * @remarks None.
//  *
//  *--*/
// POWNER_ENTRY
// FASTCALL
// ExpFindEntryForThread(IN PERESOURCE Resource,
//                       IN ERESOURCE_THREAD Thread,
//                       IN PKLOCK_QUEUE_HANDLE LockHandle,
//                       IN BOOLEAN FirstEntryInelligible)
// {
//     POWNER_ENTRY FreeEntry, Owner, Limit;
//
//     /* Start by looking in the static array */
//     Owner = &Resource->OwnerEntry;
//     if (Owner->OwnerThread == Thread) return Owner;
//
//     /* Check if this is a free entry */
//     if ((FirstEntryInelligible) || (Owner->OwnerThread))
//     {
//         /* No free entry */
//         FreeEntry = NULL;
//     }
//     else
//     {
//         /* Use the first entry as our free entry */
//         FreeEntry = Owner;
//     }
//
//     /* Get the current table pointer */
//     Owner = Resource->OwnerTable;
//     if (Owner)
//     {
//         /* Set the limit, move to the next owner and loop owner entries */
//         Limit = &Owner[Owner->TableSize];
//         Owner++;
//         while (Owner->OwnerThread != Thread)
//         {
//             /* Check if we don't have a free entry */
//             if (!FreeEntry)
//             {
//                 /* Check if this entry is free */
//                 if (!Owner->OwnerThread)
//                 {
//                     /* Save it as our free entry */
//                     FreeEntry = Owner;
//                 }
//             }
//
//             /* Move to the next one */
//             Owner++;
//
//             /* Check if the entry is free */
//             if (Owner == Limit) goto Expand;
//         }
//
//         /* Update the resource entry */
//         KeGetCurrentThread()->ResourceIndex = (UCHAR)(Owner - Resource->OwnerTable);
//         return Owner;
//     }
//     else
//     {
// Expand:
//         /* Check if it's OK to do an expansion */
//         if (!LockHandle) return NULL;
//
//         /* If we found a free entry by now, return it */
//         if (FreeEntry)
//         {
//             /* Set the resource index */
//             KeGetCurrentThread()->ResourceIndex = (UCHAR)(FreeEntry - Resource->OwnerTable);
//             return FreeEntry;
//         }
//
//         /* No free entry, expand the table */
//         ExpExpandResourceOwnerTable(Resource, LockHandle);
//         return NULL;
//     }
// }
//
// /*++
//  * @name ExpBoostOwnerThread
//  *
//  *     The ExpBoostOwnerThread routine increases the priority of a waiting
//  *     thread in an attempt to fight a possible deadlock.
//  *
//  * @param Thread
//  *        Pointer to the current thread.
//  *
//  * @param OwnerThread
//  *        Pointer to thread that owns the resource.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// FASTCALL
// ExpBoostOwnerThread(IN PKTHREAD Thread,
//                     IN PKTHREAD OwnerThread)
// {
//     /* Make sure the owner thread is a pointer, not an ID */
//     if (!((ULONG_PTR)OwnerThread & 0x3))
//     {
//         /* Check if we can actually boost it */
//         if ((OwnerThread->Priority < Thread->Priority) &&
//             (OwnerThread->Priority < 14))
//         {
//             /* Acquire the thread lock */
//             KiAcquireThreadLock(Thread);
//
//             /* Set the new priority */
//             OwnerThread->PriorityDecrement += 14 - OwnerThread->Priority;
//
//             /* Update quantum */
//             OwnerThread->Quantum = OwnerThread->QuantumReset;
//
//             /* Update the kernel state */
//             KiSetPriorityThread(OwnerThread, 14);
//
//             /* Release the thread lock */
//             KiReleaseThreadLock(Thread);
//         }
//     }
// }
//
// /*++
//  * @name ExpWaitForResource
//  *
//  *     The ExpWaitForResource routine performs a wait on the specified resource.
//  *
//  * @param Resource
//  *        Pointer to the resource to wait on.
//  *
//  * @param OwnerThread
//  *        Pointer to object (exclusive event or shared semaphore) to wait on.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// FASTCALL
// ExpWaitForResource(IN PERESOURCE Resource,
//                    IN PVOID Object)
// {
//     ULONG i;
//     ULONG Size;
//     POWNER_ENTRY Owner;
//     ULONG WaitCount = 0;
//     NTSTATUS Status;
//     LARGE_INTEGER Timeout;
//     PKTHREAD Thread, OwnerThread;
// #if DBG
//     KLOCK_QUEUE_HANDLE LockHandle;
// #endif
//
//     /* Increase contention count and use a 5 second timeout */
//     Resource->ContentionCount++;
//     Timeout.QuadPart = 500 * -10000LL;
//     for (;;)
//     {
//         /* Wait for ownership */
//         Status = KeWaitForSingleObject(Object,
//                                        WrResource,
//                                        KernelMode,
//                                        FALSE,
//                                        &Timeout);
//         if (Status != STATUS_TIMEOUT)
//             break;
//
//         /* Increase wait count */
//         WaitCount++;
//         Timeout = ExpTimeout;
//
//         /* Check if we've exceeded the limit */
//         if (WaitCount > ExpResourceTimeoutCount)
//         {
//             /* Reset wait count */
//             WaitCount = 0;
// #if DBG
//             /* Lock the resource */
//             ExAcquireResourceLock(Resource, &LockHandle);
//
//             /* Dump debug information */
//             DPRINT1("Resource @ %p\n", Resource);
//             DPRINT1(" ActiveEntries = %04lx  Flags = %s%s%s\n",
//                     Resource->ActiveEntries,
//                     IsOwnedExclusive(Resource) ? "IsOwnedExclusive " : "",
//                     IsSharedWaiting(Resource) ? "SharedWaiter "     : "",
//                     IsExclusiveWaiting(Resource) ? "ExclusiveWaiter "  : "");
//             DPRINT1(" NumberOfExclusiveWaiters = %04lx\n",
//                     Resource->NumberOfExclusiveWaiters);
//             DPRINT1("   Thread = %08lx, Count = %02x\n",
//                     Resource->OwnerEntry.OwnerThread,
//                     Resource->OwnerEntry.OwnerCount);
//
//             /* Dump out the table too */
//             Owner = Resource->OwnerTable;
//             if (Owner)
//             {
//                 /* Loop every entry */
//                 Size = Owner->TableSize;
//                 for (i = 1; i < Size; i++)
//                 {
//                     /* Print the data */
//                     Owner++;
//                     DPRINT1("   Thread = %08lx, Count = %02x\n",
//                             Owner->OwnerThread,
//                             Owner->OwnerCount);
//                 }
//             }
//
//             /* Break */
//             DbgBreakPoint();
//             DPRINT1("EX - Rewaiting\n");
//             ExReleaseResourceLock(Resource, &LockHandle);
// #endif
//         }
//
//         /* Check if we can boost */
//         if (IsBoostAllowed(Resource))
//         {
//             /* Get the current kernel thread and lock the dispatcher */
//             Thread = KeGetCurrentThread();
//             Thread->WaitIrql = KiAcquireDispatcherLock();
//             Thread->WaitNext = TRUE;
//
//             /* Get the owner thread and boost it */
//             OwnerThread = (PKTHREAD)Resource->OwnerEntry.OwnerThread;
//             if (OwnerThread) ExpBoostOwnerThread(Thread, OwnerThread);
//
//             /* If it's a shared resource */
//             if (!IsOwnedExclusive(Resource))
//             {
//                 /* Get the table */
//                 Owner = Resource->OwnerTable;
//                 if (Owner)
//                 {
//                     /* Loop every entry */
//                     Size = Owner->TableSize;
//                     for (i = 1; i < Size; i++)
//                     {
//                         /* Move to next entry */
//                         Owner++;
//
//                         /* Get the thread */
//                         OwnerThread = (PKTHREAD)Owner->OwnerThread;
//
//                         /* Boost it */
//                         if (OwnerThread) ExpBoostOwnerThread(Thread, OwnerThread);
//                     }
//                 }
//             }
//         }
//     }
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /*++
//  * @name ExAcquireResourceExclusiveLite
//  * @implemented NT4
//  *
//  *     The ExAcquireResourceExclusiveLite routine acquires the given resource
//  *     for exclusive access by the calling thread.
//  *
//  * @param Resource
//  *        Pointer to the resource to acquire.
//  *
//  * @param Wait
//  *        Specifies the routine's behavior whenever the resource cannot be
//  *        acquired immediately.
//  *
//  * @return TRUE if the resource is acquired. FALSE if the input Wait is FALSE
//  *         and exclusive access cannot be granted immediately.
//  *
//  * @remarks The caller can release the resource by calling either
//  *          ExReleaseResourceLite or ExReleaseResourceForThreadLite.
//  *
//  *          Normal kernel APC delivery must be disabled before calling this
//  *          routine. Disable normal kernel APC delivery by calling
//  *          KeEnterCriticalRegion. Delivery must remain disabled until the
//  *          resource is released, at which point it can be reenabled by calling
//  *          KeLeaveCriticalRegion.
//  *
//  *          For better performance, call ExTryToAcquireResourceExclusiveLite,
//  *          rather than calling ExAcquireResourceExclusiveLite with Wait set
//  *          to FALSE.
//  *
//  *          Callers of ExAcquireResourceExclusiveLite must be running at IRQL <
//  *          DISPATCH_LEVEL.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// ExAcquireResourceExclusiveLite(IN PERESOURCE Resource,
//                                IN BOOLEAN Wait)
// {
//     KLOCK_QUEUE_HANDLE LockHandle;
//     ERESOURCE_THREAD Thread;
//     BOOLEAN Success;
//
//     /* Sanity check */
//     ASSERT((Resource->Flag & ResourceNeverExclusive) == 0);
//
//     /* Get the thread */
//     Thread = ExGetCurrentResourceThread();
//
//     /* Sanity check and validation */
//     ASSERT(KeIsExecutingDpc() == FALSE);
//     ExpVerifyResource(Resource);
//
//     /* Acquire the lock */
//     ExAcquireResourceLock(Resource, &LockHandle);
//     ExpCheckForApcsDisabled(LockHandle.OldIrql, Resource, (PKTHREAD)Thread);
//
//     /* Check if there is a shared owner or exclusive owner */
// TryAcquire:
//     if (Resource->ActiveEntries)
//     {
//         /* Check if it's exclusively owned, and we own it */
//         if ((IsOwnedExclusive(Resource)) &&
//             (Resource->OwnerEntry.OwnerThread == Thread))
//         {
//             /* Increase the owning count */
//             Resource->OwnerEntry.OwnerCount++;
//             Success = TRUE;
//         }
//         else
//         {
//             /*
//              * If the caller doesn't want us to wait, we can't acquire the
//              * resource because someone else then us owns it. If we can wait,
//              * then we'll wait.
//              */
//             if (!Wait)
//             {
//                 Success = FALSE;
//             }
//             else
//             {
//                 /* Check if it has exclusive waiters */
//                 if (!Resource->ExclusiveWaiters)
//                 {
//                     /* It doesn't, allocate the event and try acquiring again */
//                     ExpAllocateExclusiveWaiterEvent(Resource, &LockHandle);
//                     goto TryAcquire;
//                 }
//
//                 /* Has exclusive waiters, wait on it */
//                 Resource->NumberOfExclusiveWaiters++;
//                 ExReleaseResourceLock(Resource, &LockHandle);
//                 ExpWaitForResource(Resource, Resource->ExclusiveWaiters);
//
//                 /* Set owner and return success */
//                 Resource->OwnerEntry.OwnerThread = ExGetCurrentResourceThread();
//                 return TRUE;
//             }
//         }
//     }
//     else
//     {
//         /* Nobody owns it, so let's! */
//         ASSERT(Resource->ActiveEntries == 0);
//         ASSERT(Resource->ActiveCount == 0);
//         Resource->Flag |= ResourceOwnedExclusive;
//         Resource->ActiveEntries = 1;
//         Resource->ActiveCount = 1;
//         Resource->OwnerEntry.OwnerThread = Thread;
//         Resource->OwnerEntry.OwnerCount = 1;
//         Success = TRUE;
//     }
//
//     /* Release the lock and return */
//     ExReleaseResourceLock(Resource, &LockHandle);
//     return Success;
// }
//
// /*++
//  * @name ExAcquireResourceSharedLite
//  * @implemented NT4
//  *
//  *     The ExAcquireResourceSharedLite routine acquires the given resource
//  *     for shared access by the calling thread.
//  *
//  * @param Resource
//  *        Pointer to the resource to acquire.
//  *
//  * @param Wait
//  *        Specifies the routine's behavior whenever the resource cannot be
//  *        acquired immediately.
//  *
//  * @return TRUE if the resource is acquired. FALSE if the input Wait is FALSE
//  *         and exclusive access cannot be granted immediately.
//  *
//  * @remarks The caller can release the resource by calling either
//  *          ExReleaseResourceLite or ExReleaseResourceForThreadLite.
//  *
//  *          Normal kernel APC delivery must be disabled before calling this
//  *          routine. Disable normal kernel APC delivery by calling
//  *          KeEnterCriticalRegion. Delivery must remain disabled until the
//  *          resource is released, at which point it can be reenabled by calling
//  *          KeLeaveCriticalRegion.
//  *
//  *          Callers of ExAcquireResourceExclusiveLite must be running at IRQL <
//  *          DISPATCH_LEVEL.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// ExAcquireResourceSharedLite(IN PERESOURCE Resource,
//                             IN BOOLEAN Wait)
// {
//     KLOCK_QUEUE_HANDLE LockHandle;
//     ERESOURCE_THREAD Thread;
//     POWNER_ENTRY Owner = NULL;
//     BOOLEAN FirstEntryBusy;
//
//     /* Get the thread */
//     Thread = ExGetCurrentResourceThread();
//
//     /* Sanity check and validation */
//     ASSERT(KeIsExecutingDpc() == FALSE);
//     ExpVerifyResource(Resource);
//
//     /* Acquire the lock */
//     ExAcquireResourceLock(Resource, &LockHandle);
//     ExpCheckForApcsDisabled(LockHandle.OldIrql, Resource, (PKTHREAD)Thread);
//
//     /* Check how many active entries we've got */
//     while (Resource->ActiveEntries != 0)
//     {
//         /* Check if it's exclusively owned */
//         if (IsOwnedExclusive(Resource))
//         {
//             /* Check if we own it */
//             if (Resource->OwnerEntry.OwnerThread == Thread)
//             {
//                 /* Increase the owning count */
//                 Resource->OwnerEntry.OwnerCount++;
//
//                 /* Release the lock and return */
//                 ExReleaseResourceLock(Resource, &LockHandle);
//                 return TRUE;
//             }
//
//             /* Find a free entry */
//             Owner = ExpFindFreeEntry(Resource, &LockHandle);
//             if (!Owner) continue;
//         }
//         else
//         {
//             /* Resource is shared, find who owns it */
//             FirstEntryBusy = IsExclusiveWaiting(Resource);
//             Owner = ExpFindEntryForThread(Resource,
//                                           Thread,
//                                           &LockHandle,
//                                           FirstEntryBusy);
//             if (!Owner) continue;
//
//             /* Is it us? */
//             if (Owner->OwnerThread == Thread)
//             {
//                 /* Increase acquire count and return */
//                 Owner->OwnerCount++;
//                 ASSERT(Owner->OwnerCount != 0);
//
//                 /* Release the lock and return */
//                 ExReleaseResourceLock(Resource, &LockHandle);
//                 return TRUE;
//             }
//
//             /* Try to find if there are exclusive waiters */
//             if (!FirstEntryBusy)
//             {
//                 /* There are none, so acquire it */
//                 Owner->OwnerThread = Thread;
//                 Owner->OwnerCount = 1;
//
//                 /* Check how many active entries we had */
//                 if (Resource->ActiveEntries == 0)
//                 {
//                     /* Set initial counts */
//                     ASSERT(Resource->ActiveCount == 0);
//                     Resource->ActiveEntries = 1;
//                     Resource->ActiveCount = 1;
//                 }
//                 else
//                 {
//                     /* Increase active entries */
//                     ASSERT(Resource->ActiveCount == 1);
//                     Resource->ActiveEntries++;
//                 }
//
//                 /* Release the lock and return */
//                 ExReleaseResourceLock(Resource, &LockHandle);
//                 return TRUE;
//             }
//         }
//
//         /* If we got here, then we need to wait. Are we allowed? */
//         if (!Wait)
//         {
//             /* Release the lock and return */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return FALSE;
//         }
//
//         /* Check if we have a shared waiters semaphore */
//         if (!Resource->SharedWaiters)
//         {
//             /* Allocate it and try another acquire */
//             ExpAllocateSharedWaiterSemaphore(Resource, &LockHandle);
//         }
//         else
//         {
//             /* We have shared waiters, wait for it */
//             break;
//         }
//     }
//
//     /* Did we get here because we don't have active entries? */
//     if (Resource->ActiveEntries == 0)
//     {
//         /* Acquire it */
//         ASSERT(Resource->ActiveEntries == 0);
//         ASSERT(Resource->ActiveCount == 0);
//         Resource->ActiveEntries = 1;
//         Resource->ActiveCount = 1;
//         Resource->OwnerEntry.OwnerThread = Thread;
//         Resource->OwnerEntry.OwnerCount = 1;
//
//         /* Release the lock and return */
//         ExReleaseResourceLock(Resource, &LockHandle);
//         return TRUE;
//     }
//
//     /* Now wait for the resource */
//     Owner->OwnerThread = Thread;
//     Owner->OwnerCount = 1;
//     Resource->NumberOfSharedWaiters++;
//
//     /* Release the lock and return */
//     ExReleaseResourceLock(Resource, &LockHandle);
//     ExpWaitForResource(Resource, Resource->SharedWaiters);
//     return TRUE;
// }
//
// /*++
//  * @name ExAcquireSharedStarveExclusive
//  * @implemented NT4
//  *
//  *     The ExAcquireSharedStarveExclusive routine acquires the given resource
//  *     shared access without waiting for any pending attempts to acquire
//  *     exclusive access to the same resource.
//  *
//  * @param Resource
//  *        Pointer to the resource to acquire.
//  *
//  * @param Wait
//  *        Specifies the routine's behavior whenever the resource cannot be
//  *        acquired immediately.
//  *
//  * @return TRUE if the resource is acquired. FALSE if the input Wait is FALSE
//  *         and exclusive access cannot be granted immediately.
//  *
//  * @remarks The caller can release the resource by calling either
//  *          ExReleaseResourceLite or ExReleaseResourceForThreadLite.
//  *
//  *          Normal kernel APC delivery must be disabled before calling this
//  *          routine. Disable normal kernel APC delivery by calling
//  *          KeEnterCriticalRegion. Delivery must remain disabled until the
//  *          resource is released, at which point it can be reenabled by calling
//  *          KeLeaveCriticalRegion.
//  *
//  *          Callers of ExAcquireSharedStarveExclusive usually need quick access
//  *          to a shared resource in order to save an exclusive accessor from
//  *          doing redundant work. For example, a file system might call this
//  *          routine to modify a cached resource, such as a BCB pinned in the
//  *          cache, before the Cache Manager can acquire exclusive access to the
//  *          resource and write the cache out to disk.
//  *
//  *          Callers of ExAcquireResourceExclusiveLite must be running at IRQL <
//  *          DISPATCH_LEVEL.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// ExAcquireSharedStarveExclusive(IN PERESOURCE Resource,
//                                IN BOOLEAN Wait)
// {
//     KLOCK_QUEUE_HANDLE LockHandle;
//     ERESOURCE_THREAD Thread;
//     POWNER_ENTRY Owner;
//
//     /* Get the thread */
//     Thread = ExGetCurrentResourceThread();
//
//     /* Sanity check and validation */
//     ASSERT(KeIsExecutingDpc() == FALSE);
//     ExpVerifyResource(Resource);
//
//     /* Acquire the lock */
//     ExAcquireResourceLock(Resource, &LockHandle);
//
//     /* See if anyone owns it */
// TryAcquire:
//     if (Resource->ActiveEntries == 0)
//     {
//         /* Nobody owns it, so let's take control */
//         ASSERT(Resource->ActiveEntries == 0);
//         ASSERT(Resource->ActiveCount == 0);
//         Resource->ActiveCount = 1;
//         Resource->ActiveEntries = 1;
//         Resource->OwnerEntry.OwnerThread = Thread;
//         Resource->OwnerEntry.OwnerCount = 1;
//
//         /* Release the lock and return */
//         ExReleaseResourceLock(Resource, &LockHandle);
//         return TRUE;
//     }
//
//     /* Check if it's exclusively owned */
//     if (IsOwnedExclusive(Resource))
//     {
//         /* Check if we own it */
//         if (Resource->OwnerEntry.OwnerThread == Thread)
//         {
//             /* Increase the owning count */
//             Resource->OwnerEntry.OwnerCount++;
//
//             /* Release the lock and return */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return TRUE;
//         }
//
//         /* Find a free entry */
//         Owner = ExpFindFreeEntry(Resource, &LockHandle);
//         if (!Owner) goto TryAcquire;
//     }
//     else
//     {
//         /* Resource is shared, find who owns it */
//         Owner = ExpFindEntryForThread(Resource, Thread, &LockHandle, FALSE);
//         if (!Owner) goto TryAcquire;
//
//         /* Is it us? */
//         if (Owner->OwnerThread == Thread)
//         {
//             /* Increase acquire count and return */
//             Owner->OwnerCount++;
//             ASSERT(Owner->OwnerCount != 0);
//
//             /* Release the lock and return */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return TRUE;
//         }
//
//         /* Acquire it */
//         Owner->OwnerThread = Thread;
//         Owner->OwnerCount = 1;
//
//         /* Check how many active entries we had */
//         if (Resource->ActiveEntries == 0)
//         {
//             /* Set initial counts */
//             ASSERT(Resource->ActiveCount == 0);
//             Resource->ActiveEntries = 1;
//             Resource->ActiveCount = 1;
//         }
//         else
//         {
//             /* Increase active entries */
//             ASSERT(Resource->ActiveCount == 1);
//             Resource->ActiveEntries++;
//         }
//
//         /* Release the lock and return */
//         ExReleaseResourceLock(Resource, &LockHandle);
//         return TRUE;
//     }
//
//     /* If we got here, then we need to wait. Are we allowed? */
//     if (!Wait)
//     {
//         /* Release the lock and return */
//         ExReleaseResourceLock(Resource, &LockHandle);
//         return FALSE;
//     }
//
//     /* Check if we have a shared waiters semaphore */
//     if (!Resource->SharedWaiters)
//     {
//         /* Allocate it and try another acquire */
//         ExpAllocateSharedWaiterSemaphore(Resource, &LockHandle);
//         goto TryAcquire;
//     }
//
//     /* Now wait for the resource */
//     Owner->OwnerThread = Thread;
//     Owner->OwnerCount = 1;
//     Resource->NumberOfSharedWaiters++;
//
//     /* Release the lock and return */
//     ExReleaseResourceLock(Resource, &LockHandle);
//     ExpWaitForResource(Resource, Resource->SharedWaiters);
//     return TRUE;
// }
//
// /*++
//  * @name ExAcquireSharedWaitForExclusive
//  * @implemented NT4
//  *
//  *     The ExAcquireSharedWaitForExclusive routine acquires the given resource
//  *     for shared access if shared access can be granted and there are no
//  *     exclusive waiters.
//  *
//  * @param Resource
//  *        Pointer to the resource to acquire.
//  *
//  * @param Wait
//  *        Specifies the routine's behavior whenever the resource cannot be
//  *        acquired immediately.
//  *
//  * @return TRUE if the resource is acquired. FALSE if the input Wait is FALSE
//  *         and exclusive access cannot be granted immediately.
//  *
//  * @remarks The caller can release the resource by calling either
//  *          ExReleaseResourceLite or ExReleaseResourceForThreadLite.
//  *
//  *          Normal kernel APC delivery must be disabled before calling this
//  *          routine. Disable normal kernel APC delivery by calling
//  *          KeEnterCriticalRegion. Delivery must remain disabled until the
//  *          resource is released, at which point it can be reenabled by calling
//  *          KeLeaveCriticalRegion.
//  *
//  *          Callers of ExAcquireResourceExclusiveLite must be running at IRQL <
//  *          DISPATCH_LEVEL.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// ExAcquireSharedWaitForExclusive(IN PERESOURCE Resource,
//                                 IN BOOLEAN Wait)
// {
//     KLOCK_QUEUE_HANDLE LockHandle;
//     ERESOURCE_THREAD Thread;
//     POWNER_ENTRY Owner;
//
//     /* Get the thread */
//     Thread = ExGetCurrentResourceThread();
//
//     /* Sanity check and validation */
//     ASSERT(KeIsExecutingDpc() == FALSE);
//     ExpVerifyResource(Resource);
//
//     /* Acquire the lock */
//     ExAcquireResourceLock(Resource, &LockHandle);
//
//     /* See if nobody owns us */
// TryAcquire:
//     if (!Resource->ActiveEntries)
//     {
//         /* Nobody owns it, so let's take control */
//         ASSERT(Resource->ActiveEntries == 0);
//         ASSERT(Resource->ActiveCount == 0);
//         Resource->ActiveCount = 1;
//         Resource->ActiveEntries = 1;
//         Resource->OwnerEntry.OwnerThread = Thread;
//         Resource->OwnerEntry.OwnerCount = 1;
//
//         /* Release the lock and return */
//         ExReleaseResourceLock(Resource, &LockHandle);
//         return TRUE;
//     }
//
//     /* Check if it's exclusively owned */
//     if (IsOwnedExclusive(Resource))
//     {
//         /* Check if we own it */
//         if (Resource->OwnerEntry.OwnerThread == Thread)
//         {
//             /* Increase the owning count */
//             Resource->OwnerEntry.OwnerCount++;
//
//             /* Release the lock and return */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return TRUE;
//         }
//
//         /* Find a free entry */
//         Owner = ExpFindFreeEntry(Resource, &LockHandle);
//         if (!Owner) goto TryAcquire;
//     }
//     else
//     {
//         /* Try to find if there are exclusive waiters */
//         if (IsExclusiveWaiting(Resource))
//         {
//             /* We have to wait for the exclusive waiter to be done */
//             if (!Wait)
//             {
//                 /* So bail out if we're not allowed */
//                 ExReleaseResourceLock(Resource, &LockHandle);
//                 return FALSE;
//             }
//
//             /* Check if we have a shared waiters semaphore */
//             if (!Resource->SharedWaiters)
//             {
//                 /* Allocate one and try again */
//                 ExpAllocateSharedWaiterSemaphore(Resource, &LockHandle);
//                 goto TryAcquire;
//             }
//
//             /* Now wait for the resource */
//             Resource->NumberOfSharedWaiters++;
//             ExReleaseResourceLock(Resource, &LockHandle);
//             ExpWaitForResource(Resource, Resource->SharedWaiters);
//
//             /* Get the lock back */
//             ExAcquireResourceLock(Resource, &LockHandle);
//
//             /* Find who owns it now */
//             while (!(Owner = ExpFindEntryForThread(Resource, Thread, &LockHandle, TRUE)));
//
//             /* Sanity checks */
//             ASSERT(IsOwnedExclusive(Resource) == FALSE);
//             ASSERT(Resource->ActiveEntries > 0);
//             ASSERT(Owner->OwnerThread != Thread);
//
//             /* Take control */
//             Owner->OwnerThread = Thread;
//             Owner->OwnerCount = 1;
//
//             /* Release the lock and return */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return TRUE;
//         }
//         else
//         {
//             /* Resource is shared, find who owns it */
//             Owner = ExpFindEntryForThread(Resource, Thread, &LockHandle, FALSE);
//             if (!Owner) goto TryAcquire;
//
//             /* Is it us? */
//             if (Owner->OwnerThread == Thread)
//             {
//                 /* Increase acquire count and return */
//                 Owner->OwnerCount++;
//                 ASSERT(Owner->OwnerCount != 0);
//
//                 /* Release the lock and return */
//                 ExReleaseResourceLock(Resource, &LockHandle);
//                 return TRUE;
//             }
//
//             /* No exclusive waiters, so acquire it */
//             Owner->OwnerThread = Thread;
//             Owner->OwnerCount = 1;
//
//             /* Check how many active entries we had */
//             if (Resource->ActiveEntries == 0)
//             {
//                 /* Set initial counts */
//                 ASSERT(Resource->ActiveCount == 0);
//                 Resource->ActiveEntries = 1;
//                 Resource->ActiveCount = 1;
//             }
//             else
//             {
//                 /* Increase active entries */
//                 ASSERT(Resource->ActiveCount == 1);
//                 Resource->ActiveEntries++;
//             }
//
//             /* Release the lock and return */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return TRUE;
//         }
//     }
//
//     /* We have to wait for the exclusive waiter to be done */
//     if (!Wait)
//     {
//         /* So bail out if we're not allowed */
//         ExReleaseResourceLock(Resource, &LockHandle);
//         return FALSE;
//     }
//
//     /* Check if we have a shared waiters semaphore */
//     if (!Resource->SharedWaiters)
//     {
//         /* Allocate one and try again */
//         ExpAllocateSharedWaiterSemaphore(Resource,&LockHandle);
//         goto TryAcquire;
//     }
//
//     /* Take control */
//     Owner->OwnerThread = Thread;
//     Owner->OwnerCount = 1;
//     Resource->NumberOfSharedWaiters++;
//
//     /* Release the lock and return */
//     ExReleaseResourceLock(Resource, &LockHandle);
//     ExpWaitForResource(Resource, Resource->SharedWaiters);
//     return TRUE;
// }
//
// /*++
//  * @name ExConvertExclusiveToSharedLite
//  * @implemented NT4
//  *
//  *     The ExConvertExclusiveToSharedLite routine converts an exclusively
//  *     acquired resource into a resource that can be acquired shared.
//  *
//  * @param Resource
//  *        Pointer to the resource to convert.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExConvertExclusiveToSharedLite must be running at IRQL <
//  *          DISPATCH_LEVEL.
//  *
//  *--*/
// VOID
// NTAPI
// ExConvertExclusiveToSharedLite(IN PERESOURCE Resource)
// {
//     ULONG OldWaiters;
//     KLOCK_QUEUE_HANDLE LockHandle;
//
//     /* Sanity checks */
//     ASSERT(KeIsExecutingDpc() == FALSE);
//     ExpVerifyResource(Resource);
//     ASSERT(IsOwnedExclusive(Resource));
//     ASSERT(Resource->OwnerEntry.OwnerThread == (ERESOURCE_THREAD)PsGetCurrentThread());
//
//     /* Lock the resource */
//     ExAcquireResourceLock(Resource, &LockHandle);
//
//     /* Erase the exclusive flag */
//     Resource->Flag &= ~ResourceOwnedExclusive;
//
//     /* Check if we have shared waiters */
//     if (IsSharedWaiting(Resource))
//     {
//         /* Make the waiters active owners */
//         OldWaiters = Resource->NumberOfSharedWaiters;
//         Resource->ActiveEntries += OldWaiters;
//         Resource->NumberOfSharedWaiters = 0;
//
//         /* Release lock and wake the waiters */
//         ExReleaseResourceLock(Resource, &LockHandle);
//         KeReleaseSemaphore(Resource->SharedWaiters, 0, OldWaiters, FALSE);
//     }
//     else
//     {
//         /* Release lock */
//         ExReleaseResourceLock(Resource, &LockHandle);
//     }
// }
//
// /*++
//  * @name ExDeleteResourceLite
//  * @implemented NT4
//  *
//  *     The ExConvertExclusiveToSharedLite routine deletes a given resource
//  *     from the system�s resource list.
//  *
//  * @param Resource
//  *        Pointer to the resource to delete.
//  *
//  * @return STATUS_SUCCESS if the resource was deleted.
//  *
//  * @remarks Callers of ExDeleteResourceLite must be running at IRQL <
//  *          DISPATCH_LEVEL.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// ExDeleteResourceLite(IN PERESOURCE Resource)
// {
//     KLOCK_QUEUE_HANDLE LockHandle;
//
//     /* Sanity checks */
//     ASSERT(IsSharedWaiting(Resource) == FALSE);
//     ASSERT(IsExclusiveWaiting(Resource) == FALSE);
//     ASSERT(KeIsExecutingDpc() == FALSE);
//     ExpVerifyResource(Resource);
//
//     /* Lock the resource */
//     KeAcquireInStackQueuedSpinLock(&ExpResourceSpinLock, &LockHandle);
//
//     /* Remove the resource */
//     RemoveEntryList(&Resource->SystemResourcesList);
//
//     /* Release the lock */
//     KeReleaseInStackQueuedSpinLock(&LockHandle);
//
//     /* Free every  structure */
//     if (Resource->OwnerTable) ExFreePoolWithTag(Resource->OwnerTable, TAG_RESOURCE_TABLE);
//     if (Resource->SharedWaiters) ExFreePoolWithTag(Resource->SharedWaiters, TAG_RESOURCE_SEMAPHORE);
//     if (Resource->ExclusiveWaiters) ExFreePoolWithTag(Resource->ExclusiveWaiters, TAG_RESOURCE_EVENT);
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name ExDisableResourceBoostLite
//  * @implemented NT4
//  *
//  *     The ExDisableResourceBoostLite routine disables thread boosting for
//  *     the given resource.
//  *
//  * @param Resource
//  *        Pointer to the resource whose thread boosting will be disabled.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// ExDisableResourceBoostLite(IN PERESOURCE Resource)
// {
//     KLOCK_QUEUE_HANDLE LockHandle;
//
//     /* Sanity check */
//     ExpVerifyResource(Resource);
//
//     /* Lock the resource */
//     ExAcquireResourceLock(Resource, &LockHandle);
//
//     /* Remove the flag */
//     Resource->Flag |= ResourceHasDisabledPriorityBoost;
//
//     /* Release the lock */
//     ExReleaseResourceLock(Resource, &LockHandle);
// }
//
// /*++
//  * @name ExGetExclusiveWaiterCount
//  * @implemented NT4
//  *
//  *     The ExGetExclusiveWaiterCount routine returns the number of exclusive
//  *     waiters for the given resource.
//  *
//  * @param Resource
//  *        Pointer to the resource to check.
//  *
//  * @return The number of exclusive waiters.
//  *
//  * @remarks None.
//  *
//  *--*/
// ULONG
// NTAPI
// ExGetExclusiveWaiterCount(IN PERESOURCE Resource)
// {
//     /* Return the count */
//     return Resource->NumberOfExclusiveWaiters;
// }
//
// /*++
//  * @name ExGetSharedWaiterCount
//  * @implemented NT4
//  *
//  *     The ExGetSharedWaiterCount routine returns the number of shared
//  *     waiters for the given resource.
//  *
//  * @param Resource
//  *        Pointer to the resource to check.
//  *
//  * @return The number of shared waiters.
//  *
//  * @remarks None.
//  *
//  *--*/
// ULONG
// NTAPI
// ExGetSharedWaiterCount(IN PERESOURCE Resource)
// {
//     /* Return the count */
//     return Resource->NumberOfSharedWaiters;
// }
//
// /*++
//  * @name ExInitializeResourceLite
//  * @implemented NT4
//  *
//  *     The ExInitializeResourceLite routine initializes a resource variable.
//  *
//  * @param Resource
//  *        Pointer to the resource to check.
//  *
//  * @return STATUS_SUCCESS.
//  *
//  * @remarks The storage for ERESOURCE must not be allocated from paged pool.
//  *
//  *          The storage must be 8-byte aligned.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// ExInitializeResourceLite(IN PERESOURCE Resource)
// {
//     KLOCK_QUEUE_HANDLE LockHandle;
//
//     /* Clear the structure */
//     RtlZeroMemory(Resource, sizeof(ERESOURCE));
//
//     /* Initialize the lock */
//     KeInitializeSpinLock(&Resource->SpinLock);
//
//     /* Add it into the system list */
//     KeAcquireInStackQueuedSpinLock(&ExpResourceSpinLock, &LockHandle);
//     InsertTailList(&ExpSystemResourcesList, &Resource->SystemResourcesList);
//     KeReleaseInStackQueuedSpinLock(&LockHandle);
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name ExIsResourceAcquiredExclusiveLite
//  * @implemented NT4
//  *
//  *     The ExIsResourceAcquiredExclusiveLite routine returns whether the
//  *     current thread has exclusive access to a given resource.
//  *
//  * @param Resource
//  *        Pointer to the resource to check.
//  *
//  * @return TRUE if the caller already has exclusive access to the given resource.
//  *
//  * @remarks Callers of ExIsResourceAcquiredExclusiveLite must be running at
//  *          IRQL <= DISPATCH_LEVEL.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// ExIsResourceAcquiredExclusiveLite(IN PERESOURCE Resource)
// {
//     BOOLEAN IsAcquired = FALSE;
//
//     /* Sanity check */
//     ExpVerifyResource(Resource);
//
//     /* Check if it's exclusively acquired */
//     if ((IsOwnedExclusive(Resource)) &&
//         (Resource->OwnerEntry.OwnerThread == ExGetCurrentResourceThread()))
//     {
//         /* It is acquired */
//         IsAcquired = TRUE;
//     }
//
//     /* Return if it's acquired */
//     return IsAcquired;
// }
//
// /*++
//  * @name ExIsResourceAcquiredSharedLite
//  * @implemented NT4
//  *
//  *     The ExIsResourceAcquiredSharedLite routine returns whether the
//  *     current thread has has access (either shared or exclusive) to a
//  *     given resource.
//  *
//  * @param Resource
//  *        Pointer to the resource to check.
//  *
//  * @return Number of times the caller has acquired the given resource for
//  *         shared or exclusive access.
//  *
//  * @remarks Callers of ExIsResourceAcquiredExclusiveLite must be running at
//  *          IRQL <= DISPATCH_LEVEL.
//  *
//  *--*/
// ULONG
// NTAPI
// ExIsResourceAcquiredSharedLite(IN PERESOURCE Resource)
// {
//     ERESOURCE_THREAD Thread;
//     ULONG i, Size;
//     ULONG Count = 0;
//     KLOCK_QUEUE_HANDLE LockHandle;
//     POWNER_ENTRY Owner;
//
//     /* Sanity check */
//     ExpVerifyResource(Resource);
//
//     /* Check if nobody owns us */
//     if (!Resource->ActiveEntries) return 0;
//
//     /* Get the thread */
//     Thread = ExGetCurrentResourceThread();
//
//     /* Check if we are in the thread list */
//     if (Resource->OwnerEntry.OwnerThread == Thread)
//     {
//         /* Found it, return count */
//         Count = Resource->OwnerEntry.OwnerCount;
//     }
//     else
//     {
//         /* We can't own an exclusive resource at this point */
//         if (IsOwnedExclusive(Resource)) return 0;
//
//         /* Lock the resource */
//         ExAcquireResourceLock(Resource, &LockHandle);
//
//         /* Not in the list, do a full table look up */
//         Owner = Resource->OwnerTable;
//         if (Owner)
//         {
//             /* Get the resource index */
//             i = ((PKTHREAD)Thread)->ResourceIndex;
//             Size = Owner->TableSize;
//
//             /* Check if the index is valid and check if we don't match */
//             if ((i >= Size) || (Owner[i].OwnerThread != Thread))
//             {
//                 /* Sh*t! We need to do a full search */
//                 for (i = 1; i < Size; i++)
//                 {
//                     /* Move to next owner */
//                     Owner++;
//
//                     /* Try to find a match */
//                     if (Owner->OwnerThread == Thread)
//                     {
//                         /* Finally! */
//                         Count = Owner->OwnerCount;
//                         break;
//                     }
//                 }
//             }
//             else
//             {
//                 /* We found the match directlry */
//                 Count = Owner[i].OwnerCount;
//             }
//         }
//
//         /* Release the lock */
//         ExReleaseResourceLock(Resource, &LockHandle);
//     }
//
//     /* Return count */
//     return Count;
// }
//
// /*++
//  * @name ExReinitializeResourceLite
//  * @implemented NT4
//  *
//  *     The ExReinitializeResourceLite routine routine reinitializes
//  *     an existing resource variable.
//  *
//  * @param Resource
//  *        Pointer to the resource to be reinitialized.
//  *
//  * @return STATUS_SUCCESS.
//  *
//  * @remarks With a single call to ExReinitializeResource, a driver writer can
//  *          replace three calls: one to ExDeleteResourceLite, another to
//  *          ExAllocatePool, and a third to ExInitializeResourceLite. As
//  *          contention for a resource variable increases, memory is dynamically
//  *          allocated and attached to the resource in order to track this
//  *          contention. As an optimization, ExReinitializeResourceLite retains
//  *          and zeroes this previously allocated memory.
//  *
//  *          Callers of ExReinitializeResourceLite must be running at
//  *          IRQL <= DISPATCH_LEVEL.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// ExReinitializeResourceLite(IN PERESOURCE Resource)
// {
//     PKEVENT Event;
//     PKSEMAPHORE Semaphore;
//     ULONG i, Size;
//     POWNER_ENTRY Owner;
//
//     /* Get the owner table */
//     Owner = Resource->OwnerTable;
//     if (Owner)
//     {
//         /* Get the size and loop it */
//         Size = Owner->TableSize;
//         for (i = 0; i < Size; i++)
//         {
//             /* Zero the table */
//             Owner[i].OwnerThread = 0;
//             Owner[i].OwnerCount = 0;
//         }
//     }
//
//     /* Zero the flags and count */
//     Resource->Flag = 0;
//     Resource->ActiveCount = 0;
//     Resource->ActiveEntries = 0;
//
//     /* Reset the semaphore */
//     Semaphore = Resource->SharedWaiters;
//     if (Semaphore) KeInitializeSemaphore(Semaphore, 0, MAXLONG);
//
//     /* Reset the event */
//     Event = Resource->ExclusiveWaiters;
//     if (Event) KeInitializeEvent(Event, SynchronizationEvent, FALSE);
//
//     /* Clear the resource data */
//     Resource->OwnerEntry.OwnerThread = 0;
//     Resource->OwnerEntry.OwnerCount = 0;
//     Resource->ContentionCount = 0;
//     Resource->NumberOfSharedWaiters = 0;
//     Resource->NumberOfExclusiveWaiters = 0;
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name ExReleaseResourceLite
//  * @implemented NT4
//  *
//  *     The ExReleaseResourceLite routine routine releases
//  *     a specified executive resource owned by the current thread.
//  *
//  * @param Resource
//  *        Pointer to the resource to be released.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExReleaseResourceLite must be running at
//  *          IRQL <= DISPATCH_LEVEL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExReleaseResourceLite(IN PERESOURCE Resource)
// {
//     /* Just call the For-Thread function */
//     ExReleaseResourceForThreadLite(Resource, ExGetCurrentResourceThread());
// }
//
// /*++
//  * @name ExReleaseResourceForThreadLite
//  * @implemented NT4
//  *
//  *     The ExReleaseResourceForThreadLite routine routine releases
//  *     the input resource of the indicated thread.
//  *
//  * @param Resource
//  *        Pointer to the resource to be released.
//  *
//  * @param Thread
//  *        Identifies the thread that originally acquired the resource.
//   *
//  * @return None.
//  *
//  * @remarks Callers of ExReleaseResourceForThreadLite must be running at
//  *          IRQL <= DISPATCH_LEVEL.
//  *
//  *--*/
// VOID
// NTAPI
// ExReleaseResourceForThreadLite(IN PERESOURCE Resource,
//                                IN ERESOURCE_THREAD Thread)
// {
//     ULONG i;
//     ULONG Count;
//     KLOCK_QUEUE_HANDLE LockHandle;
//     POWNER_ENTRY Owner, Limit;
//     ASSERT(Thread != 0);
//
//     /* Get the thread and lock the resource */
//     ExAcquireResourceLock(Resource, &LockHandle);
//
//     /* Sanity checks */
//     ExpVerifyResource(Resource);
//     ExpCheckForApcsDisabled(LockHandle.OldIrql, Resource, ExpGetCurrentKThread());
//
//     /* Check if it's exclusively owned */
//     if (IsOwnedExclusive(Resource))
//     {
//         /* Decrement owner count and check if we're done */
//         ASSERT(Resource->OwnerEntry.OwnerThread == Thread);
//         if (--Resource->OwnerEntry.OwnerCount)
//         {
//             /* Done, release lock! */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return;
//         }
//
//         /* Clear the owner */
//         Resource->OwnerEntry.OwnerThread = 0;
//
//         /* Decrement the number of active entries */
//         ASSERT(Resource->ActiveEntries == 1);
//         Resource->ActiveEntries--;
//
//         /* Check if there are shared waiters */
//         if (IsSharedWaiting(Resource))
//         {
//             /* Remove the exclusive flag */
//             Resource->Flag &= ~ResourceOwnedExclusive;
//
//             /* Give ownage to another thread */
//             Count = Resource->NumberOfSharedWaiters;
//             Resource->ActiveEntries = Count;
//             Resource->NumberOfSharedWaiters = 0;
//
//             /* Release lock and let someone else have it */
//             ASSERT(Resource->ActiveCount == 1);
//             ExReleaseResourceLock(Resource, &LockHandle);
//             KeReleaseSemaphore(Resource->SharedWaiters, 0, Count, FALSE);
//             return;
//         }
//         else if (IsExclusiveWaiting(Resource))
//         {
//             /* Give exclusive access */
//             Resource->OwnerEntry.OwnerThread = 1;
//             Resource->OwnerEntry.OwnerCount = 1;
//             Resource->ActiveEntries = 1;
//             Resource->NumberOfExclusiveWaiters--;
//
//             /* Release the lock and give it away */
//             ASSERT(Resource->ActiveCount == 1);
//             ExReleaseResourceLock(Resource, &LockHandle);
//             KeSetEventBoostPriority(Resource->ExclusiveWaiters,
//                                     (PKTHREAD*)&Resource->OwnerEntry.OwnerThread);
//             return;
//         }
//
//         /* Remove the exclusive flag */
//         Resource->Flag &= ~ResourceOwnedExclusive;
//         Resource->ActiveCount = 0;
//     }
//     else
//     {
//         /* Check if we are in the thread list */
//         if (Resource->OwnerEntry.OwnerThread == Thread)
//         {
//             /* Found it, get owner */
//             Owner = &Resource->OwnerEntry;
//         }
//         else
//         {
//             /* Assume no valid index */
//             i = 1;
//
//             /* If we got a valid pointer, try to get the resource index */
//             if (!((ULONG)Thread & 3)) i = ((PKTHREAD)Thread)->ResourceIndex;
//
//             /* Do a table lookup */
//             Owner = Resource->OwnerTable;
//             ASSERT(Owner != NULL);
//
//             /* Check if we're out of the size and don't match */
//             if ((i >= Owner->TableSize) || (Owner[i].OwnerThread != Thread))
//             {
//                 /* Get the last entry */
//                 Limit = &Owner[Owner->TableSize];
//                 for (;;)
//                 {
//                     /* Move to the next entry */
//                     Owner++;
//
//                     /* Make sure we're not out of bounds */
//                     if (Owner >= Limit)
//                     {
//                         /* Bugcheck, nobody owns us */
//                         KeBugCheckEx(RESOURCE_NOT_OWNED,
//                                      (ULONG_PTR)Resource,
//                                      (ULONG_PTR)Thread,
//                                      (ULONG_PTR)Resource->OwnerTable,
//                                      (ULONG_PTR)3);
//                     }
//
//                     /* Check for a match */
//                     if (Owner->OwnerThread == Thread) break;
//                 }
//             }
//             else
//             {
//                 /* Get the entry directly */
//                 Owner = &Owner[i];
//             }
//         }
//
//         /* Sanity checks */
//         ASSERT(Owner->OwnerThread == Thread);
//         ASSERT(Owner->OwnerCount > 0);
//
//         /* Check if we are the last owner */
//         if (--Owner->OwnerCount)
//         {
//             /* There are other owners, release lock */
//             ExReleaseResourceLock(Resource, &LockHandle);
//             return;
//         }
//
//         /* Clear owner */
//         Owner->OwnerThread = 0;
//
//         /* See if the resource isn't being owned anymore */
//         ASSERT(Resource->ActiveEntries > 0);
//         if (!(--Resource->ActiveEntries))
//         {
//             /* Check if there's an exclusive waiter */
//             if (IsExclusiveWaiting(Resource))
//             {
//                 /* Give exclusive access */
//                 Resource->Flag |= ResourceOwnedExclusive;
//                 Resource->OwnerEntry.OwnerThread = 1;
//                 Resource->OwnerEntry.OwnerCount = 1;
//                 Resource->ActiveEntries = 1;
//                 Resource->NumberOfExclusiveWaiters--;
//
//                 /* Release the lock and give it away */
//                 ASSERT(Resource->ActiveCount == 1);
//                 ExReleaseResourceLock(Resource, &LockHandle);
//                 KeSetEventBoostPriority(Resource->ExclusiveWaiters,
//                                         (PKTHREAD*)&Resource->OwnerEntry.OwnerThread);
//                 return;
//             }
//
//             /* Clear the active count */
//             Resource->ActiveCount = 0;
//         }
//     }
//
//     /* Release lock */
//     ExReleaseResourceLock(Resource, &LockHandle);
// }
//
// /*++
//  * @name ExSetResourceOwnerPointer
//  * @implemented NT4
//  *
//  *     The ExSetResourceOwnerPointer routine routine sets the owner thread
//  *     thread pointer for an executive resource.
//  *
//  * @param Resource
//  *        Pointer to the resource whose owner to change.
//  *
//  * @param OwnerPointer
//  *        Pointer to an owner thread pointer of type ERESOURCE_THREAD.
//   *
//  * @return None.
//  *
//  * @remarks ExSetResourceOwnerPointer, used in conjunction with
//  *          ExReleaseResourceForThreadLite, provides a means for one thread
//  *          (acting as an resource manager thread) to acquire and release
//  *          resources for use by another thread (acting as a resource user
//  *          thread).
//  *
//  *          After calling ExSetResourceOwnerPointer for a specific resource,
//  *          the only other routine that can be called for that resource is
//  *          ExReleaseResourceForThreadLite.
//  *
//  *          Callers of ExSetResourceOwnerPointer must be running at
//  *          IRQL <= DISPATCH_LEVEL.
//  *
//  *--*/
// VOID
// NTAPI
// ExSetResourceOwnerPointer(IN PERESOURCE Resource,
//                           IN PVOID OwnerPointer)
// {
//     ERESOURCE_THREAD Thread;
//     KLOCK_QUEUE_HANDLE LockHandle;
//     POWNER_ENTRY Owner, ThisOwner;
//
//     /* Sanity check */
//     ASSERT((OwnerPointer != 0) && (((ULONG_PTR)OwnerPointer & 3) == 3));
//
//     /* Get the thread */
//     Thread = ExGetCurrentResourceThread();
//
//     /* Sanity check */
//     ExpVerifyResource(Resource);
//
//     /* Lock the resource */
//     ExAcquireResourceLock(Resource, &LockHandle);
//
//     /* Check if it's exclusive */
//     if (IsOwnedExclusive(Resource))
//     {
//         /* If it's exclusive, set the first entry no matter what */
//         ASSERT(Resource->OwnerEntry.OwnerThread == Thread);
//         ASSERT(Resource->OwnerEntry.OwnerCount > 0);
//         Resource->OwnerEntry.OwnerThread = (ULONG_PTR)OwnerPointer;
//     }
//     else
//     {
//         /* Set the thread in both entries */
//         ThisOwner = ExpFindEntryForThread(Resource,
//                                           (ERESOURCE_THREAD)OwnerPointer,
//                                           0,
//                                           FALSE);
//         Owner = ExpFindEntryForThread(Resource, Thread, 0, FALSE);
//         if (!Owner)
//         {
//             /* Nobody owns it, crash */
//             KeBugCheckEx(RESOURCE_NOT_OWNED,
//                          (ULONG_PTR)Resource,
//                          Thread,
//                          (ULONG_PTR)Resource->OwnerTable,
//                          3);
//         }
//
//         /* Set if we are the owner */
//         if (ThisOwner)
//         {
//             /* Update data */
//             ThisOwner->OwnerCount += Owner->OwnerCount;
//             Owner->OwnerCount = 0;
//             Owner->OwnerThread = 0;
//             ASSERT(Resource->ActiveEntries >= 2);
//             Resource->ActiveEntries--;
//         }
//         else
//         {
//             /* Update the owner entry instead */
//             Owner->OwnerThread = (ERESOURCE_THREAD)OwnerPointer;
//         }
//     }
//
//     /* Release the resource */
//     ExReleaseResourceLock(Resource, &LockHandle);
// }
//
// /*++
//  * @name ExTryToAcquireResourceExclusiveLite
//  * @implemented NT4
//  *
//  *     The ExTryToAcquireResourceExclusiveLite routine routine attempts to
//  *     acquire the given resource for exclusive access.
//  *
//  * @param Resource
//  *        Pointer to the resource to be acquired.
//  *
//  * @return TRUE if the given resource has been acquired for the caller.
//  *
//  * @remarks Callers of ExTryToAcquireResourceExclusiveLite must be running at
//  *          IRQL < DISPATCH_LEVEL.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// ExTryToAcquireResourceExclusiveLite(IN PERESOURCE Resource)
// {
//     ERESOURCE_THREAD Thread;
//     KLOCK_QUEUE_HANDLE LockHandle;
//     BOOLEAN Acquired = FALSE;
//
//     /* Sanity check */
//     ASSERT((Resource->Flag & ResourceNeverExclusive) == 0);
//
//     /* Get the thread */
//     Thread = ExGetCurrentResourceThread();
//
//     /* Sanity check and validation */
//     ASSERT(KeIsExecutingDpc() == FALSE);
//     ExpVerifyResource(Resource);
//
//     /* Acquire the lock */
//     ExAcquireResourceLock(Resource, &LockHandle);
//
//     /* Check if there is an owner */
//     if (!Resource->ActiveCount)
//     {
//         /* No owner, give exclusive access */
//         Resource->Flag |= ResourceOwnedExclusive;
//         Resource->OwnerEntry.OwnerThread = Thread;
//         Resource->OwnerEntry.OwnerCount = 1;
//         Resource->ActiveCount = 1;
//         Resource->ActiveEntries = 1;
//         Acquired = TRUE;
//     }
//     else if ((IsOwnedExclusive(Resource)) &&
//              (Resource->OwnerEntry.OwnerThread == Thread))
//     {
//         /* Do a recursive acquire */
//         Resource->OwnerEntry.OwnerCount++;
//         Acquired = TRUE;
//     }
//
//     /* Release the resource */
//     ExReleaseResourceLock(Resource, &LockHandle);
//     return Acquired;
// }
//
// /*++
//  * @name ExEnterCriticalRegionAndAcquireResourceExclusive
//  * @implemented NT5.1
//  *
//  *     The ExEnterCriticalRegionAndAcquireResourceExclusive enters a critical
//  *     region and then exclusively acquires a resource.
//  *
//  * @param Resource
//  *        Pointer to the resource to acquire.
//  *
//  * @return Pointer to the Win32K thread pointer of the current thread.
//  *
//  * @remarks See ExAcquireResourceExclusiveLite.
//  *
//  *--*/
// PVOID
// NTAPI
// ExEnterCriticalRegionAndAcquireResourceExclusive(IN PERESOURCE Resource)
// {
//     /* Enter critical region */
//     KeEnterCriticalRegion();
//
//     /* Acquire the resource */
//     NT_VERIFY(ExAcquireResourceExclusiveLite(Resource, TRUE));
//
//     /* Return the Win32 Thread */
//     return KeGetCurrentThread()->Win32Thread;
// }
//
// /*++
//  * @name ExEnterCriticalRegionAndAcquireResourceShared
//  * @implemented NT5.2
//  *
//  *     The ExEnterCriticalRegionAndAcquireResourceShared routine
//  *     enters a critical region and then acquires a resource shared.
//  *
//  * @param Resource
//  *        Pointer to the resource to acquire.
//  *
//  * @return Pointer to the Win32K thread pointer of the current thread.
//  *
//  * @remarks See ExAcquireResourceSharedLite.
//  *
//  *--*/
// PVOID
// NTAPI
// ExEnterCriticalRegionAndAcquireResourceShared(IN PERESOURCE Resource)
// {
//     /* Enter critical region */
//     KeEnterCriticalRegion();
//
//     /* Acquire the resource */
//     NT_VERIFY(ExAcquireResourceSharedLite(Resource, TRUE));
//
//     /* Return the Win32 Thread */
//     return KeGetCurrentThread()->Win32Thread;
// }
//
// /*++
//  * @name ExEnterCriticalRegionAndAcquireSharedWaitForExclusive
//  * @implemented NT5.2
//  *
//  *     The ExEnterCriticalRegionAndAcquireSharedWaitForExclusive routine
//  *     enters a critical region and then acquires a resource shared if
//  *     shared access can be granted and there are no exclusive waiters.
//  *     It then acquires the resource exclusively.
//  *
//  * @param Resource
//  *        Pointer to the resource to acquire.
//  *
//  * @return Pointer to the Win32K thread pointer of the current thread.
//  *
//  * @remarks See ExAcquireSharedWaitForExclusive.
//  *
//  *--*/
// PVOID
// NTAPI
// ExEnterCriticalRegionAndAcquireSharedWaitForExclusive(IN PERESOURCE Resource)
// {
//     /* Enter critical region */
//     KeEnterCriticalRegion();
//
//     /* Acquire the resource */
//     NT_VERIFY(ExAcquireSharedWaitForExclusive(Resource, TRUE));
//
//     /* Return the Win32 Thread */
//     return KeGetCurrentThread()->Win32Thread;
// }
//
// /*++
//  * @name ExReleaseResourceAndLeaveCriticalRegion
//  * @implemented NT5.1
//  *
//  *     The ExReleaseResourceAndLeaveCriticalRegion release a resource and
//  *     then leaves a critical region.
//  *
//  * @param Resource
//  *        Pointer to the resource to release.
//  *
//  * @return None
//  *
//  * @remarks See ExReleaseResourceLite.
//  *
//  *--*/
// VOID
// FASTCALL
// ExReleaseResourceAndLeaveCriticalRegion(IN PERESOURCE Resource)
// {
//     /* Release the resource */
//     ExReleaseResourceLite(Resource);
//
//     /* Leave critical region */
//     KeLeaveCriticalRegion();
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
