use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn CreateGenericList() -> PGENERIC_LIST {
    let List: PGENERIC_LIST = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<GENERIC_LIST>()) as SIZE_T,
    ) as PGENERIC_LIST;

    if List.is_null() {
        return core::ptr::null_mut();
    }

    InitializeListHead(core::ptr::addr_of_mut!((*List).ListHead));
    (*List).NumOfEntries = 0;
    (*List).CurrentEntry = core::ptr::null_mut();

    List
}

#[no_mangle]
pub unsafe extern "system" fn DestroyGenericList(
    List: PGENERIC_LIST,
    FreeData: BOOLEAN,
) {
    let mut ListEntry: PGENERIC_LIST_ENTRY;
    let mut Entry: PLIST_ENTRY;

    while IsListEmpty(core::ptr::addr_of_mut!((*List).ListHead)) == false {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!((*List).ListHead));
        ListEntry = CONTAINING_RECORD!(Entry, GENERIC_LIST_ENTRY, Entry);

        if FreeData != 0 && !(*ListEntry).Data.is_null() {
            RtlFreeHeap(ProcessHeap, 0, (*ListEntry).Data);
        }

        RtlFreeHeap(ProcessHeap, 0, ListEntry.cast());
    }

    RtlFreeHeap(ProcessHeap, 0, List.cast());
}
#[no_mangle]
pub unsafe extern "system" fn AppendGenericListEntry(
    List: PGENERIC_LIST,
    Data: PVOID,
    Current: BOOLEAN,
) -> BOOLEAN {
    let Entry: PGENERIC_LIST_ENTRY = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<GENERIC_LIST_ENTRY>()) as SIZE_T,
    ) as PGENERIC_LIST_ENTRY;

    if Entry.is_null() {
        return (FALSE) as u8;
    }

    (*Entry).List = List;
    (*Entry).Data = Data;
    (*Entry).UiData = 0;

    InsertTailList(
        core::ptr::addr_of_mut!((*List).ListHead),
        core::ptr::addr_of_mut!((*Entry).Entry),
    );
    (*List).NumOfEntries = (*List).NumOfEntries.wrapping_add(1);

    if Current != 0 || (*List).CurrentEntry.is_null() {
        (*List).CurrentEntry = Entry;
    }

    (TRUE) as u8
}

#[no_mangle]
pub unsafe extern "system" fn SetCurrentListEntry(
    List: PGENERIC_LIST,
    Entry: PGENERIC_LIST_ENTRY,
) {
    if Entry.is_null() || (*Entry).List != List {
        return;
    }
    (*List).CurrentEntry = Entry;
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentListEntry(List: PGENERIC_LIST) -> PGENERIC_LIST_ENTRY {
    (*List).CurrentEntry
}

#[no_mangle]
pub unsafe extern "system" fn GetFirstListEntry(List: PGENERIC_LIST) -> PGENERIC_LIST_ENTRY {
    if IsListEmpty(core::ptr::addr_of_mut!((*List).ListHead)) != false {
        return core::ptr::null_mut::<_GENERIC_LIST_ENTRY>();
    }
    CONTAINING_RECORD!((*List).ListHead.Flink, GENERIC_LIST_ENTRY, Entry)
}

#[no_mangle]
pub unsafe extern "system" fn GetNextListEntry(
    Entry: PGENERIC_LIST_ENTRY,
) -> PGENERIC_LIST_ENTRY {
    let Next: PLIST_ENTRY = (*Entry).Entry.Flink;

    if Next == core::ptr::addr_of_mut!((*(*Entry).List).ListHead) {
        return core::ptr::null_mut();
    }

    CONTAINING_RECORD!(Next, GENERIC_LIST_ENTRY, Entry)
}

#[no_mangle]
pub unsafe extern "system" fn GetListEntryData(Entry: PGENERIC_LIST_ENTRY) -> PVOID {
    (*Entry).Data
}
#[no_mangle]
pub unsafe extern "C" fn GetListEntryUiData(Entry: PGENERIC_LIST_ENTRY) -> ULONG_PTR {
    (*Entry).UiData
}

#[no_mangle]
pub unsafe extern "system" fn GetNumberOfListEntries(List: PGENERIC_LIST) -> u32 {
    (*List).NumOfEntries
}
