use crate::crypt32::proplist::ContextPropertyList_Copy;
use crate::crypt32::proplist::ContextPropertyList_Create;
use crate::crypt32::proplist::ContextPropertyList_Free;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn Context_CreateDataContext(mut contextSize: size_t, mut vtbl: *const context_vtbl_t, mut store: *mut WINE_CRYPTCERTSTORE) -> *mut context_t {
    let mut context: *mut context_t = core::ptr::null_mut::<context_t>();
    context = CryptMemAlloc(
        (core::mem::size_of::<context_t>() as usize).wrapping_add(contextSize as usize) as ULONG,
    ) as *mut context_t;
    if context.is_null() {
        return core::ptr::null_mut::<context_t>();
    }
    (*context).properties = ContextPropertyList_Create();
    if (*context).properties.is_null() {
        CryptMemFree(context as LPVOID);
        return core::ptr::null_mut::<context_t>();
    }
    (*context).vtbl = vtbl;
    (*context).ref_ = 1 as LONG;
    (*context).linked = core::ptr::null_mut::<_context_t>();
    (*(*store).vtbl).addref.unwrap_unchecked()(store as *mut WINE_CRYPTCERTSTORE);
    (*context).store = store as *mut WINE_CRYPTCERTSTORE;
    TRACE!("returning %p\n", context);
    return context;
}
#[no_mangle]
pub unsafe extern "C" fn Context_CreateLinkContext(mut contextSize: ::core::ffi::c_uint, mut linked: *mut context_t, mut store: *mut WINE_CRYPTCERTSTORE) -> *mut context_t {
    let mut context: *mut context_t = ::core::ptr::null_mut::<context_t>();
    WINE_TRACE!("(%d, %p)\n", contextSize, linked);
    context = CryptMemAlloc(
        (::core::mem::size_of::<context_t>() as usize).wrapping_add(contextSize as usize) as ULONG,
    ) as *mut context_t;
    if context.is_null() {
        return ::core::ptr::null_mut::<context_t>();
    }
    memcpy(
        context_ptr(context),
        context_ptr(linked),
        contextSize as size_t,
    );
    (*context).vtbl = (*linked).vtbl;
    (*context).ref_ = 1 as LONG;
    (*context).linked = linked as *mut _context_t;
    (*context).properties = (*linked).properties;
    Context_AddRef(linked);
    (*(*store).vtbl).addref.unwrap_unchecked()(store as *mut WINE_CRYPTCERTSTORE);
    (*context).store = store as *mut WINE_CRYPTCERTSTORE;
    WINE_TRACE!("returning %p\n", context);
    return context;
}
#[no_mangle]
pub unsafe extern "C" fn Context_AddRef(context: *mut context_t) {
    let ref_0: LONG = InterlockedIncrement(core::ptr::addr_of_mut!((*context).ref_)) as LONG;
    WINE_TRACE!("(%p) ref=%ld\n", context, (*context).ref_);
    if ref_0 == 1 {
        /* This is the first external (non-store) reference. Increase store ref cnt. */
        ((*(*(*context).store).vtbl).addref.unwrap_unchecked())((*context).store);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Context_Free(context: *mut context_t) {
    TRACE!("(%p)\n", context);
    assert!((*context).ref_ == 0);
    if (*context).linked.is_null() {
        ContextPropertyList_Free((*context).properties);
        (*(*context).vtbl).free.unwrap_unchecked()(context);
    } else {
        Context_Release((*context).linked as *mut context_t);
    }
    CryptMemFree(context as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn Context_Release(context: *mut context_t) {
    let mut ref_0: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*context).ref_));
    WINE_TRACE!("(%p) ref=%ld\n", context, ref_0);
    assert!(ref_0 >= 0);
    if ref_0 == 0 {
        let mut store: *mut WINECRYPT_CERTSTORE = (*context).store as *mut WINECRYPT_CERTSTORE;
        (*(*store).vtbl)
            .releaseContext
            .unwrap_unchecked()(store as *mut WINE_CRYPTCERTSTORE, context);
        (*(*store).vtbl).release.unwrap_unchecked()(
            store as *mut WINE_CRYPTCERTSTORE,
            0 as DWORD,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Context_CopyProperties(to: *const ::core::ffi::c_void, from: *const ::core::ffi::c_void) {
    let mut toProperties: *mut CONTEXT_PROPERTY_LIST;
    let mut fromProperties: *mut CONTEXT_PROPERTY_LIST;

    toProperties = (*context_from_ptr(to)).properties;
    fromProperties = (*context_from_ptr(from)).properties;
    assert!(!toProperties.is_null() && !fromProperties.is_null());
    ContextPropertyList_Copy(toProperties, fromProperties);
}
