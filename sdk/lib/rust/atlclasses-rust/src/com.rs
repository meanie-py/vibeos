//! ATL's COM object runtime (atlcom.h): `CComObjectRootEx`'s reference count, the
//! `BEGIN_COM_MAP` walk behind `QueryInterface`, `CComObject<T>::CreateInstance`, and the
//! module lock count `CComObject` holds while an object lives.
//!
//! In C++ the most-derived `CComObject<T>` supplies `IUnknown` over `T`. `cpp_app` renders no
//! wrapper class: it gives `T` itself the three `IUnknown` methods with `CComObject`'s
//! semantics (calling these helpers) and implements [`ComObject`] for it, so the object a
//! module creates IS its class, laid out with the primary interface's vptr at offset 0 as
//! C++ puts it.
//!
//! Aggregation is not provided: `CComObjectRootBase` keeps `m_dwRef` in a union with the
//! outer unknown, used only by `CComAggObject`/`CComPolyObject`, and `cpp_app` refuses a
//! class whose COM map needs them.

use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};

use crate::CComPtr;

pub type HRESULT = i32;
pub const S_OK: HRESULT = 0;
pub const E_POINTER: HRESULT = 0x8000_4003_u32 as i32;
pub const E_NOINTERFACE: HRESULT = 0x8000_4002_u32 as i32;
pub const E_OUTOFMEMORY: HRESULT = 0x8007_000E_u32 as i32;

/// winerror.h `FAILED(hr)`.
#[inline]
pub fn failed(hr: HRESULT) -> bool {
    hr < 0
}

/// The data `CComObjectRootEx<ThreadModel>` adds: `CComObjectRootBase::m_dwRef`. The thread
/// model only picks the increment (`Interlocked*` for the multi-threaded models, `++`/`--`
/// for `CComSingleThreadModel`); the interlocked form is correct for both, so it is used.
/// `CComMultiThreadModel`'s critical section guards `Lock()`/`Unlock()`, which a module
/// calls by name and `cpp_app` refuses until one does.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct CComObjectRootEx {
    pub m_dwRef: i32,
}

/// `CComCoClass<T, pclsid>`: no data, only static members (`CreateInstance`, `Error`,
/// `GetObjectCLSID`), which the class reaches through `cpp_app`'s glue.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct CComCoClass;

/// One `COM_INTERFACE_ENTRY`: the IID (as `*const GUID` of the module's island) and the
/// offset of the interface subobject within the object.
#[derive(Clone, Copy)]
pub struct ComMapEntry {
    pub iid: *const c_void,
    pub offset: usize,
}

static MODULE_LOCKS: AtomicI32 = AtomicI32::new(0);

/// `_pAtlModule->Lock()`: a live `CComObject` holds the module (its constructor).
pub fn module_lock() -> i32 {
    MODULE_LOCKS.fetch_add(1, Ordering::SeqCst) + 1
}

/// `_pAtlModule->Unlock()` (the `CComObject` destructor).
pub fn module_unlock() -> i32 {
    MODULE_LOCKS.fetch_sub(1, Ordering::SeqCst) - 1
}

/// `_pAtlModule->GetLockCount()`: what `DllCanUnloadNow` asks.
pub fn module_lock_count() -> i32 {
    MODULE_LOCKS.load(Ordering::SeqCst)
}

/// `InternalAddRef`.
pub unsafe fn internal_add_ref(root: *mut CComObjectRootEx) -> u32 {
    let r = &*(core::ptr::addr_of_mut!((*root).m_dwRef) as *const AtomicI32);
    (r.fetch_add(1, Ordering::SeqCst) + 1) as u32
}

/// `InternalRelease`: the count after the decrement; the caller destroys the object at 0.
pub unsafe fn internal_release(root: *mut CComObjectRootEx) -> u32 {
    let r = &*(core::ptr::addr_of_mut!((*root).m_dwRef) as *const AtomicI32);
    (r.fetch_sub(1, Ordering::SeqCst) - 1) as u32
}

const IID_IUNKNOWN: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0xC0, 0, 0, 0, 0, 0, 0, 0x46];

unsafe fn iid_eq(a: *const c_void, b: *const c_void) -> bool {
    core::slice::from_raw_parts(a as *const u8, 16)
        == core::slice::from_raw_parts(b as *const u8, 16)
}

/// `IUnknown::AddRef` through a vtable: slot 1 of any interface.
pub unsafe fn com_add_ref(p: *mut c_void) -> u32 {
    type AddRef = unsafe extern "system" fn(*mut c_void) -> u32;
    let vtbl = *(p as *const *const AddRef);
    (*vtbl.add(1))(p)
}

/// `AtlInternalQueryInterface` over a `BEGIN_COM_MAP` table. `IID_IUnknown` answers with
/// the FIRST entry's interface; an empty map (rapps' `CDownloadManager`) has only the
/// terminator, whose offset is 0 -- the object itself, its primary interface.
pub unsafe fn internal_query_interface(
    this: *mut c_void,
    map: &[ComMapEntry],
    riid: *const c_void,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if riid.is_null() {
        return E_NOINTERFACE;
    }
    let hit = if iid_eq(riid, IID_IUNKNOWN.as_ptr() as *const c_void) {
        Some(map.first().map_or(0, |e| e.offset))
    } else {
        map.iter().find(|e| iid_eq(riid, e.iid)).map(|e| e.offset)
    };
    match hit {
        Some(off) => {
            let p = (this as *mut u8).add(off) as *mut c_void;
            com_add_ref(p);
            *ppv = p;
            S_OK
        }
        None => E_NOINTERFACE,
    }
}

/// A class with a COM map: what `CComObject<T>` provides over it.
pub trait ComObject: Sized {
    /// `CComObject<T>::CreateInstance(&p)`: a new object, constructed and `FinalConstruct`ed,
    /// reference count 0 (the caller's first `AddRef` owns it). On failure `*pp` is null and
    /// the object is already destroyed.
    unsafe fn create_instance(pp: *mut *mut Self) -> HRESULT;
}

/// shellutils.h `ShellObjectCreator(CComPtr<T>& objref)`: create, then hold it in `objref`
/// (the assignment's AddRef is the reference the caller keeps).
pub unsafe fn ShellObjectCreator<T: ComObject>(objref: &mut CComPtr<T>) -> HRESULT {
    let mut p: *mut T = core::ptr::null_mut();
    let hr = T::create_instance(&mut p);
    objref.assign(p);
    if failed(hr) {
        return hr;
    }
    S_OK
}

/// shellutils.h `ShellObjectCreator<T>(riid, ppv)`: create, AddRef, QueryInterface,
/// Release (which destroys the object if the QI did not keep it).
#[macro_export]
macro_rules! ShellObjectCreatorIid {
    ($T:ty, $riid:expr, $ppv:expr $(,)?) => {{
        let mut __pobj: *mut $T = core::ptr::null_mut();
        let mut __hr = <$T as $crate::com::ComObject>::create_instance(&mut __pobj);
        if __hr >= 0 {
            (*__pobj).AddRef();
            __hr = (*__pobj).QueryInterface(($riid) as *const _ as _, ($ppv) as *mut _ as _);
            (*__pobj).Release();
        }
        __hr
    }};
}

/// shellutils.h `ShellObjectCreatorInit<T>(args.., riid, ppv)`: create, AddRef, call
/// `Initialize(args..)`, then QueryInterface on success, and Release (which destroys the
/// object if the QI did not keep it). C++ deduces the arity from the template; spelled
/// `ShellObjectCreatorInit!(T, [args..], riid, ppv)` here, or
/// `ShellObjectCreatorInit!(T, Initialize__1, [args..], riid, ppv)` when T overloads
/// Initialize: C++ picks the overload by argument type, which a macro cannot (ntobjshex's
/// CNtObjectFolder has IPersistFolder's Initialize(pidl) beside its own (pidl, ntPath)).
#[macro_export]
macro_rules! ShellObjectCreatorInit {
    ($T:ty, $init:ident, [$($arg:expr),* $(,)?], $riid:expr, $ppv:expr $(,)?) => {{
        let mut __pobj: *mut $T = core::ptr::null_mut();
        let mut __hr = <$T as $crate::com::ComObject>::create_instance(&mut __pobj);
        if __hr >= 0 {
            (*__pobj).AddRef();
            __hr = (*__pobj).$init($($arg),*);
            if __hr >= 0 {
                __hr = (*__pobj).QueryInterface(($riid) as *const _ as _, ($ppv) as *mut _ as _);
            }
            (*__pobj).Release();
        }
        __hr
    }};
    ($T:ty, [$($arg:expr),* $(,)?], $riid:expr, $ppv:expr $(,)?) => {{
        let mut __pobj: *mut $T = core::ptr::null_mut();
        let mut __hr = <$T as $crate::com::ComObject>::create_instance(&mut __pobj);
        if __hr >= 0 {
            (*__pobj).AddRef();
            __hr = (*__pobj).Initialize($($arg),*);
            if __hr >= 0 {
                __hr = (*__pobj).QueryInterface(($riid) as *const _ as _, ($ppv) as *mut _ as _);
            }
            (*__pobj).Release();
        }
        __hr
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    // A fake object: an interface vptr at 0 (IUnknown slots) and a second interface at 16.
    #[repr(C)]
    struct Obj {
        vt0: *const [Option<Slot>; 3],
        root: CComObjectRootEx,
        _pad: u32,
        vt1: *const [Option<Slot>; 3],
    }

    unsafe extern "system" fn add_ref0(p: *mut c_void) -> u32 {
        internal_add_ref(core::ptr::addr_of_mut!((*(p as *mut Obj)).root))
    }
    unsafe extern "system" fn add_ref1(p: *mut c_void) -> u32 {
        let o = (p as *mut u8).sub(core::mem::offset_of!(Obj, vt1)) as *mut Obj;
        internal_add_ref(core::ptr::addr_of_mut!((*o).root))
    }
    type Slot = unsafe extern "system" fn(*mut c_void) -> u32;
    static VT0: [Option<Slot>; 3] = [None, Some(add_ref0), None];
    static VT1: [Option<Slot>; 3] = [None, Some(add_ref1), None];
    static IID_IFOO: [u8; 16] = [7; 16];
    static IID_IBAR: [u8; 16] = [9; 16];

    fn obj() -> Obj {
        Obj {
            vt0: &VT0,
            root: CComObjectRootEx::default(),
            _pad: 0,
            vt1: &VT1,
        }
    }

    #[test]
    fn query_interface_walks_the_map_and_add_refs() {
        let mut o = obj();
        let this = &mut o as *mut Obj as *mut c_void;
        let map = [ComMapEntry {
            iid: IID_IFOO.as_ptr() as *const c_void,
            offset: core::mem::offset_of!(Obj, vt1),
        }];
        let mut out: *mut c_void = core::ptr::null_mut();
        unsafe {
            let hr = internal_query_interface(this, &map, IID_IFOO.as_ptr() as _, &mut out);
            assert_eq!(hr, S_OK);
            assert_eq!(
                out as usize,
                this as usize + core::mem::offset_of!(Obj, vt1)
            );
            assert_eq!(o.root.m_dwRef, 1);
            // IUnknown answers with the FIRST entry's interface.
            let hr = internal_query_interface(this, &map, IID_IUNKNOWN.as_ptr() as _, &mut out);
            assert_eq!(hr, S_OK);
            assert_eq!(
                out as usize,
                this as usize + core::mem::offset_of!(Obj, vt1)
            );
            assert_eq!(o.root.m_dwRef, 2);
            let hr = internal_query_interface(this, &map, IID_IBAR.as_ptr() as _, &mut out);
            assert_eq!((hr, out), (E_NOINTERFACE, core::ptr::null_mut()));
            assert_eq!(internal_release(core::ptr::addr_of_mut!(o.root)), 1);
        }
    }

    #[test]
    fn an_empty_map_answers_iunknown_with_the_object_itself() {
        let mut o = obj();
        let this = &mut o as *mut Obj as *mut c_void;
        let mut out: *mut c_void = core::ptr::null_mut();
        unsafe {
            let hr = internal_query_interface(this, &[], IID_IUNKNOWN.as_ptr() as _, &mut out);
            assert_eq!((hr, out), (S_OK, this));
            let none = core::ptr::null_mut();
            let hr = internal_query_interface(this, &[], IID_IUNKNOWN.as_ptr() as _, none);
            assert_eq!(hr, E_POINTER);
        }
    }
}
