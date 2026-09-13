//! Platform operations from miniport.cpp/pool.cpp, and resource.cpp QueryInterface.
use crate::resource_owner;
use crate::types::*;
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::{PASSIVE_LEVEL, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS};

pub use crate::types::RUSTOWNED_TAG_PORTCLASS as TAG_PORTCLASS;
unsafe extern "C" {
    static IID_IUnknown: GUID;
}

// Placement new in miniport.cpp: tagged allocation, zero on success.
pub unsafe fn allocate_object(pool: POOL_TYPE, size: usize, tag: ULONG) -> PVOID {
    let p = ExAllocatePoolWithTag(pool, size as _, tag);
    if !p.is_null() {
        core::ptr::write_bytes(p.cast::<u8>(), 0, size);
    }
    p
}
pub unsafe fn free_object(p: PVOID) {
    ExFreePool(p);
}
pub unsafe fn allocate_item(pool: POOL_TYPE, size: usize, tag: ULONG) -> PVOID {
    ntkrnl_sys::inline::mm::ExAllocatePoolZero(pool, size as _, tag)
}
pub unsafe fn free_item(p: PVOID, tag: ULONG) {
    ExFreePoolWithTag(p, tag);
}
unsafe fn equal_guid(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}
pub unsafe extern "system" fn query_interface(
    this: *mut IResourceList,
    iid: *const IID,
    output: *mut PVOID,
) -> NTSTATUS {
    if equal_guid(iid, &raw const IID_IResourceList) || equal_guid(iid, &raw const IID_IUnknown) {
        *output = this.cast();
        ((*(*this).lpVtbl).AddRef.unwrap())(this); // nolint: silent-failure — required COM vtable slot; malformed interfaces fail loudly.
        return STATUS_SUCCESS;
    }
    let mut text: UNICODE_STRING = core::mem::zeroed();
    if RtlStringFromGUID(iid, &raw mut text) == STATUS_SUCCESS {
        DbgPrint(
            b"(resource.cpp) IResourceList_QueryInterface no interface!!! iface %S\n\0"
                .as_ptr()
                .cast(),
            text.Buffer,
        );
        RtlFreeUnicodeString(&raw mut text);
    }
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_call_CResourceList_AddEntry(
    object: PVOID,
    translated: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    untranslated: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    resource_owner::AddEntry(object.cast(), translated, untranslated)
}
#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_call_IResourceList_FindTranslatedEntry(
    object: *mut IResourceList,
    typ: CM_RESOURCE_TYPE,
    index: ULONG,
) -> PCM_PARTIAL_RESOURCE_DESCRIPTOR {
    ((*(*object).lpVtbl).FindTranslatedEntry.unwrap())(object, typ, index) // nolint: silent-failure — required COM vtable slot; malformed interfaces fail loudly.
}
#[no_mangle]
pub unsafe extern "C" fn RustBridge_CResourceList_call_IResourceList_FindUntranslatedEntry(
    object: *mut IResourceList,
    typ: CM_RESOURCE_TYPE,
    index: ULONG,
) -> PCM_PARTIAL_RESOURCE_DESCRIPTOR {
    ((*(*object).lpVtbl).FindUntranslatedEntry.unwrap())(object, typ, index) // nolint: silent-failure — required COM vtable slot; malformed interfaces fail loudly.
}
