use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    CLONG, PBASE_MCB, PLARGE_MCB, PLONGLONG, PRTL_GENERIC_TABLE,
    NPAGED_LOOKASIDE_LIST, _PAGED_LOOKASIDE_LIST,
    RtlDeleteElementGenericTable, RtlEnumerateGenericTable,
    RtlGetElementGenericTable, RtlInitializeGenericTable,
    RtlInsertElementGenericTable, RtlLookupElementGenericTable,
    RtlNumberGenericTableElements,
    ExInitializeNPagedLookasideList, ExInitializePagedLookasideList,
    RTL_GENERIC_COMPARE_RESULTS, RTL_GENERIC_TABLE,
    _RTL_GENERIC_COMPARE_RESULTS_GenericEqual,
    _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan,
    _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn ExAllocateFromPagedLookasideList(Lookaside: *mut ntkrnl_sys::ddk::_PAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToPagedLookasideList(Lookaside: *mut ntkrnl_sys::ddk::_PAGED_LOOKASIDE_LIST, Entry: PVOID);
}

extern "C" {
    fn ExAllocateFromNPagedLookasideList(Lookaside: *mut ntkrnl_sys::ddk::_NPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToNPagedLookasideList(Lookaside: *mut ntkrnl_sys::ddk::_NPAGED_LOOKASIDE_LIST, Entry: PVOID);
}

static mut FsRtlFirstMappingLookasideList: _PAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
static mut FsRtlFastMutexLookasideList: NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };

#[repr(C)]
struct LARGE_MCB_MAPPING_ENTRY {
    RunStartVbn: LARGE_INTEGER,
    RunEndVbn: LARGE_INTEGER,
    StartingLbn: LARGE_INTEGER,
}

type PLARGE_MCB_MAPPING_ENTRY = *mut LARGE_MCB_MAPPING_ENTRY;

#[repr(C)]
struct LARGE_MCB_MAPPING {
    Table: RTL_GENERIC_TABLE,
}

type PLARGE_MCB_MAPPING = *mut LARGE_MCB_MAPPING;

#[repr(C)]
struct BASE_MCB_INTERNAL {
    MaximumPairCount: ULONG,
    PairCount: ULONG,
    PoolType: USHORT,
    Flags: USHORT,
    Mapping: PLARGE_MCB_MAPPING,
}

type PBASE_MCB_INTERNAL = *mut BASE_MCB_INTERNAL;

const MAXIMUM_PAIR_COUNT: ULONG = 0xFFFFFFFF;
const IFS_POOL_TAG: ULONG = 0x74725346;

unsafe extern "system" fn McbMappingAllocate(Table: PRTL_GENERIC_TABLE, Bytes: CLONG) -> PVOID {
    let mcb = (*Table).TableContext as PBASE_MCB;
    let result = ExAllocatePoolWithTag((*mcb).PoolType as u32, Bytes as usize, 0x4C4D4342);
    result
}

unsafe extern "system" fn McbMappingFree(Table: PRTL_GENERIC_TABLE, Buffer: PVOID) {
    ExFreePoolWithTag(Buffer, 0x4C4D4342);
}

unsafe extern "system" fn McbMappingCompare(
    _Table: PRTL_GENERIC_TABLE,
    PtrA: PVOID,
    PtrB: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    let a = PtrA as PLARGE_MCB_MAPPING_ENTRY;
    let b = PtrB as PLARGE_MCB_MAPPING_ENTRY;

    if (*a).RunStartVbn.QuadPart == (*b).RunStartVbn.QuadPart
        && (*a).RunEndVbn.QuadPart == (*b).RunEndVbn.QuadPart
    {
        _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
    } else if (*a).RunEndVbn.QuadPart <= (*b).RunStartVbn.QuadPart {
        _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan
    } else if (*a).RunEndVbn.QuadPart >= (*b).RunStartVbn.QuadPart {
        _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan
    } else {
        _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
    }
}

unsafe extern "system" fn McbMappingIntersectCompare(
    _Table: PRTL_GENERIC_TABLE,
    PtrA: PVOID,
    PtrB: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    let a = PtrA as PLARGE_MCB_MAPPING_ENTRY;
    let b = PtrB as PLARGE_MCB_MAPPING_ENTRY;

    if (*a).RunStartVbn.QuadPart <= (*b).RunStartVbn.QuadPart
        && (*a).RunEndVbn.QuadPart > (*b).RunStartVbn.QuadPart
    {
        _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
    } else if (*a).RunStartVbn.QuadPart >= (*b).RunStartVbn.QuadPart
        && (*b).RunEndVbn.QuadPart > (*a).RunStartVbn.QuadPart
    {
        _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
    } else if (*a).RunStartVbn.QuadPart < (*b).RunStartVbn.QuadPart {
        _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan
    } else if (*a).RunStartVbn.QuadPart > (*b).RunStartVbn.QuadPart {
        _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan
    } else {
        _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAddBaseMcbEntry(
    OpaqueMcb: PBASE_MCB,
    Vbn: LONGLONG,
    Lbn: LONGLONG,
    SectorCount: LONGLONG,
) -> BOOLEAN {
    let mut result = 1u8;
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;
    let mut node: LARGE_MCB_MAPPING_ENTRY = core::mem::zeroed();
    let mut needle_run: LARGE_MCB_MAPPING_ENTRY = core::mem::zeroed();
    let mut new_element: BOOLEAN = 0;
    let mut int_lbn: LONGLONG = 0;
    let mut int_sector_count: LONGLONG = 0;

    if Vbn < 0 {
        result = 0;
        return result;
    }

    if SectorCount <= 0 {
        result = 0;
        return result;
    }

    let int_result = FsRtlLookupBaseMcbEntry(
        OpaqueMcb,
        Vbn,
        &mut int_lbn,
        &mut int_sector_count,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if int_result != 0 {
        if int_lbn != -1 && int_lbn != Lbn {
            result = 0;
            return result;
        }

        if (int_lbn != -1) && (int_sector_count >= SectorCount) {
            return result;
        }
    }

    FsRtlRemoveBaseMcbEntry(OpaqueMcb, Vbn, SectorCount);

    node.RunStartVbn.QuadPart = Vbn;
    node.RunEndVbn.QuadPart = Vbn + SectorCount;
    node.StartingLbn.QuadPart = Lbn;

    needle_run.RunStartVbn.QuadPart = node.RunStartVbn.QuadPart - 1;
    needle_run.RunEndVbn.QuadPart = needle_run.RunStartVbn.QuadPart + 1;
    needle_run.StartingLbn.QuadPart = !0u64 as i64;
    (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingIntersectCompare));
    let lower_run =
        RtlLookupElementGenericTable(&mut (*(*mcb).Mapping).Table, &needle_run as *const _ as PVOID)
            as PLARGE_MCB_MAPPING_ENTRY;
    if !lower_run.is_null()
        && ((*lower_run).StartingLbn.QuadPart
            + ((*lower_run).RunEndVbn.QuadPart - (*lower_run).RunStartVbn.QuadPart)
            == node.StartingLbn.QuadPart)
    {
        node.RunStartVbn.QuadPart = (*lower_run).RunStartVbn.QuadPart;
        node.StartingLbn.QuadPart = (*lower_run).StartingLbn.QuadPart;
        (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingCompare));
        RtlDeleteElementGenericTable(&mut (*(*mcb).Mapping).Table, lower_run as PVOID);
        (*mcb).PairCount -= 1;
    }

    needle_run.RunStartVbn.QuadPart = node.RunEndVbn.QuadPart;
    needle_run.RunEndVbn.QuadPart = needle_run.RunStartVbn.QuadPart + 1;
    (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingIntersectCompare));
    let higher_run =
        RtlLookupElementGenericTable(&mut (*(*mcb).Mapping).Table, &needle_run as *const _ as PVOID)
            as PLARGE_MCB_MAPPING_ENTRY;
    if !higher_run.is_null() && (node.StartingLbn.QuadPart <= (*higher_run).StartingLbn.QuadPart) {
        node.RunEndVbn.QuadPart = (*higher_run).RunEndVbn.QuadPart;
        (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingCompare));
        RtlDeleteElementGenericTable(&mut (*(*mcb).Mapping).Table, higher_run as PVOID);
        (*mcb).PairCount -= 1;
    }
    (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingCompare));

    RtlInsertElementGenericTable(
        &mut (*(*mcb).Mapping).Table,
        &node as *const _ as PVOID,
        core::mem::size_of::<LARGE_MCB_MAPPING_ENTRY>() as ULONG,
        &mut new_element,
    );
    (*mcb).PairCount += 1;

    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAddLargeMcbEntry(
    Mcb: PLARGE_MCB,
    Vbn: LONGLONG,
    Lbn: LONGLONG,
    SectorCount: LONGLONG,
) -> BOOLEAN {
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    let result = FsRtlAddBaseMcbEntry(&mut (*Mcb).BaseMcb, Vbn, Lbn, SectorCount);
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlGetNextBaseMcbEntry(
    OpaqueMcb: PBASE_MCB,
    RunIndex: ULONG,
    Vbn: PLONGLONG,
    Lbn: PLONGLONG,
    SectorCount: PLONGLONG,
) -> BOOLEAN {
    let mut result = 0u8;
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;
    let mut current_index: ULONG = 0;
    let mut last_vbn: ULONGLONG = 0;
    let mut last_sector_count: ULONGLONG = 0;

    let mut run =
        RtlEnumerateGenericTable(&mut (*(*mcb).Mapping).Table, 1u8) as PLARGE_MCB_MAPPING_ENTRY;
    while !run.is_null() {
        if (*run).RunStartVbn.QuadPart as u64 > (last_vbn + last_sector_count) {
            if RunIndex == current_index {
                *Vbn = (last_vbn + last_sector_count) as i64;
                *Lbn = -1;
                *SectorCount = (*run).RunStartVbn.QuadPart - *Vbn;

                result = 1;
                return result;
            }

            current_index += 1;
        }

        if RunIndex == current_index {
            *Vbn = (*run).RunStartVbn.QuadPart;
            *Lbn = (*run).StartingLbn.QuadPart;
            *SectorCount = (*run).RunEndVbn.QuadPart - (*run).RunStartVbn.QuadPart;

            result = 1;
            return result;
        }

        current_index += 1;
        last_vbn = (*run).RunStartVbn.QuadPart as u64;
        last_sector_count = ((*run).RunEndVbn.QuadPart - (*run).RunStartVbn.QuadPart) as u64;

        run = RtlEnumerateGenericTable(&mut (*(*mcb).Mapping).Table, 0u8) as PLARGE_MCB_MAPPING_ENTRY;
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlGetNextLargeMcbEntry(
    Mcb: PLARGE_MCB,
    RunIndex: ULONG,
    Vbn: PLONGLONG,
    Lbn: PLONGLONG,
    SectorCount: PLONGLONG,
) -> BOOLEAN {
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    let result = FsRtlGetNextBaseMcbEntry(&mut (*Mcb).BaseMcb, RunIndex, Vbn, Lbn, SectorCount);
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeBaseMcb(OpaqueMcb: PBASE_MCB, PoolType: POOL_TYPE) {
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;

    if PoolType == PagedPool {
        (*mcb).Mapping = ExAllocateFromPagedLookasideList(
            core::ptr::addr_of_mut!(FsRtlFirstMappingLookasideList),
        ) as PLARGE_MCB_MAPPING;
    } else {
        (*mcb).Mapping = ExAllocatePoolWithTag(
            PoolType | POOL_RAISE_IF_ALLOCATION_FAILURE,
            core::mem::size_of::<LARGE_MCB_MAPPING>(),
            0x46534243,
        ) as PLARGE_MCB_MAPPING;
    }

    (*mcb).PoolType = PoolType as USHORT;
    (*mcb).PairCount = 0;
    (*mcb).MaximumPairCount = MAXIMUM_PAIR_COUNT;
    RtlInitializeGenericTable(
        &mut (*(*mcb).Mapping).Table,
        Some(abi_cast!(McbMappingCompare)),
        Some(abi_cast!(McbMappingAllocate)),
        Some(abi_cast!(McbMappingFree)),
        mcb as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeLargeMcb(Mcb: PLARGE_MCB, PoolType: POOL_TYPE) {
    (*Mcb).GuardedMutex = ExAllocateFromNPagedLookasideList(
        core::ptr::addr_of_mut!(FsRtlFastMutexLookasideList),
    ) as ntkrnl_sys::ddk::PKGUARDED_MUTEX;

    KeInitializeGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);

    FsRtlInitializeBaseMcb(&mut (*Mcb).BaseMcb, PoolType);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeLargeMcbs() {
    ExInitializePagedLookasideList(
        core::ptr::addr_of_mut!(FsRtlFirstMappingLookasideList),
        None,
        None,
        POOL_RAISE_IF_ALLOCATION_FAILURE,
        core::mem::size_of::<LARGE_MCB_MAPPING>() as _,
        IFS_POOL_TAG,
        0,
    );

    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!(FsRtlFastMutexLookasideList),
        None,
        None,
        POOL_RAISE_IF_ALLOCATION_FAILURE,
        core::mem::size_of::<KGUARDED_MUTEX>() as _,
        IFS_POOL_TAG,
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupBaseMcbEntry(
    OpaqueMcb: PBASE_MCB,
    Vbn: LONGLONG,
    Lbn: PLONGLONG,
    SectorCountFromLbn: PLONGLONG,
    StartingLbn: PLONGLONG,
    SectorCountFromStartingLbn: PLONGLONG,
    Index: *mut ULONG,
) -> BOOLEAN {
    let Index = opaque_ptr(Index);
    let mut result = 0u8;
    let mut i: ULONG = 0;
    let mut last_vbn: LONGLONG = 0;
    let mut last_lbn: LONGLONG = 0;
    let mut count: LONGLONG = 0;

    loop {
        if FsRtlGetNextBaseMcbEntry(OpaqueMcb, i, &mut last_vbn, &mut last_lbn, &mut count) == 0 {
            break;
        }

        if Vbn < last_vbn + count {
            if !Lbn.is_null() {
                if last_lbn == -1 {
                    *Lbn = -1;
                } else {
                    *Lbn = last_lbn + (Vbn - last_vbn);
                }
            }

            if !SectorCountFromLbn.is_null() {
                *SectorCountFromLbn = last_vbn + count - Vbn;
            }
            if !StartingLbn.is_null() {
                *StartingLbn = last_lbn;
            }
            if !SectorCountFromStartingLbn.is_null() {
                *SectorCountFromStartingLbn = last_vbn + count - last_vbn;
            }
            if !Index.is_null() {
                *Index = i;
            }

            result = 1;
            return result;
        }

        i += 1;
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupLargeMcbEntry(
    Mcb: PLARGE_MCB,
    Vbn: LONGLONG,
    Lbn: PLONGLONG,
    SectorCountFromLbn: PLONGLONG,
    StartingLbn: PLONGLONG,
    SectorCountFromStartingLbn: PLONGLONG,
    Index: *mut ULONG,
) -> BOOLEAN {
    let Index = opaque_ptr(Index);
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    let result = FsRtlLookupBaseMcbEntry(
        &mut (*Mcb).BaseMcb,
        Vbn,
        Lbn,
        SectorCountFromLbn,
        StartingLbn,
        SectorCountFromStartingLbn,
        Index,
    );
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    result
}

unsafe fn FsRtlLookupLastLargeMcbEntryAndIndex_internal(
    Mcb: PBASE_MCB_INTERNAL,
    Vbn: PLONGLONG,
    Lbn: PLONGLONG,
    Index: *mut ULONG,
) -> BOOLEAN {
    let mut run_index: ULONG = 0;
    let mut run_found: PLARGE_MCB_MAPPING_ENTRY = core::ptr::null_mut();
    let mut last_vbn: LONGLONG = 0;

    let mut run =
        RtlEnumerateGenericTable(&mut (*(*Mcb).Mapping).Table, 1u8) as PLARGE_MCB_MAPPING_ENTRY;
    while !run.is_null() {
        if (*run).RunStartVbn.QuadPart > last_vbn {
            run_index += 1;
        }
        last_vbn = (*run).RunEndVbn.QuadPart;
        run_index += 1;
        run_found = run;

        run = RtlEnumerateGenericTable(&mut (*(*Mcb).Mapping).Table, 0u8) as PLARGE_MCB_MAPPING_ENTRY;
    }

    if run_found.is_null() {
        return 0;
    }

    if !Vbn.is_null() {
        *Vbn = (*run_found).RunEndVbn.QuadPart - 1;
    }
    if !Lbn.is_null() {
        *Lbn = (*run_found).StartingLbn.QuadPart
            + ((*run_found).RunEndVbn.QuadPart - (*run_found).RunStartVbn.QuadPart)
            - 1;
    }
    if !Index.is_null() {
        *Index = run_index - 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupLastBaseMcbEntryAndIndex(
    OpaqueMcb: PBASE_MCB,
    LargeVbn: PLONGLONG,
    LargeLbn: PLONGLONG,
    Index: *mut ULONG,
) -> BOOLEAN {
    let Index = opaque_ptr(Index);
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;
    FsRtlLookupLastLargeMcbEntryAndIndex_internal(mcb, LargeVbn, LargeLbn, Index)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupLastLargeMcbEntryAndIndex(
    OpaqueMcb: PLARGE_MCB,
    LargeVbn: PLONGLONG,
    LargeLbn: PLONGLONG,
    Index: *mut ULONG,
) -> BOOLEAN {
    let Index = opaque_ptr(Index);
    KeAcquireGuardedMutex((*OpaqueMcb).GuardedMutex as PKGUARDED_MUTEX);
    let result =
        FsRtlLookupLastBaseMcbEntryAndIndex(&mut (*OpaqueMcb).BaseMcb, LargeVbn, LargeLbn, Index);
    KeReleaseGuardedMutex((*OpaqueMcb).GuardedMutex as PKGUARDED_MUTEX);
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupLastBaseMcbEntry(
    OpaqueMcb: PBASE_MCB,
    Vbn: PLONGLONG,
    Lbn: PLONGLONG,
) -> BOOLEAN {
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;
    FsRtlLookupLastLargeMcbEntryAndIndex_internal(mcb, Vbn, Lbn, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupLastLargeMcbEntry(
    Mcb: PLARGE_MCB,
    Vbn: PLONGLONG,
    Lbn: PLONGLONG,
) -> BOOLEAN {
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    let result = FsRtlLookupLastBaseMcbEntry(&mut (*Mcb).BaseMcb, Vbn, Lbn);
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNumberOfRunsInBaseMcb(OpaqueMcb: PBASE_MCB) -> ULONG {
    let mut number_of_runs: ULONG = 0;
    let mut vbn: LONGLONG = 0;
    let mut lbn: LONGLONG = 0;
    let mut count: LONGLONG = 0;
    let mut i: i32 = 0;

    loop {
        if FsRtlGetNextBaseMcbEntry(OpaqueMcb, i as ULONG, &mut vbn, &mut lbn, &mut count) == 0 {
            break;
        }
        number_of_runs += 1;
        i += 1;
    }

    number_of_runs
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNumberOfRunsInLargeMcb(Mcb: PLARGE_MCB) -> ULONG {
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    let number_of_runs = FsRtlNumberOfRunsInBaseMcb(&mut (*Mcb).BaseMcb);
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    number_of_runs
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRemoveBaseMcbEntry(
    OpaqueMcb: PBASE_MCB,
    Vbn: LONGLONG,
    SectorCount: LONGLONG,
) -> BOOLEAN {
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;
    let mut needle_run: LARGE_MCB_MAPPING_ENTRY = core::mem::zeroed();
    let mut result = 1u8;

    if Vbn < 0 || SectorCount <= 0 {
        result = 0;
        return result;
    }

    if Vbn + SectorCount <= Vbn {
        result = 0;
        return result;
    }

    needle_run.RunStartVbn.QuadPart = Vbn;
    needle_run.RunEndVbn.QuadPart = Vbn + SectorCount;
    needle_run.StartingLbn.QuadPart = -1;

    (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingIntersectCompare));
    loop {
        let haystack_run = RtlLookupElementGenericTable(
            &mut (*(*mcb).Mapping).Table,
            &needle_run as *const _ as PVOID,
        ) as PLARGE_MCB_MAPPING_ENTRY;
        if haystack_run.is_null() {
            break;
        }

        if (*haystack_run).RunStartVbn.QuadPart < needle_run.RunStartVbn.QuadPart {
            let haystack_run_end = (*haystack_run).RunEndVbn.QuadPart;

            (*haystack_run).RunEndVbn.QuadPart = needle_run.RunStartVbn.QuadPart;

            if haystack_run_end > needle_run.RunEndVbn.QuadPart {
                let mut tail_run: LARGE_MCB_MAPPING_ENTRY = core::mem::zeroed();
                let mut new_element: BOOLEAN = 0;

                tail_run.RunStartVbn.QuadPart = needle_run.RunEndVbn.QuadPart;
                tail_run.RunEndVbn.QuadPart = haystack_run_end;
                tail_run.StartingLbn.QuadPart = (*haystack_run).StartingLbn.QuadPart
                    + (needle_run.RunEndVbn.QuadPart - (*haystack_run).RunStartVbn.QuadPart);

                (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingCompare));

                RtlInsertElementGenericTable(
                    &mut (*(*mcb).Mapping).Table,
                    &tail_run as *const _ as PVOID,
                    core::mem::size_of::<LARGE_MCB_MAPPING_ENTRY>() as ULONG,
                    &mut new_element,
                );
                (*mcb).PairCount += 1;

                (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingIntersectCompare));
            }
        } else if (*haystack_run).RunEndVbn.QuadPart > needle_run.RunEndVbn.QuadPart {
            let haystack_run_start = (*haystack_run).RunStartVbn.QuadPart;
            let haystack_starting_lbn = (*haystack_run).StartingLbn.QuadPart;

            (*haystack_run).RunStartVbn.QuadPart = needle_run.RunEndVbn.QuadPart;
            (*haystack_run).StartingLbn.QuadPart +=
                needle_run.RunEndVbn.QuadPart - haystack_run_start;

            if haystack_run_start < needle_run.RunStartVbn.QuadPart {
                let mut head_run: LARGE_MCB_MAPPING_ENTRY = core::mem::zeroed();
                let mut new_element: BOOLEAN = 0;

                head_run.RunStartVbn.QuadPart = haystack_run_start;
                head_run.RunEndVbn.QuadPart = needle_run.RunStartVbn.QuadPart;
                head_run.StartingLbn.QuadPart = haystack_starting_lbn;

                (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingCompare));

                RtlInsertElementGenericTable(
                    &mut (*(*mcb).Mapping).Table,
                    &head_run as *const _ as PVOID,
                    core::mem::size_of::<LARGE_MCB_MAPPING_ENTRY>() as ULONG,
                    &mut new_element,
                );
                (*mcb).PairCount += 1;

                (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingIntersectCompare));
            }
        } else {
            (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingCompare));
            RtlDeleteElementGenericTable(&mut (*(*mcb).Mapping).Table, haystack_run as PVOID);
            (*mcb).PairCount -= 1;
            (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingIntersectCompare));
        }
    }
    (*(*mcb).Mapping).Table.CompareRoutine = Some(abi_cast!(McbMappingCompare));

    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRemoveLargeMcbEntry(
    Mcb: PLARGE_MCB,
    Vbn: LONGLONG,
    SectorCount: LONGLONG,
) {
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    FsRtlRemoveBaseMcbEntry(&mut (*Mcb).BaseMcb, Vbn, SectorCount);
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlResetBaseMcb(OpaqueMcb: PBASE_MCB) {
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;

    while RtlNumberGenericTableElements(&mut (*(*mcb).Mapping).Table) != 0 {
        let element = RtlGetElementGenericTable(&mut (*(*mcb).Mapping).Table, 0)
            as PLARGE_MCB_MAPPING_ENTRY;
        if element.is_null() {
            break;
        }
        RtlDeleteElementGenericTable(&mut (*(*mcb).Mapping).Table, element as PVOID);
    }

    (*mcb).PairCount = 0;
    (*mcb).MaximumPairCount = 0;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlResetLargeMcb(Mcb: PLARGE_MCB, SelfSynchronized: BOOLEAN) {
    if SelfSynchronized == 0 {
        KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    }

    FsRtlResetBaseMcb(&mut (*Mcb).BaseMcb);

    if SelfSynchronized == 0 {
        KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlSplitBaseMcb(
    OpaqueMcb: PBASE_MCB,
    Vbn: LONGLONG,
    Amount: LONGLONG,
) -> BOOLEAN {
    let mcb = OpaqueMcb as PBASE_MCB_INTERNAL;
    let mut insert_lower_run: PLARGE_MCB_MAPPING_ENTRY = core::ptr::null_mut();
    let mut existing_run: PLARGE_MCB_MAPPING_ENTRY = core::ptr::null_mut();
    let mut new_element: BOOLEAN = 0;

    let mut run =
        RtlEnumerateGenericTable(&mut (*(*mcb).Mapping).Table, 1u8) as PLARGE_MCB_MAPPING_ENTRY;
    while !run.is_null() {
        if Vbn >= (*run).RunEndVbn.QuadPart {
            run = RtlEnumerateGenericTable(&mut (*(*mcb).Mapping).Table, 0u8)
                as PLARGE_MCB_MAPPING_ENTRY;
            continue;
        }

        if Vbn < (*run).RunEndVbn.QuadPart {
            (*run).RunStartVbn.QuadPart = Vbn;
            insert_lower_run = core::ptr::null_mut();
        }

        (*run).RunStartVbn.QuadPart += Amount;
        (*run).RunEndVbn.QuadPart += Amount;

        run = RtlEnumerateGenericTable(&mut (*(*mcb).Mapping).Table, 0u8) as PLARGE_MCB_MAPPING_ENTRY;
    }

    if !insert_lower_run.is_null() {
        existing_run = RtlInsertElementGenericTable(
            &mut (*(*mcb).Mapping).Table,
            insert_lower_run as PVOID,
            core::mem::size_of::<LARGE_MCB_MAPPING_ENTRY>() as ULONG,
            &mut new_element,
        ) as PLARGE_MCB_MAPPING_ENTRY;
        (*mcb).PairCount += 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlSplitLargeMcb(
    Mcb: PLARGE_MCB,
    Vbn: LONGLONG,
    Amount: LONGLONG,
) -> BOOLEAN {
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    let result = FsRtlSplitBaseMcb(&mut (*Mcb).BaseMcb, Vbn, Amount);
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlTruncateBaseMcb(OpaqueMcb: PBASE_MCB, Vbn: LONGLONG) {
    FsRtlRemoveBaseMcbEntry(OpaqueMcb, Vbn, 0x7FFFFFFF - Vbn + 1);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlTruncateLargeMcb(Mcb: PLARGE_MCB, Vbn: LONGLONG) {
    KeAcquireGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
    FsRtlTruncateBaseMcb(&mut (*Mcb).BaseMcb, Vbn);
    KeReleaseGuardedMutex((*Mcb).GuardedMutex as PKGUARDED_MUTEX);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlUninitializeBaseMcb(Mcb: PBASE_MCB) {
    FsRtlResetBaseMcb(Mcb);

    if (*Mcb).PoolType as u32 == PagedPool  {
        ExFreeToPagedLookasideList(
            core::ptr::addr_of_mut!(FsRtlFirstMappingLookasideList),
            (*Mcb).Mapping as PVOID,
        );
    } else {
        ExFreePoolWithTag((*Mcb).Mapping as PVOID, 0x46534243);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlUninitializeLargeMcb(Mcb: PLARGE_MCB) {
    if !(*Mcb).GuardedMutex.is_null() {
        ExFreeToNPagedLookasideList(
            core::ptr::addr_of_mut!(FsRtlFastMutexLookasideList),
            (*Mcb).GuardedMutex as PVOID,
        );
        FsRtlUninitializeBaseMcb(&mut (*Mcb).BaseMcb);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL v2 - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/largemcb.c
//  * PURPOSE:         Large Mapped Control Block (MCB) support for File System Drivers
//  * PROGRAMMERS:     Aleksey Bragin <aleksey@reactos.org>
//  *                  Jan Kratochvil <project-captive@jankratochvil.net>
//  *                  Trevor Thompson
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MIN(x,y) (((x)<(y))?(x):(y))
// #define MAX(x,y) (((x)>(y))?(x):(y))
//
// /* GLOBALS *******************************************************************/
//
// _PAGED_LOOKASIDE_LIST FsRtlFirstMappingLookasideList;
// NPAGED_LOOKASIDE_LIST FsRtlFastMutexLookasideList;
//
// /* We use only real 'mapping' runs; we do not store 'holes' to our GTree. */
// typedef struct _LARGE_MCB_MAPPING_ENTRY // run
// {
//     LARGE_INTEGER RunStartVbn;
//     LARGE_INTEGER RunEndVbn;   /* RunStartVbn+SectorCount; that means +1 after the last sector */
//     LARGE_INTEGER StartingLbn; /* Lbn of 'RunStartVbn' */
// } LARGE_MCB_MAPPING_ENTRY, *PLARGE_MCB_MAPPING_ENTRY;
//
// typedef struct _LARGE_MCB_MAPPING // mcb_priv
// {
//     RTL_GENERIC_TABLE Table;
// } LARGE_MCB_MAPPING, *PLARGE_MCB_MAPPING;
//
// typedef struct _BASE_MCB_INTERNAL {
//     ULONG MaximumPairCount;
//     ULONG PairCount;
//     USHORT PoolType;
//     USHORT Flags;
//     PLARGE_MCB_MAPPING Mapping;
// } BASE_MCB_INTERNAL, *PBASE_MCB_INTERNAL;
//
// /*
// static LARGE_MCB_MAPPING_ENTRY StaticRunBelow0 = {
//     {{-1}}, // ignored
//     {{0}},
//     {{-1}}, // ignored
// };
// */
//
// static PVOID NTAPI McbMappingAllocate(PRTL_GENERIC_TABLE Table, CLONG Bytes)
// {
//     PVOID Result;
//     PBASE_MCB Mcb = (PBASE_MCB)Table->TableContext;
//     Result = ExAllocatePoolWithTag(Mcb->PoolType, Bytes, 'BCML');
//     DPRINT("McbMappingAllocate(%lu) => %p\n", Bytes, Result);
//     return Result;
// }
//
// static VOID NTAPI McbMappingFree(PRTL_GENERIC_TABLE Table, PVOID Buffer)
// {
//     DPRINT("McbMappingFree(%p)\n", Buffer);
//     ExFreePoolWithTag(Buffer, 'BCML');
// }
//
// static
// RTL_GENERIC_COMPARE_RESULTS
// NTAPI
// McbMappingCompare(PRTL_GENERIC_TABLE Table,
//                   PVOID PtrA,
//                   PVOID PtrB)
// {
//     PLARGE_MCB_MAPPING_ENTRY A = PtrA, B = PtrB;
//     RTL_GENERIC_COMPARE_RESULTS Res;
//
//     ASSERT(A);
//     ASSERT(B);
//
//     if (A->RunStartVbn.QuadPart == B->RunStartVbn.QuadPart && A->RunEndVbn.QuadPart == B->RunEndVbn.QuadPart)
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
//     else if (A->RunEndVbn.QuadPart <= B->RunStartVbn.QuadPart)
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan;
//     else if (A->RunEndVbn.QuadPart >= B->RunStartVbn.QuadPart)
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan;
//     else
//     {
//         ASSERT(FALSE);
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
//     }
//
//     return Res;
// }
//
// static RTL_GENERIC_COMPARE_RESULTS NTAPI McbMappingIntersectCompare(PRTL_GENERIC_TABLE Table, PVOID PtrA, PVOID PtrB)
// {
//     PLARGE_MCB_MAPPING_ENTRY A = PtrA, B = PtrB;
//     RTL_GENERIC_COMPARE_RESULTS Res;
//
//     if (A->RunStartVbn.QuadPart <= B->RunStartVbn.QuadPart && A->RunEndVbn.QuadPart > B->RunStartVbn.QuadPart)
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
//     else if (A->RunStartVbn.QuadPart >= B->RunStartVbn.QuadPart && B->RunEndVbn.QuadPart > A->RunStartVbn.QuadPart)
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
//     else if (A->RunStartVbn.QuadPart < B->RunStartVbn.QuadPart)
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan;
//     else if (A->RunStartVbn.QuadPart > B->RunStartVbn.QuadPart)
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan;
//     else
//         Res = _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
//
//     return Res;
// }
//
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  * @Mcb: #PLARGE_MCB initialized by FsRtlInitializeLargeMcb().
//  * %NULL value is forbidden.
//  * @Vbn: Starting virtual block number of the wished range.
//  * @Lbn: Starting logical block number of the wished range.
//  * @SectorCount: Length of the wished range.
//  * Value less or equal to %0 is forbidden; FIXME: Is the reject of %0 W32 compliant?
//  *
//  * Adds the specified range @Vbn ... @Vbn+@SectorCount-1 to @Mcb.
//  * Any mappings previously in this range are deleted first.
//  *
//  * Returns: %TRUE if successful.
//  */
// BOOLEAN
// NTAPI
// FsRtlAddBaseMcbEntry(IN PBASE_MCB OpaqueMcb,
//                      IN LONGLONG Vbn,
//                      IN LONGLONG Lbn,
//                      IN LONGLONG SectorCount)
// {
//     BOOLEAN Result = TRUE;
//     BOOLEAN IntResult;
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//     LARGE_MCB_MAPPING_ENTRY Node, NeedleRun;
//     PLARGE_MCB_MAPPING_ENTRY LowerRun, HigherRun;
//     BOOLEAN NewElement;
//     LONGLONG IntLbn, IntSectorCount;
//
//     DPRINT("FsRtlAddBaseMcbEntry(%p, %I64d, %I64d, %I64d)\n", OpaqueMcb, Vbn, Lbn, SectorCount);
//
//     if (Vbn < 0)
//     {
//         Result = FALSE;
//         goto quit;
//     }
//
//     if (SectorCount <= 0)
//     {
//         Result = FALSE;
//         goto quit;
//     }
//
//     IntResult = FsRtlLookupBaseMcbEntry(OpaqueMcb, Vbn, &IntLbn, &IntSectorCount, NULL, NULL, NULL);
//     if (IntResult)
//     {
//         if (IntLbn != -1 && IntLbn != Lbn)
//         {
//             Result = FALSE;
//             goto quit;
//         }
//
//         if ((IntLbn != -1) && (IntSectorCount >= SectorCount))
//         {
//             /* This is a no-op */
//             goto quit;
//         }
//     }
//
//     /* clean any possible previous entries in our range */
//     FsRtlRemoveBaseMcbEntry(OpaqueMcb, Vbn, SectorCount);
//
//     // We need to map [Vbn, Vbn+SectorCount) to [Lbn, Lbn+SectorCount),
//     // taking in account the fact that we need to merge these runs if
//     // they are adjacent or overlap, but fail if new run fully fits into another run
//
//     /* initially we think we will be inserted as a separate run */
//     Node.RunStartVbn.QuadPart = Vbn;
//     Node.RunEndVbn.QuadPart = Vbn + SectorCount;
//     Node.StartingLbn.QuadPart = Lbn;
//
//     /* optionally merge with lower run */
//     NeedleRun.RunStartVbn.QuadPart = Node.RunStartVbn.QuadPart - 1;
//     NeedleRun.RunEndVbn.QuadPart = NeedleRun.RunStartVbn.QuadPart + 1;
//     NeedleRun.StartingLbn.QuadPart = ~0ULL;
//     Mcb->Mapping->Table.CompareRoutine = McbMappingIntersectCompare;
//     if ((LowerRun = RtlLookupElementGenericTable(&Mcb->Mapping->Table, &NeedleRun)) &&
//         (LowerRun->StartingLbn.QuadPart + (LowerRun->RunEndVbn.QuadPart - LowerRun->RunStartVbn.QuadPart) == Node.StartingLbn.QuadPart))
//     {
//         ASSERT(LowerRun->RunEndVbn.QuadPart == Node.RunStartVbn.QuadPart);
//         Node.RunStartVbn.QuadPart = LowerRun->RunStartVbn.QuadPart;
//         Node.StartingLbn.QuadPart = LowerRun->StartingLbn.QuadPart;
//         Mcb->Mapping->Table.CompareRoutine = McbMappingCompare;
//         RtlDeleteElementGenericTable(&Mcb->Mapping->Table, LowerRun);
//         --Mcb->PairCount;
//         DPRINT("Intersecting lower run found (%I64d,%I64d) Lbn: %I64d\n", LowerRun->RunStartVbn.QuadPart, LowerRun->RunEndVbn.QuadPart, LowerRun->StartingLbn.QuadPart);
//     }
//
//     /* optionally merge with higher run */
//     NeedleRun.RunStartVbn.QuadPart = Node.RunEndVbn.QuadPart;
//     NeedleRun.RunEndVbn.QuadPart = NeedleRun.RunStartVbn.QuadPart + 1;
//     Mcb->Mapping->Table.CompareRoutine = McbMappingIntersectCompare;
//     if ((HigherRun = RtlLookupElementGenericTable(&Mcb->Mapping->Table, &NeedleRun)) &&
//         (Node.StartingLbn.QuadPart <= HigherRun->StartingLbn.QuadPart))
//     {
//         ASSERT(HigherRun->RunStartVbn.QuadPart == Node.RunEndVbn.QuadPart);
//         Node.RunEndVbn.QuadPart = HigherRun->RunEndVbn.QuadPart;
//         Mcb->Mapping->Table.CompareRoutine = McbMappingCompare;
//         RtlDeleteElementGenericTable(&Mcb->Mapping->Table, HigherRun);
//         --Mcb->PairCount;
//         DPRINT("Intersecting higher run found (%I64d,%I64d) Lbn: %I64d\n", HigherRun->RunStartVbn.QuadPart, HigherRun->RunEndVbn.QuadPart, HigherRun->StartingLbn.QuadPart);
//     }
//     Mcb->Mapping->Table.CompareRoutine = McbMappingCompare;
//
//     /* finally insert the resulting run */
//     RtlInsertElementGenericTable(&Mcb->Mapping->Table, &Node, sizeof(Node), &NewElement);
//     ++Mcb->PairCount;
//     ASSERT(NewElement);
//
//     // NB: Two consecutive runs can only be merged, if actual LBNs also match!
//
//     /* 1.
//             Existing->RunStartVbn
//             |
//             |///////|
//                 |/////////////|
//                 |
//                 Node->RunStartVbn
//
//         2.
//             Existing->RunStartVbn
//             |
//             |///////|
//         |//////|
//         |
//         Node->RunStartVbn
//
//         3.
//             Existing->RunStartVbn
//             |
//             |///////|
//                 |///|
//                 |
//                 Node->RunStartVbn
//
//         4.
//             Existing->RunStartVbn
//             |
//             |///////|
//         |///////////////|
//         |
//         Node->RunStartVbn
//
//
//     Situation with holes:
//     1. Holes at both ends
//     2. Hole at the right, new run merged with the previous run
//     3. Hole at the right, new run is not merged with the previous run
//     4. Hole at the left, new run merged with the next run
//     5. Hole at the left, new run is not merged with the next run
//     6. No holes, exact fit to merge with both previous and next runs
//     7. No holes, merges only with the next run
//     8. No holes, merges only with the previous run
//     9. No holes, does not merge with next or prev runs
//
//
//     Overwriting existing mapping is not possible and results in FALSE being returned
//     */
//
// quit:
//     DPRINT("FsRtlAddBaseMcbEntry(%p, %I64d, %I64d, %I64d) = %d\n", Mcb, Vbn, Lbn, SectorCount, Result);
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlAddLargeMcbEntry(IN PLARGE_MCB Mcb,
//                       IN LONGLONG Vbn,
//                       IN LONGLONG Lbn,
//                       IN LONGLONG SectorCount)
// {
//     BOOLEAN Result;
//
//     DPRINT("FsRtlAddLargeMcbEntry(%p, %I64d, %I64d, %I64d)\n", Mcb, Vbn, Lbn, SectorCount);
//
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     Result = FsRtlAddBaseMcbEntry(&(Mcb->BaseMcb),
//                                   Vbn,
//                                   Lbn,
//                                   SectorCount);
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
//
//     DPRINT("FsRtlAddLargeMcbEntry(%p, %I64d, %I64d, %I64d) = %d\n", Mcb, Vbn, Lbn, SectorCount, Result);
//
//     return Result;
// }
//
// /*
//  * @implemented
//  * @Mcb: #PLARGE_MCB initialized by FsRtlInitializeLargeMcb().
//  * %NULL value is forbidden.
//  * @RunIndex: Requested range index to retrieve.
//  * @Vbn: Returns the starting virtual block number of the wished range.
//  * %NULL pointer is forbidden.
//  * @Lbn: Returns the starting logical block number of the wished range (or -1 if it is a hole).
//  * %NULL pointer is forbidden.
//  * @SectorCount: Returns the length of the wished range.
//  * %NULL pointer is forbidden.
//  * Value less or equal to %0 is forbidden; FIXME: Is the reject of %0 W32 compliant?
//  *
//  * Retrieves the parameters of the specified run with index @RunIndex.
//  *
//  * Mapping %0 always starts at virtual block %0, either as 'hole' or as 'real' mapping.
//  * libcaptive does not store 'hole' information to its #GTree.
//  * Last run is always a 'real' run. 'hole' runs appear as mapping to constant @Lbn value %-1.
//  *
//  * Returns: %TRUE if successful.
//  */
// BOOLEAN
// NTAPI
// FsRtlGetNextBaseMcbEntry(IN PBASE_MCB OpaqueMcb,
//     IN ULONG RunIndex,
//     OUT PLONGLONG Vbn,
//     OUT PLONGLONG Lbn,
//     OUT PLONGLONG SectorCount)
// {
//     BOOLEAN Result = FALSE;
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//     PLARGE_MCB_MAPPING_ENTRY Run = NULL;
//     ULONG CurrentIndex = 0;
//     ULONGLONG LastVbn = 0;
//     ULONGLONG LastSectorCount = 0;
//
//     // Traverse the tree
//     for (Run = (PLARGE_MCB_MAPPING_ENTRY)RtlEnumerateGenericTable(&Mcb->Mapping->Table, TRUE);
//     Run;
//         Run = (PLARGE_MCB_MAPPING_ENTRY)RtlEnumerateGenericTable(&Mcb->Mapping->Table, FALSE))
//     {
//         // is the current index a hole?
//         if (Run->RunStartVbn.QuadPart > (LastVbn + LastSectorCount))
//         {
//             // Is this the index we're looking for?
//             if (RunIndex == CurrentIndex)
//             {
//                 *Vbn = LastVbn + LastSectorCount;
//                 *Lbn = -1;
//                 *SectorCount = Run->RunStartVbn.QuadPart - *Vbn;
//
//                 Result = TRUE;
//                 goto quit;
//             }
//
//             CurrentIndex++;
//         }
//
//         if (RunIndex == CurrentIndex)
//         {
//             *Vbn = Run->RunStartVbn.QuadPart;
//             *Lbn = Run->StartingLbn.QuadPart;
//             *SectorCount = Run->RunEndVbn.QuadPart - Run->RunStartVbn.QuadPart;
//
//             Result = TRUE;
//             goto quit;
//         }
//
//         CurrentIndex++;
//         LastVbn = Run->RunStartVbn.QuadPart;
//         LastSectorCount = Run->RunEndVbn.QuadPart - Run->RunStartVbn.QuadPart;
//     }
//
// quit:
//     DPRINT("FsRtlGetNextBaseMcbEntry(%p, %d, %p, %p, %p) = %d (%I64d, %I64d, %I64d)\n", Mcb, RunIndex, Vbn, Lbn, SectorCount, Result, *Vbn, *Lbn, *SectorCount);
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlGetNextLargeMcbEntry(IN PLARGE_MCB Mcb,
//                           IN ULONG RunIndex,
//                           OUT PLONGLONG Vbn,
//                           OUT PLONGLONG Lbn,
//                           OUT PLONGLONG SectorCount)
// {
//     BOOLEAN Result;
//
//     DPRINT("FsRtlGetNextLargeMcbEntry(%p, %d, %p, %p, %p)\n", Mcb, RunIndex, Vbn, Lbn, SectorCount);
//
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     Result = FsRtlGetNextBaseMcbEntry(&(Mcb->BaseMcb),
//                                       RunIndex,
//                                       Vbn,
//                                       Lbn,
//                                       SectorCount);
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
//
//     DPRINT("FsRtlGetNextLargeMcbEntry(%p, %d, %p, %p, %p) = %d (%I64d, %I64d, %I64d)\n", Mcb, RunIndex, Vbn, Lbn, SectorCount, Result, *Vbn, *Lbn, *SectorCount);
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlInitializeBaseMcb(IN PBASE_MCB OpaqueMcb,
//                        IN POOL_TYPE PoolType)
// {
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//
//     if (PoolType == PagedPool)
//     {
//         Mcb->Mapping = ExAllocateFromPagedLookasideList(&FsRtlFirstMappingLookasideList);
//     }
//     else
//     {
//         Mcb->Mapping = ExAllocatePoolWithTag(PoolType | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                              sizeof(LARGE_MCB_MAPPING),
//                                              'CBSF');
//     }
//
//     Mcb->PoolType = PoolType;
//     Mcb->PairCount = 0;
//     Mcb->MaximumPairCount = MAXIMUM_PAIR_COUNT;
//     RtlInitializeGenericTable(&Mcb->Mapping->Table,
//                               McbMappingCompare,
//                               McbMappingAllocate,
//                               McbMappingFree,
//                               Mcb);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlInitializeLargeMcb(IN PLARGE_MCB Mcb,
//                         IN POOL_TYPE PoolType)
// {
//     DPRINT("FsRtlInitializeLargeMcb(%p, %d)\n", Mcb, PoolType);
//
//     Mcb->GuardedMutex = ExAllocateFromNPagedLookasideList(&FsRtlFastMutexLookasideList);
//
//     KeInitializeGuardedMutex(Mcb->GuardedMutex);
//
//     _SEH2_TRY
//     {
//         FsRtlInitializeBaseMcb(&(Mcb->BaseMcb), PoolType);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ExFreeToNPagedLookasideList(&FsRtlFastMutexLookasideList,
//                                     Mcb->GuardedMutex);
//         Mcb->GuardedMutex = NULL;
//     }
//     _SEH2_END;
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("INIT")
// VOID
// NTAPI
// FsRtlInitializeLargeMcbs(VOID)
// {
//     /* Initialize the list for the MCB */
//     ExInitializePagedLookasideList(&FsRtlFirstMappingLookasideList,
//                                    NULL,
//                                    NULL,
//                                    POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                    sizeof(LARGE_MCB_MAPPING),
//                                    IFS_POOL_TAG,
//                                    0); /* FIXME: Should be 4 */
//
//     /* Initialize the list for the guarded mutex */
//     ExInitializeNPagedLookasideList(&FsRtlFastMutexLookasideList,
//                                     NULL,
//                                     NULL,
//                                     POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                     sizeof(KGUARDED_MUTEX),
//                                     IFS_POOL_TAG,
//                                     0); /* FIXME: Should be 32 */
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupBaseMcbEntry(IN PBASE_MCB OpaqueMcb,
//     IN LONGLONG Vbn,
//     OUT PLONGLONG Lbn OPTIONAL,
//     OUT PLONGLONG SectorCountFromLbn OPTIONAL,
//     OUT PLONGLONG StartingLbn OPTIONAL,
//     OUT PLONGLONG SectorCountFromStartingLbn OPTIONAL,
//     OUT PULONG Index OPTIONAL)
// {
//     BOOLEAN Result = FALSE;
//     ULONG i;
//     LONGLONG LastVbn = 0, LastLbn = 0, Count = 0;   // the last values we've found during traversal
//
//     DPRINT("FsRtlLookupBaseMcbEntry(%p, %I64d, %p, %p, %p, %p, %p)\n", OpaqueMcb, Vbn, Lbn, SectorCountFromLbn, StartingLbn, SectorCountFromStartingLbn, Index);
//
//     for (i = 0; FsRtlGetNextBaseMcbEntry(OpaqueMcb, i, &LastVbn, &LastLbn, &Count); i++)
//     {
//         // have we reached the target mapping?
//         if (Vbn < LastVbn + Count)
//         {
//             if (Lbn)
//             {
//                 if (LastLbn == -1)
//                     *Lbn = -1;
//                 else
//                     *Lbn = LastLbn + (Vbn - LastVbn);
//             }
//
//             if (SectorCountFromLbn)
//                 *SectorCountFromLbn = LastVbn + Count - Vbn;
//             if (StartingLbn)
//                 *StartingLbn = LastLbn;
//             if (SectorCountFromStartingLbn)
//                 *SectorCountFromStartingLbn = LastVbn + Count - LastVbn;
//             if (Index)
//                 *Index = i;
//
//             Result = TRUE;
//             goto quit;
//         }
//     }
//
// quit:
//     DPRINT("FsRtlLookupBaseMcbEntry(%p, %I64d, %p, %p, %p, %p, %p) = %d (%I64d, %I64d, %I64d, %I64d, %d)\n",
//            OpaqueMcb, Vbn, Lbn, SectorCountFromLbn, StartingLbn, SectorCountFromStartingLbn, Index, Result,
//            (Lbn ? *Lbn : (ULONGLONG)-1), (SectorCountFromLbn ? *SectorCountFromLbn : (ULONGLONG)-1), (StartingLbn ? *StartingLbn : (ULONGLONG)-1),
//            (SectorCountFromStartingLbn ? *SectorCountFromStartingLbn : (ULONGLONG)-1), (Index ? *Index : (ULONG)-1));
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupLargeMcbEntry(IN PLARGE_MCB Mcb,
//                          IN LONGLONG Vbn,
//                          OUT PLONGLONG Lbn OPTIONAL,
//                          OUT PLONGLONG SectorCountFromLbn OPTIONAL,
//                          OUT PLONGLONG StartingLbn OPTIONAL,
//                          OUT PLONGLONG SectorCountFromStartingLbn OPTIONAL,
//                          OUT PULONG Index OPTIONAL)
// {
//     BOOLEAN Result;
//
//     DPRINT("FsRtlLookupLargeMcbEntry(%p, %I64d, %p, %p, %p, %p, %p)\n", Mcb, Vbn, Lbn, SectorCountFromLbn, StartingLbn, SectorCountFromStartingLbn, Index);
//
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     Result = FsRtlLookupBaseMcbEntry(&(Mcb->BaseMcb),
//                                      Vbn,
//                                      Lbn,
//                                      SectorCountFromLbn,
//                                      StartingLbn,
//                                      SectorCountFromStartingLbn,
//                                      Index);
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
//
//     DPRINT("FsRtlLookupLargeMcbEntry(%p, %I64d, %p, %p, %p, %p, %p) = %d (%I64d, %I64d, %I64d, %I64d, %d)\n",
//            Mcb, Vbn, Lbn, SectorCountFromLbn, StartingLbn, SectorCountFromStartingLbn, Index, Result,
//            (Lbn ? *Lbn : (ULONGLONG)-1), (SectorCountFromLbn ? *SectorCountFromLbn : (ULONGLONG)-1), (StartingLbn ? *StartingLbn : (ULONGLONG)-1),
//            (SectorCountFromStartingLbn ? *SectorCountFromStartingLbn : (ULONGLONG)-1), (Index ? *Index : (ULONG)-1));
//
//     return Result;
// }
//
// static BOOLEAN
// NTAPI
// FsRtlLookupLastLargeMcbEntryAndIndex_internal(IN PBASE_MCB_INTERNAL Mcb,
//                                               OUT PLONGLONG Vbn,
//                                               OUT PLONGLONG Lbn,
//                                               OUT PULONG Index OPTIONAL)
// {
//     ULONG RunIndex = 0;
//     PLARGE_MCB_MAPPING_ENTRY Run, RunFound = NULL;
//     LONGLONG LastVbn = 0;
//
//     for (Run = (PLARGE_MCB_MAPPING_ENTRY)RtlEnumerateGenericTable(&Mcb->Mapping->Table, TRUE);
//         Run;
//         Run = (PLARGE_MCB_MAPPING_ENTRY)RtlEnumerateGenericTable(&Mcb->Mapping->Table, FALSE))
//     {
//         /* Take care when we must emulate missing 'hole' runs. */
//         if (Run->RunStartVbn.QuadPart > LastVbn)
//         {
//             RunIndex++;
//         }
//         LastVbn = Run->RunEndVbn.QuadPart;
//         RunIndex++;
//         RunFound = Run;
//     }
//
//     if (!RunFound)
//     {
//         return FALSE;
//     }
//
//     if (Vbn)
//     {
//         *Vbn = RunFound->RunEndVbn.QuadPart - 1;
//     }
//     if (Lbn)
//     {
//         if (1)
//         {
//             *Lbn = RunFound->StartingLbn.QuadPart + (RunFound->RunEndVbn.QuadPart - RunFound->RunStartVbn.QuadPart) - 1;
//         }
//         else
//         {
//             *Lbn = ~0ULL;
//         }
//     }
//     if (Index)
//     {
//         *Index = RunIndex - 1;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupLastBaseMcbEntryAndIndex(IN PBASE_MCB OpaqueMcb,
//                                     IN OUT PLONGLONG LargeVbn,
//                                     IN OUT PLONGLONG LargeLbn,
//                                     IN OUT PULONG Index)
// {
//     BOOLEAN Result;
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//
//     DPRINT("FsRtlLookupLastBaseMcbEntryAndIndex(%p, %p, %p, %p)\n", OpaqueMcb, LargeVbn, LargeLbn, Index);
//
//     Result = FsRtlLookupLastLargeMcbEntryAndIndex_internal(Mcb, LargeVbn, LargeLbn, Index);
//
//     DPRINT("FsRtlLookupLastBaseMcbEntryAndIndex(%p, %p, %p, %p) = %d (%I64d, %I64d, %d)\n", OpaqueMcb, LargeVbn, LargeLbn, Index, Result, *LargeVbn, *LargeLbn, *Index);
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupLastLargeMcbEntryAndIndex(IN PLARGE_MCB OpaqueMcb,
//                                      OUT PLONGLONG LargeVbn,
//                                      OUT PLONGLONG LargeLbn,
//                                      OUT PULONG Index)
// {
//     BOOLEAN Result;
//
//     DPRINT("FsRtlLookupLastLargeMcbEntryAndIndex(%p, %p, %p, %p)\n", OpaqueMcb, LargeVbn, LargeLbn, Index);
//
//     KeAcquireGuardedMutex(OpaqueMcb->GuardedMutex);
//     Result = FsRtlLookupLastBaseMcbEntryAndIndex(&(OpaqueMcb->BaseMcb),
//                                                  LargeVbn,
//                                                  LargeLbn,
//                                                  Index);
//     KeReleaseGuardedMutex(OpaqueMcb->GuardedMutex);
//
//     DPRINT("FsRtlLookupLastLargeMcbEntryAndIndex(%p, %p, %p, %p) = %d (%I64d, %I64d, %d)\n", OpaqueMcb, LargeVbn, LargeLbn, Index, Result, *LargeVbn, *LargeLbn, *Index);
//
//     return Result;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupLastBaseMcbEntry(IN PBASE_MCB OpaqueMcb,
//                             OUT PLONGLONG Vbn,
//                             OUT PLONGLONG Lbn)
// {
//     BOOLEAN Result;
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//
//     DPRINT("FsRtlLookupLastBaseMcbEntry(%p, %p, %p)\n", OpaqueMcb, Vbn, Lbn);
//
//     Result = FsRtlLookupLastLargeMcbEntryAndIndex_internal(Mcb, Vbn, Lbn, NULL); /* Index */
//
//     DPRINT("FsRtlLookupLastBaseMcbEntry(%p, %p, %p) = %d (%I64d, %I64d)\n", Mcb, Vbn, Lbn, Result, *Vbn, *Lbn);
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlLookupLastLargeMcbEntry(IN PLARGE_MCB Mcb,
//                              OUT PLONGLONG Vbn,
//                              OUT PLONGLONG Lbn)
// {
//     BOOLEAN Result;
//
//     DPRINT("FsRtlLookupLastLargeMcbEntry(%p, %p, %p)\n", Mcb, Vbn, Lbn);
//
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     Result = FsRtlLookupLastBaseMcbEntry(&(Mcb->BaseMcb),
//                                          Vbn,
//                                          Lbn);
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
//
//     DPRINT("FsRtlLookupLastLargeMcbEntry(%p, %p, %p) = %d (%I64d, %I64d)\n", Mcb, Vbn, Lbn, Result, *Vbn, *Lbn);
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// FsRtlNumberOfRunsInBaseMcb(IN PBASE_MCB OpaqueMcb)
// {
//     ULONG NumberOfRuns = 0;
//     LONGLONG Vbn, Lbn, Count;
//     int i;
//
//     DPRINT("FsRtlNumberOfRunsInBaseMcb(%p)\n", OpaqueMcb);
//
//     // Count how many Mcb entries there are
//     for (i = 0; FsRtlGetNextBaseMcbEntry(OpaqueMcb, i, &Vbn, &Lbn, &Count); i++)
//     {
//         NumberOfRuns++;
//     }
//
//     DPRINT("FsRtlNumberOfRunsInBaseMcb(%p) = %d\n", OpaqueMcb, NumberOfRuns);
//     return NumberOfRuns;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// FsRtlNumberOfRunsInLargeMcb(IN PLARGE_MCB Mcb)
// {
//     ULONG NumberOfRuns;
//
//     DPRINT("FsRtlNumberOfRunsInLargeMcb(%p)\n", Mcb);
//
//     /* Read the number of runs while holding the MCB lock */
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     NumberOfRuns = FsRtlNumberOfRunsInBaseMcb(&(Mcb->BaseMcb));
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
//
//     DPRINT("FsRtlNumberOfRunsInLargeMcb(%p) = %d\n", Mcb, NumberOfRuns);
//
//     /* Return the count */
//     return NumberOfRuns;
// }
//
// /*
//  * @implemented
//  * @Mcb: #PLARGE_MCB initialized by FsRtlInitializeLargeMcb().
//  * %NULL value is forbidden.
//  * @Vbn: Starting virtual block number to specify the range to delete.
//  * @SectorCount: Length of the range to delete.
//  * Value less or equal to %0 is forbidden; FIXME: Is the reject of %0 W32 compliant?
//  *
//  * Deletes any possible @Mcb mappings in the given range @Vbn ... @Vbn+@SectorCount-1.
//  * This call has no problems if no mappings exist there yet.
//  */
// BOOLEAN
// NTAPI
// FsRtlRemoveBaseMcbEntry(IN PBASE_MCB OpaqueMcb,
//                         IN LONGLONG Vbn,
//                         IN LONGLONG SectorCount)
// {
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//     LARGE_MCB_MAPPING_ENTRY NeedleRun;
//     PLARGE_MCB_MAPPING_ENTRY HaystackRun;
//     BOOLEAN Result = TRUE;
//
//     DPRINT("FsRtlRemoveBaseMcbEntry(%p, %I64d, %I64d)\n", OpaqueMcb, Vbn, SectorCount);
//
//     if (Vbn < 0 || SectorCount <= 0)
//     {
//         Result = FALSE;
//         goto quit;
//     }
//
//     if (Vbn + SectorCount <= Vbn)
//     {
//         Result = FALSE;
//         goto quit;
//     }
//
//     NeedleRun.RunStartVbn.QuadPart = Vbn;
//     NeedleRun.RunEndVbn.QuadPart = Vbn + SectorCount;
//     NeedleRun.StartingLbn.QuadPart = -1;
//
//     /* adjust/destroy all intersecting ranges */
//     Mcb->Mapping->Table.CompareRoutine = McbMappingIntersectCompare;
//     while ((HaystackRun = RtlLookupElementGenericTable(&Mcb->Mapping->Table, &NeedleRun)))
//     {
//         if (HaystackRun->RunStartVbn.QuadPart < NeedleRun.RunStartVbn.QuadPart)
//         {
//             LONGLONG HaystackRunEnd = HaystackRun->RunEndVbn.QuadPart;
//             ASSERT(HaystackRun->RunEndVbn.QuadPart > NeedleRun.RunStartVbn.QuadPart);
//
//             HaystackRun->RunEndVbn.QuadPart = NeedleRun.RunStartVbn.QuadPart;
//
//             if (HaystackRunEnd > NeedleRun.RunEndVbn.QuadPart)
//             {
//                 /* The run we are deleting is included in the run we just truncated.
//                  * Add the tail back. */
//                 LARGE_MCB_MAPPING_ENTRY TailRun;
//                 BOOLEAN NewElement;
//
//                 TailRun.RunStartVbn.QuadPart = NeedleRun.RunEndVbn.QuadPart;
//                 TailRun.RunEndVbn.QuadPart = HaystackRunEnd;
//                 TailRun.StartingLbn.QuadPart = HaystackRun->StartingLbn.QuadPart + (NeedleRun.RunEndVbn.QuadPart - HaystackRun->RunStartVbn.QuadPart);
//
//                 Mcb->Mapping->Table.CompareRoutine = McbMappingCompare;
//
//                 RtlInsertElementGenericTable(&Mcb->Mapping->Table, &TailRun, sizeof(TailRun), &NewElement);
//                 ++Mcb->PairCount;
//                 ASSERT(NewElement);
//
//                 Mcb->Mapping->Table.CompareRoutine = McbMappingIntersectCompare;
//             }
//         }
//         else if (HaystackRun->RunEndVbn.QuadPart > NeedleRun.RunEndVbn.QuadPart)
//         {
//             LONGLONG HaystackRunStart = HaystackRun->RunStartVbn.QuadPart;
//             LONGLONG HaystackStartingLbn = HaystackRun->StartingLbn.QuadPart;
//
//             ASSERT(HaystackRun->RunStartVbn.QuadPart < NeedleRun.RunEndVbn.QuadPart);
//             HaystackRun->RunStartVbn.QuadPart = NeedleRun.RunEndVbn.QuadPart;
//             /* Adjust the starting LBN */
//             HaystackRun->StartingLbn.QuadPart += NeedleRun.RunEndVbn.QuadPart - HaystackRunStart;
//
//             if (HaystackRunStart < NeedleRun.RunStartVbn.QuadPart)
//             {
//                 /* The run we are deleting is included in the run we just truncated.
//                  * Add the head back. */
//                 LARGE_MCB_MAPPING_ENTRY HeadRun;
//                 BOOLEAN NewElement;
//
//                 HeadRun.RunStartVbn.QuadPart = HaystackRunStart;
//                 HeadRun.RunEndVbn.QuadPart = NeedleRun.RunStartVbn.QuadPart;
//                 HeadRun.StartingLbn.QuadPart = HaystackStartingLbn;
//
//                 Mcb->Mapping->Table.CompareRoutine = McbMappingCompare;
//
//                 RtlInsertElementGenericTable(&Mcb->Mapping->Table, &HeadRun, sizeof(HeadRun), &NewElement);
//                 ++Mcb->PairCount;
//                 ASSERT(NewElement);
//
//                 Mcb->Mapping->Table.CompareRoutine = McbMappingIntersectCompare;
//             }
//         }
//         else
//         {
//             //ASSERT(NeedleRun.RunStartVbn.QuadPart >= HaystackRun->RunStartVbn.QuadPart);
//             //ASSERT(NeedleRun.RunEndVbn.QuadPart <= HaystackRun->RunEndVbn.QuadPart);
//             Mcb->Mapping->Table.CompareRoutine = McbMappingCompare;
//             RtlDeleteElementGenericTable(&Mcb->Mapping->Table, HaystackRun);
//             --Mcb->PairCount;
//             Mcb->Mapping->Table.CompareRoutine = McbMappingIntersectCompare;
//         }
//     }
//     Mcb->Mapping->Table.CompareRoutine = McbMappingCompare;
//
// quit:
//     DPRINT("FsRtlRemoveBaseMcbEntry(%p, %I64d, %I64d) = %d\n", OpaqueMcb, Vbn, SectorCount, Result);
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlRemoveLargeMcbEntry(IN PLARGE_MCB Mcb,
//                          IN LONGLONG Vbn,
//                          IN LONGLONG SectorCount)
// {
//     DPRINT("FsRtlRemoveLargeMcbEntry(%p, %I64d, %I64d)\n", Mcb, Vbn, SectorCount);
//
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     FsRtlRemoveBaseMcbEntry(&(Mcb->BaseMcb), Vbn, SectorCount);
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlResetBaseMcb(IN PBASE_MCB OpaqueMcb)
// {
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//     PLARGE_MCB_MAPPING_ENTRY Element;
//
//     DPRINT("FsRtlResetBaseMcb(%p)\n", OpaqueMcb);
//
//     while (RtlNumberGenericTableElements(&Mcb->Mapping->Table) &&
//            (Element = (PLARGE_MCB_MAPPING_ENTRY)RtlGetElementGenericTable(&Mcb->Mapping->Table, 0)))
//     {
//         RtlDeleteElementGenericTable(&Mcb->Mapping->Table, Element);
//     }
//
//     Mcb->PairCount = 0;
//     Mcb->MaximumPairCount = 0;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlResetLargeMcb(IN PLARGE_MCB Mcb,
//                    IN BOOLEAN SelfSynchronized)
// {
//     DPRINT("FsRtlResetLargeMcb(%p, %d)\n", Mcb, SelfSynchronized);
//
//     if (!SelfSynchronized)
//         KeAcquireGuardedMutex(Mcb->GuardedMutex);
//
//     FsRtlResetBaseMcb(&Mcb->BaseMcb);
//
//     if (!SelfSynchronized)
//         KeReleaseGuardedMutex(Mcb->GuardedMutex);
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// FsRtlSplitBaseMcb(IN PBASE_MCB OpaqueMcb,
//                   IN LONGLONG Vbn,
//                   IN LONGLONG Amount)
// {
//     PBASE_MCB_INTERNAL Mcb = (PBASE_MCB_INTERNAL)OpaqueMcb;
//     PLARGE_MCB_MAPPING_ENTRY Run, InsertLowerRun = NULL, ExistingRun = NULL;
//     BOOLEAN NewElement;
//
//     DPRINT("FsRtlSplitBaseMcb(%p, %I64d, %I64d)\n", OpaqueMcb, Vbn, Amount);
//
//     /* Traverse the tree */
//     for (Run = (PLARGE_MCB_MAPPING_ENTRY)RtlEnumerateGenericTable(&Mcb->Mapping->Table, TRUE);
//         Run;
//         Run = (PLARGE_MCB_MAPPING_ENTRY)RtlEnumerateGenericTable(&Mcb->Mapping->Table, FALSE))
//     {
//         /* unaffected run? */
//         /* FIXME: performance: effective skip of all 'lower' runs without traversing them */
//         if (Vbn >= Run->RunEndVbn.QuadPart) { DPRINT("Skipping it\n"); continue; }
//
//         /* crossing run to be split?
//         * 'lower_run' is created on the original place; just shortened.
//         * current 'run' is shifted up later
//         */
//         if (Vbn < Run->RunEndVbn.QuadPart)
//         {
//             /* FIXME: shift 'run->Lbn_start' ? */
//             Run->RunStartVbn.QuadPart = Vbn;
//
//             InsertLowerRun = NULL;
//         }
//
//         /* Shift the current 'run'.
//         * Ordering is not changed in Generic Tree so I hope I do not need to reinsert it.
//         */
//         Run->RunStartVbn.QuadPart += Amount;
//         ASSERT(Run->RunEndVbn.QuadPart + Amount > Run->RunEndVbn.QuadPart); /* overflow? */
//         Run->RunEndVbn.QuadPart += Amount;
//         /* FIXME: shift 'run->Lbn_start' ? */
//
//         /* continue the traversal */
//     }
//
//     if (InsertLowerRun)
//     {
//         ExistingRun = RtlInsertElementGenericTable(&Mcb->Mapping->Table, InsertLowerRun, sizeof(*InsertLowerRun), &NewElement);
//         ++Mcb->PairCount;
//     }
//
//     ASSERT(ExistingRun == NULL);
//
//     DPRINT("FsRtlSplitBaseMcb(%p, %I64d, %I64d) = %d\n", OpaqueMcb, Vbn, Amount, TRUE);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlSplitLargeMcb(IN PLARGE_MCB Mcb,
//                    IN LONGLONG Vbn,
//                    IN LONGLONG Amount)
// {
//     BOOLEAN Result;
//
//     DPRINT("FsRtlSplitLargeMcb(%p, %I64d, %I64d)\n", Mcb, Vbn, Amount);
//
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     Result = FsRtlSplitBaseMcb(&(Mcb->BaseMcb),
//                                Vbn,
//                                Amount);
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
//
//     DPRINT("FsRtlSplitLargeMcb(%p, %I64d, %I64d) = %d\n", Mcb, Vbn, Amount, Result);
//
//     return Result;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// FsRtlTruncateBaseMcb(IN PBASE_MCB OpaqueMcb,
//                      IN LONGLONG Vbn)
// {
//     DPRINT("FsRtlTruncateBaseMcb(%p, %I64d)\n", OpaqueMcb, Vbn);
//
//     FsRtlRemoveBaseMcbEntry(OpaqueMcb, Vbn, MAXLONG - Vbn + 1);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlTruncateLargeMcb(IN PLARGE_MCB Mcb,
//                       IN LONGLONG Vbn)
// {
//     DPRINT("FsRtlTruncateLargeMcb(%p, %I64d)\n", Mcb, Vbn);
//
//     KeAcquireGuardedMutex(Mcb->GuardedMutex);
//     FsRtlTruncateBaseMcb(&(Mcb->BaseMcb), Vbn);
//     KeReleaseGuardedMutex(Mcb->GuardedMutex);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlUninitializeBaseMcb(IN PBASE_MCB Mcb)
// {
//     DPRINT("FsRtlUninitializeBaseMcb(%p)\n", Mcb);
//
//     FsRtlResetBaseMcb(Mcb);
//
//     if ((Mcb->PoolType == PagedPool)/* && (Mcb->MaximumPairCount == MAXIMUM_PAIR_COUNT)*/)
//     {
//         ExFreeToPagedLookasideList(&FsRtlFirstMappingLookasideList,
//                                    Mcb->Mapping);
//     }
//     else
//     {
//         ExFreePoolWithTag(Mcb->Mapping, 'CBSF');
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlUninitializeLargeMcb(IN PLARGE_MCB Mcb)
// {
//     DPRINT("FsRtlUninitializeLargeMcb(%p)\n", Mcb);
//
//     if (Mcb->GuardedMutex)
//     {
//         ExFreeToNPagedLookasideList(&FsRtlFastMutexLookasideList,
//                                     Mcb->GuardedMutex);
//         FsRtlUninitializeBaseMcb(&(Mcb->BaseMcb));
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<LARGE_MCB_MAPPING_ENTRY>() == 24);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING_ENTRY, RunStartVbn) == 0);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING_ENTRY, RunEndVbn) == 8);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING_ENTRY, StartingLbn) == 16);
    assert!(core::mem::size_of::<LARGE_MCB_MAPPING>() == 72);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING, Table) == 0);
    assert!(core::mem::size_of::<BASE_MCB_INTERNAL>() == 24);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, MaximumPairCount) == 0);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, PairCount) == 4);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, PoolType) == 8);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, Flags) == 10);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, Mapping) == 16);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<LARGE_MCB_MAPPING_ENTRY>() == 24);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING_ENTRY, RunStartVbn) == 0);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING_ENTRY, RunEndVbn) == 8);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING_ENTRY, StartingLbn) == 16);
    assert!(core::mem::size_of::<LARGE_MCB_MAPPING>() == 40);
    assert!(core::mem::offset_of!(LARGE_MCB_MAPPING, Table) == 0);
    assert!(core::mem::size_of::<BASE_MCB_INTERNAL>() == 16);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, MaximumPairCount) == 0);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, PairCount) == 4);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, PoolType) == 8);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, Flags) == 10);
    assert!(core::mem::offset_of!(BASE_MCB_INTERNAL, Mapping) == 12);
};
// END OFFSET CHECKS
