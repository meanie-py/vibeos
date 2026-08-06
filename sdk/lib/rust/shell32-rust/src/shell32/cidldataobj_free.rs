use crate::prelude::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------------------
// `ShellObjectCreatorInit<T>(args..., riid, ppv)` is the ATL template that allocates T,
// runs its ctor, calls `T::Initialize(args...)` and QIs. The COM scaffold generates exactly
// that per class as `<T>_CreateInstance`, with the same argument order, so the template
// instantiation maps 1:1 onto the generated factory — there is nothing generic left to model.
// ---------------------------------------------------------------------------------------
use crate::shell32::CIDLDataObj::CIDLDataObj_CreateInstance;
use crate::win32::IsEqualIID;
use crate::shell32::IEnumFORMATETCImpl::IEnumFORMATETCImpl_CreateInstance;

type LPFORMATETC = *mut FORMATETC;

extern "C" {
    pub static IID_IDataObject: IID;
    pub static IID_IEnumFORMATETC: IID;
}

// ---- file-scope preamble (translated) ----

pub type PIEnumFORMATETCImpl = *mut crate::shell32::IEnumFORMATETCImpl::IEnumFORMATETCImpl;


pub type PCIDLDataObj = *mut crate::shell32::CIDLDataObj::CIDLDataObj;


#[no_mangle]
pub unsafe extern "C" fn IEnumFORMATETC_Constructor(
    cfmt: UINT,
    afmt: *const FORMATETC,
    ppFormat: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    IEnumFORMATETCImpl_CreateInstance(cfmt, afmt, core::ptr::addr_of!(IID_IEnumFORMATETC), (ppFormat).cast())
}
#[no_mangle]
pub unsafe extern "C" fn IDataObject_Constructor(
    hwndOwner: HWND,
    pMyPidl: PCIDLIST_ABSOLUTE,
    apidl: PCUIDLIST_RELATIVE_ARRAY,
    cidl: UINT,
    bExtendedObject: BOOL,
    dataObject: *mut *mut IDataObject,
) -> HRESULT {
    if dataObject.is_null() {
        return E_INVALIDARG;
    }
    return CIDLDataObj_CreateInstance(
        hwndOwner,
        pMyPidl,
        apidl,
        cidl,
        bExtendedObject,
        core::ptr::addr_of!(IID_IDataObject), (dataObject).cast(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateDataObject(
    pidlFolder: PCIDLIST_ABSOLUTE,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    pdtInner: *mut IDataObject,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if IsEqualIID(riid.cast(), core::ptr::addr_of!(IID_IDataObject).cast()) {
        if !pdtInner.is_null() {
            // UNIMPLEMENTED (ReactOS debug macro; no-op in a release build)
        }
        return IDataObject_Constructor(
            core::ptr::null_mut(),
            pidlFolder,
            apidl,
            cidl,
            TRUE,
            ppv as *mut *mut IDataObject,
        );
    }
    E_FAIL
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateFileDataObject(
    pidlFolder: PCIDLIST_ABSOLUTE,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    pDataInner: *mut IDataObject,
    ppDataObj: *mut *mut IDataObject,
) -> HRESULT {
    if !pDataInner.is_null() {
        // UNIMPLEMENTED (ReactOS debug macro; no-op in a release build)
    }
    IDataObject_Constructor(
        core::ptr::null_mut(),
        pidlFolder,
        apidl,
        cidl,
        TRUE,
        ppDataObj,
    )
}
