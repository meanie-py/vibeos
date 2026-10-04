//! oleproxy.c — ole32's proxy/stub class factory + the `DllGetClassObject` export.
//!
//! One `IClassFactory` vtbl shape (`ClassFactory_{QueryInterface,AddRef,Release,
//! LockServer}`) shared by eight non-heap-based, statically-allocated class
//! factories; only the `CreateInstance` slot differs per CLSID.
use crate::prelude::*;
use core::ffi::c_void;

// ---- IID/CLSID data symbols (uuid.lib) not (yet) declared in crate::iids ----
// IID_IUnknown / IID_IClassFactory / CLSID_*Moniker / CLSID_DfMarshal come from the
// prelude. These two do not; both are real `R` symbols in libuuid.a, so DECLARE them
// (never transcribe a GUID by hand — a wrong byte is a silent QueryInterface miss).
extern "C" {
    static CLSID_StdGlobalInterfaceTable: GUID;
    static CLSID_StdComponentCategoriesMgr: GUID;
}

// ---- ole32-internal entry points ----
// compobj/marshal carry their own hand-rolled FFI surfaces (their `IClassFactory` is a
// DIFFERENT Rust type from com-sys's), so they are not reachable through the prelude;
// reach them at the symbol level instead. ABIs are per compobj_private.h:
//   * MARSHAL_GetStandardMarshalCF / StdGlobalInterfaceTable_GetFactory /
//     Handler_DllGetClassObject — plain `HRESULT foo(...)`, no WINAPI → cdecl.
//   * OLE32_DllGetClassObject (widl dlldata.c) — `HRESULT WINAPI` → stdcall on i386.
//     Declaring THIS one `extern "C"` (as the raw translation did) links fine on
//     amd64/arm64 and silently corrupts the stack on i386.
//   * GlobalOptions_CreateInstance — `HRESULT WINAPI` (compobj_private.h:316).
extern "C" {
    fn Handler_DllGetClassObject(rclsid: *const GUID, riid: *const GUID, ppv: *mut LPVOID) -> HRESULT;
    fn MARSHAL_GetStandardMarshalCF(ppv: *mut LPVOID) -> HRESULT;
    fn StdGlobalInterfaceTable_GetFactory(ppv: *mut LPVOID) -> HRESULT;
}
extern "system" {
    fn OLE32_DllGetClassObject(rclsid: *const GUID, iid: *const GUID, ppv: *mut LPVOID) -> HRESULT;
    fn GlobalOptions_CreateInstance(
        iface: *mut IClassFactory,
        pUnkOuter: *mut IUnknown,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}

// ---- the shared IClassFactory methods (C: static, WINAPI) ----

stdmethod! {
    fn ClassFactory_QueryInterface(
        iface: *mut IClassFactory,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        TRACE!("(%p)->(%s %p)\n", iface, debugstr_guid(riid as *const c_void), ppv);

        if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown))
            || IsEqualIID(riid, core::ptr::addr_of!(IID_IClassFactory))
        {
            *ppv = iface as LPVOID;
            return S_OK;
        }

        *ppv = core::ptr::null_mut();
        E_NOINTERFACE
    }
}

stdmethod! {
    fn ClassFactory_AddRef(_iface: *mut IClassFactory) -> ULONG {
        2 /* non-heap based object */
    }
}

stdmethod! {
    fn ClassFactory_Release(_iface: *mut IClassFactory) -> ULONG {
        1 /* non-heap based object */
    }
}

stdmethod! {
    fn ClassFactory_LockServer(_iface: *mut IClassFactory, fLock: BOOL) -> HRESULT {
        TRACE!("(%x)\n", fLock);
        S_OK
    }
}

/// Define one of oleproxy.c's static class factories.
///
/// The ring's `*_CreateInstance` entry points are Rust items declared
/// `extern "system"` (their C originals are WINAPI). com-sys spells the
/// `IClassFactoryVtbl::CreateInstance` slot CONCRETELY — `extern "C"` on
/// amd64/arm64, `extern "stdcall"` on i386, i.e. exactly what STDMETHODCALLTYPE
/// compiles to — and rustc keeps `extern "system"` a *nominally distinct* fn type
/// even where it lowers identically, so a `*_CreateInstance` cannot be dropped into
/// the slot directly. The `stdmethod!` thunk below does the (ABI-identical) forward.
///
/// `IClassFactory` holds a raw `lpVtbl` pointer, so it is `!Sync` → the factory
/// objects are `static mut` (as in C, where they are mutable globals) and are only
/// ever handed out by address.
macro_rules! class_factory {
    ($vtbl:ident, $cf:ident, $thunk:ident, $create:ident $(, #[$attr:meta])?) => {
        stdmethod! {
            fn $thunk(
                iface: *mut IClassFactory,
                pUnkOuter: *mut IUnknown,
                riid: *const IID,
                ppv: *mut *mut c_void,
            ) -> HRESULT {
                $create(iface, pUnkOuter, riid, ppv)
            }
        }

        static $vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
            QueryInterface: Some(ClassFactory_QueryInterface),
            AddRef: Some(ClassFactory_AddRef),
            Release: Some(ClassFactory_Release),
            CreateInstance: Some($thunk),
            LockServer: Some(ClassFactory_LockServer),
        };

        $(#[$attr])?
        pub static mut $cf: IClassFactory = IClassFactory {
            lpVtbl: &$vtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
        };
    };
}

class_factory!(FileMonikerCFVtbl, FileMonikerCF, FileMonikerCF_CreateInstance, FileMoniker_CreateInstance);
class_factory!(ItemMonikerCFVtbl, ItemMonikerCF, ItemMonikerCF_CreateInstance, ItemMoniker_CreateInstance);
class_factory!(AntiMonikerCFVtbl, AntiMonikerCF, AntiMonikerCF_CreateInstance, AntiMoniker_CreateInstance);
class_factory!(CompositeMonikerCFVtbl, CompositeMonikerCF, CompositeMonikerCF_CreateInstance, CompositeMoniker_CreateInstance);
class_factory!(ClassMonikerCFVtbl, ClassMonikerCF, ClassMonikerCF_CreateInstance, ClassMoniker_CreateInstance);
class_factory!(PointerMonikerCFVtbl, PointerMonikerCF, PointerMonikerCF_CreateInstance, PointerMoniker_CreateInstance);
class_factory!(ObjrefMonikerCFVtbl, ObjrefMonikerCF, ObjrefMonikerCF_CreateInstance, ObjrefMoniker_CreateInstance);
class_factory!(ComCatCFVtbl, ComCatCF, ComCatCF_CreateInstance, ComCat_CreateInstance);

// C: `IClassFactory GlobalOptionsCF = { &GlobalOptionsCFVtbl };` — the ONE object in
// oleproxy.c that is NOT `static`. compobj.c reaches it by name
// (compobj_private.h:318 `extern IClassFactory GlobalOptionsCF`), and so does our
// compobj.rs (`extern { pub static mut GlobalOptionsCF: IClassFactory; }`), so this
// definition MUST carry #[no_mangle] — without it the Rust static is name-mangled and
// compobj's extern goes unresolved the moment oleproxy.c is cohab-flipped out.
class_factory!(GlobalOptionsCFVtbl, GlobalOptionsCF, GlobalOptionsCF_CreateInstance, GlobalOptions_CreateInstance, #[no_mangle]);

/***********************************************************************
 *           DllGetClassObject [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    iid: *const GUID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let hr: HRESULT;

    *ppv = core::ptr::null_mut();
    if IsEqualIID(rclsid, core::ptr::addr_of!(CLSID_DfMarshal))
        && (IsEqualIID(iid, core::ptr::addr_of!(IID_IClassFactory))
            || IsEqualIID(iid, core::ptr::addr_of!(IID_IUnknown)))
    {
        return MARSHAL_GetStandardMarshalCF(ppv);
    }
    if IsEqualIID(rclsid, core::ptr::addr_of!(CLSID_StdGlobalInterfaceTable))
        && (IsEqualIID(iid, core::ptr::addr_of!(IID_IClassFactory))
            || IsEqualIID(iid, core::ptr::addr_of!(IID_IUnknown)))
    {
        return StdGlobalInterfaceTable_GetFactory(ppv);
    }
    if IsEqualCLSID(rclsid, core::ptr::addr_of!(CLSID_FileMoniker)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(FileMonikerCF), iid, ppv);
    }
    if IsEqualCLSID(rclsid, core::ptr::addr_of!(CLSID_ItemMoniker)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(ItemMonikerCF), iid, ppv);
    }
    if IsEqualCLSID(rclsid, core::ptr::addr_of!(CLSID_AntiMoniker)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(AntiMonikerCF), iid, ppv);
    }
    if IsEqualCLSID(rclsid, core::ptr::addr_of!(CLSID_CompositeMoniker)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(CompositeMonikerCF), iid, ppv);
    }
    if IsEqualCLSID(rclsid, core::ptr::addr_of!(CLSID_ClassMoniker)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(ClassMonikerCF), iid, ppv);
    }
    if IsEqualCLSID(rclsid, core::ptr::addr_of!(CLSID_PointerMoniker)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(PointerMonikerCF), iid, ppv);
    }
    if IsEqualCLSID(rclsid, core::ptr::addr_of!(CLSID_ObjrefMoniker)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(ObjrefMonikerCF), iid, ppv);
    }
    if IsEqualGUID(rclsid, core::ptr::addr_of!(CLSID_StdComponentCategoriesMgr)) {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(ComCatCF), iid, ppv);
    }

    hr = OLE32_DllGetClassObject(rclsid, iid, ppv);
    if SUCCEEDED(hr) {
        return hr;
    }

    Handler_DllGetClassObject(rclsid, iid, ppv)
}
