//! resource.cpp factories. Allocation order and failure output behavior follow the C++.
use crate::resource_owner::*;
use crate::resource_support::*;
use crate::types::*;
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::{PASSIVE_LEVEL, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS};

// The active clang/GNU source branch computes in size_t, then assigns to ULONG.
fn resource_size(count: ULONG) -> ULONG {
    core::mem::size_of::<CM_RESOURCE_LIST>()
        .wrapping_sub(core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>())
        .wrapping_add(
            (count as usize).wrapping_mul(core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>()),
        ) as ULONG
}
#[no_mangle]
pub unsafe extern "system" fn PcNewResourceList(
    out: *mut PRESOURCELIST,
    outer: PUNKNOWN,
    pool: POOL_TYPE,
    translated: PCM_RESOURCE_LIST,
    untranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    if translated.is_null() != untranslated.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let list = construct(pool, TAG_PORTCLASS, outer);
    if list.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let status = query_interface(list.cast(), &raw const IID_IResourceList, out.cast());
    if status < 0 {
        destroy(list);
        return STATUS_INVALID_PARAMETER;
    }
    if translated.is_null() {
        return STATUS_SUCCESS;
    }
    kernel_assert!(
        (*untranslated).List[0].PartialResourceList.Count
            == (*translated).List[0].PartialResourceList.Count
    );
    let count = (*untranslated).List[0].PartialResourceList.Count;
    let size = resource_size(count) as usize;
    let new_translated = allocate_item(pool, size, TAG_PORTCLASS) as PCM_RESOURCE_LIST;
    if new_translated.is_null() {
        destroy(list);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let new_untranslated = allocate_item(pool, size, TAG_PORTCLASS) as PCM_RESOURCE_LIST;
    if new_untranslated.is_null() {
        destroy(list);
        free_item(new_translated.cast(), TAG_PORTCLASS);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopyMemory(new_translated.cast(), translated.cast(), size);
    RtlCopyMemory(new_untranslated.cast(), untranslated.cast(), size);
    (*list).m_TranslatedResourceList = new_translated;
    (*list).m_UntranslatedResourceList = new_untranslated;
    (*list).m_NumberOfEntries = count;
    (*list).m_MaxEntries = count;
    (*list).m_PoolType = pool;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn PcNewResourceSublist(
    out: *mut PRESOURCELIST,
    outer: PUNKNOWN,
    pool: POOL_TYPE,
    parent: PRESOURCELIST,
    maximum: ULONG,
) -> NTSTATUS {
    if out.is_null() || parent.is_null() || maximum == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let list = construct(pool, TAG_PORTCLASS, outer);
    if list.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let size = resource_size(maximum) as usize;
    (*list).m_TranslatedResourceList = allocate_item(pool, size, TAG_PORTCLASS).cast();
    if (*list).m_TranslatedResourceList.is_null() {
        destroy(list);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*list).m_UntranslatedResourceList = allocate_item(pool, size, TAG_PORTCLASS).cast();
    if (*list).m_UntranslatedResourceList.is_null() {
        destroy(list);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopyMemory(
        (*list).m_TranslatedResourceList.cast(),
        ((*(*parent).lpVtbl).TranslatedList.unwrap())(parent).cast(), // nolint: silent-failure — mandatory COM slot, malformed input fails loudly.
        core::mem::size_of::<CM_RESOURCE_LIST>(),
    ); // nolint: silent-failure — required COM vtable slot; malformed interfaces fail loudly.
    RtlCopyMemory(
        (*list).m_UntranslatedResourceList.cast(),
        ((*(*parent).lpVtbl).UntranslatedList.unwrap())(parent).cast(), // nolint: silent-failure — mandatory COM slot, malformed input fails loudly.
        core::mem::size_of::<CM_RESOURCE_LIST>(),
    ); // nolint: silent-failure — required COM vtable slot; malformed interfaces fail loudly.
    (*(*list).m_UntranslatedResourceList).List[0]
        .PartialResourceList
        .Count = 0;
    (*(*list).m_TranslatedResourceList).List[0]
        .PartialResourceList
        .Count = 0;
    (*list).m_OuterUnknown = outer;
    (*list).m_PoolType = pool;
    add_ref(list.cast());
    (*list).m_NumberOfEntries = 0;
    (*list).m_MaxEntries = maximum;
    *out = list.cast();
    STATUS_SUCCESS
}
