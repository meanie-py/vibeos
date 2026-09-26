// rustconv-assembly: free-functions
//! `CFileSysBindData.cpp`'s three file-scope functions — the bind-context helpers that
//! wrap the COM class in `CFileSysBindData.rs`.
//!
//! The class half and this half are separate rustconv runs on one TU (`--cpp-com --class`
//! and `--free-functions`), which is why they are two modules: a re-run overwrites its own
//! generated file wholesale, and rustconv refuses to let one slice land on the other's path.
//! Same split as `CIDLDataObj.rs` / `cidldataobj_free.rs`.
//!
//! **All three bodies are HAND-WRITTEN against the C**, and that is worth recording rather
//! than hiding. `FileSystemBindData_GetFindData` was an AUDIT REJECT ("CComPtr local pUnk:
//! expected one null raw-pointer declaration, found 0") and was `--skip-fn`'d. Reading the
//! rejected `.partial` against the C showed the audit was RIGHT — but it also showed the two
//! bodies that PASSED were worse: method-call syntax on raw COM pointers (`(*pbc).GetObject
//! Param(..)`), a mangled `IID_IFileSystemBindData!(..)` macro invocation, a `let ret` bound
//! once and assigned three times, and — in both — a `CppComPtrGuard` *plus* hand-rolled
//! `Release()` calls, which is the double-release class. None of it compiled, so the compiler
//! caught this one; the lesson is the field manual's, that the flagged list is not the whole
//! risk and a passing audit is not a correct body.

use crate::prelude::*;
use crate::shell32::utils_support::STR_FILE_SYS_BIND_DATA_W;
use crate::shell32::CFileSysBindData::{
    CFileSysBindData_CreateInstance, IFileSystemBindData, IID_IFileSystemBindData,
};

/// The `CComPtr` destructor, as the rest of the crate spells it: release on every exit path
/// and NULL the slot, so a `Detach` is expressed by nulling the slot before returning.
pub struct CppComPtrGuard<T> {
    slot: *mut *mut T,
}
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self {
        Self { slot }
    }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

/// `CFileSysBindData.cpp:50` `IFileSystemBindData_Constructor`.
///
/// `*ppV = bindContext.Detach()` is the one subtlety: the bind context is handed to the
/// caller, so the guard must NOT release it. Nulling the slot before returning IS `Detach`.
#[no_mangle]
pub unsafe extern "system" fn IFileSystemBindData_Constructor(
    pfd: *const WIN32_FIND_DATAW,
    ppV: *mut *mut IBindCtx,
) -> HRESULT {
    let mut fileSystemBindData: *mut IFileSystemBindData = core::ptr::null_mut();
    let _fileSystemBindData_cpp_com_guard =
        CppComPtrGuard::new(core::ptr::addr_of_mut!(fileSystemBindData));
    let mut bindContext: *mut IBindCtx = core::ptr::null_mut();
    let _bindContext_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(bindContext));
    let mut bindOpts: BIND_OPTS = core::mem::zeroed();
    let mut hResult: HRESULT;

    if ppV.is_null() {
        return E_INVALIDARG;
    }

    *ppV = core::ptr::null_mut();

    hResult = CFileSysBindData_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IFileSystemBindData).cast(),
        core::ptr::addr_of_mut!(fileSystemBindData).cast(),
    );
    if hResult < 0 {
        return hResult;
    }
    hResult = ((*(*fileSystemBindData).lpVtbl).SetFindData)(fileSystemBindData, pfd.cast());
    if hResult < 0 {
        return hResult;
    }

    hResult = CreateBindCtx(0, core::ptr::addr_of_mut!(bindContext).cast());
    if hResult < 0 {
        return hResult;
    }
    bindOpts.cbStruct = core::mem::size_of::<BIND_OPTS>() as DWORD;
    bindOpts.grfFlags = 0;
    bindOpts.grfMode = STGM_CREATE;
    bindOpts.dwTickCountDeadline = 0;
    hResult = ((*(*bindContext).lpVtbl).SetBindOptions)(bindContext, core::ptr::addr_of_mut!(bindOpts));
    if hResult < 0 {
        return hResult;
    }
    hResult = ((*(*bindContext).lpVtbl).RegisterObjectParam)(
        bindContext,
        STR_FILE_SYS_BIND_DATA_W.as_ptr() as LPOLESTR,
        fileSystemBindData as *mut IUnknown,
    );
    if hResult < 0 {
        return hResult;
    }

    // `bindContext.Detach()` — ownership moves to the caller, so the guard must not release.
    *ppV = bindContext;
    bindContext = core::ptr::null_mut();
    let _ = bindContext;

    S_OK
}

/// `CFileSysBindData.cpp:92` `FileSystemBindData_GetFindData`.
#[no_mangle]
pub unsafe extern "system" fn FileSystemBindData_GetFindData(
    pbc: *mut IBindCtx,
    pfd: *mut WIN32_FIND_DATAW,
) -> HRESULT {
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    let _pUnk_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pUnk));
    let mut pfsbd: *mut IFileSystemBindData = core::ptr::null_mut();
    let _pfsbd_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pfsbd));
    let mut ret: HRESULT;

    if pfd.is_null() {
        return E_INVALIDARG;
    }

    ret = ((*(*pbc).lpVtbl).GetObjectParam)(
        pbc,
        STR_FILE_SYS_BIND_DATA_W.as_ptr() as LPOLESTR,
        core::ptr::addr_of_mut!(pUnk),
    );
    if ret >= 0 {
        ret = ((*(*pUnk).lpVtbl).QueryInterface)(
            pUnk,
            core::ptr::addr_of!(IID_IFileSystemBindData).cast(),
            core::ptr::addr_of_mut!(pfsbd).cast(),
        );
        if ret >= 0 {
            ret = ((*(*pfsbd).lpVtbl).GetFindData)(pfsbd, pfd.cast());
        }
    }
    ret
}

/// `CFileSysBindData.cpp:113` `FileSystemBindData_SetFindData`.
///
/// The C has NO `if (!pfd)` guard here, unlike `GetFindData` — carried as written.
#[no_mangle]
pub unsafe extern "system" fn FileSystemBindData_SetFindData(
    pbc: *mut IBindCtx,
    pfd: *const WIN32_FIND_DATAW,
) -> HRESULT {
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    let _pUnk_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pUnk));
    let mut pfsbd: *mut IFileSystemBindData = core::ptr::null_mut();
    let _pfsbd_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pfsbd));
    let mut ret: HRESULT;

    ret = ((*(*pbc).lpVtbl).GetObjectParam)(
        pbc,
        STR_FILE_SYS_BIND_DATA_W.as_ptr() as LPOLESTR,
        core::ptr::addr_of_mut!(pUnk),
    );
    if ret >= 0 {
        ret = ((*(*pUnk).lpVtbl).QueryInterface)(
            pUnk,
            core::ptr::addr_of!(IID_IFileSystemBindData).cast(),
            core::ptr::addr_of_mut!(pfsbd).cast(),
        );
        if ret >= 0 {
            ret = ((*(*pfsbd).lpVtbl).SetFindData)(pfsbd, pfd.cast());
        }
    }
    ret
}
