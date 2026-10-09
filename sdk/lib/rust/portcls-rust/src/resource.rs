// Retained paid workers from .responses/20260912-221905-resource_rust_reference.c.
use crate::resource_support::{
    RustBridge_CResourceList_call_CResourceList_AddEntry,
    RustBridge_CResourceList_call_IResourceList_FindTranslatedEntry,
    RustBridge_CResourceList_call_IResourceList_FindUntranslatedEntry,
};
use crate::types::*;
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::{PASSIVE_LEVEL, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS};

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_NumberOfEntries(
    context: *mut RustBridge_CResourceList_Context,
) -> u32 {
    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);
    return *(*context).m_NumberOfEntries;
}

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_NumberOfEntriesOfType(
    context: *mut RustBridge_CResourceList_Context,
    Type: CM_RESOURCE_TYPE,
) -> u32 {
    let mut Index: u32;
    let mut Count: u32 = 0;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);

    if (*(*context).m_UntranslatedResourceList).is_null() {
        return 0;
    }

    Index = 0;
    while Index < *(*context).m_NumberOfEntries {
        PartialDescriptor = (core::ptr::addr_of_mut!(
            (*(*(*context).m_UntranslatedResourceList)).List[0]
                .PartialResourceList
                .PartialDescriptors
        ) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset(Index as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

        if ((*PartialDescriptor).Type) as i32 == Type {
            Count = Count.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }

    return Count;
}

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_FindTranslatedEntry(
    context: *mut RustBridge_CResourceList_Context,
    Type: CM_RESOURCE_TYPE,
    Index: u32,
) -> PCM_PARTIAL_RESOURCE_DESCRIPTOR {
    let mut DescIndex: u32;
    let mut Count: u32 = 0;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);

    if (*(*context).m_TranslatedResourceList).is_null() {
        return core::ptr::null_mut();
    }

    DescIndex = 0;
    while DescIndex < *(*context).m_NumberOfEntries {
        PartialDescriptor = core::ptr::addr_of_mut!(
            (**(*context).m_TranslatedResourceList).List[0]
                .PartialResourceList
                .PartialDescriptors
        )
        .cast::<CM_PARTIAL_RESOURCE_DESCRIPTOR>()
        .add(DescIndex as usize);

        if ((*PartialDescriptor).Type) as i32 == Type {
            if Index == Count {
                return PartialDescriptor;
            }
            Count = Count.wrapping_add(1);
        }
        DescIndex = DescIndex.wrapping_add(1);
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_FindUntranslatedEntry(
    context: *mut RustBridge_CResourceList_Context,
    Type: CM_RESOURCE_TYPE,
    Index: u32,
) -> PCM_PARTIAL_RESOURCE_DESCRIPTOR {
    let mut DescIndex: u32;
    let mut Count: u32 = 0;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);

    if (*(*context).m_UntranslatedResourceList).is_null() {
        return core::ptr::null_mut();
    }

    DescIndex = 0;
    while DescIndex < *(*context).m_NumberOfEntries {
        let resource_list = *(*context).m_UntranslatedResourceList;
        let full_descriptor_ptr = (*resource_list).List.as_mut_ptr().add(0 as usize);

        PartialDescriptor = core::ptr::addr_of_mut!(
            (*full_descriptor_ptr)
                .PartialResourceList
                .PartialDescriptors
        )
        .cast::<CM_PARTIAL_RESOURCE_DESCRIPTOR>()
        .add(DescIndex as usize);

        if ((*PartialDescriptor).Type) as i32 == Type {
            if Index == Count {
                return PartialDescriptor;
            }

            Count = Count.wrapping_add(1);
        }

        DescIndex = DescIndex.wrapping_add(1);
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_AddEntry(
    context: *mut RustBridge_CResourceList_Context,
    Translated: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Untranslated: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);

    if *(*context).m_NumberOfEntries >= *(*context).m_MaxEntries {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    PartialDescriptor = (&raw mut (*(&raw mut (**(*context).m_UntranslatedResourceList).List
        as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize))
    .PartialResourceList
    .PartialDescriptors as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
        .offset(*(*context).m_NumberOfEntries as isize)
        as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    RtlCopyMemory(
        PartialDescriptor as *mut core::ffi::c_void,
        Untranslated as *const core::ffi::c_void,
        core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
    );

    PartialDescriptor = (&raw mut (*(&raw mut (**(*context).m_TranslatedResourceList).List
        as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize))
    .PartialResourceList
    .PartialDescriptors as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
        .offset(*(*context).m_NumberOfEntries as isize)
        as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    RtlCopyMemory(
        PartialDescriptor as *mut core::ffi::c_void,
        Translated as *const core::ffi::c_void,
        core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
    );

    *(*context).m_NumberOfEntries += 1;
    (*(&raw mut (**(*context).m_UntranslatedResourceList).List
        as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize))
    .PartialResourceList
    .Count += 1;
    (*(&raw mut (**(*context).m_TranslatedResourceList).List
        as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize))
    .PartialResourceList
    .Count += 1;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_AddEntryFromParent(
    context: *mut RustBridge_CResourceList_Context,
    Parent: *mut IResourceList,
    Type: CM_RESOURCE_TYPE,
    Index: ULONG,
) -> NTSTATUS {
    let Translated: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let Untranslated: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);

    Translated =
        RustBridge_CResourceList_call_IResourceList_FindTranslatedEntry(Parent, Type, Index);
    Untranslated =
        RustBridge_CResourceList_call_IResourceList_FindUntranslatedEntry(Parent, Type, Index);

    if !Translated.is_null() && !Untranslated.is_null() {
        return RustBridge_CResourceList_call_CResourceList_AddEntry(
            (*context).object,
            Translated,
            Untranslated,
        );
    }

    return STATUS_INVALID_PARAMETER;
}

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_TranslatedList(
    context: *mut RustBridge_CResourceList_Context,
) -> PCM_RESOURCE_LIST {
    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);
    return *(*context).m_TranslatedResourceList;
}

#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_UntranslatedList(
    context: *mut RustBridge_CResourceList_Context,
) -> PCM_RESOURCE_LIST {
    PC_ASSERT_IRQL_EQUAL!(PASSIVE_LEVEL);

    return *(*context).m_UntranslatedResourceList;
}
