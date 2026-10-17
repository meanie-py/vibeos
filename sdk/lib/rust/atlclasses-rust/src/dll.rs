//! ATL's in-process server (atlbase.h `CComModule`, atlcom.h's object map and
//! `CComClassFactory`): what a shell extension's `DllGetClassObject`, `DllCanUnloadNow` and
//! `DllRegisterServer` delegate to.
//!
//! - **The object map** (`BEGIN_OBJECT_MAP` / `OBJECT_ENTRY(clsid, class)`) is a static
//!   array of [`ObjMapEntry`] ending in [`ObjMapEntry::END`], which `cpp_app` renders from the
//!   C++ map. Each entry carries the class's `UpdateRegistry` and its `_CreatorClass`
//!   ([`creator`]), plus the cached class factory (`pCF`).
//! - **The class factory** is `CComObjectCached<CComClassFactory>`, what `DECLARE_CLASSFACTORY`
//!   (CComCoClass's default) names for a DLL: the map's own reference does not hold the
//!   module, every other one does (`AddRef` to 2 locks, `Release` to 1 unlocks), so
//!   `DllCanUnloadNow` answers S_OK exactly when no client holds a factory or an object.
//!   This EXCEEDS the C as ReactOS builds it: atlcom.h picks CComObjectCached only
//!   `#if defined(_WINDLL) | defined(_USRDLL)`, which no ReactOS DLL defines (they get `_DLL`),
//!   so every C++ shell extension takes the EXE arm -- CComObjectNoLock, "class factory should
//!   not change lock count" -- and says it can unload while a client still holds its factory.
//!   mmcshext's modrun suite declares the difference.
//! - **Registration** runs each entry's `UpdateRegistry`; `DECLARE_REGISTRY_RESOURCEID(id)`'s
//!   is [`update_registry_from_resource`] over the `.rgs` script (`crate::registrar`).
//!
//! Not provided, and refused by `cpp_app` rather than approximated: aggregation (a class
//! without `DECLARE_NOT_AGGREGATABLE`), `OBJECT_ENTRY_AUTO`, a class's own `ObjectMain`,
//! other class factories (`DECLARE_CLASSFACTORY_*`), and type library registration (every
//! ReactOS shell extension passes `bRegTypeLib = FALSE`; TRUE answers E_NOTIMPL).

use core::ffi::c_void;
use core::sync::atomic::{AtomicPtr, Ordering};

use crate::com::{
    failed, internal_add_ref, internal_query_interface, internal_release, module_lock,
    module_lock_count, module_unlock, CComObjectRootEx, ComMapEntry, ComObject, E_OUTOFMEMORY,
    E_POINTER, HRESULT, S_OK,
};
use crate::registrar::{hresult_from_win32, RegObject};

pub const S_FALSE: HRESULT = 1;
pub const E_NOTIMPL: HRESULT = 0x8000_4001_u32 as i32;
pub const CLASS_E_NOAGGREGATION: HRESULT = 0x8004_0110_u32 as i32;
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = 0x8004_0111_u32 as i32;

const IID_IUNKNOWN: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0xC0, 0, 0, 0, 0, 0, 0, 0x46];
const IID_ICLASSFACTORY: [u8; 16] = [1, 0, 0, 0, 0, 0, 0, 0, 0xC0, 0, 0, 0, 0, 0, 0, 0x46];

unsafe fn guid_eq(a: *const c_void, b: *const c_void) -> bool {
    core::slice::from_raw_parts(a as *const u8, 16) == core::slice::from_raw_parts(b as *const u8, 16)
}

/// `_ATL_CREATORFUNC`: `(pv, riid, ppv)`, where `pv` is the outer unknown (or, for a class
/// factory's creator, the object creator it is given).
pub type CreatorFn =
    unsafe extern "system" fn(pv: *mut c_void, riid: *const c_void, ppv: *mut *mut c_void) -> HRESULT;
/// `HRESULT (WINAPI *pfnUpdateRegistry)(BOOL bRegister)`.
pub type UpdateRegistryFn = unsafe extern "system" fn(register: i32) -> HRESULT;

/// One `OBJECT_ENTRY(clsid, class)` (`_ATL_OBJMAP_ENTRY`): the members `CComModule` reads.
#[repr(C)]
pub struct ObjMapEntry {
    pub pclsid: *const c_void,
    pub pfnUpdateRegistry: Option<UpdateRegistryFn>,
    /// `class::_CreatorClass::CreateInstance`.
    pub pfnCreateInstance: Option<CreatorFn>,
    /// The cached class factory, created on the first `GetClassObject` for the CLSID.
    pub pCF: AtomicPtr<c_void>,
}

// The map is a static that every thread reads; pCF is the only mutable member, and atomic.
unsafe impl Sync for ObjMapEntry {}

impl ObjMapEntry {
    /// `END_OBJECT_MAP()`: the all-NULL terminator.
    pub const END: ObjMapEntry = ObjMapEntry {
        pclsid: core::ptr::null(),
        pfnUpdateRegistry: None,
        pfnCreateInstance: None,
        pCF: AtomicPtr::new(core::ptr::null_mut()),
    };

    pub const fn new(clsid: *const c_void, update: UpdateRegistryFn, create: CreatorFn) -> Self {
        ObjMapEntry {
            pclsid: clsid,
            pfnUpdateRegistry: Some(update),
            pfnCreateInstance: Some(create),
            pCF: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

/// A class in an object map: [`ComObject`] plus what `CComCreator` does after creating it.
pub trait CoClass: ComObject {
    /// The object's `QueryInterface` (its COM map).
    unsafe fn query_interface(p: *mut Self, riid: *const c_void, ppv: *mut *mut c_void) -> HRESULT;
    /// `delete` of the `CComObject<T>`: FinalRelease, the module Unlock, the destructor.
    unsafe fn destroy(p: *mut Self);
}

/// `DECLARE_NOT_AGGREGATABLE(T)`'s `_CreatorClass::CreateInstance`:
/// `CComCreator2<CComCreator<CComObject<T>>, CComFailCreator<CLASS_E_NOAGGREGATION>>`.
pub unsafe extern "system" fn creator<T: CoClass>(
    pv: *mut c_void,
    riid: *const c_void,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if !pv.is_null() {
        return CLASS_E_NOAGGREGATION; // CComFailCreator
    }
    // CComCreator<CComObject<T>>: construct, FinalConstruct (a failure is already destroyed),
    // then the interface asked for; an object nobody holds is deleted.
    let mut p: *mut T = core::ptr::null_mut();
    let hr = T::create_instance(&mut p);
    if failed(hr) {
        return hr;
    }
    let hr = T::query_interface(p, riid, ppv);
    if failed(hr) {
        T::destroy(p);
    }
    hr
}

// ---- CComObjectCached<CComClassFactory> --------------------------------------------------

#[repr(C)]
struct ClassFactoryVtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut ClassFactory, *const c_void, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut ClassFactory) -> u32,
    Release: unsafe extern "system" fn(*mut ClassFactory) -> u32,
    CreateInstance: unsafe extern "system" fn(
        *mut ClassFactory,
        *mut c_void,
        *const c_void,
        *mut *mut c_void,
    ) -> HRESULT,
    LockServer: unsafe extern "system" fn(*mut ClassFactory, i32) -> HRESULT,
}

/// `CComClassFactory`: `IClassFactory` at offset 0, the reference count, `m_pfnCreateInstance`.
#[repr(C)]
struct ClassFactory {
    vtbl: *const ClassFactoryVtbl,
    root: CComObjectRootEx,
    m_pfnCreateInstance: Option<CreatorFn>,
}

static CLASS_FACTORY_VTBL: ClassFactoryVtbl = ClassFactoryVtbl {
    QueryInterface: cf_query_interface,
    AddRef: cf_add_ref,
    Release: cf_release,
    CreateInstance: cf_create_instance,
    LockServer: cf_lock_server,
};

extern "C" {
    fn calloc(n: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

unsafe extern "system" fn cf_query_interface(
    this: *mut ClassFactory,
    riid: *const c_void,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // BEGIN_COM_MAP(CComClassFactory) COM_INTERFACE_ENTRY_IID(IID_IClassFactory, ..)
    let map = [ComMapEntry { iid: IID_ICLASSFACTORY.as_ptr() as *const c_void, offset: 0 }];
    internal_query_interface(this as *mut c_void, &map, riid, ppv)
}

/// `CComObjectCached::AddRef`: the second reference (the first a client takes) locks.
unsafe extern "system" fn cf_add_ref(this: *mut ClassFactory) -> u32 {
    let n = internal_add_ref(core::ptr::addr_of_mut!((*this).root));
    if n == 2 {
        module_lock();
    }
    n
}

/// `CComObjectCached::Release`: back to the map's one reference unlocks; 0 deletes.
unsafe extern "system" fn cf_release(this: *mut ClassFactory) -> u32 {
    let n = internal_release(core::ptr::addr_of_mut!((*this).root));
    if n == 0 {
        free(this as *mut c_void);
    } else if n == 1 {
        module_unlock();
    }
    n
}

unsafe extern "system" fn cf_create_instance(
    this: *mut ClassFactory,
    outer: *mut c_void,
    riid: *const c_void,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() && !guid_eq(riid, IID_IUNKNOWN.as_ptr() as *const c_void) {
        return CLASS_E_NOAGGREGATION;
    }
    match (*this).m_pfnCreateInstance {
        Some(create) => create(outer, riid, ppv),
        None => E_POINTER, // ATLASSERT(m_pfnCreateInstance != NULL)
    }
}

unsafe extern "system" fn cf_lock_server(_this: *mut ClassFactory, lock: i32) -> HRESULT {
    if lock != 0 {
        module_lock();
    } else {
        module_unlock();
    }
    S_OK
}

/// `CComCreator<CComObjectCached<CComClassFactory>>::CreateInstance(pfnCreateInstance,
/// IID_IUnknown, &pCF)`: the factory with its one (map) reference, which does not lock.
unsafe fn new_class_factory(create: CreatorFn, out: *mut *mut c_void) -> HRESULT {
    let cf = calloc(1, core::mem::size_of::<ClassFactory>()) as *mut ClassFactory;
    if cf.is_null() {
        return E_OUTOFMEMORY;
    }
    (*cf).vtbl = &CLASS_FACTORY_VTBL;
    (*cf).m_pfnCreateInstance = Some(create); // SetVoid(pv)
    let hr = cf_query_interface(cf, IID_IUNKNOWN.as_ptr() as *const c_void, out);
    if failed(hr) {
        free(cf as *mut c_void);
    }
    hr
}

unsafe fn com_release(p: *mut c_void) -> u32 {
    type Release = unsafe extern "system" fn(*mut c_void) -> u32;
    let vtbl = *(p as *const *const Release);
    (*vtbl.add(2))(p)
}

unsafe fn com_query_interface(p: *mut c_void, riid: *const c_void, ppv: *mut *mut c_void) -> HRESULT {
    type Qi = unsafe extern "system" fn(*mut c_void, *const c_void, *mut *mut c_void) -> HRESULT;
    let vtbl = *(p as *const *const Qi);
    (*vtbl)(p, riid, ppv)
}

// ---- CComModule ---------------------------------------------------------------------------

/// atlbase.h `_ATL_REGMAP_ENTRY`: one `%szKey%` -> `szData` replacement for a script.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct _ATL_REGMAP_ENTRY {
    pub szKey: *const u16,
    pub szData: *const u16,
}

/// `CComModule`: the object map it was `Init`ed with. Its lock count is the module's
/// (`crate::com::module_lock`), the one every `CComObject` holds.
#[repr(C)]
pub struct CComModule {
    pub m_pObjMap: *const ObjMapEntry,
}

impl Default for CComModule {
    fn default() -> Self {
        Self::new()
    }
}

impl CComModule {
    /// `CComModule g_Module;` -- a `const fn`, so the global needs no constructor call.
    pub const fn new() -> Self {
        CComModule { m_pObjMap: core::ptr::null() }
    }

    fn entries(&self) -> &'static [ObjMapEntry] {
        if self.m_pObjMap.is_null() {
            return &[];
        }
        let mut n = 0;
        unsafe {
            while !(*self.m_pObjMap.add(n)).pclsid.is_null() {
                n += 1;
            }
            core::slice::from_raw_parts(self.m_pObjMap, n)
        }
    }

    /// `Init(ObjectMap, hInstance, plibid)`. The instance is not stored: ATL takes the
    /// module's own from `_AtlBaseModule` (`crate::module_instance`). Each entry's
    /// `ObjectMain(true)` is CComObjectRootBase's empty one (cpp_app refuses a class's own).
    pub unsafe fn Init<H, G>(&mut self, p: *const ObjMapEntry, _h: *mut H, _plibid: *const G) -> HRESULT {
        if p as usize != usize::MAX {
            self.m_pObjMap = p;
        }
        S_OK
    }

    /// `Term()`: release every cached class factory.
    pub unsafe fn Term(&mut self) {
        for e in self.entries() {
            let cf = e.pCF.swap(core::ptr::null_mut(), Ordering::AcqRel);
            if !cf.is_null() {
                com_release(cf);
            }
        }
    }

    /// `GetClassObject(rclsid, riid, ppv)`: the CLSID's class factory, created once.
    pub unsafe fn GetClassObject<C, I, P>(
        &mut self,
        rclsid: *const C,
        riid: *const I,
        ppv: *mut *mut P,
    ) -> HRESULT {
        if ppv.is_null() {
            return E_POINTER;
        }
        let ppv = ppv as *mut *mut c_void;
        *ppv = core::ptr::null_mut();
        let mut hr = S_OK;
        for e in self.entries() {
            let Some(create) = e.pfnCreateInstance else { continue };
            if !guid_eq(rclsid as *const c_void, e.pclsid) {
                continue;
            }
            let mut cf = e.pCF.load(Ordering::Acquire);
            if cf.is_null() {
                // ATL creates it under `m_csObjMap`; a loser of the race releases its own.
                let mut new: *mut c_void = core::ptr::null_mut();
                hr = new_class_factory(create, &mut new);
                if !new.is_null() {
                    match e.pCF.compare_exchange(
                        core::ptr::null_mut(),
                        new,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    ) {
                        Ok(_) => cf = new,
                        Err(winner) => {
                            com_release(new);
                            cf = winner;
                        }
                    }
                }
            }
            if !cf.is_null() {
                hr = com_query_interface(cf, riid as *const c_void, ppv);
            }
            break;
        }
        if hr == S_OK && (*ppv).is_null() {
            // AtlComModuleGetClassObject over the OBJECT_ENTRY_AUTO map, which is empty.
            hr = CLASS_E_CLASSNOTAVAILABLE;
        }
        hr
    }

    pub unsafe fn DllGetClassObject<C, I, P>(
        &mut self,
        rclsid: *const C,
        riid: *const I,
        ppv: *mut *mut P,
    ) -> HRESULT {
        self.GetClassObject(rclsid, riid, ppv)
    }

    /// `DllCanUnloadNow()`: S_OK when nothing holds the module.
    pub fn DllCanUnloadNow(&self) -> HRESULT {
        if self.GetLockCount() == 0 {
            S_OK
        } else {
            S_FALSE
        }
    }

    pub fn GetLockCount(&self) -> i32 {
        module_lock_count()
    }

    pub fn Lock(&self) -> i32 {
        module_lock()
    }

    pub fn Unlock(&self) -> i32 {
        module_unlock()
    }

    /// `_AtlBaseModule.GetModuleInstance()`.
    pub fn GetModuleInstance(&self) -> *mut c_void {
        crate::module_instance()
    }

    /// `RegisterServer(bRegTypeLib, pCLSID)`: each entry's (or the one CLSID's)
    /// `UpdateRegistry(TRUE)`, stopping at the first failure.
    pub unsafe fn RegisterServer<C>(&mut self, bRegTypeLib: i32, pCLSID: *const C) -> HRESULT {
        self.update_registry(true, bRegTypeLib, pCLSID as *const c_void)
    }

    pub unsafe fn UnregisterServer<C>(&mut self, bUnRegTypeLib: i32, pCLSID: *const C) -> HRESULT {
        self.update_registry(false, bUnRegTypeLib, pCLSID as *const c_void)
    }

    pub unsafe fn DllRegisterServer(&mut self, bRegTypeLib: i32) -> HRESULT {
        self.RegisterServer(bRegTypeLib, core::ptr::null::<c_void>())
    }

    pub unsafe fn DllUnregisterServer(&mut self, bUnRegTypeLib: i32) -> HRESULT {
        self.UnregisterServer(bUnRegTypeLib, core::ptr::null::<c_void>())
    }

    unsafe fn update_registry(&self, register: bool, typelib: i32, clsid: *const c_void) -> HRESULT {
        let mut hr = S_OK;
        for e in self.entries() {
            if clsid.is_null() || guid_eq(clsid, e.pclsid) {
                if let Some(update) = e.pfnUpdateRegistry {
                    hr = update(register as i32);
                }
                if failed(hr) {
                    break;
                }
            }
        }
        if !failed(hr) && typelib != 0 {
            // CAtlModuleT::RegisterServer -> AtlComModuleRegisterServer's AtlRegisterTypeLib.
            hr = E_NOTIMPL;
        }
        hr
    }

    /// `UpdateRegistryFromResource(nResID, bRegister, pMapEntries)`.
    pub unsafe fn UpdateRegistryFromResource<M>(
        &self,
        nResID: u32,
        bRegister: i32,
        pMapEntries: *const M,
    ) -> HRESULT {
        update_registry_from_resource(nResID, bRegister, pMapEntries as *const _ATL_REGMAP_ENTRY)
    }
}

extern "system" {
    fn GetModuleFileNameW(h: *mut c_void, name: *mut u16, cch: u32) -> u32;
    fn GetLastError() -> u32;
}

/// `_pAtlModule->UpdateRegistryFromResource(nResID, bRegister, pMapEntries)`: the module's
/// `REGISTRY` resource `nResID`, with `%MODULE%`/`%MODULE_RAW%` (this DLL's path), `%APPID%`
/// (empty: `CAtlModuleT::GetAppId`) and the caller's replacements. What
/// `DECLARE_REGISTRY_RESOURCEID(nResID)`'s `UpdateRegistry` does.
pub unsafe fn update_registry_from_resource(
    nResID: u32,
    bRegister: i32,
    pMapEntries: *const _ATL_REGMAP_ENTRY,
) -> HRESULT {
    const MAX_PATH: u32 = 260;
    const ERROR_INSUFFICIENT_BUFFER: i32 = 122;
    let mut path = [0u16; MAX_PATH as usize];
    let n = GetModuleFileNameW(crate::module_instance(), path.as_mut_ptr(), MAX_PATH);
    if n == MAX_PATH {
        return hresult_from_win32(ERROR_INSUFFICIENT_BUFFER);
    } else if n == 0 {
        return hresult_from_win32(GetLastError() as i32);
    }
    let Some(mut reg) = RegObject::new() else {
        return E_OUTOFMEMORY;
    };
    let mut e = pMapEntries;
    while !e.is_null() && !(*e).szKey.is_null() {
        let hr = reg.AddReplacement((*e).szKey, (*e).szData);
        if failed(hr) {
            return hr;
        }
        e = e.add(1);
    }
    let w = |s: &str, out: &mut [u16; 16]| {
        for (i, c) in s.encode_utf16().enumerate() {
            out[i] = c;
        }
    };
    let (mut appid, mut empty, mut module, mut module_raw) = ([0u16; 16], [0u16; 16], [0u16; 16], [0u16; 16]);
    w("APPID", &mut appid);
    w("", &mut empty);
    w("Module", &mut module);
    w("Module_Raw", &mut module_raw);
    for (k, v) in [
        (appid.as_ptr(), empty.as_ptr()),
        (module.as_ptr(), path.as_ptr()),
        (module_raw.as_ptr(), path.as_ptr()),
    ] {
        let hr = reg.AddReplacement(k, v);
        if failed(hr) {
            return hr;
        }
    }
    let mut ty = [0u16; 16];
    w("REGISTRY", &mut ty);
    reg.resource_register(path.as_ptr(), nResID, ty.as_ptr(), bRegister != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::ComMapEntry;

    // A one-interface CComObject<T>: IUnknown-shaped vtable at 0, the count, a canary.
    #[repr(C)]
    struct Obj {
        vt: *const [unsafe extern "system" fn(*mut c_void) -> u32; 3],
        root: CComObjectRootEx,
    }
    static IID_IFOO: [u8; 16] = [7; 16];
    static CLSID_FOO: [u8; 16] = [3; 16];
    static CLSID_BAR: [u8; 16] = [4; 16];

    unsafe extern "system" fn add_ref(p: *mut c_void) -> u32 {
        internal_add_ref(core::ptr::addr_of_mut!((*(p as *mut Obj)).root))
    }
    unsafe extern "system" fn release(p: *mut c_void) -> u32 {
        let n = internal_release(core::ptr::addr_of_mut!((*(p as *mut Obj)).root));
        if n == 0 {
            <Obj as CoClass>::destroy(p as *mut Obj);
        }
        n
    }
    unsafe extern "system" fn unused(_p: *mut c_void) -> u32 {
        0
    }
    static VT: [unsafe extern "system" fn(*mut c_void) -> u32; 3] = [unused, add_ref, release];

    impl ComObject for Obj {
        unsafe fn create_instance(pp: *mut *mut Self) -> i32 {
            let p = calloc(1, core::mem::size_of::<Obj>()) as *mut Obj;
            (*p).vt = &VT;
            module_lock();
            *pp = p;
            0
        }
    }
    impl CoClass for Obj {
        unsafe fn query_interface(p: *mut Self, riid: *const c_void, ppv: *mut *mut c_void) -> HRESULT {
            let map = [ComMapEntry { iid: IID_IFOO.as_ptr() as *const c_void, offset: 0 }];
            internal_query_interface(p as *mut c_void, &map, riid, ppv)
        }
        unsafe fn destroy(p: *mut Self) {
            module_unlock();
            free(p as *mut c_void);
        }
    }
    unsafe extern "system" fn no_registry(_b: i32) -> HRESULT {
        S_OK
    }

    #[test]
    fn class_object_lifecycle_tracks_the_module_lock() {
        let map = [
            ObjMapEntry::new(CLSID_FOO.as_ptr() as *const c_void, no_registry, creator::<Obj>),
            ObjMapEntry::END,
        ];
        let mut m = CComModule::new();
        unsafe {
            m.Init(map.as_ptr(), core::ptr::null_mut::<c_void>(), core::ptr::null::<c_void>());
            assert_eq!(m.DllCanUnloadNow(), S_OK);
            let mut cf: *mut c_void = core::ptr::null_mut();
            let hr = m.DllGetClassObject(CLSID_FOO.as_ptr(), IID_ICLASSFACTORY.as_ptr(), &mut cf);
            assert_eq!(hr, S_OK);
            assert!(!cf.is_null());
            assert_eq!(m.DllCanUnloadNow(), S_FALSE); // a client holds the factory

            // IClassFactory::CreateInstance(NULL, IID_IFoo): an object, holding the module.
            let vt = *(cf as *const *const ClassFactoryVtbl);
            let mut obj: *mut c_void = core::ptr::null_mut();
            let hr = ((*vt).CreateInstance)(cf as _, core::ptr::null_mut(), IID_IFOO.as_ptr() as _, &mut obj);
            assert_eq!(hr, S_OK);
            // Aggregation is refused before the creator runs.
            let mut agg: *mut c_void = core::ptr::null_mut();
            let hr = ((*vt).CreateInstance)(cf as _, cf, IID_IFOO.as_ptr() as _, &mut agg);
            assert_eq!((hr, agg), (CLASS_E_NOAGGREGATION, core::ptr::null_mut()));
            let hr = ((*vt).CreateInstance)(cf as _, cf, IID_IUNKNOWN.as_ptr() as _, &mut agg);
            assert_eq!((hr, agg), (CLASS_E_NOAGGREGATION, core::ptr::null_mut()));

            // An interface the object lacks: E_NOINTERFACE, and the object is gone again.
            let before = module_lock_count();
            let mut none: *mut c_void = core::ptr::null_mut();
            let hr = ((*vt).CreateInstance)(cf as _, core::ptr::null_mut(), CLSID_BAR.as_ptr() as _, &mut none);
            assert!(failed(hr));
            assert_eq!(module_lock_count(), before);

            ((*vt).Release)(cf as _); // back to the map's own reference: unlocks
            assert_eq!(m.DllCanUnloadNow(), S_FALSE); // the object still holds it
            release(obj);
            assert_eq!(m.DllCanUnloadNow(), S_OK);

            // A second request reuses the cached factory.
            let cached = map[0].pCF.load(Ordering::Acquire);
            let mut cf2: *mut c_void = core::ptr::null_mut();
            assert_eq!(m.DllGetClassObject(CLSID_FOO.as_ptr(), IID_IUNKNOWN.as_ptr(), &mut cf2), S_OK);
            assert_eq!(cf2, cached);
            com_release(cf2);

            // An unknown CLSID.
            let mut x: *mut c_void = core::ptr::null_mut();
            let hr = m.DllGetClassObject(CLSID_BAR.as_ptr(), IID_ICLASSFACTORY.as_ptr(), &mut x);
            assert_eq!((hr, x), (CLASS_E_CLASSNOTAVAILABLE, core::ptr::null_mut()));

            m.Term();
            assert!(map[0].pCF.load(Ordering::Acquire).is_null());
            assert_eq!(m.DllCanUnloadNow(), S_OK);
        }
    }

    #[test]
    fn register_server_runs_every_entry_and_refuses_a_typelib() {
        use core::sync::atomic::AtomicI32;
        static CALLS: AtomicI32 = AtomicI32::new(0);
        unsafe extern "system" fn count(b: i32) -> HRESULT {
            CALLS.fetch_add(if b != 0 { 1 } else { 100 }, Ordering::SeqCst);
            S_OK
        }
        let map = [
            ObjMapEntry::new(CLSID_FOO.as_ptr() as *const c_void, count, creator::<Obj>),
            ObjMapEntry::new(CLSID_BAR.as_ptr() as *const c_void, count, creator::<Obj>),
            ObjMapEntry::END,
        ];
        let mut m = CComModule::new();
        unsafe {
            m.Init(map.as_ptr(), core::ptr::null_mut::<c_void>(), core::ptr::null::<c_void>());
            assert_eq!(m.DllRegisterServer(0), S_OK);
            assert_eq!(m.DllUnregisterServer(0), S_OK);
            assert_eq!(CALLS.load(Ordering::SeqCst), 202);
            assert_eq!(m.RegisterServer(0, CLSID_BAR.as_ptr()), S_OK);
            assert_eq!(CALLS.load(Ordering::SeqCst), 203);
            assert_eq!(m.DllRegisterServer(1), E_NOTIMPL);
        }
    }
}
