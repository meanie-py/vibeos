use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    // Handle table types
    EXHANDLE, PEXHANDLE,
    _HANDLE_TABLE, HANDLE_TABLE, PHANDLE_TABLE,
    _HANDLE_TABLE_ENTRY, PHANDLE_TABLE_ENTRY,
    HANDLE_TABLE_ENTRY_INFO,
    // HANDLE/PHANDLE (not in base.rs)
    HANDLE, PHANDLE,
    // Push lock types
    EX_PUSH_LOCK,
    EX_PUSH_LOCK_WAIT_BLOCK,
    // EPROCESS/PEPROCESS from bindgen (= *mut _EPROCESS, needed for ddk quota functions)
    _EPROCESS, PEPROCESS,
    // Callback types
    PEX_DUPLICATE_HANDLE_CALLBACK,
    PEX_CHANGE_HANDLE_CALLBACK,
    PEX_SWEEP_HANDLE_CALLBACK,
    PEX_ENUM_HANDLE_CALLBACK,
    // Push lock operations
    ExAcquirePushLockExclusive,
    ExReleasePushLockExclusive,
    ExAcquirePushLockShared,
    ExReleasePushLockShared,
    ExBlockPushLock,
    ExfUnblockPushLock,
    ExWaitOnPushLock,
    ExWaitForUnblockPushLock,
    // Critical region
    KeEnterCriticalRegion,
    KeLeaveCriticalRegion,
    // Process quota
    PsChargeProcessPagedPoolQuota,
    PsReturnProcessPagedPoolQuota,
};

// Pool tag for handle tables
const TAG_OBJECT_TABLE: ULONG = 0x6274624F;

// InterlockedOr — atomic bitwise OR, returns OLD value.
#[inline]
unsafe fn InterlockedOr(Target: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Target as *const AtomicI32);
    atomic.fetch_or(Value, Ordering::SeqCst)
}

// InterlockedCompareExchange — CAS on LONG, returns OLD value.
#[inline]
unsafe fn InterlockedCompareExchange(Destination: *mut LONG, Exchange: LONG, Comparand: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Destination as *const AtomicI32);
    match atomic.compare_exchange(Comparand, Exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

// InterlockedExchange — atomically exchange a LONG, returns OLD value.
#[inline]
unsafe fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Target as *const AtomicI32);
    atomic.swap(Value, Ordering::SeqCst)
}

// ExInitializePushLock takes PULONG_PTR (= *mut u64 on amd64, *mut u32 on i386)
#[inline]
unsafe fn ExInitializePushLockLocal(PushLock: *mut EX_PUSH_LOCK) {
    ntkrnl_sys::ddk::ExInitializePushLock(PushLock as *mut _);
}

// Accessor helpers for _EXHANDLE bitfields (via __bindgen_anon_2 = LowIndex/MidIndex/HighIndex/TagBits2)
// and __bindgen_anon_1 (= TagBits/Index/KernelFlag)
// Note: bindgen methods return/accept u64 (bindgen's ULONG_PTR), we use usize here (base.rs ULONG_PTR).
#[inline]
unsafe fn exhandle_tagbits(h: *const EXHANDLE) -> usize {
    (*h).__bindgen_anon_1.TagBits() as usize
}
#[inline]
unsafe fn exhandle_set_tagbits(h: *mut EXHANDLE, v: usize) {
    (*h).__bindgen_anon_1.set_TagBits(v as _);
}
#[inline]
unsafe fn exhandle_index(h: *const EXHANDLE) -> usize {
    (*h).__bindgen_anon_1.Index() as usize
}
#[inline]
unsafe fn exhandle_set_index(h: *mut EXHANDLE, v: usize) {
    (*h).__bindgen_anon_1.set_Index(v as _);
}
#[inline]
unsafe fn exhandle_lowindex(h: *const EXHANDLE) -> usize {
    (*h).__bindgen_anon_2.LowIndex() as usize
}
#[inline]
unsafe fn exhandle_midindex(h: *const EXHANDLE) -> usize {
    (*h).__bindgen_anon_2.MidIndex() as usize
}
#[inline]
unsafe fn exhandle_highindex(h: *const EXHANDLE) -> usize {
    (*h).__bindgen_anon_2.HighIndex() as usize
}

// Accessor helpers for _HANDLE_TABLE_ENTRY bitfields
// Note: Value is ULONG_PTR (u64 in bindgen), cast to/from usize at boundary.
#[inline]
unsafe fn hte_value(e: *const _HANDLE_TABLE_ENTRY) -> usize {
    (*e).__bindgen_anon_1.Value as usize
}
#[inline]
unsafe fn hte_set_value(e: *mut _HANDLE_TABLE_ENTRY, v: usize) {
    (*e).__bindgen_anon_1.Value = v as _;
}
#[inline]
unsafe fn hte_nextfree(e: *const _HANDLE_TABLE_ENTRY) -> LONG {
    (*e).__bindgen_anon_2.NextFreeTableEntry
}
#[inline]
unsafe fn hte_set_nextfree(e: *mut _HANDLE_TABLE_ENTRY, v: LONG) {
    (*e).__bindgen_anon_2.NextFreeTableEntry = v;
}

// Accessor helpers for _HANDLE_TABLE bitfields
#[inline]
unsafe fn ht_flags(t: *const _HANDLE_TABLE) -> ULONG {
    (*t).__bindgen_anon_1.Flags
}
#[inline]
unsafe fn ht_set_flags(t: *mut _HANDLE_TABLE, v: ULONG) {
    (*t).__bindgen_anon_1.Flags = v;
}
#[inline]
unsafe fn ht_strictfifo(t: *const _HANDLE_TABLE) -> UCHAR {
    (*t).__bindgen_anon_1.StrictFIFO()
}

#[no_mangle]
static mut HandleTableListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
static mut HandleTableListLock: EX_PUSH_LOCK = unsafe { core::mem::zeroed() };

const HANDLE_TAG_BITS: u32 = 2;
const LOW_LEVEL_ENTRIES: u32 = PAGE_SIZE as u32 / core::mem::size_of::<_HANDLE_TABLE_ENTRY>() as u32;
const MID_LEVEL_ENTRIES: u32 = PAGE_SIZE as u32 / core::mem::size_of::<PHANDLE_TABLE_ENTRY>() as u32;
const HIGH_LEVEL_ENTRIES: u32 = PAGE_SIZE as u32 / core::mem::size_of::<*mut PHANDLE_TABLE_ENTRY>() as u32;
const MAX_MID_INDEX: u32 = LOW_LEVEL_ENTRIES * MID_LEVEL_ENTRIES;
// ex.h has -1; free-list links are handle values with clear tag bits, so masking
// them is a no-op. nolint: shadowed-c-define
const FREE_HANDLE_MASK: u32 = !((1 << HANDLE_TAG_BITS) - 1);
const EXHANDLE_TABLE_ENTRY_LOCK_BIT: LONG_PTR = 1;

/// Handle value type — matches _EXHANDLE.Value (ULONG_PTR from bindgen).
/// On amd64/arm64 this is u64 (c_ulonglong), on i386 this is u32 (c_ulong).
#[cfg(target_pointer_width = "64")]
type HandleVal = u64;
#[cfg(target_pointer_width = "32")]
type HandleVal = u32;

fn SizeOfHandle(x: u32) -> usize {
    core::mem::size_of::<HANDLE>() * x as usize
}

fn INDEX_TO_HANDLE_VALUE(x: u32) -> u32 {
    x << HANDLE_TAG_BITS
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitializeHandleTables() {
    InitializeListHead(core::ptr::addr_of_mut!(HandleTableListHead));
    ExInitializePushLockLocal(core::ptr::addr_of_mut!(HandleTableListLock));
}

#[no_mangle]
pub unsafe extern "system" fn ExpLookupHandleTableEntry(
    HandleTable: PHANDLE_TABLE,
    mut Handle: EXHANDLE,
) -> PHANDLE_TABLE_ENTRY {
    let HandleTable = opaque_ptr(HandleTable);
    let mut TableLevel: ULONG;
    let mut TableBase: ULONG_PTR;
    let mut HandleArray: PHANDLE_TABLE_ENTRY;
    let mut Entry: PHANDLE_TABLE_ENTRY;
    let mut PointerArray: *mut PVOID;

    exhandle_set_tagbits(core::ptr::addr_of_mut!(Handle), 0);

    if Handle.Value >= (*HandleTable).NextHandleNeedingPool as _ {
        return core::ptr::null_mut();
    }

    TableBase = (*HandleTable).TableCode as usize;
    TableLevel = (TableBase & 3) as ULONG;
    TableBase &= !3;

    PointerArray = TableBase as *mut PVOID;
    HandleArray = TableBase as PHANDLE_TABLE_ENTRY;

    match TableLevel {
        2 => {
            PointerArray = *PointerArray.add(exhandle_highindex(core::ptr::addr_of!(Handle)) as usize) as *mut PVOID;
            HandleArray = *PointerArray.add(exhandle_midindex(core::ptr::addr_of!(Handle)) as usize) as PHANDLE_TABLE_ENTRY;
            Entry = HandleArray.add(exhandle_lowindex(core::ptr::addr_of!(Handle)) as usize);
        }
        1 => {
            HandleArray = *PointerArray.add(exhandle_midindex(core::ptr::addr_of!(Handle)) as usize) as PHANDLE_TABLE_ENTRY;
            Entry = HandleArray.add(exhandle_lowindex(core::ptr::addr_of!(Handle)) as usize);
        }
        0 => {
            Entry = HandleArray.add(exhandle_lowindex(core::ptr::addr_of!(Handle)) as usize);
        }
        _ => {
            Entry = core::ptr::null_mut();
        }
    }

    Entry
}

unsafe fn ExpAllocateTablePagedPool(
    Process: PEPROCESS,
    Size: SIZE_T,
) -> PVOID {
    let Buffer: PVOID;
    let Status: NTSTATUS;

    Buffer = ExAllocatePoolWithTag(PagedPool, Size, TAG_OBJECT_TABLE);
    if !Buffer.is_null() {
        core::ptr::write_bytes(Buffer as *mut u8, 0, Size);

        if !Process.is_null() {
            Status = PsChargeProcessPagedPoolQuota(Process, Size as _);
            if Status < 0 {
                ExFreePoolWithTag(Buffer, TAG_OBJECT_TABLE);
                return core::ptr::null_mut();
            }
        }
    }

    Buffer
}

unsafe fn ExpAllocateTablePagedPoolNoZero(
    Process: PEPROCESS,
    Size: SIZE_T,
) -> PVOID {
    let Buffer: PVOID;
    let Status: NTSTATUS;

    Buffer = ExAllocatePoolWithTag(PagedPool, Size, TAG_OBJECT_TABLE);
    if !Buffer.is_null() {
        if !Process.is_null() {
            Status = PsChargeProcessPagedPoolQuota(Process, Size as _);
            if Status < 0 {
                ExFreePoolWithTag(Buffer, TAG_OBJECT_TABLE);
                return core::ptr::null_mut();
            }
        }
    }

    Buffer
}

unsafe fn ExpFreeTablePagedPool(
    Process: PEPROCESS,
    Buffer: PVOID,
    Size: SIZE_T,
) {
    ExFreePoolWithTag(Buffer, TAG_OBJECT_TABLE);
    if !Process.is_null() {
        PsReturnProcessPagedPoolQuota(Process, Size as _);
    }
}

unsafe fn ExpFreeLowLevelTable(
    Process: PEPROCESS,
    TableEntry: PHANDLE_TABLE_ENTRY,
) {
    if !(*TableEntry.add(0)).__bindgen_anon_1.Object.is_null() {
        ExpFreeTablePagedPool(
            Process,
            (*TableEntry.add(0)).__bindgen_anon_1.Object,
            LOW_LEVEL_ENTRIES as usize * core::mem::size_of::<HANDLE_TABLE_ENTRY_INFO>(),
        );
    }

    ExpFreeTablePagedPool(Process, TableEntry as PVOID, PAGE_SIZE);
}

unsafe fn ExpFreeHandleTable(HandleTable: PHANDLE_TABLE) {
    let Process = (*HandleTable).QuotaProcess;
    let mut i: ULONG;
    let mut j: ULONG;
    let TableCode = (*HandleTable).TableCode as usize;
    let TableBase = TableCode & !3;
    let TableLevel = (TableCode & 3) as ULONG;
    let mut Level1: PHANDLE_TABLE_ENTRY;
    let mut Level2: *mut PHANDLE_TABLE_ENTRY;
    let mut Level3: *mut *mut PHANDLE_TABLE_ENTRY;

    if TableLevel == 0 {
        Level1 = TableBase as PVOID as PHANDLE_TABLE_ENTRY;
        ExpFreeLowLevelTable(Process, Level1);
    } else if TableLevel == 1 {
        Level2 = TableBase as PVOID as *mut PHANDLE_TABLE_ENTRY;

        i = 0;
        while i < MID_LEVEL_ENTRIES {
            if (*Level2.add(i as usize)).is_null() {
                break;
            }

            ExpFreeLowLevelTable(Process, *Level2.add(i as usize));
            i += 1;
        }

        ExpFreeTablePagedPool(Process, Level2 as PVOID, PAGE_SIZE);
    } else {
        Level3 = TableBase as PVOID as *mut *mut PHANDLE_TABLE_ENTRY;

        i = 0;
        while i < HIGH_LEVEL_ENTRIES {
            if (*Level3.add(i as usize)).is_null() {
                break;
            }

            j = 0;
            while j < MID_LEVEL_ENTRIES {
                if (*(*Level3.add(i as usize)).add(j as usize)).is_null() {
                    break;
                }

                ExpFreeLowLevelTable(Process, *(*Level3.add(i as usize)).add(j as usize));
                j += 1;
            }

            ExpFreeTablePagedPool(Process, *Level3.add(i as usize) as PVOID, PAGE_SIZE);
            i += 1;
        }

        ExpFreeTablePagedPool(
            Process,
            Level3 as PVOID,
            SizeOfHandle(HIGH_LEVEL_ENTRIES),
        );
    }

    ExFreePoolWithTag(HandleTable as PVOID, TAG_OBJECT_TABLE);
    if !Process.is_null() {
        PsReturnProcessPagedPoolQuota(Process, core::mem::size_of::<HANDLE_TABLE>() as _);
    }
}

unsafe fn ExpFreeHandleTableEntry(
    HandleTable: PHANDLE_TABLE,
    mut Handle: EXHANDLE,
    HandleTableEntry: PHANDLE_TABLE_ENTRY,
) {
    let mut OldValue: ULONG;
    let mut Free: *mut ULONG;
    let LockIndex: ULONG;

    InterlockedDecrement(core::ptr::addr_of_mut!((*HandleTable).HandleCount));

    exhandle_set_tagbits(core::ptr::addr_of_mut!(Handle), 0);

    if ht_strictfifo(HandleTable) == 0 {
        LockIndex = (exhandle_index(core::ptr::addr_of!(Handle)) % 4) as ULONG;

        Free = if (*HandleTable).HandleTableLock[LockIndex as usize].__bindgen_anon_1.__bindgen_anon_1.Locked() != 0 {
            core::ptr::addr_of_mut!((*HandleTable).FirstFree)
        } else {
            core::ptr::addr_of_mut!((*HandleTable).LastFree)
        };
    } else {
        Free = core::ptr::addr_of_mut!((*HandleTable).LastFree);
    }

    loop {
        OldValue = *Free;
        hte_set_nextfree(HandleTableEntry, OldValue as LONG);
        if InterlockedCompareExchange(Free as *mut LONG, Handle.AsULONG as LONG, OldValue as LONG) == OldValue as LONG {
            break;
        }
    }
}

unsafe fn ExpAllocateHandleTable(
    Process: PEPROCESS,
    NewTable: BOOLEAN,
) -> PHANDLE_TABLE {
    let HandleTable: PHANDLE_TABLE;
    let mut HandleTableTable: PHANDLE_TABLE_ENTRY;
    let mut HandleEntry: PHANDLE_TABLE_ENTRY;
    let mut i: ULONG;
    let Status: NTSTATUS;

    // DBG: before ExAllocatePoolWithTag for HANDLE_TABLE

    HandleTable = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<HANDLE_TABLE>(),
        TAG_OBJECT_TABLE,
    ) as PHANDLE_TABLE;
    if HandleTable.is_null() {
        return core::ptr::null_mut();
    }

    // DBG: after ExAllocatePoolWithTag returned non-null

    if !Process.is_null() {
        Status = PsChargeProcessPagedPoolQuota(Process, core::mem::size_of::<HANDLE_TABLE>() as _);
        if Status < 0 {
            ExFreePoolWithTag(HandleTable as PVOID, TAG_OBJECT_TABLE);
            return core::ptr::null_mut();
        }
    }

    // DBG: before write_bytes (zeroing HANDLE_TABLE — triggers demand-zero page fault)

    core::ptr::write_bytes(HandleTable as *mut u8, 0, core::mem::size_of::<HANDLE_TABLE>());

    // DBG: after write_bytes completed

    // DBG: before ExpAllocateTablePagedPoolNoZero

    HandleTableTable = ExpAllocateTablePagedPoolNoZero(Process, PAGE_SIZE) as PHANDLE_TABLE_ENTRY;
    if HandleTableTable.is_null() {
        ExFreePoolWithTag(HandleTable as PVOID, TAG_OBJECT_TABLE);

        if !Process.is_null() {
            PsReturnProcessPagedPoolQuota(Process, core::mem::size_of::<HANDLE_TABLE>() as _);
        }

        return core::ptr::null_mut();
    }

    // DBG: after ExpAllocateTablePagedPoolNoZero returned non-null

    (*HandleTable).TableCode = HandleTableTable as usize as _;

    HandleEntry = &mut (*HandleTableTable.add(0));
    hte_set_nextfree(HandleEntry, -2);
    hte_set_value(HandleEntry, 0);

    // DBG: before handle entry setup loop

    if NewTable != 0 {
        HandleEntry = HandleEntry.add(1);

        i = 1;
        while i < (LOW_LEVEL_ENTRIES - 1) {
            hte_set_value(HandleEntry, 0);
            hte_set_nextfree(HandleEntry, INDEX_TO_HANDLE_VALUE(i + 1) as LONG);

            HandleEntry = HandleEntry.add(1);
            i += 1;
        }

        hte_set_value(HandleEntry, 0);
        hte_set_nextfree(HandleEntry, 0);
        (*HandleTable).FirstFree = INDEX_TO_HANDLE_VALUE(1);
    }

    // DBG: after handle entry loop, before metadata setup

    (*HandleTable).NextHandleNeedingPool = INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES);

    (*HandleTable).QuotaProcess = Process;
    (*HandleTable).UniqueProcessId = (*(PsGetCurrentProcess() as *mut _EPROCESS)).UniqueProcessId;
    ht_set_flags(HandleTable, 0);

    i = 0;
    while i < 4 {
        ExInitializePushLockLocal(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[i as usize]));
        i += 1;
    }

    ExInitializePushLockLocal(core::ptr::addr_of_mut!((*HandleTable).HandleContentionEvent));

    // DBG: ExpAllocateHandleTable returning

    HandleTable
}

unsafe fn ExpAllocateLowLevelTable(
    HandleTable: PHANDLE_TABLE,
    DoInit: BOOLEAN,
) -> PHANDLE_TABLE_ENTRY {
    let mut i: ULONG;
    let Base: ULONG;
    let Low: PHANDLE_TABLE_ENTRY;
    let mut HandleEntry: PHANDLE_TABLE_ENTRY;

    Low = ExpAllocateTablePagedPoolNoZero(
        (*HandleTable).QuotaProcess,
        PAGE_SIZE,
    ) as PHANDLE_TABLE_ENTRY;
    if Low.is_null() {
        return core::ptr::null_mut();
    }

    HandleEntry = &mut (*Low.add(0));
    hte_set_nextfree(HandleEntry, -2);
    hte_set_value(HandleEntry, 0);

    if DoInit != 0 {
        HandleEntry = HandleEntry.add(1);
        Base = (*HandleTable).NextHandleNeedingPool + INDEX_TO_HANDLE_VALUE(2);

        i = Base;
        while i < Base + INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES - 2) {
            hte_set_nextfree(HandleEntry, i as LONG);
            hte_set_value(HandleEntry, 0);
            HandleEntry = HandleEntry.add(1);
            i += INDEX_TO_HANDLE_VALUE(1);
        }

        hte_set_nextfree(HandleEntry, 0);
        hte_set_value(HandleEntry, 0);
    }

    Low
}

unsafe fn ExpAllocateMidLevelTable(
    HandleTable: PHANDLE_TABLE,
    DoInit: BOOLEAN,
    LowTableEntry: *mut PHANDLE_TABLE_ENTRY,
) -> *mut PHANDLE_TABLE_ENTRY {
    let Mid: *mut PHANDLE_TABLE_ENTRY;
    let Low: PHANDLE_TABLE_ENTRY;

    Mid = ExpAllocateTablePagedPool((*HandleTable).QuotaProcess, PAGE_SIZE) as *mut PHANDLE_TABLE_ENTRY;
    if Mid.is_null() {
        return core::ptr::null_mut();
    }

    Low = ExpAllocateLowLevelTable(HandleTable, DoInit);
    if Low.is_null() {
        ExpFreeTablePagedPool((*HandleTable).QuotaProcess, Mid as PVOID, PAGE_SIZE);
        return core::ptr::null_mut();
    }

    *Mid.add(0) = Low;
    *LowTableEntry = Low;
    Mid
}

unsafe fn ExpAllocateHandleTableEntrySlow(
    HandleTable: PHANDLE_TABLE,
    DoInit: BOOLEAN,
) -> BOOLEAN {
    let mut i: ULONG;
    let mut j: ULONG;
    let mut Index: ULONG;
    let mut Low: PHANDLE_TABLE_ENTRY = core::ptr::null_mut();
    let mut Mid: *mut PHANDLE_TABLE_ENTRY;
    let mut High: *mut *mut PHANDLE_TABLE_ENTRY;
    let mut SecondLevel: *mut PHANDLE_TABLE_ENTRY;
    let mut ThirdLevel: *mut *mut PHANDLE_TABLE_ENTRY;
    let mut NewFree: ULONG;
    let mut FirstFree: ULONG;
    let mut Value: PVOID;
    let TableCode = (*HandleTable).TableCode as ULONG_PTR;
    let TableBase = TableCode & !3;
    let TableLevel = (TableCode & 3) as ULONG;

    if TableLevel == 0 {
        Mid = ExpAllocateMidLevelTable(HandleTable, DoInit, &mut Low);
        if Mid.is_null() {
            return 0u8;
        }

        *Mid.add(1) = *Mid.add(0);
        *Mid.add(0) = TableBase as PVOID as PHANDLE_TABLE_ENTRY;

        let new_base = (Mid as ULONG_PTR) | 1;
        Value = InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*HandleTable).TableCode) as *mut PVOID,
            new_base as PVOID,
        );
    } else if TableLevel == 1 {
        SecondLevel = TableBase as PVOID as *mut PHANDLE_TABLE_ENTRY;

        i = (*HandleTable).NextHandleNeedingPool / INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES);
        if i < MID_LEVEL_ENTRIES {
            Low = ExpAllocateLowLevelTable(HandleTable, DoInit);
            if Low.is_null() {
                return 0u8;
            }

            Value = InterlockedExchangePointer(
                SecondLevel.add(i as usize) as *mut PVOID,
                Low as PVOID,
            );
        } else {
            High = ExpAllocateTablePagedPool(
                (*HandleTable).QuotaProcess,
                SizeOfHandle(HIGH_LEVEL_ENTRIES),
            ) as *mut *mut PHANDLE_TABLE_ENTRY;
            if High.is_null() {
                return 0u8;
            }

            Mid = ExpAllocateMidLevelTable(HandleTable, DoInit, &mut Low);
            if Mid.is_null() {
                ExpFreeTablePagedPool(
                    (*HandleTable).QuotaProcess,
                    High as PVOID,
                    SizeOfHandle(HIGH_LEVEL_ENTRIES),
                );
                return 0u8;
            }

            *High.add(0) = TableBase as PVOID as *mut PHANDLE_TABLE_ENTRY;
            *High.add(1) = Mid;

            let new_base = (High as ULONG_PTR) | 2;
            Value = InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*HandleTable).TableCode) as *mut PVOID,
                new_base as PVOID,
            );
        }
    } else if TableLevel == 2 {
        ThirdLevel = TableBase as PVOID as *mut *mut PHANDLE_TABLE_ENTRY;

        i = (*HandleTable).NextHandleNeedingPool / INDEX_TO_HANDLE_VALUE(MAX_MID_INDEX);
        if i >= HIGH_LEVEL_ENTRIES {
            return 0u8;
        }

        if (*ThirdLevel.add(i as usize)).is_null() {
            Mid = ExpAllocateMidLevelTable(HandleTable, DoInit, &mut Low);
            if Mid.is_null() {
                return 0u8;
            }

            Value = InterlockedExchangePointer(
                ThirdLevel.add(i as usize) as *mut PVOID,
                Mid as PVOID,
            );
        } else {
            Index = ((*HandleTable).NextHandleNeedingPool / INDEX_TO_HANDLE_VALUE(1)) - i * MAX_MID_INDEX;
            j = Index / LOW_LEVEL_ENTRIES;

            Low = ExpAllocateLowLevelTable(HandleTable, DoInit);
            if Low.is_null() {
                return 0u8;
            }

            Value = InterlockedExchangePointer(
                (*ThirdLevel.add(i as usize)).add(j as usize) as *mut PVOID,
                Low as PVOID,
            );
        }
    } else {
        return 0u8;
    }

    Index = InterlockedExchangeAdd(
        core::ptr::addr_of_mut!((*HandleTable).NextHandleNeedingPool) as *mut LONG,
        INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES) as LONG,
    ) as ULONG;

    if DoInit != 0 {
        Index += INDEX_TO_HANDLE_VALUE(1);

        loop {
            FirstFree = (*HandleTable).FirstFree;
            hte_set_nextfree(Low.add((LOW_LEVEL_ENTRIES - 1) as usize), FirstFree as LONG);

            NewFree = InterlockedCompareExchange(
                core::ptr::addr_of_mut!((*HandleTable).FirstFree) as *mut LONG,
                Index as LONG,
                FirstFree as LONG,
            ) as ULONG;
            if NewFree == FirstFree {
                break;
            }
        }
    }

    1u8
}

unsafe fn ExpMoveFreeHandles(HandleTable: PHANDLE_TABLE) -> ULONG {
    let LastFree: ULONG;
    let mut i: ULONG;

    LastFree = InterlockedExchange(
        core::ptr::addr_of_mut!((*HandleTable).LastFree) as *mut LONG,
        0,
    ) as ULONG;

    if LastFree == 0 {
        return LastFree;
    }

    i = 1;
    while i < 4 {
        ExWaitOnPushLock(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[i as usize]));
        i += 1;
    }

    if ht_strictfifo(HandleTable) == 0 {
        if InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*HandleTable).FirstFree) as *mut LONG,
            LastFree as LONG,
            0,
        ) == 0 {
            return LastFree;
        }
    }

    LastFree
}

unsafe fn ExpAllocateHandleTableEntry(
    HandleTable: PHANDLE_TABLE,
    NewHandle: PEXHANDLE,
) -> PHANDLE_TABLE_ENTRY {
    let mut OldValue: ULONG;
    let mut NewValue: ULONG;
    let mut Entry: PHANDLE_TABLE_ENTRY;
    let mut Handle: EXHANDLE = core::mem::zeroed();
    let mut Result: BOOLEAN;

    loop {
        OldValue = (*HandleTable).FirstFree;
        while OldValue == 0 {
            KeEnterCriticalRegion();
            ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[0]));

            OldValue = (*HandleTable).FirstFree;
            if OldValue != 0 {
                ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[0]));
                KeLeaveCriticalRegion();
                break;
            }

            OldValue = ExpMoveFreeHandles(HandleTable);
            if OldValue != 0 {
                ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[0]));
                KeLeaveCriticalRegion();
                break;
            }

            Result = ExpAllocateHandleTableEntrySlow(HandleTable, 1u8);

            ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[0]));
            KeLeaveCriticalRegion();
            OldValue = (*HandleTable).FirstFree;

            if Result == 0 {
                if OldValue == 0 {
                    (*NewHandle).GenericHandleOverlay = core::ptr::null_mut();
                    return core::ptr::null_mut();
                }
            }
        }

        Handle.Value = (OldValue & FREE_HANDLE_MASK) as _;

        Entry = ExpLookupHandleTableEntry(HandleTable, Handle);

        let mut OldHandle: EXHANDLE = core::mem::zeroed();
        OldHandle.Value = OldValue as _;
        let i = (exhandle_index(core::ptr::addr_of!(OldHandle)) % 4) as ULONG;
        KeEnterCriticalRegion();
        ExAcquirePushLockShared(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[i as usize]));

        if OldValue != *(core::ptr::addr_of!((*HandleTable).FirstFree) as *const ULONG) {
            ExReleasePushLockShared(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[i as usize]));
            KeLeaveCriticalRegion();
            continue;
        }

        NewValue = hte_nextfree(Entry) as ULONG;
        let NewValue1 = InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*HandleTable).FirstFree) as *mut LONG,
            NewValue as LONG,
            OldValue as LONG,
        ) as ULONG;

        ExReleasePushLockShared(core::ptr::addr_of_mut!((*HandleTable).HandleTableLock[i as usize]));
        KeLeaveCriticalRegion();

        if NewValue1 == OldValue {
            break;
        }
    }

    InterlockedIncrement(core::ptr::addr_of_mut!((*HandleTable).HandleCount));

    *NewHandle = Handle;
    Entry
}

#[no_mangle]
pub unsafe extern "system" fn ExCreateHandleTable(Process: PEPROCESS) -> PHANDLE_TABLE {
    let Process = opaque_ptr(Process);
    let HandleTable: PHANDLE_TABLE;

    HandleTable = ExpAllocateHandleTable(Process, 1u8);
    if HandleTable.is_null() {
        return core::ptr::null_mut();
    }

    // DBG: ExCreateHandleTable after ExpAllocateHandleTable

    KeEnterCriticalRegion();

    // DBG: after KeEnterCriticalRegion

    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(HandleTableListLock));

    // DBG: after ExAcquirePushLockExclusive

    InsertTailList(
        core::ptr::addr_of_mut!(HandleTableListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*HandleTable).HandleTableList) as PLIST_ENTRY,
    );

    // DBG: after InsertTailList

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!(HandleTableListLock));
    KeLeaveCriticalRegion();

    // DBG: ExCreateHandleTable returning

    HandleTable
}

#[no_mangle]
pub unsafe extern "system" fn ExCreateHandle(
    HandleTable: PHANDLE_TABLE,
    HandleTableEntry: PHANDLE_TABLE_ENTRY,
) -> HANDLE {
    let HandleTable = opaque_ptr(HandleTable);
    let HandleTableEntry = opaque_ptr(HandleTableEntry);
    let mut Handle: EXHANDLE = core::mem::zeroed();
    let NewEntry: PHANDLE_TABLE_ENTRY;

    Handle.GenericHandleOverlay = core::ptr::null_mut();

    NewEntry = ExpAllocateHandleTableEntry(HandleTable, &mut Handle);
    if !NewEntry.is_null() {
        KeEnterCriticalRegion();

        *NewEntry = *HandleTableEntry;

        ExUnlockHandleTableEntry(HandleTable, NewEntry);
        KeLeaveCriticalRegion();
    }

    Handle.GenericHandleOverlay
}

unsafe fn ExpBlockOnLockedHandleEntry(
    HandleTable: PHANDLE_TABLE,
    HandleTableEntry: PHANDLE_TABLE_ENTRY,
) {
    let OldValue: LONG_PTR;
    // 16-aligned wrapper: bindgen dropped C's __ALIGNED(16) and the pushlock
    // word encodes flag bits in the block pointer's low nibble (see
    // ex/pushlock.rs AlignedPlWaitBlock).
    let mut WaitBlock: crate::pushlock::AlignedPlWaitBlock =
        crate::pushlock::AlignedPlWaitBlock(core::mem::zeroed());

    ExBlockPushLock(
        core::ptr::addr_of_mut!((*HandleTable).HandleContentionEvent),
        core::ptr::addr_of_mut!(WaitBlock.0) as PVOID,
    );

    OldValue = hte_value(HandleTableEntry) as LONG_PTR;
    if OldValue == 0 || (OldValue & EXHANDLE_TABLE_ENTRY_LOCK_BIT) != 0 {
        ExfUnblockPushLock(
            core::ptr::addr_of_mut!((*HandleTable).HandleContentionEvent),
            core::ptr::addr_of_mut!(WaitBlock.0) as PVOID,
        );
    } else {
        ExWaitForUnblockPushLock(
            core::ptr::addr_of_mut!((*HandleTable).HandleContentionEvent),
            core::ptr::addr_of_mut!(WaitBlock.0) as PVOID,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExpLockHandleTableEntry(
    HandleTable: PHANDLE_TABLE,
    HandleTableEntry: PHANDLE_TABLE_ENTRY,
) -> BOOLEAN {
    let HandleTable = opaque_ptr(HandleTable);
    let HandleTableEntry = opaque_ptr(HandleTableEntry);
    let mut NewValue: LONG_PTR;
    let mut OldValue: LONG_PTR;

    loop {
        OldValue = *(core::ptr::addr_of!((*HandleTableEntry).__bindgen_anon_1.Object) as *const LONG_PTR);
        if (OldValue & EXHANDLE_TABLE_ENTRY_LOCK_BIT) != 0 {
            NewValue = OldValue & !EXHANDLE_TABLE_ENTRY_LOCK_BIT;
            if InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*HandleTableEntry).__bindgen_anon_1.Object),
                NewValue as PVOID,
                OldValue as PVOID,
            ) == OldValue as PVOID {
                return 1u8;
            }
        } else {
            if OldValue == 0 {
                return 0u8;
            }
        }

        ExpBlockOnLockedHandleEntry(HandleTable, HandleTableEntry);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExUnlockHandleTableEntry(
    HandleTable: PHANDLE_TABLE,
    HandleTableEntry: PHANDLE_TABLE_ENTRY,
) {
    let HandleTable = opaque_ptr(HandleTable);
    let HandleTableEntry = opaque_ptr(HandleTableEntry);
    let OldValue: LONG_PTR;

    OldValue = InterlockedOr(
        core::ptr::addr_of_mut!((*HandleTableEntry).__bindgen_anon_1.Value) as *mut LONG,
        EXHANDLE_TABLE_ENTRY_LOCK_BIT as LONG,
    ) as LONG_PTR;

    ExfUnblockPushLock(
        core::ptr::addr_of_mut!((*HandleTable).HandleContentionEvent),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExRemoveHandleTable(HandleTable: PHANDLE_TABLE) {
    let HandleTable = opaque_ptr(HandleTable);
    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(HandleTableListLock));

    RemoveEntryList(core::ptr::addr_of_mut!((*HandleTable).HandleTableList) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!((*HandleTable).HandleTableList));

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!(HandleTableListLock));
    KeLeaveCriticalRegion();
}

#[no_mangle]
pub unsafe extern "system" fn ExDestroyHandleTable(
    HandleTable: PHANDLE_TABLE,
    DestroyHandleProcedure: PVOID,
) {
    let HandleTable = opaque_ptr(HandleTable);
    let DestroyHandleProcedure = opaque_ptr(DestroyHandleProcedure);
    ExRemoveHandleTable(HandleTable);

    if !DestroyHandleProcedure.is_null() {
    }

    ExpFreeHandleTable(HandleTable);
}

#[no_mangle]
pub unsafe extern "system" fn ExDestroyHandle(
    HandleTable: PHANDLE_TABLE,
    Handle: HANDLE,
    HandleTableEntry: PHANDLE_TABLE_ENTRY,
) -> BOOLEAN {
    let HandleTable = opaque_ptr(HandleTable);
    let Handle = opaque_ptr(Handle);
    let HandleTableEntry = opaque_ptr(HandleTableEntry);
    let mut ExHandle: EXHANDLE = core::mem::zeroed();
    let Object: PVOID;
    let mut entry = HandleTableEntry;

    ExHandle.GenericHandleOverlay = Handle;

    KeEnterCriticalRegion();
    if entry.is_null() {
        entry = ExpLookupHandleTableEntry(HandleTable, ExHandle);

        if entry.is_null() ||
            (*entry).__bindgen_anon_1.Object.is_null() ||
            hte_nextfree(entry) == -2 {
            KeLeaveCriticalRegion();
            return 0u8;
        }

        if ExpLockHandleTableEntry(HandleTable, entry) == 0 {
            KeLeaveCriticalRegion();
            return 0u8;
        }
    }

    Object = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*entry).__bindgen_anon_1.Object),
        core::ptr::null_mut(),
    );

    ExfUnblockPushLock(
        core::ptr::addr_of_mut!((*HandleTable).HandleContentionEvent),
        core::ptr::null_mut(),
    );

    ExpFreeHandleTableEntry(HandleTable, ExHandle, entry);

    KeLeaveCriticalRegion();
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ExMapHandleToPointer(
    HandleTable: PHANDLE_TABLE,
    Handle: HANDLE,
) -> PHANDLE_TABLE_ENTRY {
    let HandleTable = opaque_ptr(HandleTable);
    let Handle = opaque_ptr(Handle);
    let mut ExHandle: EXHANDLE = core::mem::zeroed();
    let HandleTableEntry: PHANDLE_TABLE_ENTRY;

    ExHandle.GenericHandleOverlay = Handle;

    if (exhandle_index(core::ptr::addr_of!(ExHandle)) & (LOW_LEVEL_ENTRIES as usize - 1)) == 0 {
        return core::ptr::null_mut();
    }

    HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, ExHandle);
    if HandleTableEntry.is_null() {
        return core::ptr::null_mut();
    }

    if ExpLockHandleTableEntry(HandleTable, HandleTableEntry) == 0 {
        return core::ptr::null_mut();
    }

    HandleTableEntry
}

#[no_mangle]
pub unsafe extern "system" fn ExDupHandleTable(
    Process: PEPROCESS,
    HandleTable: PHANDLE_TABLE,
    DupHandleProcedure: PEX_DUPLICATE_HANDLE_CALLBACK,
    Mask: ULONG_PTR,
) -> PHANDLE_TABLE {
    let Process = opaque_ptr(Process);
    let HandleTable = opaque_ptr(HandleTable);
    let NewTable: PHANDLE_TABLE;
    let mut Handle: EXHANDLE = core::mem::zeroed();
    let mut HandleTableEntry: PHANDLE_TABLE_ENTRY;
    let mut NewEntry: PHANDLE_TABLE_ENTRY;
    let mut Failed: BOOLEAN;

    NewTable = ExpAllocateHandleTable(Process, 0u8);
    if NewTable.is_null() {
        return core::ptr::null_mut();
    }

    while (*NewTable).NextHandleNeedingPool < (*HandleTable).NextHandleNeedingPool {
        if ExpAllocateHandleTableEntrySlow(NewTable, 0u8) == 0 {
            ExpFreeHandleTable(NewTable);
            return core::ptr::null_mut();
        }
    }

    (*NewTable).HandleCount = 0;
    (*NewTable).ExtraInfoPages = 0;
    (*NewTable).FirstFree = 0;

    Handle.Value = INDEX_TO_HANDLE_VALUE(1) as _;

    KeEnterCriticalRegion();
    loop {
        NewEntry = ExpLookupHandleTableEntry(NewTable, Handle);
        if NewEntry.is_null() {
            break;
        }

        HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, Handle);

        loop {
            if (hte_value(HandleTableEntry) as isize & Mask as isize) == 0 {
                Failed = 1u8;
            } else {
                if ExpLockHandleTableEntry(HandleTable, HandleTableEntry) == 0 {
                    Failed = 1u8;
                } else {
                    *NewEntry = *HandleTableEntry;

                    if (DupHandleProcedure.unwrap())(
                        Process,
                        HandleTable,
                        HandleTableEntry,
                        NewEntry,
                    ) != 0 {
                        Failed = 0u8;

                        // Set lock bit in Value
                        let v = hte_value(NewEntry);
                        hte_set_value(NewEntry, v | EXHANDLE_TABLE_ENTRY_LOCK_BIT as usize);
                        (*NewTable).HandleCount += 1;
                    } else {
                        Failed = 1u8;
                    }
                }
            }

            if Failed != 0 {
                (*NewEntry).__bindgen_anon_1.Object = core::ptr::null_mut();
                hte_set_nextfree(NewEntry, (*NewTable).FirstFree as LONG);
                (*NewTable).FirstFree = Handle.Value as ULONG;
            }

            Handle.Value += INDEX_TO_HANDLE_VALUE(1) as HandleVal;
            NewEntry = NewEntry.add(1);
            HandleTableEntry = HandleTableEntry.add(1);

            if (Handle.Value % INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES) as HandleVal) == 0 {
                break;
            }
        }

        Handle.Value += INDEX_TO_HANDLE_VALUE(1) as HandleVal;
    }

    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(HandleTableListLock));
    InsertTailList(
        core::ptr::addr_of_mut!(HandleTableListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*NewTable).HandleTableList) as PLIST_ENTRY,
    );
    ExReleasePushLockExclusive(core::ptr::addr_of_mut!(HandleTableListLock));

    KeLeaveCriticalRegion();
    NewTable
}

#[no_mangle]
pub unsafe extern "system" fn ExChangeHandle(
    HandleTable: PHANDLE_TABLE,
    Handle: HANDLE,
    ChangeRoutine: PEX_CHANGE_HANDLE_CALLBACK,
    Context: ULONG_PTR,
) -> BOOLEAN {
    let HandleTable = opaque_ptr(HandleTable);
    let Handle = opaque_ptr(Handle);
    let mut ExHandle: EXHANDLE = core::mem::zeroed();
    let HandleTableEntry: PHANDLE_TABLE_ENTRY;
    let mut Result: BOOLEAN = 0u8;

    ExHandle.GenericHandleOverlay = Handle;

    HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, ExHandle);

    if HandleTableEntry.is_null() ||
        (*HandleTableEntry).__bindgen_anon_1.Object.is_null() ||
        hte_nextfree(HandleTableEntry) == -2 {
        return 0u8;
    }

    KeEnterCriticalRegion();

    if ExpLockHandleTableEntry(HandleTable, HandleTableEntry) != 0 {
        Result = (ChangeRoutine.unwrap())(HandleTableEntry, Context as _);
        ExUnlockHandleTableEntry(HandleTable, HandleTableEntry);
    }

    KeLeaveCriticalRegion();
    Result
}

#[no_mangle]
pub unsafe extern "system" fn ExSweepHandleTable(
    HandleTable: PHANDLE_TABLE,
    EnumHandleProcedure: PEX_SWEEP_HANDLE_CALLBACK,
    Context: PVOID,
) {
    let HandleTable = opaque_ptr(HandleTable);
    let Context = opaque_ptr(Context);
    let mut Handle: EXHANDLE = core::mem::zeroed();
    let mut HandleTableEntry: PHANDLE_TABLE_ENTRY;

    Handle.Value = INDEX_TO_HANDLE_VALUE(1) as _;
    loop {
        HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, Handle);
        if HandleTableEntry.is_null() {
            break;
        }

        loop {
            if ExpLockHandleTableEntry(HandleTable, HandleTableEntry) != 0 {
                (EnumHandleProcedure.unwrap())(
                    HandleTableEntry,
                    Handle.GenericHandleOverlay,
                    Context,
                );
            }

            Handle.Value += INDEX_TO_HANDLE_VALUE(1) as HandleVal;
            HandleTableEntry = HandleTableEntry.add(1);

            if (Handle.Value % INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES) as HandleVal) == 0 {
                break;
            }
        }

        Handle.Value += INDEX_TO_HANDLE_VALUE(1) as HandleVal;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExEnumHandleTable(
    HandleTable: PHANDLE_TABLE,
    EnumHandleProcedure: PEX_ENUM_HANDLE_CALLBACK,
    Context: PVOID,
    EnumHandle: PHANDLE,
) -> BOOLEAN {
    let HandleTable = opaque_ptr(HandleTable);
    let Context = opaque_ptr(Context);
    let EnumHandle = opaque_ptr(EnumHandle);
    let mut Handle: EXHANDLE = core::mem::zeroed();
    let mut HandleTableEntry: PHANDLE_TABLE_ENTRY;
    let mut Result: BOOLEAN = 0u8;

    KeEnterCriticalRegion();

    Handle.Value = 0;
    loop {
        HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, Handle);
        if HandleTableEntry.is_null() {
            break;
        }

        if !(*HandleTableEntry).__bindgen_anon_1.Object.is_null() &&
            hte_nextfree(HandleTableEntry) != -2 {
            if ExpLockHandleTableEntry(HandleTable, HandleTableEntry) != 0 {
                Result = (EnumHandleProcedure.unwrap())(
                    HandleTableEntry,
                    Handle.GenericHandleOverlay,
                    Context,
                );

                ExUnlockHandleTableEntry(HandleTable, HandleTableEntry);

                if Result != 0 {
                    if !EnumHandle.is_null() {
                        *EnumHandle = Handle.GenericHandleOverlay;
                    }
                    break;
                }
            }
        }

        Handle.Value += INDEX_TO_HANDLE_VALUE(1) as HandleVal;
    }

    KeLeaveCriticalRegion();
    Result
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/handle.c
//  * PURPOSE:         Generic Executive Handle Tables
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Thomas Weidenmueller <w3seek@reactos.com>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// LIST_ENTRY HandleTableListHead;
// EX_PUSH_LOCK HandleTableListLock;
// #define SizeOfHandle(x) (sizeof(HANDLE) * (x))
// #define INDEX_TO_HANDLE_VALUE(x) ((x) << HANDLE_TAG_BITS)
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// #ifdef _WIN64
// #define strtoulptr strtoull
// #else
// #define strtoulptr strtoul
// #endif
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpInitializeHandleTables(VOID)
// {
//     /* Initialize the list of handle tables and the lock */
//     InitializeListHead(&HandleTableListHead);
//     ExInitializePushLock(&HandleTableListLock);
// }
//
// PHANDLE_TABLE_ENTRY
// NTAPI
// ExpLookupHandleTableEntry(IN PHANDLE_TABLE HandleTable,
//                           IN EXHANDLE Handle)
// {
//     ULONG TableLevel;
//     ULONG_PTR TableBase;
//     PHANDLE_TABLE_ENTRY HandleArray, Entry;
//     PVOID *PointerArray;
//
//     /* Clear the tag bits */
//     Handle.TagBits = 0;
//
//     /* Check if the handle is in the allocated range */
//     if (Handle.Value >= HandleTable->NextHandleNeedingPool)
//     {
//         return NULL;
//     }
//
//     /* Get the table code */
//     TableBase = HandleTable->TableCode;
//
//     /* Extract the table level and actual table base */
//     TableLevel = (ULONG)(TableBase & 3);
//     TableBase &= ~3;
//
//     PointerArray = (PVOID*)TableBase;
//     HandleArray = (PHANDLE_TABLE_ENTRY)TableBase;
//
//     /* Check what level we're running at */
//     switch (TableLevel)
//     {
//         case 2:
//
//             /* Get the mid level pointer array */
//             PointerArray = PointerArray[Handle.HighIndex];
//             ASSERT(PointerArray != NULL);
//
//             /* Fall through */
//         case 1:
//
//             /* Get the handle array */
//             HandleArray = PointerArray[Handle.MidIndex];
//             ASSERT(HandleArray != NULL);
//
//             /* Fall through */
//         case 0:
//
//             /* Get the entry using the low index */
//             Entry = &HandleArray[Handle.LowIndex];
//
//             /* All done */
//             break;
//
//         default:
//
//             ASSERT(FALSE);
//             Entry = NULL;
//     }
//
//     /* Return the handle entry */
//     return Entry;
// }
//
// PVOID
// NTAPI
// ExpAllocateTablePagedPool(IN PEPROCESS Process OPTIONAL,
//                           IN SIZE_T Size)
// {
//     PVOID Buffer;
//     NTSTATUS Status;
//
//     /* Do the allocation */
//     Buffer = ExAllocatePoolWithTag(PagedPool, Size, TAG_OBJECT_TABLE);
//     if (Buffer)
//     {
//         /* Clear the memory */
//         RtlZeroMemory(Buffer, Size);
//
//         /* Check if we have a process to charge quota */
//         if (Process)
//         {
//             /* Charge quota */
//             Status = PsChargeProcessPagedPoolQuota(Process, Size);
//             if (!NT_SUCCESS(Status))
//             {
//                 ExFreePoolWithTag(Buffer, TAG_OBJECT_TABLE);
//                 return NULL;
//             }
//         }
//     }
//
//     /* Return the allocated memory */
//     return Buffer;
// }
//
// PVOID
// NTAPI
// ExpAllocateTablePagedPoolNoZero(IN PEPROCESS Process OPTIONAL,
//                                 IN SIZE_T Size)
// {
//     PVOID Buffer;
//     NTSTATUS Status;
//
//     /* Do the allocation */
//     Buffer = ExAllocatePoolWithTag(PagedPool, Size, TAG_OBJECT_TABLE);
//     if (Buffer)
//     {
//         /* Check if we have a process to charge quota */
//         if (Process)
//         {
//             /* Charge quota */
//             Status = PsChargeProcessPagedPoolQuota(Process, Size);
//             if (!NT_SUCCESS(Status))
//             {
//                 ExFreePoolWithTag(Buffer, TAG_OBJECT_TABLE);
//                 return NULL;
//             }
//         }
//     }
//
//     /* Return the allocated memory */
//     return Buffer;
// }
//
// VOID
// NTAPI
// ExpFreeTablePagedPool(IN PEPROCESS Process OPTIONAL,
//                       IN PVOID Buffer,
//                       IN SIZE_T Size)
// {
//     /* Free the buffer */
//     ExFreePoolWithTag(Buffer, TAG_OBJECT_TABLE);
//     if (Process)
//     {
//         /* Release quota */
//         PsReturnProcessPagedPoolQuota(Process, Size);
//     }
// }
//
// VOID
// NTAPI
// ExpFreeLowLevelTable(IN PEPROCESS Process,
//                      IN PHANDLE_TABLE_ENTRY TableEntry)
// {
//     /* Check if we have an entry */
//     if (TableEntry[0].Object)
//     {
//         /* Free the entry */
//         ExpFreeTablePagedPool(Process,
//                               TableEntry[0].Object,
//                               LOW_LEVEL_ENTRIES *
//                               sizeof(HANDLE_TABLE_ENTRY_INFO));
//     }
//
//     /* Free the table */
//     ExpFreeTablePagedPool(Process, TableEntry, PAGE_SIZE);
// }
//
// VOID
// NTAPI
// ExpFreeHandleTable(IN PHANDLE_TABLE HandleTable)
// {
//     PEPROCESS Process = HandleTable->QuotaProcess;
//     ULONG i, j;
//     ULONG_PTR TableCode = HandleTable->TableCode;
//     ULONG_PTR TableBase = TableCode & ~3;
//     ULONG TableLevel = (ULONG)(TableCode & 3);
//     PHANDLE_TABLE_ENTRY Level1, *Level2, **Level3;
//     PAGED_CODE();
//
//     /* Check which level we're at */
//     if (TableLevel == 0)
//     {
//         /* Select the first level table base and just free it */
//         Level1 = (PVOID)TableBase;
//         ExpFreeLowLevelTable(Process, Level1);
//     }
//     else if (TableLevel == 1)
//     {
//         /* Select the second level table base */
//         Level2 = (PVOID)TableBase;
//
//         /* Loop each mid level entry */
//         for (i = 0; i < MID_LEVEL_ENTRIES; i++)
//         {
//             /* Leave if we've reached the last entry */
//             if (!Level2[i]) break;
//
//             /* Free the second level table */
//             ExpFreeLowLevelTable(Process, Level2[i]);
//         }
//
//         /* Free the second level table */
//         ExpFreeTablePagedPool(Process, Level2, PAGE_SIZE);
//     }
//     else
//     {
//         /* Select the third level table base */
//         Level3 = (PVOID)TableBase;
//
//         /* Loop each high level entry */
//         for (i = 0; i < HIGH_LEVEL_ENTRIES; i++)
//         {
//             /* Leave if we've reached the last entry */
//             if (!Level3[i]) break;
//
//             /* Loop each mid level entry */
//             for (j = 0; j < MID_LEVEL_ENTRIES; j++)
//             {
//                 /* Leave if we've reached the last entry */
//                 if (!Level3[i][j]) break;
//
//                 /* Free the second level table */
//                 ExpFreeLowLevelTable(Process, Level3[i][j]);
//             }
//
//             /* Free the third level table entry */
//             ExpFreeTablePagedPool(Process, Level3[i], PAGE_SIZE);
//         }
//
//         /* Free the third level table */
//         ExpFreeTablePagedPool(Process,
//                               Level3,
//                               SizeOfHandle(HIGH_LEVEL_ENTRIES));
//     }
//
//     /* Free the actual table and check if we need to release quota */
//     ExFreePoolWithTag(HandleTable, TAG_OBJECT_TABLE);
//     if (Process)
//     {
//         /* Release the quota it was taking up */
//         PsReturnProcessPagedPoolQuota(Process, sizeof(HANDLE_TABLE));
//     }
// }
//
// VOID
// NTAPI
// ExpFreeHandleTableEntry(IN PHANDLE_TABLE HandleTable,
//                         IN EXHANDLE Handle,
//                         IN PHANDLE_TABLE_ENTRY HandleTableEntry)
// {
//     ULONG OldValue, *Free;
//     ULONG LockIndex;
//     PAGED_CODE();
//
//     /* Sanity checks */
//     ASSERT(HandleTableEntry->Object == NULL);
//     ASSERT(HandleTableEntry == ExpLookupHandleTableEntry(HandleTable, Handle));
//
//     /* Decrement the handle count */
//     InterlockedDecrement(&HandleTable->HandleCount);
//
//     /* Mark the handle as free */
//     Handle.TagBits = 0;
//
//     /* Check if we're FIFO */
//     if (!HandleTable->StrictFIFO)
//     {
//         /* Select a lock index */
//         LockIndex = Handle.Index % 4;
//
//         /* Select which entry to use */
//         Free = (HandleTable->HandleTableLock[LockIndex].Locked) ?
//                 &HandleTable->FirstFree : &HandleTable->LastFree;
//     }
//     else
//     {
//         /* No need to worry about locking, take the last entry */
//         Free = &HandleTable->LastFree;
//     }
//
//     /* Start value change loop */
//     for (;;)
//     {
//         /* Get the current value and write */
//         OldValue = *Free;
//         HandleTableEntry->NextFreeTableEntry = OldValue;
//         if (InterlockedCompareExchange((PLONG)Free, Handle.AsULONG, OldValue) == OldValue)
//         {
//             /* Break out, we're done. Make sure the handle value makes sense */
//             ASSERT((OldValue & FREE_HANDLE_MASK) <
//                    HandleTable->NextHandleNeedingPool);
//             break;
//         }
//     }
// }
//
// PHANDLE_TABLE
// NTAPI
// ExpAllocateHandleTable(IN PEPROCESS Process OPTIONAL,
//                        IN BOOLEAN NewTable)
// {
//     PHANDLE_TABLE HandleTable;
//     PHANDLE_TABLE_ENTRY HandleTableTable, HandleEntry;
//     ULONG i;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Allocate the table */
//     HandleTable = ExAllocatePoolWithTag(PagedPool,
//                                         sizeof(HANDLE_TABLE),
//                                         TAG_OBJECT_TABLE);
//     if (!HandleTable) return NULL;
//
//     /* Check if we have a process */
//     if (Process)
//     {
//         /* Charge quota */
//         Status = PsChargeProcessPagedPoolQuota(Process, sizeof(HANDLE_TABLE));
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePoolWithTag(HandleTable, TAG_OBJECT_TABLE);
//             return NULL;
//         }
//     }
//
//     /* Clear the table */
//     RtlZeroMemory(HandleTable, sizeof(HANDLE_TABLE));
//
//     /* Now allocate the first level structures */
//     HandleTableTable = ExpAllocateTablePagedPoolNoZero(Process, PAGE_SIZE);
//     if (!HandleTableTable)
//     {
//         /* Failed, free the table */
//         ExFreePoolWithTag(HandleTable, TAG_OBJECT_TABLE);
//
//         /* Return the quota it was taking up */
//         if (Process)
//         {
//             PsReturnProcessPagedPoolQuota(Process, sizeof(HANDLE_TABLE));
//         }
//
//         return NULL;
//     }
//
//     /* Write the pointer to our first level structures */
//     HandleTable->TableCode = (ULONG_PTR)HandleTableTable;
//
//     /* Initialize the first entry */
//     HandleEntry = &HandleTableTable[0];
//     HandleEntry->NextFreeTableEntry = -2;
//     HandleEntry->Value = 0;
//
//     /* Check if this is a new table */
//     if (NewTable)
//     {
//         /* Go past the root entry */
//         HandleEntry++;
//
//         /* Loop every low level entry */
//         for (i = 1; i < (LOW_LEVEL_ENTRIES - 1); i++)
//         {
//             /* Set up the free data */
//             HandleEntry->Value = 0;
//             HandleEntry->NextFreeTableEntry = INDEX_TO_HANDLE_VALUE(i + 1);
//
//             /* Move to the next entry */
//             HandleEntry++;
//         }
//
//         /* Terminate the last entry */
//         HandleEntry->Value = 0;
//         HandleEntry->NextFreeTableEntry = 0;
//         HandleTable->FirstFree = INDEX_TO_HANDLE_VALUE(1);
//     }
//
//     /* Set the next handle needing pool after our allocated page from above */
//     HandleTable->NextHandleNeedingPool = INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES);
//
//     /* Setup the rest of the handle table data */
//     HandleTable->QuotaProcess = Process;
//     HandleTable->UniqueProcessId = PsGetCurrentProcess()->UniqueProcessId;
//     HandleTable->Flags = 0;
//
//     /* Loop all the handle table locks */
//     for (i = 0; i < 4; i++)
//     {
//         /* Initialize the handle table lock */
//         ExInitializePushLock(&HandleTable->HandleTableLock[i]);
//     }
//
//     /* Initialize the contention event lock and return the lock */
//     ExInitializePushLock(&HandleTable->HandleContentionEvent);
//     return HandleTable;
// }
//
// PHANDLE_TABLE_ENTRY
// NTAPI
// ExpAllocateLowLevelTable(IN PHANDLE_TABLE HandleTable,
//                          IN BOOLEAN DoInit)
// {
//     ULONG i, Base;
//     PHANDLE_TABLE_ENTRY Low, HandleEntry;
//
//     /* Allocate the low level table */
//     Low = ExpAllocateTablePagedPoolNoZero(HandleTable->QuotaProcess,
//                                           PAGE_SIZE);
//     if (!Low) return NULL;
//
//     /* Setup the initial entry */
//     HandleEntry = &Low[0];
//     HandleEntry->NextFreeTableEntry = -2;
//     HandleEntry->Value = 0;
//
//     /* Check if we're initializing */
//     if (DoInit)
//     {
//         /* Go to the next entry and the base entry */
//         HandleEntry++;
//         Base = HandleTable->NextHandleNeedingPool + INDEX_TO_HANDLE_VALUE(2);
//
//         /* Loop each entry */
//         for (i = Base;
//              i < Base + INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES - 2);
//              i += INDEX_TO_HANDLE_VALUE(1))
//         {
//             /* Free this entry and move on to the next one */
//             HandleEntry->NextFreeTableEntry = i;
//             HandleEntry->Value = 0;
//             HandleEntry++;
//         }
//
//         /* Terminate the last entry */
//         HandleEntry->NextFreeTableEntry = 0;
//         HandleEntry->Value = 0;
//     }
//
//     /* Return the low level table */
//     return Low;
// }
//
// PHANDLE_TABLE_ENTRY*
// NTAPI
// ExpAllocateMidLevelTable(IN PHANDLE_TABLE HandleTable,
//                          IN BOOLEAN DoInit,
//                          OUT PHANDLE_TABLE_ENTRY *LowTableEntry)
// {
//     PHANDLE_TABLE_ENTRY *Mid, Low;
//
//     /* Allocate the mid level table */
//     Mid = ExpAllocateTablePagedPool(HandleTable->QuotaProcess, PAGE_SIZE);
//     if (!Mid) return NULL;
//
//     /* Allocate a new low level for it */
//     Low = ExpAllocateLowLevelTable(HandleTable, DoInit);
//     if (!Low)
//     {
//         /* We failed, free the mid table */
//         ExpFreeTablePagedPool(HandleTable->QuotaProcess, Mid, PAGE_SIZE);
//         return NULL;
//     }
//
//     /* Link the tables and return the pointer */
//     Mid[0] = Low;
//     *LowTableEntry = Low;
//     return Mid;
// }
//
// BOOLEAN
// NTAPI
// ExpAllocateHandleTableEntrySlow(IN PHANDLE_TABLE HandleTable,
//                                 IN BOOLEAN DoInit)
// {
//     ULONG i, j, Index;
//     PHANDLE_TABLE_ENTRY Low = NULL, *Mid, **High, *SecondLevel, **ThirdLevel;
//     ULONG NewFree, FirstFree;
//     PVOID Value;
//     ULONG_PTR TableCode = HandleTable->TableCode;
//     ULONG_PTR TableBase = TableCode & ~3;
//     ULONG TableLevel = (ULONG)(TableCode & 3);
//     PAGED_CODE();
//
//     /* Check how many levels we already have */
//     if (TableLevel == 0)
//     {
//         /* Allocate a mid level, since we only have a low level */
//         Mid = ExpAllocateMidLevelTable(HandleTable, DoInit, &Low);
//         if (!Mid) return FALSE;
//
//         /* Link up the tables */
//         Mid[1] = Mid[0];
//         Mid[0] = (PVOID)TableBase;
//
//         /* Write the new level and attempt to change the table code */
//         TableBase = ((ULONG_PTR)Mid) | 1;
//         Value = InterlockedExchangePointer((PVOID*)&HandleTable->TableCode, (PVOID)TableBase);
//     }
//     else if (TableLevel == 1)
//     {
//         /* Setup the 2nd level table */
//         SecondLevel = (PVOID)TableBase;
//
//         /* Get if the next index can fit in the table */
//         i = HandleTable->NextHandleNeedingPool /
//             INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES);
//         if (i < MID_LEVEL_ENTRIES)
//         {
//             /* We need to allocate a new table */
//             Low = ExpAllocateLowLevelTable(HandleTable, DoInit);
//             if (!Low) return FALSE;
//
//             /* Update the table */
//             Value = InterlockedExchangePointer((PVOID*)&SecondLevel[i], Low);
//             ASSERT(Value == NULL);
//         }
//         else
//         {
//             /* We need a new high level table */
//             High = ExpAllocateTablePagedPool(HandleTable->QuotaProcess,
//                                              SizeOfHandle(HIGH_LEVEL_ENTRIES));
//             if (!High) return FALSE;
//
//             /* Allocate a new mid level table as well */
//             Mid = ExpAllocateMidLevelTable(HandleTable, DoInit, &Low);
//             if (!Mid)
//             {
//                 /* We failed, free the high level table as well */
//                 ExpFreeTablePagedPool(HandleTable->QuotaProcess,
//                                       High,
//                                       SizeOfHandle(HIGH_LEVEL_ENTRIES));
//                 return FALSE;
//             }
//
//             /* Link up the tables */
//             High[0] = (PVOID)TableBase;
//             High[1] = Mid;
//
//             /* Write the new table and change the table code */
//             TableBase = ((ULONG_PTR)High) | 2;
//             Value = InterlockedExchangePointer((PVOID*)&HandleTable->TableCode,
//                                                (PVOID)TableBase);
//         }
//     }
//     else if (TableLevel == 2)
//     {
//         /* Setup the 3rd level table */
//         ThirdLevel = (PVOID)TableBase;
//
//         /* Get the index and check if it can fit */
//         i = HandleTable->NextHandleNeedingPool / INDEX_TO_HANDLE_VALUE(MAX_MID_INDEX);
//         if (i >= HIGH_LEVEL_ENTRIES) return FALSE;
//
//         /* Check if there's no mid-level table */
//         if (!ThirdLevel[i])
//         {
//             /* Allocate a new mid level table */
//             Mid = ExpAllocateMidLevelTable(HandleTable, DoInit, &Low);
//             if (!Mid) return FALSE;
//
//             /* Update the table pointer */
//             Value = InterlockedExchangePointer((PVOID*)&ThirdLevel[i], Mid);
//             ASSERT(Value == NULL);
//         }
//         else
//         {
//             /* We have one, check at which index we should insert our entry */
//             Index = (HandleTable->NextHandleNeedingPool / INDEX_TO_HANDLE_VALUE(1)) -
//                      i * MAX_MID_INDEX;
//             j = Index / LOW_LEVEL_ENTRIES;
//
//             /* Allocate a new low level */
//             Low = ExpAllocateLowLevelTable(HandleTable, DoInit);
//             if (!Low) return FALSE;
//
//             /* Update the table pointer */
//             Value = InterlockedExchangePointer((PVOID*)&ThirdLevel[i][j], Low);
//             ASSERT(Value == NULL);
//         }
//     }
//     else
//     {
//         /* Something is really broken */
//         ASSERT(FALSE);
//     }
//
//     /* Update the index of the next handle */
//     Index = InterlockedExchangeAdd((PLONG) &HandleTable->NextHandleNeedingPool,
//                                    INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES));
//
//     /* Check if need to initialize the table */
//     if (DoInit)
//     {
//         /* Create a new index number */
//         Index += INDEX_TO_HANDLE_VALUE(1);
//
//         /* Start free index change loop */
//         for (;;)
//         {
//             /* Setup the first free index */
//             FirstFree = HandleTable->FirstFree;
//             Low[LOW_LEVEL_ENTRIES - 1].NextFreeTableEntry = FirstFree;
//
//             /* Change the index */
//             NewFree = InterlockedCompareExchange((PLONG) &HandleTable->FirstFree,
//                                                  Index,
//                                                  FirstFree);
//             if (NewFree == FirstFree) break;
//         }
//     }
//
//     /* All done */
//     return TRUE;
// }
//
// ULONG
// NTAPI
// ExpMoveFreeHandles(IN PHANDLE_TABLE HandleTable)
// {
//     ULONG LastFree, i;
//
//     /* Clear the last free index */
//     LastFree = InterlockedExchange((PLONG) &HandleTable->LastFree, 0);
//
//     /* Check if we had no index */
//     if (!LastFree) return LastFree;
//
//     /* Acquire the locks we need */
//     for (i = 1; i < 4; i++)
//     {
//         /* Acquire this lock exclusively */
//         ExWaitOnPushLock(&HandleTable->HandleTableLock[i]);
//     }
//
//     /* Check if we're not strict FIFO */
//     if (!HandleTable->StrictFIFO)
//     {
//         /* Update the first free index */
//         if (!InterlockedCompareExchange((PLONG) &HandleTable->FirstFree, LastFree, 0))
//         {
//             /* We're done, exit */
//             return LastFree;
//         }
//     }
//
//     /* We are strict FIFO, we need to reverse the entries */
//     ASSERT(FALSE);
//     return LastFree;
// }
//
// PHANDLE_TABLE_ENTRY
// NTAPI
// ExpAllocateHandleTableEntry(IN PHANDLE_TABLE HandleTable,
//                             OUT PEXHANDLE NewHandle)
// {
//     ULONG OldValue, NewValue, NewValue1;
//     PHANDLE_TABLE_ENTRY Entry;
//     EXHANDLE Handle, OldHandle;
//     BOOLEAN Result;
//     ULONG i;
//
//     /* Start allocation loop */
//     for (;;)
//     {
//         /* Get the current link */
//         OldValue = HandleTable->FirstFree;
//         while (!OldValue)
//         {
//             /* No free entries remain, lock the handle table */
//             KeEnterCriticalRegion();
//             ExAcquirePushLockExclusive(&HandleTable->HandleTableLock[0]);
//
//             /* Check the value again */
//             OldValue = HandleTable->FirstFree;
//             if (OldValue)
//             {
//                 /* Another thread has already created a new level, bail out */
//                 ExReleasePushLockExclusive(&HandleTable->HandleTableLock[0]);
//                 KeLeaveCriticalRegion();
//                 break;
//             }
//
//             /* Now move any free handles */
//             OldValue = ExpMoveFreeHandles(HandleTable);
//             if (OldValue)
//             {
//                 /* Another thread has already moved them, bail out */
//                 ExReleasePushLockExclusive(&HandleTable->HandleTableLock[0]);
//                 KeLeaveCriticalRegion();
//                 break;
//             }
//
//             /* We're the first one through, so do the actual allocation */
//             Result = ExpAllocateHandleTableEntrySlow(HandleTable, TRUE);
//
//             /* Unlock the table and get the value now */
//             ExReleasePushLockExclusive(&HandleTable->HandleTableLock[0]);
//             KeLeaveCriticalRegion();
//             OldValue = HandleTable->FirstFree;
//
//             /* Check if allocation failed */
//             if (!Result)
//             {
//                 /* Check if nobody else went through here */
//                 if (!OldValue)
//                 {
//                     /* We're still the only thread around, so fail */
//                     NewHandle->GenericHandleOverlay = NULL;
//                     return NULL;
//                 }
//             }
//         }
//
//         /* We made it, write the current value */
//         Handle.Value = (OldValue & FREE_HANDLE_MASK);
//
//         /* Lookup the entry for this handle */
//         Entry = ExpLookupHandleTableEntry(HandleTable, Handle);
//
//         /* Get an available lock and acquire it */
//         OldHandle.Value = OldValue;
//         i = OldHandle.Index % 4;
//         KeEnterCriticalRegion();
//         ExAcquirePushLockShared(&HandleTable->HandleTableLock[i]);
//
//         /* Check if the value changed after acquiring the lock */
//         if (OldValue != *(volatile ULONG*)&HandleTable->FirstFree)
//         {
//             /* It did, so try again */
//             ExReleasePushLockShared(&HandleTable->HandleTableLock[i]);
//             KeLeaveCriticalRegion();
//             continue;
//         }
//
//         /* Now get the next value and do the compare */
//         NewValue = *(volatile ULONG*)&Entry->NextFreeTableEntry;
//         NewValue1 = InterlockedCompareExchange((PLONG) &HandleTable->FirstFree,
//                                                NewValue,
//                                                OldValue);
//
//         /* The change was done, so release the lock */
//         ExReleasePushLockShared(&HandleTable->HandleTableLock[i]);
//         KeLeaveCriticalRegion();
//
//         /* Check if the compare was successful */
//         if (NewValue1 == OldValue)
//         {
//             /* Make sure that the new handle is in range, and break out */
//             ASSERT((NewValue & FREE_HANDLE_MASK) <
//                    HandleTable->NextHandleNeedingPool);
//             break;
//         }
//         else
//         {
//             /* The compare failed, make sure we expected it */
//             ASSERT((NewValue1 & FREE_HANDLE_MASK) !=
//                    (OldValue & FREE_HANDLE_MASK));
//         }
//     }
//
//     /* Increase the number of handles */
//     InterlockedIncrement(&HandleTable->HandleCount);
//
//     /* Return the handle and the entry */
//     *NewHandle = Handle;
//     return Entry;
// }
//
// PHANDLE_TABLE
// NTAPI
// ExCreateHandleTable(IN PEPROCESS Process OPTIONAL)
// {
//     PHANDLE_TABLE HandleTable;
//     PAGED_CODE();
//
//     /* Allocate the handle table */
//     HandleTable = ExpAllocateHandleTable(Process, TRUE);
//     if (!HandleTable) return NULL;
//
//     /* Acquire the handle table lock */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&HandleTableListLock);
//
//     /* Insert it into the list */
//     InsertTailList(&HandleTableListHead, &HandleTable->HandleTableList);
//
//     /* Release the lock */
//     ExReleasePushLockExclusive(&HandleTableListLock);
//     KeLeaveCriticalRegion();
//
//     /* Return the handle table */
//     return HandleTable;
// }
//
// HANDLE
// NTAPI
// ExCreateHandle(IN PHANDLE_TABLE HandleTable,
//                IN PHANDLE_TABLE_ENTRY HandleTableEntry)
// {
//     EXHANDLE Handle;
//     PHANDLE_TABLE_ENTRY NewEntry;
//     PAGED_CODE();
//
//     /* Start with a clean handle */
//     Handle.GenericHandleOverlay = NULL;
//
//     /* Allocate a new entry */
//     NewEntry = ExpAllocateHandleTableEntry(HandleTable, &Handle);
//     if (NewEntry)
//     {
//         /* Enter a critical region */
//         KeEnterCriticalRegion();
//
//         /* Write the entry */
//         *NewEntry = *HandleTableEntry;
//
//         /* Unlock it and leave the critical region */
//         ExUnlockHandleTableEntry(HandleTable, NewEntry);
//         KeLeaveCriticalRegion();
//     }
//
//     /* Return the handle value */
//     return Handle.GenericHandleOverlay;
// }
//
// VOID
// NTAPI
// ExpBlockOnLockedHandleEntry(IN PHANDLE_TABLE HandleTable,
//                             IN PHANDLE_TABLE_ENTRY HandleTableEntry)
// {
//     LONG_PTR OldValue;
//     EX_PUSH_LOCK_WAIT_BLOCK WaitBlock;
//
//     /* Block on the pushlock */
//     ExBlockPushLock(&HandleTable->HandleContentionEvent, &WaitBlock);
//
//     /* Get the current value and check if it's been unlocked */
//     OldValue = HandleTableEntry->Value;
//     if (!(OldValue) || (OldValue & EXHANDLE_TABLE_ENTRY_LOCK_BIT))
//     {
//         /* Unblock the pushlock and return */
//         ExfUnblockPushLock(&HandleTable->HandleContentionEvent, &WaitBlock);
//     }
//     else
//     {
//         /* Wait for it to be unblocked */
//         ExWaitForUnblockPushLock(&HandleTable->HandleContentionEvent,
//                                  &WaitBlock);
//     }
// }
//
// BOOLEAN
// NTAPI
// ExpLockHandleTableEntry(IN PHANDLE_TABLE HandleTable,
//                         IN PHANDLE_TABLE_ENTRY HandleTableEntry)
// {
//     LONG_PTR NewValue, OldValue;
//
//     /* Sanity check */
//     ASSERT((KeGetCurrentThread()->CombinedApcDisable != 0) ||
//            (KeGetCurrentIrql() == APC_LEVEL));
//
//     /* Start lock loop */
//     for (;;)
//     {
//         /* Get the current value and check if it's locked */
//         OldValue = *(volatile LONG_PTR *)&HandleTableEntry->Object;
//         if (OldValue & EXHANDLE_TABLE_ENTRY_LOCK_BIT)
//         {
//             /* It's not locked, remove the lock bit to lock it */
//             NewValue = OldValue & ~EXHANDLE_TABLE_ENTRY_LOCK_BIT;
//             if (InterlockedCompareExchangePointer(&HandleTableEntry->Object,
//                                                   (PVOID)NewValue,
//                                                   (PVOID)OldValue) == (PVOID)OldValue)
//             {
//                 /* We locked it, get out */
//                 return TRUE;
//             }
//         }
//         else
//         {
//             /* We couldn't lock it, bail out if it's been freed */
//             if (!OldValue) return FALSE;
//         }
//
//         /* It's locked, wait for it to be unlocked */
//         ExpBlockOnLockedHandleEntry(HandleTable, HandleTableEntry);
//     }
// }
//
// VOID
// NTAPI
// ExUnlockHandleTableEntry(IN PHANDLE_TABLE HandleTable,
//                          IN PHANDLE_TABLE_ENTRY HandleTableEntry)
// {
//     LONG_PTR OldValue;
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT((KeGetCurrentThread()->CombinedApcDisable != 0) ||
//            (KeGetCurrentIrql() == APC_LEVEL));
//
//     /* Set the lock bit and make sure it wasn't earlier */
//     OldValue = InterlockedOr((PLONG) &HandleTableEntry->Value,
//                              EXHANDLE_TABLE_ENTRY_LOCK_BIT);
//     ASSERT((OldValue & EXHANDLE_TABLE_ENTRY_LOCK_BIT) == 0);
//
//     /* Unblock any waiters */
//     ExfUnblockPushLock(&HandleTable->HandleContentionEvent, NULL);
// }
//
// VOID
// NTAPI
// ExRemoveHandleTable(IN PHANDLE_TABLE HandleTable)
// {
//     PAGED_CODE();
//
//     /* Acquire the table lock */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&HandleTableListLock);
//
//     /* Remove the table and reset the list */
//     RemoveEntryList(&HandleTable->HandleTableList);
//     InitializeListHead(&HandleTable->HandleTableList);
//
//     /* Release the lock */
//     ExReleasePushLockExclusive(&HandleTableListLock);
//     KeLeaveCriticalRegion();
// }
//
// VOID
// NTAPI
// ExDestroyHandleTable(IN PHANDLE_TABLE HandleTable,
//                      IN PVOID DestroyHandleProcedure OPTIONAL)
// {
//     PAGED_CODE();
//
//     /* Remove the handle from the list */
//     ExRemoveHandleTable(HandleTable);
//
//     /* Check if we have a destroy callback */
//     if (DestroyHandleProcedure)
//     {
//         /* FIXME: */
//         ASSERT(FALSE);
//     }
//
//     /* Free the handle table */
//     ExpFreeHandleTable(HandleTable);
// }
//
// BOOLEAN
// NTAPI
// ExDestroyHandle(IN PHANDLE_TABLE HandleTable,
//                 IN HANDLE Handle,
//                 IN PHANDLE_TABLE_ENTRY HandleTableEntry OPTIONAL)
// {
//     EXHANDLE ExHandle;
//     PVOID Object;
//     PAGED_CODE();
//
//     /* Setup the actual handle value */
//     ExHandle.GenericHandleOverlay = Handle;
//
//     /* Enter a critical region and check if we have to lookup the handle */
//     KeEnterCriticalRegion();
//     if (!HandleTableEntry)
//     {
//         /* Lookup the entry */
//         HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, ExHandle);
//
//         /* Make sure that we found an entry, and that it's valid */
//         if (!(HandleTableEntry) ||
//             !(HandleTableEntry->Object) ||
//             (HandleTableEntry->NextFreeTableEntry == -2))
//         {
//             /* Invalid handle, fail */
//             KeLeaveCriticalRegion();
//             return FALSE;
//         }
//
//         /* Lock the entry */
//         if (!ExpLockHandleTableEntry(HandleTable, HandleTableEntry))
//         {
//             /* Couldn't lock, fail */
//             KeLeaveCriticalRegion();
//             return FALSE;
//         }
//     }
//     else
//     {
//         /* Make sure the handle is locked */
//         ASSERT((HandleTableEntry->Value & EXHANDLE_TABLE_ENTRY_LOCK_BIT) == 0);
//     }
//
//     /* Clear the handle */
//     Object = InterlockedExchangePointer((PVOID*)&HandleTableEntry->Object, NULL);
//
//     /* Sanity checks */
//     ASSERT(Object != NULL);
//     ASSERT((((ULONG_PTR)Object) & EXHANDLE_TABLE_ENTRY_LOCK_BIT) == 0);
//
//     /* Unblock the pushlock */
//     ExfUnblockPushLock(&HandleTable->HandleContentionEvent, NULL);
//
//     /* Free the actual entry */
//     ExpFreeHandleTableEntry(HandleTable, ExHandle, HandleTableEntry);
//
//     /* If we got here, return success */
//     KeLeaveCriticalRegion();
//     return TRUE;
// }
//
// PHANDLE_TABLE_ENTRY
// NTAPI
// ExMapHandleToPointer(IN PHANDLE_TABLE HandleTable,
//                      IN HANDLE Handle)
// {
//     EXHANDLE ExHandle;
//     PHANDLE_TABLE_ENTRY HandleTableEntry;
//     PAGED_CODE();
//
//     /* Set the handle value */
//     ExHandle.GenericHandleOverlay = Handle;
//
//     /* Fail if we got an invalid index */
//     if (!(ExHandle.Index & (LOW_LEVEL_ENTRIES - 1))) return NULL;
//
//     /* Do the lookup */
//     HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, ExHandle);
//     if (!HandleTableEntry) return NULL;
//
//     /* Lock it */
//     if (!ExpLockHandleTableEntry(HandleTable, HandleTableEntry)) return NULL;
//
//     /* Return the entry */
//     return HandleTableEntry;
// }
//
// PHANDLE_TABLE
// NTAPI
// ExDupHandleTable(IN PEPROCESS Process,
//                  IN PHANDLE_TABLE HandleTable,
//                  IN PEX_DUPLICATE_HANDLE_CALLBACK DupHandleProcedure,
//                  IN ULONG_PTR Mask)
// {
//     PHANDLE_TABLE NewTable;
//     EXHANDLE Handle;
//     PHANDLE_TABLE_ENTRY HandleTableEntry, NewEntry;
//     BOOLEAN Failed = FALSE;
//     PAGED_CODE();
//
//     /* Allocate the duplicated copy */
//     NewTable = ExpAllocateHandleTable(Process, FALSE);
//     if (!NewTable) return NULL;
//
//     /* Loop each entry */
//     while (NewTable->NextHandleNeedingPool <
//            HandleTable->NextHandleNeedingPool)
//     {
//         /* Insert it into the duplicated copy */
//         if (!ExpAllocateHandleTableEntrySlow(NewTable, FALSE))
//         {
//             /* Insert failed, free the new copy and return */
//             ExpFreeHandleTable(NewTable);
//             return NULL;
//         }
//     }
//
//     /* Setup the initial handle table data */
//     NewTable->HandleCount = 0;
//     NewTable->ExtraInfoPages = 0;
//     NewTable->FirstFree = 0;
//
//     /* Setup the first handle value  */
//     Handle.Value = INDEX_TO_HANDLE_VALUE(1);
//
//     /* Enter a critical region and lookup the new entry */
//     KeEnterCriticalRegion();
//     while ((NewEntry = ExpLookupHandleTableEntry(NewTable, Handle)))
//     {
//         /* Lookup the old entry */
//         HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, Handle);
//
//         /* Loop each entry */
//         do
//         {
//             /* Check if it doesn't match the audit mask */
//             if (!(HandleTableEntry->Value & Mask))
//             {
//                 /* Free it since we won't use it */
//                 Failed = TRUE;
//             }
//             else
//             {
//                 /* Lock the entry */
//                 if (!ExpLockHandleTableEntry(HandleTable, HandleTableEntry))
//                 {
//                     /* Free it since we can't lock it, so we won't use it */
//                     Failed = TRUE;
//                 }
//                 else
//                 {
//                     /* Copy the handle value */
//                     *NewEntry = *HandleTableEntry;
//
//                     /* Call the duplicate callback */
//                     if (DupHandleProcedure(Process,
//                                            HandleTable,
//                                            HandleTableEntry,
//                                            NewEntry))
//                     {
//                         /* Clear failure flag */
//                         Failed = FALSE;
//
//                         /* Lock the entry, increase the handle count */
//                         NewEntry->Value |= EXHANDLE_TABLE_ENTRY_LOCK_BIT;
//                         NewTable->HandleCount++;
//                     }
//                     else
//                     {
//                         /* Duplication callback refused, fail */
//                         Failed = TRUE;
//                     }
//                 }
//             }
//
//             /* Check if we failed earlier and need to free */
//             if (Failed)
//             {
//                 /* Free this entry */
//                 NewEntry->Object = NULL;
//                 NewEntry->NextFreeTableEntry = NewTable->FirstFree;
//                 NewTable->FirstFree = (ULONG)Handle.Value;
//             }
//
//             /* Increase the handle value and move to the next entry */
//             Handle.Value += INDEX_TO_HANDLE_VALUE(1);
//             NewEntry++;
//             HandleTableEntry++;
//         } while (Handle.Value % INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES));
//
//         /* We're done, skip the last entry */
//         Handle.Value += INDEX_TO_HANDLE_VALUE(1);
//     }
//
//     /* Acquire the table lock and insert this new table into the list */
//     ExAcquirePushLockExclusive(&HandleTableListLock);
//     InsertTailList(&HandleTableListHead, &NewTable->HandleTableList);
//     ExReleasePushLockExclusive(&HandleTableListLock);
//
//     /* Leave the critical region we entered previously and return the table */
//     KeLeaveCriticalRegion();
//     return NewTable;
// }
//
// BOOLEAN
// NTAPI
// ExChangeHandle(IN PHANDLE_TABLE HandleTable,
//                IN HANDLE Handle,
//                IN PEX_CHANGE_HANDLE_CALLBACK ChangeRoutine,
//                IN ULONG_PTR Context)
// {
//     EXHANDLE ExHandle;
//     PHANDLE_TABLE_ENTRY HandleTableEntry;
//     BOOLEAN Result = FALSE;
//     PAGED_CODE();
//
//     /* Set the handle value */
//     ExHandle.GenericHandleOverlay = Handle;
//
//     /* Find the entry for this handle */
//     HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, ExHandle);
//
//     /* Make sure that we found an entry, and that it's valid */
//     if (!(HandleTableEntry) ||
//         !(HandleTableEntry->Object) ||
//         (HandleTableEntry->NextFreeTableEntry == -2))
//     {
//         /* It isn't, fail */
//         return FALSE;
//     }
//
//     /* Enter a critical region */
//     KeEnterCriticalRegion();
//
//     /* Try locking the handle entry */
//     if (ExpLockHandleTableEntry(HandleTable, HandleTableEntry))
//     {
//         /* Call the change routine and unlock the entry */
//         Result = ChangeRoutine(HandleTableEntry, Context);
//         ExUnlockHandleTableEntry(HandleTable, HandleTableEntry);
//     }
//
//     /* Leave the critical region and return the callback result */
//     KeLeaveCriticalRegion();
//     return Result;
// }
//
// VOID
// NTAPI
// ExSweepHandleTable(IN PHANDLE_TABLE HandleTable,
//                    IN PEX_SWEEP_HANDLE_CALLBACK EnumHandleProcedure,
//                    IN PVOID Context)
// {
//     EXHANDLE Handle;
//     PHANDLE_TABLE_ENTRY HandleTableEntry;
//     PAGED_CODE();
//
//     /* Set the initial value and loop the entries */
//     Handle.Value = INDEX_TO_HANDLE_VALUE(1);
//     while ((HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, Handle)))
//     {
//         /* Loop each handle */
//         do
//         {
//             /* Lock the entry */
//             if (ExpLockHandleTableEntry(HandleTable, HandleTableEntry))
//             {
//                 /* Notify the callback routine */
//                 EnumHandleProcedure(HandleTableEntry,
//                                     Handle.GenericHandleOverlay,
//                                     Context);
//             }
//
//             /* Go to the next handle and entry */
//             Handle.Value += INDEX_TO_HANDLE_VALUE(1);
//             HandleTableEntry++;
//         } while (Handle.Value % INDEX_TO_HANDLE_VALUE(LOW_LEVEL_ENTRIES));
//
//         /* Skip past the last entry */
//         Handle.Value += INDEX_TO_HANDLE_VALUE(1);
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// ExEnumHandleTable(IN PHANDLE_TABLE HandleTable,
//                   IN PEX_ENUM_HANDLE_CALLBACK EnumHandleProcedure,
//                   IN OUT PVOID Context,
//                   OUT PHANDLE EnumHandle OPTIONAL)
// {
//     EXHANDLE Handle;
//     PHANDLE_TABLE_ENTRY HandleTableEntry;
//     BOOLEAN Result = FALSE;
//     PAGED_CODE();
//
//     /* Enter a critical region */
//     KeEnterCriticalRegion();
//
//     /* Set the initial value and loop the entries */
//     Handle.Value = 0;
//     while ((HandleTableEntry = ExpLookupHandleTableEntry(HandleTable, Handle)))
//     {
//         /* Validate the entry */
//         if ((HandleTableEntry->Object) &&
//             (HandleTableEntry->NextFreeTableEntry != -2))
//         {
//             /* Lock the entry */
//             if (ExpLockHandleTableEntry(HandleTable, HandleTableEntry))
//             {
//                 /* Notify the callback routine */
//                 Result = EnumHandleProcedure(HandleTableEntry,
//                                              Handle.GenericHandleOverlay,
//                                              Context);
//
//                 /* Unlock it */
//                 ExUnlockHandleTableEntry(HandleTable, HandleTableEntry);
//
//                 /* Was this the one looked for? */
//                 if (Result)
//                 {
//                     /* If so, return it if requested */
//                     if (EnumHandle) *EnumHandle = Handle.GenericHandleOverlay;
//                     break;
//                 }
//             }
//         }
//
//         /* Go to the next entry */
//         Handle.Value += INDEX_TO_HANDLE_VALUE(1);
//     }
//
//     /* Leave the critical region and return callback result */
//     KeLeaveCriticalRegion();
//     return Result;
// }
//
// #if DBG && defined(KDBG)
//
// #include <kdbg/kdb.h>
//
// BOOLEAN ExpKdbgExtHandle(ULONG Argc, PCHAR Argv[])
// {
//     USHORT i;
//     char *endptr;
//     HANDLE ProcessId;
//     EXHANDLE ExHandle;
//     PLIST_ENTRY Entry;
//     PEPROCESS Process;
//     WCHAR KeyPath[256];
//     PFILE_OBJECT FileObject;
//     PHANDLE_TABLE HandleTable;
//     POBJECT_HEADER ObjectHeader;
//     PHANDLE_TABLE_ENTRY TableEntry;
//     ULONG NeededLength = 0;
//     ULONG NameLength;
//     PCM_KEY_CONTROL_BLOCK Kcb, CurrentKcb;
//     POBJECT_HEADER_NAME_INFO ObjectNameInfo;
//
//     if (Argc > 1)
//     {
//         /* Get _EPROCESS address or PID */
//         i = 0;
//         while (Argv[1][i])
//         {
//             if (!isdigit(Argv[1][i]))
//             {
//                 i = 0;
//                 break;
//             }
//
//             ++i;
//         }
//
//         if (i == 0)
//         {
//             if (!KdbpGetHexNumber(Argv[1], (PVOID)&Process))
//             {
//                 KdbpPrint("Invalid parameter: %s\n", Argv[1]);
//                 return TRUE;
//             }
//
//             /* In the end, we always want a PID */
//             ProcessId = PsGetProcessId(Process);
//         }
//         else
//         {
//             ProcessId = (HANDLE)strtoulptr(Argv[1], &endptr, 10);
//             if (*endptr != '\0')
//             {
//                 KdbpPrint("Invalid parameter: %s\n", Argv[1]);
//                 return TRUE;
//             }
//         }
//     }
//     else
//     {
//         ProcessId = PsGetCurrentProcessId();
//     }
//
//     for (Entry = HandleTableListHead.Flink;
//          Entry != &HandleTableListHead;
//          Entry = Entry->Flink)
//     {
//         /* Only return matching PID
//          * 0 matches everything
//          */
//         HandleTable = CONTAINING_RECORD(Entry, HANDLE_TABLE, HandleTableList);
//         if (ProcessId != 0 && HandleTable->UniqueProcessId != ProcessId)
//         {
//             continue;
//         }
//
//         KdbpPrint("\n");
//
//         KdbpPrint("Handle table at %p with %d entries in use\n", HandleTable, HandleTable->HandleCount);
//
//         ExHandle.Value = 0;
//         while ((TableEntry = ExpLookupHandleTableEntry(HandleTable, ExHandle)))
//         {
//             if ((TableEntry->Object) &&
//                 (TableEntry->NextFreeTableEntry != -2))
//             {
//                 ObjectHeader = ObpGetHandleObject(TableEntry);
//
//                 KdbpPrint("%p: Object: %p GrantedAccess: %x Entry: %p\n", ExHandle.Value, &ObjectHeader->Body, TableEntry->GrantedAccess, TableEntry);
//                 KdbpPrint("Object: %p Type: (%x) %wZ\n", &ObjectHeader->Body, ObjectHeader->Type, &ObjectHeader->Type->Name);
//                 KdbpPrint("\tObjectHeader: %p\n", ObjectHeader);
//                 KdbpPrint("\t\tHandleCount: %u PointerCount: %u\n", ObjectHeader->HandleCount, ObjectHeader->PointerCount);
//
//                 /* Specific objects debug prints */
//
//                 /* For file, display path */
//                 if (ObjectHeader->Type == IoFileObjectType)
//                 {
//                     FileObject = (PFILE_OBJECT)&ObjectHeader->Body;
//
//                     KdbpPrint("\t\t\tName: %wZ\n", &FileObject->FileName);
//                 }
//
//                 /* For directory, and win32k objects, display object name */
//                 else if (ObjectHeader->Type == ObpDirectoryObjectType ||
//                          ObjectHeader->Type == ExWindowStationObjectType ||
//                          ObjectHeader->Type == ExDesktopObjectType ||
//                          ObjectHeader->Type == MmSectionObjectType)
//                 {
//                     ObjectNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
//                     if (ObjectNameInfo != NULL && ObjectNameInfo->Name.Buffer != NULL)
//                     {
//                         KdbpPrint("\t\t\tName: %wZ\n", &ObjectNameInfo->Name);
//                     }
//                 }
//
//                 /* For registry keys, display full path */
//                 else if (ObjectHeader->Type == CmpKeyObjectType)
//                 {
//                     Kcb = ((PCM_KEY_BODY)&ObjectHeader->Body)->KeyControlBlock;
//                     if (!Kcb->Delete)
//                     {
//                         CurrentKcb = Kcb;
//
//                         /* See: CmpQueryNameInformation() */
//
//                         while (CurrentKcb != NULL)
//                         {
//                             if (CurrentKcb->NameBlock->Compressed)
//                                 NeededLength += CmpCompressedNameSize(CurrentKcb->NameBlock->Name, CurrentKcb->NameBlock->NameLength);
//                             else
//                                 NeededLength += CurrentKcb->NameBlock->NameLength;
//
//                             NeededLength += sizeof(OBJ_NAME_PATH_SEPARATOR);
//
//                             CurrentKcb = CurrentKcb->ParentKcb;
//                         }
//
//                         if (NeededLength < sizeof(KeyPath))
//                         {
//                             CurrentKcb = Kcb;
//
//                             while (CurrentKcb != NULL)
//                             {
//                                 if (CurrentKcb->NameBlock->Compressed)
//                                 {
//                                     NameLength = CmpCompressedNameSize(CurrentKcb->NameBlock->Name, CurrentKcb->NameBlock->NameLength);
//                                     CmpCopyCompressedName(&KeyPath[(NeededLength - NameLength)/sizeof(WCHAR)],
//                                                           NameLength,
//                                                           CurrentKcb->NameBlock->Name,
//                                                           CurrentKcb->NameBlock->NameLength);
//                                 }
//                                 else
//                                 {
//                                     NameLength = CurrentKcb->NameBlock->NameLength;
//                                     RtlCopyMemory(&KeyPath[(NeededLength - NameLength)/sizeof(WCHAR)],
//                                                   CurrentKcb->NameBlock->Name,
//                                                   NameLength);
//                                 }
//
//                                 NeededLength -= NameLength;
//                                 NeededLength -= sizeof(OBJ_NAME_PATH_SEPARATOR);
//                                 KeyPath[NeededLength/sizeof(WCHAR)] = OBJ_NAME_PATH_SEPARATOR;
//
//                                 CurrentKcb = CurrentKcb->ParentKcb;
//                             }
//                         }
//
//                         KdbpPrint("\t\t\tName: %S\n", KeyPath);
//                     }
//                 }
//             }
//
//             ExHandle.Value += INDEX_TO_HANDLE_VALUE(1);
//         }
//     }
//
//     return TRUE;
// }
//
// #endif // DBG && defined(KDBG)
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// ============================================================================
// ExpKdbgExtHandle — KDBG `!handle` extension. Walks HandleTableListHead and
// dumps every entry of the matching process's handle table. Translated from
// `ntoskrnl/kernel_c_helpers.c` 2026-05-07.
//
// Uses `kdbp_print!` macro instead of variadic KdbpPrint; format-string
// parsing happens in Rust ntprintf via rust_kdbp_print → KdbpPagerInternal.
// ============================================================================

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "system" {
    fn KdbpGetHexNumber(s: *mut core::ffi::c_char, out: *mut core::ffi::c_void)
        -> ntkrnl_sys::base::ULONG;
    static mut IoFileObjectType: ntkrnl_sys::ddk::POBJECT_TYPE;
    static CmpKeyObjectType: ntkrnl_sys::ddk::POBJECT_TYPE;
    // NTAPI on i386 (stdcall) — wrong ABI here breaks i386 boot via the
    // rustc 1.92 cdecl-emit-for-stdcall path documented in
    // `feedback_extern_c_abi.md`. amd64 / arm64 have NTAPI = cdecl so the
    // bug is invisible there.
    fn PsGetCurrentProcessId() -> ntkrnl_sys::ddk::HANDLE;
    fn PsGetProcessId(Process: ntkrnl_sys::ddk::PEPROCESS) -> ntkrnl_sys::ddk::HANDLE;
}

/// Parses a NUL-terminated decimal C string. Mirrors strtoul/strtoull behaviour:
/// stops at the first non-digit; sets `endptr_offset` to that position. Returns
/// (parsed_value, all_chars_consumed).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn parse_decimal_pid(s: *const core::ffi::c_char) -> Option<usize> {
    let mut p = s;
    let mut acc: usize = 0;
    if *p == 0 {
        return None;
    }
    while *p != 0 {
        let c = *p as u8;
        if !(b'0'..=b'9').contains(&c) {
            return None;
        }
        acc = acc.wrapping_mul(10).wrapping_add((c - b'0') as usize);
        p = p.add(1);
    }
    Some(acc)
}

/// `ExpKdbgExtHandle(Argc, Argv)` — i386/amd64 KDBG extension. ARM64 has no
/// KDBG path here so we don't compile this on aarch64.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn ExpKdbgExtHandle(
    argc: ntkrnl_sys::base::ULONG,
    argv: *mut *mut core::ffi::c_char,
) -> ntkrnl_sys::base::BOOLEAN {
    use ntkrnl_sys::base::*;
    use ntkrnl_sys::constants::OBJ_HANDLE_ATTRIBUTES;
    use ntkrnl_sys::ddk::{
        PCM_KEY_BODY, PCM_KEY_CONTROL_BLOCK, PFILE_OBJECT, _HANDLE_TABLE, _HANDLE_TABLE_ENTRY,
        _LIST_ENTRY, _OBJECT_HEADER,
    };

    // Resolve target ProcessId. If user passed an argument it's either a
    // hex pointer to an EPROCESS (then PsGetProcessId) or a decimal PID.
    let process_id: ntkrnl_sys::ddk::HANDLE = if argc > 1 {
        let arg = *argv.add(1);
        match parse_decimal_pid(arg) {
            Some(pid) => pid as ntkrnl_sys::ddk::HANDLE,
            None => {
                let mut process: ntkrnl_sys::ddk::PEPROCESS = core::ptr::null_mut();
                if KdbpGetHexNumber(arg, &mut process as *mut _ as *mut core::ffi::c_void) == 0 {
                    crate::kdbp_print!(b"Invalid parameter: %s\n\0", arg);
                    return 1;
                }
                PsGetProcessId(process)
            }
        }
    } else {
        PsGetCurrentProcessId()
    };

    // Walk HandleTableListHead — circular doubly-linked list of HANDLE_TABLEs.
    let head = core::ptr::addr_of_mut!(HandleTableListHead) as *mut _LIST_ENTRY;
    let mut entry = (*head).Flink;
    let table_list_offset = core::mem::offset_of!(_HANDLE_TABLE, HandleTableList);
    while entry != head {
        let handle_table =
            (entry as *mut u8).sub(table_list_offset).cast::<_HANDLE_TABLE>();
        if !process_id.is_null()
            && (*handle_table).UniqueProcessId != process_id as *mut core::ffi::c_void
        {
            entry = (*entry).Flink;
            continue;
        }
        crate::kdbp_print!(b"\n\0");
        crate::kdbp_print!(
            b"Handle table at %p with %d entries in use\n\0",
            handle_table,
            (*handle_table).HandleCount
        );

        let mut ex_handle: ntkrnl_sys::ddk::EXHANDLE = core::mem::zeroed();
        ex_handle.Value = 0;

        loop {
            let table_entry: *mut _HANDLE_TABLE_ENTRY =
                ExpLookupHandleTableEntry(handle_table, ex_handle);
            if table_entry.is_null() {
                break;
            }
            let object_raw = (*table_entry).__bindgen_anon_1.Object;
            let next_free = (*table_entry).__bindgen_anon_2.NextFreeTableEntry;
            if !object_raw.is_null() && next_free != -2 {
                let object_header = ((object_raw as ULONG_PTR)
                    & !(OBJ_HANDLE_ATTRIBUTES as ULONG_PTR))
                    as *mut _OBJECT_HEADER;
                let granted_access = (*table_entry).__bindgen_anon_2.GrantedAccess;
                let body_ptr = core::ptr::addr_of_mut!((*object_header).Body);

                crate::kdbp_print!(
                    b"%p: Object: %p GrantedAccess: %x Entry: %p\n\0",
                    ex_handle.Value,
                    body_ptr,
                    granted_access,
                    table_entry
                );

                let object_type = (*object_header).Type;
                crate::kdbp_print!(
                    b"Object: %p Type: (%x) %wZ\n\0",
                    body_ptr,
                    object_type,
                    core::ptr::addr_of_mut!((*object_type).Name)
                );
                crate::kdbp_print!(b"\tObjectHeader: %p\n\0", object_header);
                // PointerCount is a top-level field; HandleCount lives inside
                // the anon union `__bindgen_anon_1` (sharing storage with NextToFree).
                crate::kdbp_print!(
                    b"\t\tHandleCount: %u PointerCount: %u\n\0",
                    (*object_header).__bindgen_anon_1.HandleCount,
                    (*object_header).PointerCount
                );

                if object_type == IoFileObjectType {
                    let file_object = body_ptr as PFILE_OBJECT;
                    crate::kdbp_print!(
                        b"\t\tName: %wZ\n\0",
                        core::ptr::addr_of_mut!((*file_object).FileName)
                    );
                } else if object_type == CmpKeyObjectType {
                    let mut key_path: [WCHAR; 256] = [0; 256];
                    let kcb = (*(body_ptr as PCM_KEY_BODY)).KeyControlBlock
                        as PCM_KEY_CONTROL_BLOCK;

                    // Pass 1: compute total length needed (one byte for each
                    // key path component plus a leading separator).
                    let mut needed_length: ULONG = 0;
                    let mut current_kcb = kcb;
                    while !current_kcb.is_null() {
                        let name_block = (*current_kcb).NameBlock;
                        // Both union arms (NameHash / __bindgen_anon_1) have
                        // NameLength at the same offset; pick NameHash.
                        let name_length =
                            (*name_block).__bindgen_anon_1.NameHash.NameLength as ULONG;
                        if (*name_block).Compressed != 0 {
                            needed_length += name_length + 1;
                        } else {
                            needed_length += name_length / core::mem::size_of::<WCHAR>() as ULONG
                                + 1;
                        }
                        current_kcb = (*current_kcb).ParentKcb;
                    }

                    // Pass 2: walk again writing components into key_path
                    // back-to-front (the C version mutates `needed_length` as
                    // a write cursor).
                    if needed_length < 256 {
                        let mut cursor = needed_length as usize;
                        key_path[cursor] = 0; // UNICODE_NULL
                        let mut current_kcb = kcb;
                        while !current_kcb.is_null() {
                            let name_block = (*current_kcb).NameBlock;
                            // `Name` is a flexible array — use pointer arithmetic.
                            let name_ptr = core::ptr::addr_of!(
                                (*name_block).__bindgen_anon_1.NameHash.Name
                            ) as *const WCHAR;
                            let raw_name_length =
                                (*name_block).__bindgen_anon_1.NameHash.NameLength as ULONG;
                            let name_length = if (*name_block).Compressed != 0 {
                                raw_name_length as usize
                            } else {
                                (raw_name_length / core::mem::size_of::<WCHAR>() as ULONG)
                                    as usize
                            };
                            cursor = cursor.saturating_sub(name_length + 1);
                            for i in 0..name_length {
                                let ch = if (*name_block).Compressed != 0 {
                                    *(name_ptr as *const u8).add(i) as WCHAR
                                } else {
                                    *name_ptr.add(i)
                                };
                                key_path[cursor + 1 + i] = ch;
                            }
                            key_path[cursor] = b'\\' as WCHAR; // OBJ_NAME_PATH_SEPARATOR
                            current_kcb = (*current_kcb).ParentKcb;
                        }
                        crate::kdbp_print!(b"\t\tName: %S\n\0", key_path.as_ptr());
                    }
                }

                let object_name_info =
                    ntkrnl_sys::macros::OBJECT_HEADER_TO_NAME_INFO(object_header);
                if !object_name_info.is_null()
                    && !(*object_name_info).Name.Buffer.is_null()
                {
                    crate::kdbp_print!(
                        b"\t\tName: %wZ\n\0",
                        core::ptr::addr_of_mut!((*object_name_info).Name)
                    );
                }
            }
            // SizeOfHandle(1) — handle values are encoded as `sizeof(HANDLE) * index`.
            // EXHANDLE.Value is bindgen's ULONG_PTR (= c_ulong on i386 / c_ulonglong on
            // amd64), distinct from base::ULONG_PTR (= usize). Cast through usize.
            ex_handle.Value = ex_handle.Value.wrapping_add(
                core::mem::size_of::<ntkrnl_sys::ddk::HANDLE>() as ntkrnl_sys::ddk::ULONG_PTR,
            );
        }
        entry = (*entry).Flink;
    }
    1 // TRUE
}

// ============================================================================
// ExpKdbgExtFileCache / ExpKdbgExtDefWrites — smaller KDBG `!filecache` and
// `!defwrites` extensions. Same `kdbp_print!` macro pattern as
// ExpKdbgExtHandle. Both are i386/amd64 only.
// ============================================================================

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "C" {
    static mut CcTotalDirtyPages: ntkrnl_sys::base::ULONG;
    static mut CcDirtyPageThreshold: ntkrnl_sys::base::ULONG;
    static mut MmAvailablePages: ntkrnl_sys::ddk::PFN_NUMBER;
    static mut MmThrottleTop: ntkrnl_sys::base::ULONG;
    static mut MmThrottleBottom: ntkrnl_sys::base::ULONG;
    static mut MmModifiedPageListHead: ntkrnl_sys::ddk::_MMPFNLIST;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn ExpKdbgExtFileCache(
    _argc: ntkrnl_sys::base::ULONG,
    _argv: *mut *mut core::ffi::c_char,
) -> ntkrnl_sys::base::BOOLEAN {
    crate::kdbp_print!(b"  File cache display not available (CC subsystem is in Rust)\n\0");
    1
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn ExpKdbgExtDefWrites(
    _argc: ntkrnl_sys::base::ULONG,
    _argv: *mut *mut core::ffi::c_char,
) -> ntkrnl_sys::base::BOOLEAN {
    use ntkrnl_sys::base::{PAGE_SIZE, ULONG};
    let page_size = PAGE_SIZE as ULONG;

    crate::kdbp_print!(
        b"CcTotalDirtyPages:\t%lu (%lu Kb)\n\0",
        CcTotalDirtyPages,
        (CcTotalDirtyPages.wrapping_mul(page_size)) / 1024
    );
    crate::kdbp_print!(
        b"CcDirtyPageThreshold:\t%lu (%lu Kb)\n\0",
        CcDirtyPageThreshold,
        (CcDirtyPageThreshold.wrapping_mul(page_size)) / 1024
    );
    crate::kdbp_print!(
        b"MmAvailablePages:\t%lu (%lu Kb)\n\0",
        MmAvailablePages as ULONG,
        ((MmAvailablePages as ULONG).wrapping_mul(page_size)) / 1024
    );
    crate::kdbp_print!(
        b"MmThrottleTop:\t\t%lu (%lu Kb)\n\0",
        MmThrottleTop,
        (MmThrottleTop.wrapping_mul(page_size)) / 1024
    );
    crate::kdbp_print!(
        b"MmThrottleBottom:\t%lu (%lu Kb)\n\0",
        MmThrottleBottom,
        (MmThrottleBottom.wrapping_mul(page_size)) / 1024
    );
    let modified_total = MmModifiedPageListHead.Total as ULONG;
    crate::kdbp_print!(
        b"MmModifiedPageListHead.Total:\t%lu (%lu Kb)\n\0",
        modified_total,
        (modified_total.wrapping_mul(page_size)) / 1024
    );

    if CcTotalDirtyPages >= CcDirtyPageThreshold {
        crate::kdbp_print!(
            b"CcTotalDirtyPages above the threshold, writes should be throttled\n\0"
        );
    } else if CcTotalDirtyPages.wrapping_add(64) >= CcDirtyPageThreshold {
        crate::kdbp_print!(
            b"CcTotalDirtyPages within 64 pages of threshold, writes may be throttled\n\0"
        );
    } else {
        crate::kdbp_print!(
            b"CcTotalDirtyPages below the threshold, writes should not be throttled\n\0"
        );
    }
    1
}
