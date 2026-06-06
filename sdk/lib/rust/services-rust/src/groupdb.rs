//! groupdb.c — the SCM service-group database. First consumer of the SERVICE /
//! SERVICE_GROUP mirror. Faithful translation: ScmGetServiceGroupByName /
//! ScmSetServiceGroup / ScmCreateGroupList are cdecl exports; the two
//! RtlQueryRegistryValues callbacks are NTAPI (stdcall on i386) so they are
//! declared `extern "system"` and transmuted into the bindgen
//! PRTL_QUERY_REGISTRY_ROUTINE slot (which is `extern "stdcall"` on i386,
//! `extern "C"` on 64-bit — `extern "system"` is the matching machine ABI per
//! arch). `GroupListHead`/`UnknownGroupListHead` are exported globals.

use crate::types::*;
use crate::wstr;
use core::mem::{size_of, transmute, zeroed};
use core::ptr::{addr_of_mut, null_mut};
use winsrv_sys::winsrv::*;
// List inlines + RtlCopyMemory + the STATUS_* codes are hand-written in
// winsrv_sys::nt (FORCEINLINE macros / cast-macro consts), not the winsrv glob.
use winsrv_sys::nt::{
    InitializeListHead, InsertTailList, RemoveEntryList,
    STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};

// CRT string fns (cdecl), not in winsrv-sys — link against msvcrt.
extern "C" {
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
}

// Exported globals (database.c and others reference them).
#[no_mangle]
pub static mut GroupListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: null_mut(),
    Blink: null_mut(),
};
#[no_mangle]
pub static mut UnknownGroupListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: null_mut(),
    Blink: null_mut(),
};

/// CONTAINING_RECORD(entry, SERVICE_GROUP, GroupListEntry).
#[inline]
unsafe fn group_from_entry(e: *mut LIST_ENTRY) -> PSERVICE_GROUP {
    (e as *mut u8).sub(core::mem::offset_of!(SERVICE_GROUP, GroupListEntry)) as PSERVICE_GROUP
}

#[no_mangle]
pub unsafe extern "C" fn ScmGetServiceGroupByName(lpGroupName: LPCWSTR) -> PSERVICE_GROUP {
    let mut GroupEntry = GroupListHead.Flink;
    while GroupEntry != addr_of_mut!(GroupListHead) {
        let lpGroup = group_from_entry(GroupEntry);
        if _wcsicmp((*lpGroup).lpGroupName as _, lpGroupName) == 0 {
            return lpGroup;
        }
        GroupEntry = (*GroupEntry).Flink;
    }

    GroupEntry = UnknownGroupListHead.Flink;
    while GroupEntry != addr_of_mut!(UnknownGroupListHead) {
        let lpGroup = group_from_entry(GroupEntry);
        if _wcsicmp((*lpGroup).lpGroupName as _, lpGroupName) == 0 {
            return lpGroup;
        }
        GroupEntry = (*GroupEntry).Flink;
    }

    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn ScmSetServiceGroup(lpService: PSERVICE, lpGroupName: LPCWSTR) -> DWORD {
    if !(*lpService).lpGroup.is_null() {
        // ASSERTs are NDEBUG no-ops in the C; dropped.
        let g = (*lpService).lpGroup;
        if (*g).dwRefCount != u32::MAX {
            (*g).dwRefCount -= 1;
            if (*g).dwRefCount == 0 {
                RemoveEntryList(addr_of_mut!((*g).GroupListEntry));
                HeapFree(GetProcessHeap(), 0, g as LPVOID);
                (*lpService).lpGroup = null_mut();
            }
        }
    }

    if lpGroupName.is_null() {
        return ERROR_SUCCESS;
    }

    let mut GroupEntry = GroupListHead.Flink;
    while GroupEntry != addr_of_mut!(GroupListHead) {
        let lpGroup = group_from_entry(GroupEntry);
        if _wcsicmp((*lpGroup).lpGroupName as _, lpGroupName) == 0 {
            (*lpService).lpGroup = lpGroup;
            return ERROR_SUCCESS;
        }
        GroupEntry = (*GroupEntry).Flink;
    }

    GroupEntry = UnknownGroupListHead.Flink;
    while GroupEntry != addr_of_mut!(UnknownGroupListHead) {
        let lpGroup = group_from_entry(GroupEntry);
        if _wcsicmp((*lpGroup).lpGroupName as _, lpGroupName) == 0 {
            (*lpGroup).dwRefCount += 1;
            (*lpService).lpGroup = lpGroup;
            return ERROR_SUCCESS;
        }
        GroupEntry = (*GroupEntry).Flink;
    }

    let lpGroup = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (size_of::<SERVICE_GROUP>() + (wcslen(lpGroupName) + 1) * size_of::<u16>()) as _,
    ) as PSERVICE_GROUP;
    if lpGroup.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    wcscpy((*lpGroup).szGroupName.as_mut_ptr(), lpGroupName);
    (*lpGroup).lpGroupName = (*lpGroup).szGroupName.as_mut_ptr();
    (*lpGroup).dwRefCount = 1;
    (*lpService).lpGroup = lpGroup;

    InsertTailList(addr_of_mut!(UnknownGroupListHead), addr_of_mut!((*lpGroup).GroupListEntry));

    ERROR_SUCCESS
}

unsafe extern "system" fn CreateGroupOrderListRoutine(
    _ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    _EntryContext: PVOID,
) -> NTSTATUS {
    if ValueType == REG_BINARY
        && !ValueData.is_null()
        && ValueLength >= size_of::<DWORD>() as ULONG
        && ValueLength
            >= (*(ValueData as *const ULONG))
                .wrapping_add(1)
                .wrapping_mul(size_of::<DWORD>() as ULONG)
    {
        let Group = Context as PSERVICE_GROUP;
        (*Group).TagCount = *(ValueData as *const ULONG); // ((PULONG)ValueData)[0]
        if (*Group).TagCount > 0 {
            if ValueLength >= ((*Group).TagCount + 1).wrapping_mul(size_of::<DWORD>() as ULONG) {
                (*Group).TagArray = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    ((*Group).TagCount as usize * size_of::<DWORD>()) as _,
                ) as PULONG;
                if (*Group).TagArray.is_null() {
                    (*Group).TagCount = 0;
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                // RtlCopyMemory(dst, (PULONG)ValueData + 1, TagCount*sizeof(DWORD))
                core::ptr::copy_nonoverlapping(
                    (ValueData as *const ULONG).add(1) as *const u8,
                    (*Group).TagArray as *mut u8,
                    (*Group).TagCount as usize * size_of::<DWORD>(),
                );
            } else {
                (*Group).TagCount = 0;
                return STATUS_UNSUCCESSFUL;
            }
        }
    }

    STATUS_SUCCESS
}

unsafe extern "system" fn CreateGroupListRoutine(
    _ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    _ValueLength: ULONG,
    _Context: PVOID,
    _EntryContext: PVOID,
) -> NTSTATUS {
    if ValueType == REG_SZ {
        let Group = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (size_of::<SERVICE_GROUP>() + (wcslen(ValueData as *const u16) + 1) * size_of::<u16>())
                as _,
        ) as PSERVICE_GROUP;
        if Group.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        wcscpy((*Group).szGroupName.as_mut_ptr(), ValueData as *const u16);
        (*Group).lpGroupName = (*Group).szGroupName.as_mut_ptr();
        (*Group).dwRefCount = u32::MAX; // (DWORD)-1

        let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = zeroed();
        QueryTable[0].Name = ValueData as PCWSTR;
        QueryTable[0].QueryRoutine = transmute::<
            unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
            PRTL_QUERY_REGISTRY_ROUTINE,
        >(CreateGroupOrderListRoutine);

        RtlQueryRegistryValues(
            RTL_REGISTRY_CONTROL,
            wstr!("GroupOrderList").as_ptr(),
            QueryTable.as_mut_ptr(),
            Group as PVOID,
            null_mut(),
        );

        InsertTailList(addr_of_mut!(GroupListHead), addr_of_mut!((*Group).GroupListEntry));
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ScmCreateGroupList() -> DWORD {
    InitializeListHead(addr_of_mut!(GroupListHead));
    InitializeListHead(addr_of_mut!(UnknownGroupListHead));

    /* Build group order list */
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = zeroed();
    QueryTable[0].Name = wstr!("List").as_ptr();
    QueryTable[0].QueryRoutine = transmute::<
        unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
        PRTL_QUERY_REGISTRY_ROUTINE,
    >(CreateGroupListRoutine);

    let Status = RtlQueryRegistryValues(
        RTL_REGISTRY_CONTROL,
        wstr!("ServiceGroupOrder").as_ptr(),
        QueryTable.as_mut_ptr(),
        null_mut(),
        null_mut(),
    );

    RtlNtStatusToDosError(Status)
}
