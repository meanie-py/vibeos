//! Ro* WinRT activation / error entry points (combase/roapi.c), hosted in ole32
//! because this tree does not build combase.dll. Semantics follow Wine's roapi.c,
//! including its stubs - where Wine says "stub" we return exactly what Wine returns.
//!
//! Deliberate gaps vs Wine (reported, not hidden):
//!  * the activation-context (manifest `winrt` section) lookup in
//!    get_library_for_classid is not done; only the HKLM registry path is.
//!  * ensure_mta() is approximated by one process-wide CoIncrementMTAUsage; Wine
//!    keeps a per-thread implicit-MTA cookie in its TLS block.
//!  * RoGetAgileReference does not pre-check InternalIsProcessInitialized (not
//!    exported here); CoMarshalInterface reports an uninitialised process itself.
//!  * RoRevokeActivationFactories / RoUnregisterForApartmentShutdown do not exist
//!    in Wine's roapi.c; they return S_OK, mirroring the Register* stubs.
use crate::compobj::ffi::CRITICAL_SECTION;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

type HRESULT = i32;
type BOOL = i32;
type HSTRING = *mut c_void;

const S_OK: HRESULT = 0;
const E_NOTIMPL: HRESULT = 0x80004001u32 as i32;
const E_NOINTERFACE: HRESULT = 0x80004002u32 as i32;
const E_FAIL: HRESULT = 0x80004005u32 as i32;
const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;
const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as i32;
const REGDB_E_READREGDB: HRESULT = 0x80040150u32 as i32;
const REGDB_E_CLASSNOTREG: HRESULT = 0x80040154u32 as i32;
const CO_E_NOT_SUPPORTED: HRESULT = 0x80004021u32 as i32;

#[derive(PartialEq, Clone, Copy)]
#[repr(C)]
pub struct Guid {
    pub d1: u32,
    d2: u16,
    d3: u16,
    d4: [u8; 8],
}
const IID_IUNKNOWN: Guid = Guid { d1: 0, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
const IID_IACTIVATIONFACTORY: Guid = Guid { d1: 0x35, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
const IID_IAGILEOBJECT: Guid =
    Guid { d1: 0x94ea2b94, d2: 0xe9cc, d3: 0x49e0, d4: [0xc0, 0xff, 0xee, 0x64, 0xca, 0x8f, 0x5b, 0x90] };
const IID_IAGILEREFERENCE: Guid =
    Guid { d1: 0xc03f6a43, d2: 0x65a4, d3: 0x9818, d4: [0x98, 0x7e, 0xe0, 0xb8, 0x10, 0xd2, 0xa6, 0xf2] };
const IID_INOMARSHAL: Guid =
    Guid { d1: 0xecc8691b, d2: 0xc1db, d3: 0x4dc0, d4: [0x85, 0x5e, 0x65, 0xf6, 0xc5, 0x51, 0xaf, 0x49] };

const fn wide<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}
// NUL-terminated
const ACTIVATABLE_ROOT: [u16; 53] = wide(b"Software\\Microsoft\\WindowsRuntime\\ActivatableClassId\0");
const DLLPATH: [u16; 8] = wide(b"DllPath\0");

const HKEY_LOCAL_MACHINE: *mut c_void = 0x80000002u32 as i32 as isize as *mut c_void;
const KEY_READ: u32 = 0x20019;
const REG_SZ: u32 = 1;
const REG_EXPAND_SZ: u32 = 2;
const HEAP_ZERO_MEMORY: u32 = 8;

mod sys {
    use core::ffi::c_void;
    extern "system" {
        pub fn GetProcessHeap() -> *mut c_void;
        pub fn HeapAlloc(heap: *mut c_void, flags: u32, bytes: usize) -> *mut c_void;
        pub fn HeapFree(heap: *mut c_void, flags: u32, mem: *mut c_void) -> i32;
        pub fn GetLastError() -> u32;
        pub fn LoadLibraryW(name: *const u16) -> *mut c_void;
        pub fn FreeLibrary(module: *mut c_void) -> i32;
        pub fn GetProcAddress(module: *mut c_void, name: *const core::ffi::c_char) -> *mut c_void;
        pub fn ExpandEnvironmentStringsW(src: *const u16, dst: *mut u16, n: u32) -> u32;
        pub fn RegOpenKeyExW(k: *mut c_void, sub: *const u16, opt: u32, sam: u32, out: *mut *mut c_void) -> i32;
        pub fn RegQueryValueExW(
            k: *mut c_void,
            name: *const u16,
            reserved: *mut u32,
            ty: *mut u32,
            data: *mut u8,
            size: *mut u32,
        ) -> i32;
        pub fn RegCloseKey(k: *mut c_void) -> i32;
        pub fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
        pub fn CoUninitialize();
        pub fn CoIncrementMTAUsage(cookie: *mut *mut c_void) -> i32;
        pub fn InitializeCriticalSection(cs: *mut crate::compobj::ffi::CRITICAL_SECTION);
        pub fn EnterCriticalSection(cs: *mut crate::compobj::ffi::CRITICAL_SECTION);
        pub fn LeaveCriticalSection(cs: *mut crate::compobj::ffi::CRITICAL_SECTION);
        pub fn DeleteCriticalSection(cs: *mut crate::compobj::ffi::CRITICAL_SECTION);
        pub fn WindowsGetStringRawBuffer(s: *mut c_void, len: *mut u32) -> *const u16;
    }
}

#[inline]
fn hresult_from_win32(e: u32) -> HRESULT {
    if (e as i32) <= 0 {
        e as i32
    } else {
        ((e & 0xFFFF) | 0x8007_0000) as i32
    }
}

// ---- minimal vtable views (IUnknown + the slots we call) ----
#[repr(C)]
struct IUnknownVtbl {
    qi: unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}
#[repr(C)]
struct IUnknownObj {
    vtbl: *const IUnknownVtbl,
}
#[repr(C)]
struct IActivationFactoryVtbl {
    base: IUnknownVtbl,
    get_iids: usize,
    get_runtime_class_name: usize,
    get_trust_level: usize,
    activate_instance: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}
#[repr(C)]
struct IStreamVtbl {
    base: IUnknownVtbl,
    read: usize,
    write: usize,
    seek: unsafe extern "system" fn(*mut c_void, i64, u32, *mut u64) -> HRESULT,
}

unsafe fn com_release(p: *mut c_void) -> u32 {
    ((*(*(p as *mut IUnknownObj)).vtbl).release)(p)
}
unsafe fn com_add_ref(p: *mut c_void) -> u32 {
    ((*(*(p as *mut IUnknownObj)).vtbl).add_ref)(p)
}
unsafe fn com_qi(p: *mut c_void, iid: *const Guid, out: *mut *mut c_void) -> HRESULT {
    ((*(*(p as *mut IUnknownObj)).vtbl).qi)(p, iid, out)
}
unsafe fn stream_rewind(stm: *mut c_void) -> HRESULT {
    let vt = (*(stm as *mut IUnknownObj)).vtbl as *const IStreamVtbl;
    ((*vt).seek)(stm, 0, 0, ptr::null_mut())
}

// ---- library lookup ----

unsafe fn heap_alloc(bytes: usize, flags: u32) -> *mut c_void {
    sys::HeapAlloc(sys::GetProcessHeap(), flags, bytes)
}
unsafe fn heap_free(p: *mut c_void) {
    if !p.is_null() {
        sys::HeapFree(sys::GetProcessHeap(), 0, p);
    }
}

/// HKLM\Software\Microsoft\WindowsRuntime\ActivatableClassId\<classid>\DllPath.
unsafe fn get_library_for_classid(classid: *const u16, out: *mut *mut u16) -> HRESULT {
    *out = ptr::null_mut();
    let mut root: *mut c_void = ptr::null_mut();
    if sys::RegOpenKeyExW(HKEY_LOCAL_MACHINE, ACTIVATABLE_ROOT.as_ptr(), 0, KEY_READ, &mut root) != 0 {
        // No runtime class is registered at all, so this one is not: Windows 11 answers
        // REGDB_E_CLASSNOTREG for an unknown class (winref, Win8Surface); Wine's
        // REGDB_E_READREGDB here was a guest-only difference.
        return REGDB_E_CLASSNOTREG;
    }
    let mut class: *mut c_void = ptr::null_mut();
    let rc = sys::RegOpenKeyExW(root, classid, 0, KEY_READ, &mut class);
    sys::RegCloseKey(root);
    if rc != 0 {
        return REGDB_E_CLASSNOTREG;
    }

    let mut ty = 0u32;
    let mut size = 0u32;
    let mut buf: *mut u16 = ptr::null_mut();
    let hr = 'done: {
        if sys::RegQueryValueExW(class, DLLPATH.as_ptr(), ptr::null_mut(), &mut ty, ptr::null_mut(), &mut size) != 0
            || (ty != REG_SZ && ty != REG_EXPAND_SZ)
        {
            break 'done REGDB_E_READREGDB;
        }
        // +2: guarantee NUL termination even for an unterminated REG_SZ.
        buf = heap_alloc(size as usize + 2, HEAP_ZERO_MEMORY) as *mut u16;
        if buf.is_null() {
            break 'done E_OUTOFMEMORY;
        }
        if sys::RegQueryValueExW(class, DLLPATH.as_ptr(), ptr::null_mut(), ptr::null_mut(), buf as *mut u8, &mut size)
            != 0
        {
            break 'done REGDB_E_READREGDB;
        }
        if ty == REG_EXPAND_SZ {
            let len = sys::ExpandEnvironmentStringsW(buf, ptr::null_mut(), 0);
            let expanded = heap_alloc(len as usize * 2 + 2, HEAP_ZERO_MEMORY) as *mut u16;
            if expanded.is_null() {
                break 'done E_OUTOFMEMORY;
            }
            sys::ExpandEnvironmentStringsW(buf, expanded, len);
            heap_free(buf as *mut c_void);
            buf = expanded;
        }
        *out = buf;
        buf = ptr::null_mut();
        S_OK
    };
    heap_free(buf as *mut c_void);
    sys::RegCloseKey(class);
    hr
}

static MTA_ENSURED: AtomicBool = AtomicBool::new(false);

unsafe fn ensure_mta() -> HRESULT {
    if MTA_ENSURED.load(Ordering::Acquire) {
        return S_OK;
    }
    let mut cookie: *mut c_void = ptr::null_mut();
    let hr = sys::CoIncrementMTAUsage(&mut cookie);
    if hr >= 0 {
        MTA_ENSURED.store(true, Ordering::Release);
        return S_OK;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RoInitialize(ty: i32) -> HRESULT {
    // RO_INIT_SINGLETHREADED=0 -> COINIT_APARTMENTTHREADED(2); everything else MTA(0).
    sys::CoInitializeEx(ptr::null_mut(), if ty == 0 { 2 } else { 0 })
}

#[no_mangle]
pub unsafe extern "system" fn RoUninitialize() {
    sys::CoUninitialize();
}

#[no_mangle]
pub unsafe extern "system" fn RoGetActivationFactory(
    classid: HSTRING,
    iid: *const Guid,
    class_factory: *mut *mut c_void,
) -> HRESULT {
    if iid.is_null() || class_factory.is_null() {
        return E_INVALIDARG;
    }
    // Windows clears the out-pointer on every failure (winref, Win8Surface).
    *class_factory = ptr::null_mut();
    let hr = ensure_mta();
    if hr < 0 {
        return hr;
    }
    let mut library: *mut u16 = ptr::null_mut();
    let hr = get_library_for_classid(sys::WindowsGetStringRawBuffer(classid, ptr::null_mut()), &mut library);
    if hr < 0 {
        return hr;
    }
    let mut module = sys::LoadLibraryW(library);
    let hr = 'done: {
        if module.is_null() {
            break 'done hresult_from_win32(sys::GetLastError());
        }
        let f = sys::GetProcAddress(module, b"DllGetActivationFactory\0".as_ptr() as *const core::ffi::c_char);
        if f.is_null() {
            break 'done E_FAIL;
        }
        let get: unsafe extern "system" fn(HSTRING, *mut *mut c_void) -> HRESULT = core::mem::transmute(f);
        let mut factory: *mut c_void = ptr::null_mut();
        let mut hr = get(classid, &mut factory);
        if hr >= 0 {
            hr = com_qi(factory, iid, class_factory);
            if hr >= 0 {
                module = ptr::null_mut(); // the factory now pins the DLL
            }
            com_release(factory);
        }
        hr
    };
    heap_free(library as *mut c_void);
    if !module.is_null() {
        sys::FreeLibrary(module);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RoActivateInstance(classid: HSTRING, instance: *mut *mut c_void) -> HRESULT {
    let mut factory: *mut c_void = ptr::null_mut();
    let hr = RoGetActivationFactory(classid, &IID_IACTIVATIONFACTORY, &mut factory);
    if hr < 0 {
        return hr;
    }
    let vt = (*(factory as *mut IUnknownObj)).vtbl as *const IActivationFactoryVtbl;
    let hr = ((*vt).activate_instance)(factory, instance);
    com_release(factory);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RoRegisterActivationFactories(
    _classes: *mut HSTRING,
    _callbacks: *mut c_void,
    _count: u32,
    _cookie: *mut c_void,
) -> HRESULT {
    S_OK // Wine: FIXME stub, S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RoRevokeActivationFactories(_cookie: *mut c_void) -> HRESULT {
    S_OK
}

extern "system" {
    fn GetCurrentThreadId() -> u32;
}

#[no_mangle]
pub unsafe extern "system" fn RoGetApartmentIdentifier(identifier: *mut u64) -> HRESULT {
    if identifier.is_null() {
        return E_INVALIDARG;
    }
    // Opaque, but it must name THIS apartment: callers compare it to match shutdown
    // notifications, so Wine's constant 0xdeadbeef would make every apartment the same.
    // The MTA is one per process; an STA is its thread.
    let mut kind: i32 = 0;
    let mut qualifier: i32 = 0;
    let hr = crate::compobj::compobj::CoGetApartmentType(&mut kind as *mut _ as _, &mut qualifier as *mut _ as _);
    if hr < 0 {
        return hr; // CO_E_NOTINITIALIZED outside an apartment, as on Windows
    }
    const APTTYPE_MTA: i32 = 1;
    *identifier = if kind == APTTYPE_MTA {
        1
    } else {
        (1u64 << 32) | GetCurrentThreadId() as u64
    };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RoRegisterForApartmentShutdown(
    _callback: *mut c_void,
    identifier: *mut u64,
    cookie: *mut *mut c_void,
) -> HRESULT {
    let hr = RoGetApartmentIdentifier(identifier);
    if hr < 0 {
        return hr;
    }
    if !cookie.is_null() {
        *cookie = 0xcafecafeu32 as usize as *mut c_void;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RoUnregisterForApartmentShutdown(_cookie: *mut c_void) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn GetRestrictedErrorInfo(_info: *mut *mut c_void) -> HRESULT {
    E_NOTIMPL // Wine stub
}

#[no_mangle]
pub unsafe extern "system" fn SetRestrictedErrorInfo(_info: *mut c_void) -> HRESULT {
    E_NOTIMPL // Wine stub
}

#[no_mangle]
pub unsafe extern "system" fn RoOriginateLanguageException(_error: HRESULT, _message: HSTRING, _exc: *mut c_void) -> BOOL {
    0 // Wine stub returns FALSE
}

#[no_mangle]
pub unsafe extern "system" fn RoOriginateError(_error: HRESULT, _message: HSTRING) -> BOOL {
    0 // Wine stub returns FALSE
}

#[no_mangle]
pub unsafe extern "system" fn RoOriginateErrorW(_error: HRESULT, _max_len: u32, _message: *const u16) -> BOOL {
    0 // Wine stub returns FALSE
}

// ---- RoGetAgileReference ----

#[repr(C)]
struct AgileVtbl {
    base: IUnknownVtbl,
    resolve: unsafe extern "system" fn(*mut AgileRef, *const Guid, *mut *mut c_void) -> HRESULT,
}

#[repr(C)]
struct AgileRef {
    vtbl: *const AgileVtbl,
    option: i32,
    marshal_stream: *mut c_void,
    cs: CRITICAL_SECTION,
    obj: *mut c_void,
    refs: i32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<CRITICAL_SECTION>() == 40);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<CRITICAL_SECTION>() == 24);

const AGILEREFERENCE_DEFAULT: i32 = 0;
const AGILEREFERENCE_DELAYEDMARSHAL: i32 = 1;
const MSHCTX_INPROC: u32 = 3;
const MSHLFLAGS_TABLESTRONG: u32 = 1;

unsafe fn marshal_object_in_agile_reference(r: *mut AgileRef, riid: *const Guid, obj: *mut c_void) -> HRESULT {
    let hr = crate::ring::hglobalstream::CreateStreamOnHGlobal(ptr::null_mut(), 1, ptr::addr_of_mut!((*r).marshal_stream) as *mut _);
    if hr < 0 {
        return hr;
    }
    let hr = crate::ole32_exports::CoMarshalInterface(
        (*r).marshal_stream as *mut _,
        riid as *const _,
        obj as *mut _,
        MSHCTX_INPROC,
        ptr::null_mut(),
        MSHLFLAGS_TABLESTRONG,
    );
    if hr < 0 {
        com_release((*r).marshal_stream);
        (*r).marshal_stream = ptr::null_mut();
    }
    hr
}

unsafe extern "system" fn agile_qi(this: *mut c_void, riid: *const Guid, obj: *mut *mut c_void) -> HRESULT {
    if riid.is_null() || obj.is_null() {
        return E_INVALIDARG;
    }
    let g = *riid;
    if g == IID_IUNKNOWN || g == IID_IAGILEOBJECT || g == IID_IAGILEREFERENCE {
        agile_add_ref(this);
        *obj = this;
        return S_OK;
    }
    *obj = ptr::null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn agile_add_ref(this: *mut c_void) -> u32 {
    let r = this as *mut AgileRef;
    (AtomicI32::from_ptr(ptr::addr_of_mut!((*r).refs)).fetch_add(1, Ordering::AcqRel) + 1) as u32
}

unsafe extern "system" fn agile_release(this: *mut c_void) -> u32 {
    let r = this as *mut AgileRef;
    let n = AtomicI32::from_ptr(ptr::addr_of_mut!((*r).refs)).fetch_sub(1, Ordering::AcqRel) - 1;
    if n == 0 {
        if !(*r).obj.is_null() {
            com_release((*r).obj);
        }
        if !(*r).marshal_stream.is_null() {
            stream_rewind((*r).marshal_stream);
            crate::ole32_exports::CoReleaseMarshalData((*r).marshal_stream as *mut _);
            com_release((*r).marshal_stream);
        }
        sys::DeleteCriticalSection(ptr::addr_of_mut!((*r).cs));
        heap_free(this);
    }
    n as u32
}

unsafe extern "system" fn agile_resolve(r: *mut AgileRef, riid: *const Guid, obj: *mut *mut c_void) -> HRESULT {
    sys::EnterCriticalSection(ptr::addr_of_mut!((*r).cs));
    let hr = 'out: {
        if (*r).option == AGILEREFERENCE_DELAYEDMARSHAL && (*r).marshal_stream.is_null() {
            let hr = marshal_object_in_agile_reference(r, riid, (*r).obj);
            if hr < 0 {
                break 'out hr;
            }
            com_release((*r).obj);
            (*r).obj = ptr::null_mut();
        }
        let hr = stream_rewind((*r).marshal_stream);
        if hr < 0 {
            break 'out hr;
        }
        crate::ole32_exports::CoUnmarshalInterface((*r).marshal_stream as *mut _, riid as *const _, obj as *mut _)
    };
    sys::LeaveCriticalSection(ptr::addr_of_mut!((*r).cs));
    hr
}

static AGILE_VTBL: AgileVtbl = AgileVtbl {
    base: IUnknownVtbl { qi: agile_qi, add_ref: agile_add_ref, release: agile_release },
    resolve: agile_resolve,
};

#[no_mangle]
pub unsafe extern "system" fn RoGetAgileReference(
    option: i32,
    riid: *const Guid,
    obj: *mut c_void,
    agile_reference: *mut *mut c_void,
) -> HRESULT {
    if option != AGILEREFERENCE_DEFAULT && option != AGILEREFERENCE_DELAYEDMARSHAL {
        return E_INVALIDARG;
    }
    if riid.is_null() || obj.is_null() || agile_reference.is_null() {
        return E_INVALIDARG;
    }
    let mut unk: *mut c_void = ptr::null_mut();
    if com_qi(obj, riid, &mut unk) < 0 {
        return E_NOINTERFACE;
    }
    com_release(unk);
    if com_qi(obj, &IID_INOMARSHAL, &mut unk) >= 0 {
        com_release(unk);
        return CO_E_NOT_SUPPORTED;
    }

    let r = heap_alloc(size_of::<AgileRef>(), HEAP_ZERO_MEMORY) as *mut AgileRef;
    if r.is_null() {
        return E_OUTOFMEMORY;
    }
    (*r).vtbl = &AGILE_VTBL;
    (*r).option = option;
    (*r).refs = 1;
    if option == AGILEREFERENCE_DEFAULT {
        let hr = marshal_object_in_agile_reference(r, riid, obj);
        if hr < 0 {
            heap_free(r as *mut c_void);
            return hr;
        }
    } else {
        (*r).obj = obj;
        com_add_ref(obj);
    }
    // CS initialised at its final (heap) address.
    sys::InitializeCriticalSection(ptr::addr_of_mut!((*r).cs));
    *agile_reference = r as *mut c_void;
    S_OK
}
