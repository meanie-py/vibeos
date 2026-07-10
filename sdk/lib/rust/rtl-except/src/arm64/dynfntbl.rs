//! ARM64 dynamic function table support (`RtlAddFunctionTable` & friends).
//!
//! Ported from `amd64/dynfntbl.rs` (upstream `sdk/lib/rtl/amd64/dynfntbl.c`),
//! which arm64 never had — its `RtlAddFunctionTable`/`RtlInstallFunctionTable
//! Callback`/`RtlDeleteFunctionTable` were no-op stubs, and
//! `RtlLookupFunctionEntry` only consulted static `.pdata`. That makes
//! JIT-registered unwind info (FEX guest frames, any runtime codegen) invisible
//! to the unwinder. This module makes registration real and is consulted by
//! `RtlLookupFunctionEntry` when the static table misses.
//!
//! The only arm64-specific difference from amd64: `RUNTIME_FUNCTION` has no
//! `EndAddress` field, so the per-function end address is derived from the
//! packed/`.xdata` unwind info via `super::except::RtlpGetFunctionEndAddress`.

use crate::types::*;
use ntkrnl_sys::ddk::*;

const TAG_RTLDYNFNTBL: ULONG = u32::from_le_bytes(*b"tfDP"); // 'tfDP'

pub type PGET_RUNTIME_FUNCTION_CALLBACK =
    Option<unsafe extern "system" fn(ControlPc: DWORD64, Context: PVOID) -> PRUNTIME_FUNCTION>;

#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
enum FUNCTION_TABLE_TYPE {
    RF_SORTED = 0x0,
    RF_UNSORTED = 0x1,
    RF_CALLBACK = 0x2,
    RF_KERNEL_DYNAMIC = 0x3,
}

#[repr(C)]
struct DYNAMIC_FUNCTION_TABLE {
    ListEntry: LIST_ENTRY,
    FunctionTable: PRUNTIME_FUNCTION,
    TimeStamp: LARGE_INTEGER,
    MinimumAddress: ULONG64,
    MaximumAddress: ULONG64,
    BaseAddress: ULONG64,
    Callback: PGET_RUNTIME_FUNCTION_CALLBACK,
    Context: PVOID,
    OutOfProcessCallbackDll: *mut u16,
    Type: FUNCTION_TABLE_TYPE,
    EntryCount: ULONG,
}

#[no_mangle]
pub static mut RtlpDynamicFunctionTableLock: RTL_SRWLOCK = RTL_SRWLOCK {
    Ptr: core::ptr::null_mut(),
};

/// Self-referential list head — same static-initializer relocation the C uses.
#[no_mangle]
pub static mut RtlpDynamicFunctionTableList: LIST_ENTRY = LIST_ENTRY {
    Flink: unsafe { core::ptr::addr_of!(RtlpDynamicFunctionTableList) as *mut LIST_ENTRY },
    Blink: unsafe { core::ptr::addr_of!(RtlpDynamicFunctionTableList) as *mut LIST_ENTRY },
};

#[inline]
unsafe fn AcquireDynamicFunctionTableLockExclusive() {
    RtlAcquireSRWLockExclusive(core::ptr::addr_of_mut!(RtlpDynamicFunctionTableLock));
}

#[inline]
unsafe fn ReleaseDynamicFunctionTableLockExclusive() {
    RtlReleaseSRWLockExclusive(core::ptr::addr_of_mut!(RtlpDynamicFunctionTableLock));
}

#[inline]
unsafe fn AcquireDynamicFunctionTableLockShared() {
    RtlAcquireSRWLockShared(core::ptr::addr_of_mut!(RtlpDynamicFunctionTableLock));
}

#[inline]
unsafe fn ReleaseDynamicFunctionTableLockShared() {
    RtlReleaseSRWLockShared(core::ptr::addr_of_mut!(RtlpDynamicFunctionTableLock));
}

#[inline]
unsafe fn InsertTailList(ListHead: *mut LIST_ENTRY, Entry: *mut LIST_ENTRY) {
    let OldBlink = (*ListHead).Blink;
    (*Entry).Flink = ListHead;
    (*Entry).Blink = OldBlink;
    (*OldBlink).Flink = Entry;
    (*ListHead).Blink = Entry;
}

#[inline]
unsafe fn RemoveEntryList(Entry: *mut LIST_ENTRY) {
    let OldFlink = (*Entry).Flink;
    let OldBlink = (*Entry).Blink;
    (*OldFlink).Blink = OldBlink;
    (*OldBlink).Flink = OldFlink;
}

/// arm64 end-RVA of one dynamic-table entry (no EndAddress field on arm64).
#[inline]
unsafe fn EntryEndRva(entry: PRUNTIME_FUNCTION, base: ULONG64) -> ULONG {
    super::except::RtlpGetFunctionEndAddress(entry, base)
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetFunctionTableListHead() -> PLIST_ENTRY {
    core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList)
}

unsafe fn RtlpInsertDynamicFunctionTable(DynamicTable: *mut DYNAMIC_FUNCTION_TABLE) {
    AcquireDynamicFunctionTableLockExclusive();
    InsertTailList(
        core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList),
        core::ptr::addr_of_mut!((*DynamicTable).ListEntry),
    );
    ReleaseDynamicFunctionTableLockExclusive();
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddFunctionTable(
    FunctionTable: PRUNTIME_FUNCTION,
    EntryCount: ULONG,
    BaseAddress: DWORD64,
) -> BOOLEAN {
    let dynamicTable = RtlpAllocateMemory(
        core::mem::size_of::<DYNAMIC_FUNCTION_TABLE>() as SIZE_T,
        TAG_RTLDYNFNTBL,
    ) as *mut DYNAMIC_FUNCTION_TABLE;
    if dynamicTable.is_null() {
        return 0;
    }

    (*dynamicTable).FunctionTable = FunctionTable;
    (*dynamicTable).EntryCount = EntryCount;
    (*dynamicTable).BaseAddress = BaseAddress;
    (*dynamicTable).Callback = None;
    (*dynamicTable).Context = core::ptr::null_mut();
    (*dynamicTable).Type = FUNCTION_TABLE_TYPE::RF_UNSORTED;

    /* Find the address margins (arm64: derive each end from unwind info). */
    (*dynamicTable).MinimumAddress = u64::MAX;
    (*dynamicTable).MaximumAddress = 0;
    for i in 0..EntryCount as usize {
        let entry = FunctionTable.add(i);
        let begin = (*entry).BeginAddress as ULONG64;
        let end = EntryEndRva(entry, BaseAddress) as ULONG64;
        (*dynamicTable).MinimumAddress = core::cmp::min((*dynamicTable).MinimumAddress, begin);
        (*dynamicTable).MaximumAddress = core::cmp::max((*dynamicTable).MaximumAddress, end);
    }

    (*dynamicTable).MinimumAddress += BaseAddress;
    (*dynamicTable).MaximumAddress += BaseAddress;

    RtlpInsertDynamicFunctionTable(dynamicTable);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RtlInstallFunctionTableCallback(
    TableIdentifier: DWORD64,
    BaseAddress: DWORD64,
    Length: ULONG,
    Callback: PGET_RUNTIME_FUNCTION_CALLBACK,
    Context: PVOID,
    OutOfProcessCallbackDll: PCWSTR,
) -> BOOLEAN {
    if (TableIdentifier & 3) != 3 {
        return 0;
    }

    let stringLength: SIZE_T = if !OutOfProcessCallbackDll.is_null() {
        (wcslen(OutOfProcessCallbackDll) + 1) as SIZE_T
    } else {
        0
    };

    let allocationSize = core::mem::size_of::<DYNAMIC_FUNCTION_TABLE>() as SIZE_T
        + stringLength * core::mem::size_of::<u16>() as SIZE_T;

    let dynamicTable =
        RtlpAllocateMemory(allocationSize, TAG_RTLDYNFNTBL) as *mut DYNAMIC_FUNCTION_TABLE;
    if dynamicTable.is_null() {
        return 0;
    }

    (*dynamicTable).FunctionTable = TableIdentifier as PRUNTIME_FUNCTION;
    (*dynamicTable).EntryCount = 0;
    (*dynamicTable).BaseAddress = BaseAddress;
    (*dynamicTable).Callback = Callback;
    (*dynamicTable).Context = Context;
    (*dynamicTable).Type = FUNCTION_TABLE_TYPE::RF_CALLBACK;
    (*dynamicTable).MinimumAddress = BaseAddress;
    (*dynamicTable).MaximumAddress = BaseAddress + Length as ULONG64;

    if !OutOfProcessCallbackDll.is_null() {
        (*dynamicTable).OutOfProcessCallbackDll = dynamicTable.add(1) as *mut u16;
        core::ptr::copy_nonoverlapping(
            OutOfProcessCallbackDll,
            (*dynamicTable).OutOfProcessCallbackDll,
            stringLength as usize,
        );
    } else {
        (*dynamicTable).OutOfProcessCallbackDll = core::ptr::null_mut();
    }

    RtlpInsertDynamicFunctionTable(dynamicTable);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteFunctionTable(FunctionTable: PRUNTIME_FUNCTION) -> BOOLEAN {
    let mut removed = false;
    let mut dynamicTable: *mut DYNAMIC_FUNCTION_TABLE = core::ptr::null_mut();

    AcquireDynamicFunctionTableLockExclusive();

    let head = core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList);
    let mut listLink = (*head).Flink;
    while listLink != head {
        dynamicTable = listLink as *mut DYNAMIC_FUNCTION_TABLE;
        if (*dynamicTable).FunctionTable == FunctionTable {
            RemoveEntryList(core::ptr::addr_of_mut!((*dynamicTable).ListEntry));
            removed = true;
            break;
        }
        listLink = (*listLink).Flink;
    }

    ReleaseDynamicFunctionTableLockExclusive();

    if removed {
        RtlpFreeMemory(dynamicTable as PVOID, TAG_RTLDYNFNTBL);
    }

    removed as BOOLEAN
}

/// Consulted by `RtlLookupFunctionEntry` when the static `.pdata` search misses.
#[no_mangle]
pub unsafe extern "system" fn RtlpLookupDynamicFunctionEntry(
    ControlPc: DWORD64,
    ImageBase: PDWORD64,
    _HistoryTable: *mut UNWIND_HISTORY_TABLE,
) -> PRUNTIME_FUNCTION {
    let mut foundEntry: PRUNTIME_FUNCTION = core::ptr::null_mut();

    AcquireDynamicFunctionTableLockShared();

    let head = core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList);
    let mut listLink = (*head).Flink;
    'outer: while listLink != head {
        let dynamicTable = listLink as *mut DYNAMIC_FUNCTION_TABLE;

        if (ControlPc >= (*dynamicTable).MinimumAddress)
            && (ControlPc < (*dynamicTable).MaximumAddress)
        {
            if let Some(callback) = (*dynamicTable).Callback {
                let context = (*dynamicTable).Context;
                *ImageBase = (*dynamicTable).BaseAddress;
                ReleaseDynamicFunctionTableLockShared();
                return callback(ControlPc, context);
            }

            let functionTable = (*dynamicTable).FunctionTable;
            let base = (*dynamicTable).BaseAddress;
            let ipOffset = ControlPc - base;
            for i in 0..(*dynamicTable).EntryCount as usize {
                let entry = functionTable.add(i);
                let begin = (*entry).BeginAddress as DWORD64;
                let end = EntryEndRva(entry, base) as DWORD64;
                if ipOffset >= begin && ipOffset < end {
                    foundEntry = entry;
                    *ImageBase = base;
                    break 'outer;
                }
            }
        }

        listLink = (*listLink).Flink;
    }

    ReleaseDynamicFunctionTableLockShared();
    foundEntry
}
