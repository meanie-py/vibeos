//! Translation of sdk/lib/rtl/amd64/dynfntbl.c — dynamic function table
//! support routines (RtlAddFunctionTable & friends).

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

/* ndk list macros (InsertTailList / RemoveEntryList) */
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

/// https://docs.microsoft.com/en-us/windows/win32/devnotes/rtlgetfunctiontablelisthead
#[no_mangle]
pub unsafe extern "system" fn RtlGetFunctionTableListHead() -> PLIST_ENTRY {
    core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList)
}

unsafe fn RtlpInsertDynamicFunctionTable(DynamicTable: *mut DYNAMIC_FUNCTION_TABLE) {
    AcquireDynamicFunctionTableLockExclusive();

    /* Insert it into the list */
    InsertTailList(
        core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList),
        core::ptr::addr_of_mut!((*DynamicTable).ListEntry),
    );

    // TODO: insert into RB-trees

    ReleaseDynamicFunctionTableLockExclusive();
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddFunctionTable(
    FunctionTable: PRUNTIME_FUNCTION,
    EntryCount: ULONG,
    BaseAddress: DWORD64,
) -> BOOLEAN {
    /* Allocate a dynamic function table */
    let dynamicTable = RtlpAllocateMemory(
        core::mem::size_of::<DYNAMIC_FUNCTION_TABLE>() as SIZE_T,
        TAG_RTLDYNFNTBL,
    ) as *mut DYNAMIC_FUNCTION_TABLE;
    if dynamicTable.is_null() {
        /* DPRINT1 elided (NDEBUG) */
        return 0;
    }

    /* Initialize fields */
    (*dynamicTable).FunctionTable = FunctionTable;
    (*dynamicTable).EntryCount = EntryCount;
    (*dynamicTable).BaseAddress = BaseAddress;
    (*dynamicTable).Callback = None;
    (*dynamicTable).Context = core::ptr::null_mut();
    (*dynamicTable).Type = FUNCTION_TABLE_TYPE::RF_UNSORTED;

    /* Loop all entries to find the margins */
    (*dynamicTable).MinimumAddress = u64::MAX;
    (*dynamicTable).MaximumAddress = 0;
    for i in 0..EntryCount as usize {
        let entry = FunctionTable.add(i);
        (*dynamicTable).MinimumAddress =
            core::cmp::min((*dynamicTable).MinimumAddress, (*entry).BeginAddress as ULONG64);
        (*dynamicTable).MaximumAddress =
            core::cmp::max((*dynamicTable).MaximumAddress, (*entry).EndAddress as ULONG64);
    }

    /* Adjust the margins to be absolute addresses */
    (*dynamicTable).MinimumAddress += BaseAddress;
    (*dynamicTable).MaximumAddress += BaseAddress;

    /* Insert the table into the list */
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
    /* Make sure the identifier is valid */
    if (TableIdentifier & 3) != 3 {
        return 0;
    }

    /* Check if we have a DLL name */
    let stringLength: SIZE_T = if !OutOfProcessCallbackDll.is_null() {
        (wcslen(OutOfProcessCallbackDll) + 1) as SIZE_T
    } else {
        0
    };

    /* Calculate required size */
    let allocationSize = core::mem::size_of::<DYNAMIC_FUNCTION_TABLE>() as SIZE_T
        + stringLength * core::mem::size_of::<u16>() as SIZE_T;

    /* Allocate a dynamic function table */
    let dynamicTable =
        RtlpAllocateMemory(allocationSize, TAG_RTLDYNFNTBL) as *mut DYNAMIC_FUNCTION_TABLE;
    if dynamicTable.is_null() {
        /* DPRINT1 elided (NDEBUG) */
        return 0;
    }

    /* Initialize fields */
    (*dynamicTable).FunctionTable = TableIdentifier as PRUNTIME_FUNCTION;
    (*dynamicTable).EntryCount = 0;
    (*dynamicTable).BaseAddress = BaseAddress;
    (*dynamicTable).Callback = Callback;
    (*dynamicTable).Context = Context;
    (*dynamicTable).Type = FUNCTION_TABLE_TYPE::RF_CALLBACK;
    (*dynamicTable).MinimumAddress = BaseAddress;
    (*dynamicTable).MaximumAddress = BaseAddress + Length as ULONG64;

    /* If we have a DLL name, copy that, too */
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

    /* Insert the table into the list */
    RtlpInsertDynamicFunctionTable(dynamicTable);

    1
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteFunctionTable(
    FunctionTable: PRUNTIME_FUNCTION,
) -> BOOLEAN {
    let mut removed = false;
    let mut dynamicTable: *mut DYNAMIC_FUNCTION_TABLE = core::ptr::null_mut();

    AcquireDynamicFunctionTableLockExclusive();

    /* Loop all tables to find the one to delete */
    let head = core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList);
    let mut listLink = (*head).Flink;
    while listLink != head {
        /* CONTAINING_RECORD(listLink, DYNAMIC_FUNCTION_TABLE, ListEntry) — ListEntry is field 0 */
        dynamicTable = listLink as *mut DYNAMIC_FUNCTION_TABLE;

        if (*dynamicTable).FunctionTable == FunctionTable {
            RemoveEntryList(core::ptr::addr_of_mut!((*dynamicTable).ListEntry));
            removed = true;
            break;
        }

        listLink = (*listLink).Flink;
    }

    ReleaseDynamicFunctionTableLockExclusive();

    /* If we were successful, free the memory */
    if removed {
        RtlpFreeMemory(dynamicTable as PVOID, TAG_RTLDYNFNTBL);
    }

    removed as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn RtlpLookupDynamicFunctionEntry(
    ControlPc: DWORD64,
    ImageBase: PDWORD64,
    _HistoryTable: *mut UNWIND_HISTORY_TABLE,
) -> PRUNTIME_FUNCTION {
    let mut foundEntry: PRUNTIME_FUNCTION = core::ptr::null_mut();

    AcquireDynamicFunctionTableLockShared();

    /* Loop all tables to find the one matching ControlPc */
    let head = core::ptr::addr_of_mut!(RtlpDynamicFunctionTableList);
    let mut listLink = (*head).Flink;
    'outer: while listLink != head {
        let dynamicTable = listLink as *mut DYNAMIC_FUNCTION_TABLE;

        if (ControlPc >= (*dynamicTable).MinimumAddress)
            && (ControlPc < (*dynamicTable).MaximumAddress)
        {
            /* Check if there is a callback */
            if let Some(callback) = (*dynamicTable).Callback {
                let context = (*dynamicTable).Context;

                *ImageBase = (*dynamicTable).BaseAddress;
                ReleaseDynamicFunctionTableLockShared();
                return callback(ControlPc, context);
            }

            /* Loop all entries in the function table */
            let functionTable = (*dynamicTable).FunctionTable;
            let ipOffset = ControlPc - (*dynamicTable).BaseAddress;
            for i in 0..(*dynamicTable).EntryCount as usize {
                let entry = functionTable.add(i);
                /* Check if this entry contains the address */
                if (ipOffset >= (*entry).BeginAddress as DWORD64)
                    && (ipOffset < (*entry).EndAddress as DWORD64)
                {
                    foundEntry = entry;
                    *ImageBase = (*dynamicTable).BaseAddress;
                    break 'outer; // goto Exit
                }
            }
        }

        listLink = (*listLink).Flink;
    }

    /* Exit: */
    ReleaseDynamicFunctionTableLockShared();

    foundEntry
}
