//! comcat.c — the component-category manager (`CLSID_StdComponentCategoriesMgr`):
//! `ICatRegister` + `ICatInformation` over `HKCR\Component Categories` and
//! `HKCR\CLSID\{...}\{Implemented,Required} Categories`, plus the three enumerators
//! (`IEnumCATEGORYINFO`, and `IEnumGUID` in its `IEnumCLSID` / `IEnumCATID` flavors).
//!
//! ## Why the comcat interfaces are declared HERE and not taken from com-sys
//!
//! com-sys's bindgen input does not cover `comcat.h` — it has **zero** ICatRegister /
//! ICatInformation / IEnumGUID / IEnumCATEGORYINFO / CATEGORYINFO coverage on any of
//! the three arches. Until it does, this module carries the declarations, transcribed
//! from `sdk/include/psdk/comcat.idl` (and cross-checked against the widl-generated
//! `build-*/sdk/include/psdk/comcat.h`).
//!
//! They are deliberately **private** to this module, so they do not leak through
//! `prelude`'s `pub use crate::ring::comcat::*` and cannot collide with a future
//! com-sys definition (a local item shadows a glob import). The vtbl slots are spelled
//! exactly the way com-sys/bindgen spells COM slots — `Option<unsafe extern "C" fn>` on
//! amd64/arm64, `Option<unsafe extern "stdcall" fn>` on i386, i.e. what
//! STDMETHODCALLTYPE compiles to per target — so the day com-sys grows comcat, deleting
//! this block is a no-op. `stdmethod!` produces exactly those method types.
//!
//! The IIDs are NOT transcribed: `IID_ICatRegister`, `IID_ICatInformation`,
//! `IID_IEnumGUID` and `IID_IEnumCATEGORYINFO` are all real `R` symbols in `libuuid.a`,
//! so they are declared and linked, never hand-written.
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Import surface
// ---------------------------------------------------------------------------

extern "C" {
    // uuid.lib data symbols (see module docs — never transcribe a GUID by hand).
    static IID_ICatRegister: GUID;
    static IID_ICatInformation: GUID;
    static IID_IEnumCATEGORYINFO: GUID;
    static IID_IEnumGUID: GUID;

    // compobj.c's HKCR helpers (compobj_private.h — plain C linkage, no WINAPI).
    // They live in `crate::compobj`, which carries its own hand-rolled FFI surface and
    // is therefore not reachable through `prelude`; bind them at the symbol level.
    fn open_classes_key(hkey: HKEY, name: LPCWSTR, access: REGSAM, retkey: *mut HKEY) -> LSTATUS;
    fn create_classes_key(hkey: HKEY, name: LPCWSTR, access: REGSAM, retkey: *mut HKEY) -> LSTATUS;

    // CRT (genuinely cdecl on every arch). Not in `crate::crt` yet.
    fn wcsicmp(s1: *const WCHAR, s2: *const WCHAR) -> c_int;
}

/// winnt.h — the registry APIs' return type. Not exported by com-sys; `crate::user32`
/// spells the same thing `LONG`.
type LSTATUS = LONG;

/// wine/debug.h `TRACE_ON(ole)`. The ring's `TRACE!` is a compile-time no-op
/// (`crate::debug_shims`), so the channel is statically off. Kept as a named constant
/// so the guarded diagnostic block in `IsClassOfCategories` stays intact — and can be
/// switched back on the day `TRACE!` grows a real backend.
const TRACE_ON_OLE: bool = false;

// ---------------------------------------------------------------------------
// comcat.idl interface declarations (see module docs)
// ---------------------------------------------------------------------------

/// Spell a COM vtbl slot's fn-pointer type the way STDMETHODCALLTYPE compiles — the
/// type-level twin of `stdmethod!`, and it must stay in lockstep with it. Both are `system`
/// now: that lowers to stdcall on i386 and C on the 64-bit pair, which is what this used to
/// spell per arch, and it is the spelling `stdmethod!` and the com-sys slots now use, so the
/// three unify nominally instead of only lowering the same.
macro_rules! stdmethod_ty {
    ($($sig:tt)*) => { Option<unsafe extern "system" $($sig)*> };
}

/// comcat.idl: `typedef GUID CATID;` / `typedef REFGUID REFCATID;`
type CATID = GUID;
type REFCATID = *const CATID;

/// comcat.idl: `#define IEnumCATID IEnumGUID` / `#define IEnumCLSID IEnumGUID`
type IEnumCATID = IEnumGUID;
type IEnumCLSID = IEnumGUID;
type LPENUMCLSID = *mut IEnumCLSID;
type LPCATREGISTER = *mut ICatRegister;
type LPCATINFORMATION = *mut ICatInformation;

/// comcat.idl: `struct tagCATEGORYINFO`
#[repr(C)]
#[derive(Copy, Clone)]
struct CATEGORYINFO {
    catid: CATID,
    lcid: LCID,
    szDescription: [WCHAR; 128],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ICatRegister {
    lpVtbl: *mut ICatRegisterVtbl,
}

#[repr(C)]
struct ICatRegisterVtbl {
    QueryInterface: stdmethod_ty!(fn(*mut ICatRegister, REFIID, *mut LPVOID) -> HRESULT),
    AddRef: stdmethod_ty!(fn(*mut ICatRegister) -> ULONG),
    Release: stdmethod_ty!(fn(*mut ICatRegister) -> ULONG),
    RegisterCategories: stdmethod_ty!(fn(*mut ICatRegister, ULONG, *mut CATEGORYINFO) -> HRESULT),
    UnRegisterCategories: stdmethod_ty!(fn(*mut ICatRegister, ULONG, *mut CATID) -> HRESULT),
    RegisterClassImplCategories:
        stdmethod_ty!(fn(*mut ICatRegister, REFCLSID, ULONG, *mut CATID) -> HRESULT),
    UnRegisterClassImplCategories:
        stdmethod_ty!(fn(*mut ICatRegister, REFCLSID, ULONG, *mut CATID) -> HRESULT),
    RegisterClassReqCategories:
        stdmethod_ty!(fn(*mut ICatRegister, REFCLSID, ULONG, *mut CATID) -> HRESULT),
    UnRegisterClassReqCategories:
        stdmethod_ty!(fn(*mut ICatRegister, REFCLSID, ULONG, *mut CATID) -> HRESULT),
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ICatInformation {
    lpVtbl: *mut ICatInformationVtbl,
}

#[repr(C)]
struct ICatInformationVtbl {
    QueryInterface: stdmethod_ty!(fn(*mut ICatInformation, REFIID, *mut LPVOID) -> HRESULT),
    AddRef: stdmethod_ty!(fn(*mut ICatInformation) -> ULONG),
    Release: stdmethod_ty!(fn(*mut ICatInformation) -> ULONG),
    EnumCategories:
        stdmethod_ty!(fn(*mut ICatInformation, LCID, *mut *mut IEnumCATEGORYINFO) -> HRESULT),
    GetCategoryDesc:
        stdmethod_ty!(fn(*mut ICatInformation, REFCATID, LCID, *mut LPWSTR) -> HRESULT),
    EnumClassesOfCategories: stdmethod_ty!(
        fn(
            *mut ICatInformation,
            ULONG,
            *mut CATID,
            ULONG,
            *mut CATID,
            *mut *mut IEnumGUID,
        ) -> HRESULT
    ),
    IsClassOfCategories: stdmethod_ty!(
        fn(*mut ICatInformation, REFCLSID, ULONG, *mut CATID, ULONG, *mut CATID) -> HRESULT
    ),
    EnumImplCategoriesOfClass:
        stdmethod_ty!(fn(*mut ICatInformation, REFCLSID, *mut *mut IEnumGUID) -> HRESULT),
    EnumReqCategoriesOfClass:
        stdmethod_ty!(fn(*mut ICatInformation, REFCLSID, *mut *mut IEnumGUID) -> HRESULT),
}

#[repr(C)]
#[derive(Copy, Clone)]
struct IEnumGUID {
    lpVtbl: *mut IEnumGUIDVtbl,
}

#[repr(C)]
struct IEnumGUIDVtbl {
    QueryInterface: stdmethod_ty!(fn(*mut IEnumGUID, REFIID, *mut LPVOID) -> HRESULT),
    AddRef: stdmethod_ty!(fn(*mut IEnumGUID) -> ULONG),
    Release: stdmethod_ty!(fn(*mut IEnumGUID) -> ULONG),
    Next: stdmethod_ty!(fn(*mut IEnumGUID, ULONG, *mut GUID, *mut ULONG) -> HRESULT),
    Skip: stdmethod_ty!(fn(*mut IEnumGUID, ULONG) -> HRESULT),
    Reset: stdmethod_ty!(fn(*mut IEnumGUID) -> HRESULT),
    Clone: stdmethod_ty!(fn(*mut IEnumGUID, *mut *mut IEnumGUID) -> HRESULT),
}

#[repr(C)]
#[derive(Copy, Clone)]
struct IEnumCATEGORYINFO {
    lpVtbl: *mut IEnumCATEGORYINFOVtbl,
}

#[repr(C)]
struct IEnumCATEGORYINFOVtbl {
    QueryInterface: stdmethod_ty!(fn(*mut IEnumCATEGORYINFO, REFIID, *mut LPVOID) -> HRESULT),
    AddRef: stdmethod_ty!(fn(*mut IEnumCATEGORYINFO) -> ULONG),
    Release: stdmethod_ty!(fn(*mut IEnumCATEGORYINFO) -> ULONG),
    Next: stdmethod_ty!(
        fn(*mut IEnumCATEGORYINFO, ULONG, *mut CATEGORYINFO, *mut ULONG) -> HRESULT
    ),
    Skip: stdmethod_ty!(fn(*mut IEnumCATEGORYINFO, ULONG) -> HRESULT),
    Reset: stdmethod_ty!(fn(*mut IEnumCATEGORYINFO) -> HRESULT),
    Clone: stdmethod_ty!(fn(*mut IEnumCATEGORYINFO, *mut *mut IEnumCATEGORYINFO) -> HRESULT),
}

// COBJMACRO vtbl calls for the interfaces above (comcat.h's `ICatRegister_AddRef(This)`
// & co. — macros in C, so there is nothing to link against; these are the real calls).
#[inline]
unsafe fn ICatRegister_QueryInterface(
    This: *mut ICatRegister,
    riid: REFIID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject)
}
#[inline]
unsafe fn ICatRegister_AddRef(This: *mut ICatRegister) -> ULONG {
    ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This)
}
#[inline]
unsafe fn ICatRegister_Release(This: *mut ICatRegister) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap_unchecked())(This)
}
#[inline]
unsafe fn IEnumGUID_AddRef(This: *mut IEnumGUID) -> ULONG {
    ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This)
}

// ---------------------------------------------------------------------------
// The static ComCatMgr instance
// ---------------------------------------------------------------------------

#[repr(C)]
struct ComCatMgrImpl {
    ICatRegister_iface: ICatRegister,
    ICatInformation_iface: ICatInformation,
}

/// C: `static ComCatMgrImpl COMCAT_ComCatMgr = {{ &vtbl }, { &vtbl }};`
/// `static mut` because the interface structs hold raw vtbl pointers (`!Sync`); the
/// object is never mutated, only handed out by address.
static mut COMCAT_ComCatMgr: ComCatMgrImpl = ComCatMgrImpl {
    ICatRegister_iface: ICatRegister {
        lpVtbl: &COMCAT_ICatRegister_Vtbl as *const ICatRegisterVtbl as *mut ICatRegisterVtbl,
    },
    ICatInformation_iface: ICatInformation {
        lpVtbl: &COMCAT_ICatInformation_Vtbl as *const ICatInformationVtbl
            as *mut ICatInformationVtbl,
    },
};

#[repr(C)]
struct class_categories {
    size: ULONG, /* total length, including structure itself */
    impl_offset: ULONG,
    req_offset: ULONG,
}

// ---------------------------------------------------------------------------
// File-scope string constants
// ---------------------------------------------------------------------------

/// `L"Component Categories"` (20 chars + NUL)
static comcat_keyname: [WCHAR; 21] = [
    'C' as WCHAR, 'o' as WCHAR, 'm' as WCHAR, 'p' as WCHAR, 'o' as WCHAR, 'n' as WCHAR,
    'e' as WCHAR, 'n' as WCHAR, 't' as WCHAR, ' ' as WCHAR, 'C' as WCHAR, 'a' as WCHAR,
    't' as WCHAR, 'e' as WCHAR, 'g' as WCHAR, 'o' as WCHAR, 'r' as WCHAR, 'i' as WCHAR,
    'e' as WCHAR, 's' as WCHAR, 0,
];

/// `L"Implemented Categories"` (22 chars + NUL)
static impl_keyname: [WCHAR; 23] = [
    'I' as WCHAR, 'm' as WCHAR, 'p' as WCHAR, 'l' as WCHAR, 'e' as WCHAR, 'm' as WCHAR,
    'e' as WCHAR, 'n' as WCHAR, 't' as WCHAR, 'e' as WCHAR, 'd' as WCHAR, ' ' as WCHAR,
    'C' as WCHAR, 'a' as WCHAR, 't' as WCHAR, 'e' as WCHAR, 'g' as WCHAR, 'o' as WCHAR,
    'r' as WCHAR, 'i' as WCHAR, 'e' as WCHAR, 's' as WCHAR, 0,
];

/// `L"Required Categories"` (19 chars + NUL)
static req_keyname: [WCHAR; 20] = [
    'R' as WCHAR, 'e' as WCHAR, 'q' as WCHAR, 'u' as WCHAR, 'i' as WCHAR, 'r' as WCHAR,
    'e' as WCHAR, 'd' as WCHAR, ' ' as WCHAR, 'C' as WCHAR, 'a' as WCHAR, 't' as WCHAR,
    'e' as WCHAR, 'g' as WCHAR, 'o' as WCHAR, 'r' as WCHAR, 'i' as WCHAR, 'e' as WCHAR,
    's' as WCHAR, 0,
];

/// `L"CLSID"`
static clsid_keyname: [WCHAR; 6] = [
    'C' as WCHAR, 'L' as WCHAR, 'S' as WCHAR, 'I' as WCHAR, 'D' as WCHAR, 0,
];

/// `L"%lX"` — the per-LCID description value name.
static lcid_fmt: [WCHAR; 4] = ['%' as WCHAR, 'l' as WCHAR, 'X' as WCHAR, 0];

/// compobj_private.h: `#define CHARS_IN_GUID 39` (includes the NUL).
const CHARS_IN_GUID_U: usize = CHARS_IN_GUID as usize;

// ---------------------------------------------------------------------------
// COMCAT_RegisterClassCategories
// ---------------------------------------------------------------------------

unsafe fn COMCAT_RegisterClassCategories(
    rclsid: REFCLSID,
    type_: LPCWSTR,
    mut cCategories: ULONG,
    mut rgcatid: *const CATID,
) -> HRESULT {
    let mut keyname: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];
    let mut res: HRESULT;
    let mut clsid_key: HKEY = core::ptr::null_mut();
    let mut class_key: HKEY = core::ptr::null_mut();
    let mut type_key: HKEY = core::ptr::null_mut();

    if cCategories != 0 && rgcatid.is_null() {
        return E_POINTER;
    }

    /* Format the class key name. */
    res = StringFromGUID2(rclsid, keyname.as_mut_ptr(), CHARS_IN_GUID) as HRESULT;
    if FAILED(res) {
        return res;
    }

    /* Create (or open) the CLSID key. */
    res = create_classes_key(
        HKEY_CLASSES_ROOT,
        clsid_keyname.as_ptr(),
        KEY_READ | KEY_WRITE,
        &mut clsid_key,
    );
    if res != ERROR_SUCCESS {
        return E_FAIL;
    }

    /* Create (or open) the class key. */
    res = create_classes_key(
        clsid_key,
        keyname.as_ptr(),
        KEY_READ | KEY_WRITE,
        &mut class_key,
    );
    if res == ERROR_SUCCESS {
        /* Create (or open) the category type key. */
        res = create_classes_key(class_key, type_, KEY_READ | KEY_WRITE, &mut type_key);
        if res == ERROR_SUCCESS {
            while cCategories != 0 {
                let mut key: HKEY = core::ptr::null_mut();

                /* Format the category key name. */
                res = StringFromGUID2(rgcatid, keyname.as_mut_ptr(), CHARS_IN_GUID) as HRESULT;
                if !FAILED(res) {
                    /* Do the register. */
                    res = create_classes_key(
                        type_key,
                        keyname.as_ptr(),
                        KEY_READ | KEY_WRITE,
                        &mut key,
                    );
                    if res == ERROR_SUCCESS {
                        RegCloseKey(key);
                    }
                }

                cCategories -= 1;
                rgcatid = rgcatid.add(1);
            }
            // NOTE (fix vs the C oracle): comcat.c LEAKS `type_key` — it closes
            // class_key and clsid_key but never type_key, so every
            // Register{Impl,Req}Categories call burns a registry handle.
            RegCloseKey(type_key);
            res = S_OK;
        } else {
            res = E_FAIL;
        }
        RegCloseKey(class_key);
    } else {
        res = E_FAIL;
    }
    RegCloseKey(clsid_key);

    res
}

// ---------------------------------------------------------------------------
// COMCAT_UnRegisterClassCategories
// ---------------------------------------------------------------------------

unsafe fn COMCAT_UnRegisterClassCategories(
    rclsid: REFCLSID,
    type_: LPCWSTR,
    mut cCategories: ULONG,
    mut rgcatid: *const CATID,
) -> HRESULT {
    // C: `WCHAR keyname[68] = { 'C','L','S','I','D','\\' };` — prefix, then the class
    // GUID at +6, a '\\' at 44, and the category-type name at +45.
    let mut keyname: [WCHAR; 68] = [0; 68];
    keyname[0] = 'C' as WCHAR;
    keyname[1] = 'L' as WCHAR;
    keyname[2] = 'S' as WCHAR;
    keyname[3] = 'I' as WCHAR;
    keyname[4] = 'D' as WCHAR;
    keyname[5] = '\\' as WCHAR;

    let mut res: HRESULT;
    let mut type_key: HKEY = core::ptr::null_mut();

    if cCategories != 0 && rgcatid.is_null() {
        return E_POINTER;
    }

    /* Format the class category type key name. */
    res = StringFromGUID2(rclsid, keyname.as_mut_ptr().add(6), CHARS_IN_GUID) as HRESULT;
    if FAILED(res) {
        return res;
    }
    keyname[44] = '\\' as WCHAR;
    lstrcpyW(keyname.as_mut_ptr().add(45), type_);

    /* Open the class category type key. */
    res = open_classes_key(
        HKEY_CLASSES_ROOT,
        keyname.as_ptr(),
        KEY_READ | KEY_WRITE,
        &mut type_key,
    );
    if res != ERROR_SUCCESS {
        return E_FAIL;
    }

    while cCategories != 0 {
        /* Format the category key name. (Deliberately reuses `keyname` from the
         * front — `type_key` is already open, so clobbering the path is harmless;
         * this mirrors the C.) */
        res = StringFromGUID2(rgcatid, keyname.as_mut_ptr(), CHARS_IN_GUID) as HRESULT;
        if !FAILED(res) {
            /* Do the unregister. */
            RegDeleteKeyW(type_key, keyname.as_ptr());
        }

        cCategories -= 1;
        rgcatid = rgcatid.add(1);
    }
    RegCloseKey(type_key);

    S_OK
}

// ---------------------------------------------------------------------------
// COMCAT_GetCategoryDesc
// ---------------------------------------------------------------------------

unsafe fn COMCAT_GetCategoryDesc(
    key: HKEY,
    lcid: LCID,
    pszDesc: *mut WCHAR,
    buf_wchars: ULONG,
) -> HRESULT {
    let mut valname: [WCHAR; 5] = [0; 5];
    let mut type_: DWORD = 0;
    // `size` is in BYTES on the way in (RegQueryValueExW's lpcbData always is) and
    // comes back as the byte count actually written.
    let mut size: DWORD = (buf_wchars - 1) * core::mem::size_of::<WCHAR>() as DWORD;

    if pszDesc.is_null() {
        return E_INVALIDARG;
    }

    /* FIXME: lcid comparisons are more complex than this! */
    wsprintfW(valname.as_mut_ptr(), lcid_fmt.as_ptr(), lcid);
    let res = RegQueryValueExW(
        key,
        valname.as_ptr(),
        core::ptr::null_mut(),
        &mut type_,
        pszDesc as *mut u8,
        &mut size,
    );
    if res != ERROR_SUCCESS || type_ != REG_SZ {
        FIXME!("Simplified lcid comparison\n");
        return CAT_E_NODESCRIPTION;
    }
    *pszDesc.add(size as usize / core::mem::size_of::<WCHAR>()) = 0;

    S_OK
}

// ---------------------------------------------------------------------------
// COMCAT_PrepareClassCategories
// ---------------------------------------------------------------------------

unsafe fn COMCAT_PrepareClassCategories(
    mut impl_count: ULONG,
    mut impl_catids: *const CATID,
    mut req_count: ULONG,
    mut req_catids: *const CATID,
) -> *mut class_categories {
    let size: ULONG = core::mem::size_of::<class_categories>() as ULONG
        + ((impl_count + req_count) * CHARS_IN_GUID as ULONG + 2)
            * core::mem::size_of::<WCHAR>() as ULONG;

    let categories =
        HeapAlloc(GetProcessHeap(), 0, size as usize) as *mut class_categories;
    if categories.is_null() {
        return categories;
    }

    (*categories).size = size;
    (*categories).impl_offset = core::mem::size_of::<class_categories>() as ULONG;
    (*categories).req_offset = (*categories).impl_offset
        + (impl_count * CHARS_IN_GUID as ULONG + 1) * core::mem::size_of::<WCHAR>() as ULONG;

    let mut strings = categories.add(1) as *mut WCHAR;
    while impl_count != 0 {
        StringFromGUID2(impl_catids, strings, CHARS_IN_GUID);
        impl_catids = impl_catids.add(1);
        strings = strings.add(CHARS_IN_GUID_U);
        impl_count -= 1;
    }
    *strings = 0;
    strings = strings.add(1);

    while req_count != 0 {
        StringFromGUID2(req_catids, strings, CHARS_IN_GUID);
        req_catids = req_catids.add(1);
        strings = strings.add(CHARS_IN_GUID_U);
        req_count -= 1;
    }
    *strings = 0;

    categories
}

// ---------------------------------------------------------------------------
// COMCAT_IsClassOfCategories
// ---------------------------------------------------------------------------

unsafe fn COMCAT_IsClassOfCategories(key: HKEY, categories: *const class_categories) -> HRESULT {
    let mut subkey: HKEY = core::ptr::null_mut();
    let mut res: LSTATUS;
    let mut string: LPCWSTR;

    let impl_strings =
        (categories as *const u8).add((*categories).impl_offset as usize) as LPCWSTR;
    let req_strings =
        (categories as *const u8).add((*categories).req_offset as usize) as LPCWSTR;

    /* Check that every given category is implemented by class. */
    if *impl_strings != 0 {
        res = open_classes_key(key, impl_keyname.as_ptr(), KEY_READ, &mut subkey);
        if res != ERROR_SUCCESS {
            return S_FALSE;
        }
        string = impl_strings;
        while *string != 0 {
            let mut catkey: HKEY = core::ptr::null_mut();
            res = open_classes_key(subkey, string, READ_CONTROL, &mut catkey);
            if res != ERROR_SUCCESS {
                RegCloseKey(subkey);
                return S_FALSE;
            }
            RegCloseKey(catkey);
            string = string.add(CHARS_IN_GUID_U);
        }
        RegCloseKey(subkey);
    }

    /* Check that all categories required by class are given. */
    res = open_classes_key(key, req_keyname.as_ptr(), KEY_READ, &mut subkey);
    if res == ERROR_SUCCESS {
        let mut index: DWORD = 0;
        loop {
            let mut keyname: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];
            let mut size: DWORD = CHARS_IN_GUID as DWORD;

            res = RegEnumKeyExW(
                subkey,
                index,
                keyname.as_mut_ptr(),
                &mut size,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if res != ERROR_SUCCESS && res != ERROR_MORE_DATA as LSTATUS {
                break;
            }
            index += 1;
            if size != CHARS_IN_GUID as DWORD - 1 {
                continue; /* bogus catid in registry */
            }
            string = req_strings;
            while *string != 0 {
                if wcsicmp(string, keyname.as_ptr()) == 0 {
                    break;
                }
                string = string.add(CHARS_IN_GUID_U);
            }
            if *string == 0 {
                RegCloseKey(subkey);
                return S_FALSE;
            }
        }
        RegCloseKey(subkey);
    }

    S_OK
}

// ---------------------------------------------------------------------------
// ICatRegister
// ---------------------------------------------------------------------------

stdmethod! {
    fn COMCAT_ICatRegister_QueryInterface(
        iface: LPCATREGISTER,
        riid: REFIID,
        ppvObj: *mut LPVOID,
    ) -> HRESULT {
        TRACE!("%s\n", debugstr_guid(riid as *const c_void));

        if ppvObj.is_null() {
            return E_POINTER;
        }

        if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown))
            || IsEqualGUID(riid, core::ptr::addr_of!(IID_ICatRegister))
        {
            *ppvObj = iface as LPVOID;
            ICatRegister_AddRef(iface);
            return S_OK;
        }

        if IsEqualGUID(riid, core::ptr::addr_of!(IID_ICatInformation)) {
            *ppvObj = core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatInformation_iface) as LPVOID;
            ICatRegister_AddRef(iface);
            return S_OK;
        }

        E_NOINTERFACE
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_AddRef(_iface: LPCATREGISTER) -> ULONG {
        2 /* non-heap based object */
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_Release(_iface: LPCATREGISTER) -> ULONG {
        1 /* non-heap based object */
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_RegisterCategories(
        _iface: LPCATREGISTER,
        cCategories: ULONG,
        rgci: *mut CATEGORYINFO,
    ) -> HRESULT {
        // (`stdmethod!` params cannot be `mut`; the C walks both by increment.)
        let mut cCategories = cCategories;
        let mut rgci = rgci;

        TRACE!("\n");

        if cCategories != 0 && rgci.is_null() {
            return E_POINTER;
        }

        /* Create (or open) the component categories key. */
        let mut comcat_key: HKEY = core::ptr::null_mut();
        let res = create_classes_key(
            HKEY_CLASSES_ROOT,
            comcat_keyname.as_ptr(),
            KEY_READ | KEY_WRITE,
            &mut comcat_key,
        );
        if res != ERROR_SUCCESS {
            return E_FAIL;
        }

        while cCategories != 0 {
            let mut keyname: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];
            let mut valname: [WCHAR; 9] = [0; 9];
            let mut cat_key: HKEY = core::ptr::null_mut();

            /* Create (or open) the key for this category. */
            if StringFromGUID2(
                core::ptr::addr_of!((*rgci).catid),
                keyname.as_mut_ptr(),
                CHARS_IN_GUID,
            ) == 0
            {
                cCategories -= 1;
                rgci = rgci.add(1);
                continue;
            }
            let res = create_classes_key(
                comcat_key,
                keyname.as_ptr(),
                KEY_READ | KEY_WRITE,
                &mut cat_key,
            );
            if res != ERROR_SUCCESS {
                cCategories -= 1;
                rgci = rgci.add(1);
                continue;
            }

            /* Set the value for this locale's description. */
            wsprintfW(valname.as_mut_ptr(), lcid_fmt.as_ptr(), (*rgci).lcid);
            RegSetValueExW(
                cat_key,
                valname.as_ptr(),
                0,
                REG_SZ,
                core::ptr::addr_of!((*rgci).szDescription) as *const u8,
                ((lstrlenW((*rgci).szDescription.as_ptr()) + 1) as DWORD)
                    * core::mem::size_of::<WCHAR>() as DWORD,
            );

            RegCloseKey(cat_key);

            cCategories -= 1;
            rgci = rgci.add(1);
        }

        RegCloseKey(comcat_key);
        S_OK
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_UnRegisterCategories(
        _iface: LPCATREGISTER,
        cCategories: ULONG,
        rgcatid: *mut CATID,
    ) -> HRESULT {
        let mut cCategories = cCategories;
        let mut rgcatid = rgcatid;

        TRACE!("\n");

        if cCategories != 0 && rgcatid.is_null() {
            return E_POINTER;
        }

        /* Open the component categories key. */
        let mut comcat_key: HKEY = core::ptr::null_mut();
        let res = open_classes_key(
            HKEY_CLASSES_ROOT,
            comcat_keyname.as_ptr(),
            KEY_READ | KEY_WRITE,
            &mut comcat_key,
        );
        if res != ERROR_SUCCESS {
            return E_FAIL;
        }

        while cCategories != 0 {
            let mut keyname: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];

            /* Delete the key for this category. */
            if StringFromGUID2(rgcatid, keyname.as_mut_ptr(), CHARS_IN_GUID) != 0 {
                RegDeleteKeyW(comcat_key, keyname.as_ptr());
            }

            cCategories -= 1;
            rgcatid = rgcatid.add(1);
        }

        RegCloseKey(comcat_key);
        S_OK
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_RegisterClassImplCategories(
        _iface: LPCATREGISTER,
        rclsid: REFCLSID,
        cCategories: ULONG,
        rgcatid: *mut CATID,
    ) -> HRESULT {
        TRACE!("\n");

        COMCAT_RegisterClassCategories(rclsid, impl_keyname.as_ptr(), cCategories, rgcatid)
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_UnRegisterClassImplCategories(
        _iface: LPCATREGISTER,
        rclsid: REFCLSID,
        cCategories: ULONG,
        rgcatid: *mut CATID,
    ) -> HRESULT {
        TRACE!("\n");

        COMCAT_UnRegisterClassCategories(rclsid, impl_keyname.as_ptr(), cCategories, rgcatid)
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_RegisterClassReqCategories(
        _iface: LPCATREGISTER,
        rclsid: REFCLSID,
        cCategories: ULONG,
        rgcatid: *mut CATID,
    ) -> HRESULT {
        TRACE!("\n");

        COMCAT_RegisterClassCategories(rclsid, req_keyname.as_ptr(), cCategories, rgcatid)
    }
}

stdmethod! {
    fn COMCAT_ICatRegister_UnRegisterClassReqCategories(
        _iface: LPCATREGISTER,
        rclsid: REFCLSID,
        cCategories: ULONG,
        rgcatid: *mut CATID,
    ) -> HRESULT {
        TRACE!("\n");

        COMCAT_UnRegisterClassCategories(rclsid, req_keyname.as_ptr(), cCategories, rgcatid)
    }
}

static COMCAT_ICatRegister_Vtbl: ICatRegisterVtbl = ICatRegisterVtbl {
    QueryInterface: Some(COMCAT_ICatRegister_QueryInterface),
    AddRef: Some(COMCAT_ICatRegister_AddRef),
    Release: Some(COMCAT_ICatRegister_Release),
    RegisterCategories: Some(COMCAT_ICatRegister_RegisterCategories),
    UnRegisterCategories: Some(COMCAT_ICatRegister_UnRegisterCategories),
    RegisterClassImplCategories: Some(COMCAT_ICatRegister_RegisterClassImplCategories),
    UnRegisterClassImplCategories: Some(COMCAT_ICatRegister_UnRegisterClassImplCategories),
    RegisterClassReqCategories: Some(COMCAT_ICatRegister_RegisterClassReqCategories),
    UnRegisterClassReqCategories: Some(COMCAT_ICatRegister_UnRegisterClassReqCategories),
};

// ---------------------------------------------------------------------------
// ICatInformation
// ---------------------------------------------------------------------------

stdmethod! {
    fn COMCAT_ICatInformation_QueryInterface(
        _iface: LPCATINFORMATION,
        riid: REFIID,
        ppvObj: *mut LPVOID,
    ) -> HRESULT {
        ICatRegister_QueryInterface(
            core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatRegister_iface),
            riid,
            ppvObj,
        )
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_AddRef(_iface: LPCATINFORMATION) -> ULONG {
        ICatRegister_AddRef(core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatRegister_iface))
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_Release(_iface: LPCATINFORMATION) -> ULONG {
        ICatRegister_Release(core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatRegister_iface))
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_EnumCategories(
        _iface: LPCATINFORMATION,
        lcid: LCID,
        ppenumCatInfo: *mut *mut IEnumCATEGORYINFO,
    ) -> HRESULT {
        TRACE!("\n");

        if ppenumCatInfo.is_null() {
            return E_POINTER;
        }

        EnumCATEGORYINFO_Construct(lcid, ppenumCatInfo)
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_GetCategoryDesc(
        _iface: LPCATINFORMATION,
        rcatid: REFCATID,
        lcid: LCID,
        ppszDesc: *mut LPWSTR,
    ) -> HRESULT {
        // C: `WCHAR keyname[60] = { 'C','o','m',...,'s','\\', 0 };` — "Component
        // Categories\" is 21 WCHARs, the category GUID lands at +21.
        let mut keyname: [WCHAR; 60] = [0; 60];
        keyname[..comcat_keyname.len() - 1].copy_from_slice(&comcat_keyname[..comcat_keyname.len() - 1]);
        keyname[20] = '\\' as WCHAR;

        TRACE!("CATID: %s LCID: %x\n", debugstr_guid(rcatid as *const c_void), lcid);

        if rcatid.is_null() || ppszDesc.is_null() {
            return E_INVALIDARG;
        }

        /* Open the key for this category. */
        if StringFromGUID2(rcatid, keyname.as_mut_ptr().add(21), CHARS_IN_GUID) == 0 {
            return E_FAIL;
        }
        let mut key: HKEY = core::ptr::null_mut();
        let res = open_classes_key(HKEY_CLASSES_ROOT, keyname.as_ptr(), KEY_READ, &mut key);
        if res != ERROR_SUCCESS {
            return CAT_E_CATIDNOEXIST;
        }

        /* Allocate a sensible amount of memory for the description. */
        *ppszDesc = CoTaskMemAlloc(128 * core::mem::size_of::<WCHAR>() as SIZE_T) as LPWSTR;
        if (*ppszDesc).is_null() {
            RegCloseKey(key);
            return E_OUTOFMEMORY;
        }

        /* Get the description, and make sure it's null terminated. */
        let res = COMCAT_GetCategoryDesc(key, lcid, *ppszDesc, 128);
        RegCloseKey(key);
        if FAILED(res) {
            CoTaskMemFree(*ppszDesc as LPVOID);
            return res;
        }

        S_OK
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_EnumClassesOfCategories(
        _iface: LPCATINFORMATION,
        cImplemented: ULONG,
        rgcatidImpl: *mut CATID,
        cRequired: ULONG,
        rgcatidReq: *mut CATID,
        ppenumCLSID: *mut LPENUMCLSID,
    ) -> HRESULT {
        let mut cImplemented = cImplemented;
        let mut cRequired = cRequired;

        TRACE!("\n");

        if cImplemented == ULONG::MAX {
            cImplemented = 0;
        }
        if cRequired == ULONG::MAX {
            cRequired = 0;
        }

        if ppenumCLSID.is_null()
            || (cImplemented != 0 && rgcatidImpl.is_null())
            || (cRequired != 0 && rgcatidReq.is_null())
        {
            return E_POINTER;
        }

        let categories =
            COMCAT_PrepareClassCategories(cImplemented, rgcatidImpl, cRequired, rgcatidReq);
        if categories.is_null() {
            return E_OUTOFMEMORY;
        }

        let hr = CLSIDEnumGUID_Construct(categories, ppenumCLSID);
        if FAILED(hr) {
            HeapFree(GetProcessHeap(), 0, categories as LPVOID);
            return hr;
        }

        hr
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_IsClassOfCategories(
        _iface: LPCATINFORMATION,
        rclsid: REFCLSID,
        cImplemented: ULONG,
        rgcatidImpl: *mut CATID,
        cRequired: ULONG,
        rgcatidReq: *mut CATID,
    ) -> HRESULT {
        // C: `WCHAR keyname[45] = { 'C','L','S','I','D','\\', 0 };` — 6 + CHARS_IN_GUID.
        let mut keyname: [WCHAR; 45] = [0; 45];
        keyname[0] = 'C' as WCHAR;
        keyname[1] = 'L' as WCHAR;
        keyname[2] = 'S' as WCHAR;
        keyname[3] = 'I' as WCHAR;
        keyname[4] = 'D' as WCHAR;
        keyname[5] = '\\' as WCHAR;

        if TRACE_ON_OLE {
            let mut count: ULONG;
            TRACE!("CLSID: %s Implemented %u\n", debugstr_guid(rclsid as *const c_void), cImplemented);
            count = 0;
            while count < cImplemented {
                TRACE!("    %s\n", debugstr_guid(rgcatidImpl.add(count as usize) as *const c_void));
                count += 1;
            }
            TRACE!("Required %u\n", cRequired);
            count = 0;
            while count < cRequired {
                TRACE!("    %s\n", debugstr_guid(rgcatidReq.add(count as usize) as *const c_void));
                count += 1;
            }
        }

        if (cImplemented != 0 && rgcatidImpl.is_null())
            || (cRequired != 0 && rgcatidReq.is_null())
        {
            return E_POINTER;
        }

        let mut res: HRESULT =
            StringFromGUID2(rclsid, keyname.as_mut_ptr().add(6), CHARS_IN_GUID) as HRESULT;
        if FAILED(res) {
            return res;
        }

        let categories =
            COMCAT_PrepareClassCategories(cImplemented, rgcatidImpl, cRequired, rgcatidReq);
        if categories.is_null() {
            return E_OUTOFMEMORY;
        }

        let mut key: HKEY = core::ptr::null_mut();
        res = open_classes_key(HKEY_CLASSES_ROOT, keyname.as_ptr(), KEY_READ, &mut key);
        if res == ERROR_SUCCESS {
            res = COMCAT_IsClassOfCategories(key, categories);
            RegCloseKey(key);
        } else {
            res = S_FALSE;
        }

        HeapFree(GetProcessHeap(), 0, categories as LPVOID);

        res
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_EnumImplCategoriesOfClass(
        _iface: LPCATINFORMATION,
        rclsid: REFCLSID,
        ppenumCATID: *mut *mut IEnumCATID,
    ) -> HRESULT {
        /// `L"\\Implemented Categories"` (23 chars + NUL)
        static POSTFIX: [WCHAR; 24] = [
            '\\' as WCHAR, 'I' as WCHAR, 'm' as WCHAR, 'p' as WCHAR, 'l' as WCHAR, 'e' as WCHAR,
            'm' as WCHAR, 'e' as WCHAR, 'n' as WCHAR, 't' as WCHAR, 'e' as WCHAR, 'd' as WCHAR,
            ' ' as WCHAR, 'C' as WCHAR, 'a' as WCHAR, 't' as WCHAR, 'e' as WCHAR, 'g' as WCHAR,
            'o' as WCHAR, 'r' as WCHAR, 'i' as WCHAR, 'e' as WCHAR, 's' as WCHAR, 0,
        ];

        TRACE!("%s\n", debugstr_guid(rclsid as *const c_void));

        if rclsid.is_null() || ppenumCATID.is_null() {
            return E_POINTER;
        }

        CATIDEnumGUID_Construct(rclsid, POSTFIX.as_ptr(), ppenumCATID)
    }
}

stdmethod! {
    fn COMCAT_ICatInformation_EnumReqCategoriesOfClass(
        _iface: LPCATINFORMATION,
        rclsid: REFCLSID,
        ppenumCATID: *mut *mut IEnumCATID,
    ) -> HRESULT {
        /// `L"\\Required Categories"` (20 chars + NUL)
        static POSTFIX: [WCHAR; 21] = [
            '\\' as WCHAR, 'R' as WCHAR, 'e' as WCHAR, 'q' as WCHAR, 'u' as WCHAR, 'i' as WCHAR,
            'r' as WCHAR, 'e' as WCHAR, 'd' as WCHAR, ' ' as WCHAR, 'C' as WCHAR, 'a' as WCHAR,
            't' as WCHAR, 'e' as WCHAR, 'g' as WCHAR, 'o' as WCHAR, 'r' as WCHAR, 'i' as WCHAR,
            'e' as WCHAR, 's' as WCHAR, 0,
        ];

        TRACE!("%s\n", debugstr_guid(rclsid as *const c_void));

        if rclsid.is_null() || ppenumCATID.is_null() {
            return E_POINTER;
        }

        CATIDEnumGUID_Construct(rclsid, POSTFIX.as_ptr(), ppenumCATID)
    }
}

static COMCAT_ICatInformation_Vtbl: ICatInformationVtbl = ICatInformationVtbl {
    QueryInterface: Some(COMCAT_ICatInformation_QueryInterface),
    AddRef: Some(COMCAT_ICatInformation_AddRef),
    Release: Some(COMCAT_ICatInformation_Release),
    EnumCategories: Some(COMCAT_ICatInformation_EnumCategories),
    GetCategoryDesc: Some(COMCAT_ICatInformation_GetCategoryDesc),
    EnumClassesOfCategories: Some(COMCAT_ICatInformation_EnumClassesOfCategories),
    IsClassOfCategories: Some(COMCAT_ICatInformation_IsClassOfCategories),
    EnumImplCategoriesOfClass: Some(COMCAT_ICatInformation_EnumImplCategoriesOfClass),
    EnumReqCategoriesOfClass: Some(COMCAT_ICatInformation_EnumReqCategoriesOfClass),
};

// ---------------------------------------------------------------------------
// ComCat_CreateInstance — the class factory hook (moniker.h: `HRESULT WINAPI`)
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn ComCat_CreateInstance(
    _iface: *mut IClassFactory,
    pUnkOuter: *mut IUnknown,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    TRACE!("%s\n", debugstr_guid(riid as *const c_void));

    if ppvObj.is_null() {
        return E_POINTER;
    }

    /* Don't support aggregation (Windows doesn't) */
    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let res = ICatRegister_QueryInterface(
        core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatRegister_iface),
        riid,
        ppvObj,
    );
    if SUCCEEDED(res) {
        return res;
    }

    CLASS_E_CLASSNOTAVAILABLE
}

// ---------------------------------------------------------------------------
// IEnumCATEGORYINFO implementation
//
// This implementation is not thread-safe. The manager itself is, but I can't imagine a
// valid use of an enumerator in several threads.
// ---------------------------------------------------------------------------

#[repr(C)]
struct IEnumCATEGORYINFOImpl {
    IEnumCATEGORYINFO_iface: IEnumCATEGORYINFO,
    /// C's field is `LONG ref` — `ref` is a Rust keyword.
    ref_: LONG,
    lcid: LCID,
    key: HKEY,
    next_index: DWORD,
}

#[inline]
unsafe fn impl_from_IEnumCATEGORYINFO(iface: *mut IEnumCATEGORYINFO) -> *mut IEnumCATEGORYINFOImpl {
    CONTAINING_RECORD!(iface, IEnumCATEGORYINFOImpl, IEnumCATEGORYINFO_iface)
}

stdmethod! {
    fn COMCAT_IEnumCATEGORYINFO_AddRef(iface: *mut IEnumCATEGORYINFO) -> ULONG {
        let This = impl_from_IEnumCATEGORYINFO(iface);

        TRACE!("\n");

        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
    }
}

stdmethod! {
    fn COMCAT_IEnumCATEGORYINFO_QueryInterface(
        iface: *mut IEnumCATEGORYINFO,
        riid: REFIID,
        ppvObj: *mut LPVOID,
    ) -> HRESULT {
        TRACE!("%s\n", debugstr_guid(riid as *const c_void));

        if ppvObj.is_null() {
            return E_POINTER;
        }

        if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown))
            || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumCATEGORYINFO))
        {
            *ppvObj = iface as LPVOID;
            COMCAT_IEnumCATEGORYINFO_AddRef(iface);
            return S_OK;
        }

        E_NOINTERFACE
    }
}

stdmethod! {
    fn COMCAT_IEnumCATEGORYINFO_Release(iface: *mut IEnumCATEGORYINFO) -> ULONG {
        let This = impl_from_IEnumCATEGORYINFO(iface);

        TRACE!("\n");

        let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));
        if ref_count == 0 {
            if !(*This).key.is_null() {
                RegCloseKey((*This).key);
            }
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
            return 0;
        }
        ref_count as ULONG
    }
}

stdmethod! {
    fn COMCAT_IEnumCATEGORYINFO_Next(
        iface: *mut IEnumCATEGORYINFO,
        celt: ULONG,
        rgelt: *mut CATEGORYINFO,
        pceltFetched: *mut ULONG,
    ) -> HRESULT {
        let mut rgelt = rgelt;
        let This = impl_from_IEnumCATEGORYINFO(iface);
        let mut fetched: ULONG = 0;

        TRACE!("\n");

        if rgelt.is_null() {
            return E_POINTER;
        }

        if !(*This).key.is_null() {
            while fetched < celt {
                let mut catid: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];
                let mut cName: DWORD = CHARS_IN_GUID as DWORD;
                let mut subkey: HKEY = core::ptr::null_mut();

                let res = RegEnumKeyExW(
                    (*This).key,
                    (*This).next_index,
                    catid.as_mut_ptr(),
                    &mut cName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if res != ERROR_SUCCESS && res != ERROR_MORE_DATA as LSTATUS {
                    break;
                }
                (*This).next_index += 1;

                let hr = CLSIDFromString(catid.as_ptr(), core::ptr::addr_of_mut!((*rgelt).catid));
                if FAILED(hr) {
                    continue;
                }

                let res = open_classes_key(
                    (*This).key,
                    catid.as_ptr(),
                    KEY_READ,
                    core::ptr::addr_of_mut!(subkey),
                );
                if res != ERROR_SUCCESS {
                    continue;
                }

                let hr = COMCAT_GetCategoryDesc(
                    subkey,
                    (*This).lcid,
                    (*rgelt).szDescription.as_mut_ptr(),
                    128,
                );
                RegCloseKey(subkey);
                if FAILED(hr) {
                    continue;
                }

                (*rgelt).lcid = (*This).lcid;
                fetched += 1;
                rgelt = rgelt.add(1);
            }
        }

        if !pceltFetched.is_null() {
            *pceltFetched = fetched;
        }
        if fetched == celt { S_OK } else { S_FALSE }
    }
}

stdmethod! {
    fn COMCAT_IEnumCATEGORYINFO_Skip(iface: *mut IEnumCATEGORYINFO, celt: ULONG) -> HRESULT {
        let This = impl_from_IEnumCATEGORYINFO(iface);

        TRACE!("\n");

        (*This).next_index = (*This).next_index.wrapping_add(celt);
        /* This should return S_FALSE when there aren't celt elems to skip. */
        S_OK
    }
}

stdmethod! {
    fn COMCAT_IEnumCATEGORYINFO_Reset(iface: *mut IEnumCATEGORYINFO) -> HRESULT {
        let This = impl_from_IEnumCATEGORYINFO(iface);

        TRACE!("\n");

        (*This).next_index = 0;
        S_OK
    }
}

stdmethod! {
    fn COMCAT_IEnumCATEGORYINFO_Clone(
        iface: *mut IEnumCATEGORYINFO,
        ppenum: *mut *mut IEnumCATEGORYINFO,
    ) -> HRESULT {
        let This = impl_from_IEnumCATEGORYINFO(iface);

        TRACE!("\n");

        if ppenum.is_null() {
            return E_POINTER;
        }

        let new_this = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<IEnumCATEGORYINFOImpl>() as usize,
        ) as *mut IEnumCATEGORYINFOImpl;
        if new_this.is_null() {
            return E_OUTOFMEMORY;
        }

        (*new_this).IEnumCATEGORYINFO_iface = (*This).IEnumCATEGORYINFO_iface;
        (*new_this).ref_ = 1;
        (*new_this).lcid = (*This).lcid;
        /* FIXME: could we more efficiently use DuplicateHandle? */
        open_classes_key(
            HKEY_CLASSES_ROOT,
            comcat_keyname.as_ptr(),
            KEY_READ,
            core::ptr::addr_of_mut!((*new_this).key),
        );
        (*new_this).next_index = (*This).next_index;

        *ppenum = core::ptr::addr_of_mut!((*new_this).IEnumCATEGORYINFO_iface);
        S_OK
    }
}

static COMCAT_IEnumCATEGORYINFO_Vtbl: IEnumCATEGORYINFOVtbl = IEnumCATEGORYINFOVtbl {
    QueryInterface: Some(COMCAT_IEnumCATEGORYINFO_QueryInterface),
    AddRef: Some(COMCAT_IEnumCATEGORYINFO_AddRef),
    Release: Some(COMCAT_IEnumCATEGORYINFO_Release),
    Next: Some(COMCAT_IEnumCATEGORYINFO_Next),
    Skip: Some(COMCAT_IEnumCATEGORYINFO_Skip),
    Reset: Some(COMCAT_IEnumCATEGORYINFO_Reset),
    Clone: Some(COMCAT_IEnumCATEGORYINFO_Clone),
};

unsafe fn EnumCATEGORYINFO_Construct(lcid: LCID, ret: *mut *mut IEnumCATEGORYINFO) -> HRESULT {
    *ret = core::ptr::null_mut();

    let This = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<IEnumCATEGORYINFOImpl>() as usize,
    ) as *mut IEnumCATEGORYINFOImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IEnumCATEGORYINFO_iface.lpVtbl =
        &COMCAT_IEnumCATEGORYINFO_Vtbl as *const IEnumCATEGORYINFOVtbl as *mut IEnumCATEGORYINFOVtbl;
    (*This).ref_ = 1;
    (*This).lcid = lcid;
    open_classes_key(
        HKEY_CLASSES_ROOT,
        comcat_keyname.as_ptr(),
        KEY_READ,
        core::ptr::addr_of_mut!((*This).key),
    );

    *ret = core::ptr::addr_of_mut!((*This).IEnumCATEGORYINFO_iface);
    S_OK
}

// ---------------------------------------------------------------------------
// ClassesOfCategories IEnumCLSID (IEnumGUID) implementation
// ---------------------------------------------------------------------------

#[repr(C)]
struct CLSID_IEnumGUIDImpl {
    IEnumGUID_iface: IEnumGUID,
    ref_: LONG,
    categories: *mut class_categories,
    key: HKEY,
    next_index: DWORD,
}

#[inline]
unsafe fn impl_from_IEnumCLSID(iface: *mut IEnumGUID) -> *mut CLSID_IEnumGUIDImpl {
    CONTAINING_RECORD!(iface, CLSID_IEnumGUIDImpl, IEnumGUID_iface)
}

stdmethod! {
    fn CLSIDEnumGUID_QueryInterface(
        iface: *mut IEnumGUID,
        riid: REFIID,
        ppvObj: *mut LPVOID,
    ) -> HRESULT {
        TRACE!("%s\n", debugstr_guid(riid as *const c_void));

        if ppvObj.is_null() {
            return E_POINTER;
        }

        if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown))
            || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumGUID))
        {
            *ppvObj = iface as LPVOID;
            IEnumGUID_AddRef(iface);
            return S_OK;
        }

        E_NOINTERFACE
    }
}

stdmethod! {
    fn CLSIDEnumGUID_AddRef(iface: *mut IEnumGUID) -> ULONG {
        let This = impl_from_IEnumCLSID(iface);

        TRACE!("\n");

        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
    }
}

stdmethod! {
    fn CLSIDEnumGUID_Release(iface: *mut IEnumGUID) -> ULONG {
        let This = impl_from_IEnumCLSID(iface);

        TRACE!("\n");

        let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));
        if ref_count == 0 {
            if !(*This).key.is_null() {
                RegCloseKey((*This).key);
            }
            HeapFree(GetProcessHeap(), 0, (*This).categories as LPVOID);
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
            return 0;
        }
        ref_count as ULONG
    }
}

stdmethod! {
    fn CLSIDEnumGUID_Next(
        iface: *mut IEnumGUID,
        celt: ULONG,
        rgelt: *mut GUID,
        pceltFetched: *mut ULONG,
    ) -> HRESULT {
        let mut rgelt = rgelt;
        let This = impl_from_IEnumCLSID(iface);
        let mut fetched: ULONG = 0;

        TRACE!("\n");

        if rgelt.is_null() {
            return E_POINTER;
        }

        if !(*This).key.is_null() {
            while fetched < celt {
                let mut clsid: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];
                let mut cName: DWORD = CHARS_IN_GUID as DWORD;
                let mut subkey: HKEY = core::ptr::null_mut();

                let res = RegEnumKeyExW(
                    (*This).key,
                    (*This).next_index,
                    clsid.as_mut_ptr(),
                    &mut cName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if res != ERROR_SUCCESS && res != ERROR_MORE_DATA as LSTATUS {
                    break;
                }
                (*This).next_index += 1;

                let hr = CLSIDFromString(clsid.as_ptr(), rgelt);
                if FAILED(hr) {
                    continue;
                }

                let res = open_classes_key(
                    (*This).key,
                    clsid.as_ptr(),
                    KEY_READ,
                    core::ptr::addr_of_mut!(subkey),
                );
                if res != ERROR_SUCCESS {
                    continue;
                }

                let hr = COMCAT_IsClassOfCategories(subkey, (*This).categories);
                RegCloseKey(subkey);
                if hr != S_OK {
                    continue;
                }

                fetched += 1;
                rgelt = rgelt.add(1);
            }
        }

        if !pceltFetched.is_null() {
            *pceltFetched = fetched;
        }
        if fetched == celt { S_OK } else { S_FALSE }
    }
}

stdmethod! {
    fn CLSIDEnumGUID_Skip(iface: *mut IEnumGUID, celt: ULONG) -> HRESULT {
        let This = impl_from_IEnumCLSID(iface);

        TRACE!("\n");

        (*This).next_index = (*This).next_index.wrapping_add(celt);
        FIXME!("Never returns S_FALSE\n");
        S_OK
    }
}

stdmethod! {
    fn CLSIDEnumGUID_Reset(iface: *mut IEnumGUID) -> HRESULT {
        let This = impl_from_IEnumCLSID(iface);

        TRACE!("\n");

        (*This).next_index = 0;
        S_OK
    }
}

stdmethod! {
    fn CLSIDEnumGUID_Clone(iface: *mut IEnumGUID, ppenum: *mut *mut IEnumGUID) -> HRESULT {
        let This = impl_from_IEnumCLSID(iface);

        TRACE!("(%p)->(%p)\n", This, ppenum);

        if ppenum.is_null() {
            return E_POINTER;
        }

        *ppenum = core::ptr::null_mut();

        let cloned = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<CLSID_IEnumGUIDImpl>() as usize,
        ) as *mut CLSID_IEnumGUIDImpl;
        if cloned.is_null() {
            return E_OUTOFMEMORY;
        }

        (*cloned).IEnumGUID_iface.lpVtbl = (*This).IEnumGUID_iface.lpVtbl;
        (*cloned).ref_ = 1;

        (*cloned).categories = HeapAlloc(
            GetProcessHeap(),
            0,
            (*(*This).categories).size as usize,
        ) as *mut class_categories;
        if (*cloned).categories.is_null() {
            HeapFree(GetProcessHeap(), 0, cloned as LPVOID);
            return E_OUTOFMEMORY;
        }
        memcpy(
            (*cloned).categories as *mut c_void,
            (*This).categories as *const c_void,
            (*(*This).categories).size as usize,
        );

        (*cloned).key = core::ptr::null_mut();
        open_classes_key(
            HKEY_CLASSES_ROOT,
            clsid_keyname.as_ptr(),
            KEY_READ,
            core::ptr::addr_of_mut!((*cloned).key),
        );
        (*cloned).next_index = (*This).next_index;

        *ppenum = core::ptr::addr_of_mut!((*cloned).IEnumGUID_iface);
        S_OK
    }
}

static CLSIDEnumGUIDVtbl: IEnumGUIDVtbl = IEnumGUIDVtbl {
    QueryInterface: Some(CLSIDEnumGUID_QueryInterface),
    AddRef: Some(CLSIDEnumGUID_AddRef),
    Release: Some(CLSIDEnumGUID_Release),
    Next: Some(CLSIDEnumGUID_Next),
    Skip: Some(CLSIDEnumGUID_Skip),
    Reset: Some(CLSIDEnumGUID_Reset),
    Clone: Some(CLSIDEnumGUID_Clone),
};

unsafe fn CLSIDEnumGUID_Construct(
    categories: *mut class_categories,
    ret: *mut *mut IEnumCLSID,
) -> HRESULT {
    *ret = core::ptr::null_mut();

    let This = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CLSID_IEnumGUIDImpl>() as usize,
    ) as *mut CLSID_IEnumGUIDImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IEnumGUID_iface.lpVtbl =
        &CLSIDEnumGUIDVtbl as *const IEnumGUIDVtbl as *mut IEnumGUIDVtbl;
    (*This).ref_ = 1;
    (*This).categories = categories;
    open_classes_key(
        HKEY_CLASSES_ROOT,
        clsid_keyname.as_ptr(),
        KEY_READ,
        core::ptr::addr_of_mut!((*This).key),
    );

    *ret = core::ptr::addr_of_mut!((*This).IEnumGUID_iface);

    S_OK
}

// ---------------------------------------------------------------------------
// CategoriesOfClass IEnumCATID (IEnumGUID) implementation
// ---------------------------------------------------------------------------

#[repr(C)]
struct CATID_IEnumGUIDImpl {
    IEnumGUID_iface: IEnumGUID,
    ref_: LONG,
    keyname: [WCHAR; 68],
    key: HKEY,
    next_index: DWORD,
}

#[inline]
unsafe fn impl_from_IEnumCATID(iface: *mut IEnumGUID) -> *mut CATID_IEnumGUIDImpl {
    CONTAINING_RECORD!(iface, CATID_IEnumGUIDImpl, IEnumGUID_iface)
}

stdmethod! {
    fn CATIDEnumGUID_QueryInterface(
        iface: *mut IEnumGUID,
        riid: REFIID,
        ppvObj: *mut LPVOID,
    ) -> HRESULT {
        TRACE!("%s\n", debugstr_guid(riid as *const c_void));

        if ppvObj.is_null() {
            return E_POINTER;
        }

        if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown))
            || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumGUID))
        {
            *ppvObj = iface as LPVOID;
            IEnumGUID_AddRef(iface);
            return S_OK;
        }

        E_NOINTERFACE
    }
}

stdmethod! {
    fn CATIDEnumGUID_AddRef(iface: *mut IEnumGUID) -> ULONG {
        let This = impl_from_IEnumCATID(iface);

        TRACE!("\n");

        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
    }
}

stdmethod! {
    fn CATIDEnumGUID_Release(iface: *mut IEnumGUID) -> ULONG {
        let This = impl_from_IEnumCATID(iface);

        TRACE!("\n");

        let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));
        if ref_count == 0 {
            if !(*This).key.is_null() {
                RegCloseKey((*This).key);
            }
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
            return 0;
        }
        ref_count as ULONG
    }
}

stdmethod! {
    fn CATIDEnumGUID_Next(
        iface: *mut IEnumGUID,
        celt: ULONG,
        rgelt: *mut GUID,
        pceltFetched: *mut ULONG,
    ) -> HRESULT {
        let mut rgelt = rgelt;
        let This = impl_from_IEnumCATID(iface);
        let mut fetched: ULONG = 0;

        TRACE!("\n");

        if rgelt.is_null() {
            return E_POINTER;
        }

        if !(*This).key.is_null() {
            while fetched < celt {
                let mut catid: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];
                let mut cName: DWORD = CHARS_IN_GUID as DWORD;

                let res = RegEnumKeyExW(
                    (*This).key,
                    (*This).next_index,
                    catid.as_mut_ptr(),
                    &mut cName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if res != ERROR_SUCCESS && res != ERROR_MORE_DATA as LSTATUS {
                    break;
                }
                (*This).next_index += 1;

                let hr = CLSIDFromString(catid.as_ptr(), rgelt);
                if FAILED(hr) {
                    continue;
                }

                fetched += 1;
                rgelt = rgelt.add(1);
            }
        }

        if !pceltFetched.is_null() {
            *pceltFetched = fetched;
        }
        if fetched == celt { S_OK } else { S_FALSE }
    }
}

stdmethod! {
    fn CATIDEnumGUID_Skip(iface: *mut IEnumGUID, celt: ULONG) -> HRESULT {
        let This = impl_from_IEnumCATID(iface);

        TRACE!("\n");

        (*This).next_index = (*This).next_index.wrapping_add(celt);
        FIXME!("Never returns S_FALSE\n");
        S_OK
    }
}

stdmethod! {
    fn CATIDEnumGUID_Reset(iface: *mut IEnumGUID) -> HRESULT {
        let This = impl_from_IEnumCATID(iface);

        TRACE!("\n");

        (*This).next_index = 0;
        S_OK
    }
}

stdmethod! {
    fn CATIDEnumGUID_Clone(iface: *mut IEnumGUID, ppenum: *mut *mut IEnumGUID) -> HRESULT {
        let This = impl_from_IEnumCATID(iface);

        TRACE!("\n");

        if ppenum.is_null() {
            return E_POINTER;
        }

        let new_this = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<CATID_IEnumGUIDImpl>() as usize,
        ) as *mut CATID_IEnumGUIDImpl;
        if new_this.is_null() {
            return E_OUTOFMEMORY;
        }

        (*new_this).IEnumGUID_iface.lpVtbl = (*This).IEnumGUID_iface.lpVtbl;
        (*new_this).ref_ = 1;
        lstrcpyW((*new_this).keyname.as_mut_ptr(), (*This).keyname.as_ptr());
        /* FIXME: could we more efficiently use DuplicateHandle? */
        open_classes_key(
            HKEY_CLASSES_ROOT,
            (*new_this).keyname.as_ptr(),
            KEY_READ,
            core::ptr::addr_of_mut!((*new_this).key),
        );
        (*new_this).next_index = (*This).next_index;

        *ppenum = core::ptr::addr_of_mut!((*new_this).IEnumGUID_iface);
        S_OK
    }
}

static CATIDEnumGUIDVtbl: IEnumGUIDVtbl = IEnumGUIDVtbl {
    QueryInterface: Some(CATIDEnumGUID_QueryInterface),
    AddRef: Some(CATIDEnumGUID_AddRef),
    Release: Some(CATIDEnumGUID_Release),
    Next: Some(CATIDEnumGUID_Next),
    Skip: Some(CATIDEnumGUID_Skip),
    Reset: Some(CATIDEnumGUID_Reset),
    Clone: Some(CATIDEnumGUID_Clone),
};

unsafe fn CATIDEnumGUID_Construct(
    rclsid: REFCLSID,
    postfix: LPCWSTR,
    ret: *mut *mut IEnumCATID,
) -> HRESULT {
    /// `L"CLSID\\"`
    static prefixW: [WCHAR; 7] = [
        'C' as WCHAR, 'L' as WCHAR, 'S' as WCHAR, 'I' as WCHAR, 'D' as WCHAR, '\\' as WCHAR, 0,
    ];
    let mut clsidW: [WCHAR; CHARS_IN_GUID_U] = [0; CHARS_IN_GUID_U];

    *ret = core::ptr::null_mut();

    let This = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CATID_IEnumGUIDImpl>() as usize,
    ) as *mut CATID_IEnumGUIDImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    StringFromGUID2(rclsid, clsidW.as_mut_ptr(), CHARS_IN_GUID);

    (*This).IEnumGUID_iface.lpVtbl =
        &CATIDEnumGUIDVtbl as *const IEnumGUIDVtbl as *mut IEnumGUIDVtbl;
    (*This).ref_ = 1;
    // NOTE (faithful to the C, and an UPSTREAM WINE BUG): comcat.c builds the path in a
    // LOCAL `WCHAR keyname[100]` and never copies it into `This->keyname`, which stays
    // "" (HEAP_ZERO_MEMORY). CATIDEnumGUID_Clone then lstrcpyW's that empty string and
    // re-opens HKCR itself, so a cloned CATID enumerator enumerates the wrong key.
    // Left as-is so this translation stays behaviour-identical to the oracle; fixing it
    // is a separate, deliberate change (build the path INTO (*This).keyname).
    let mut keyname: [WCHAR; 100] = [0; 100];
    lstrcpyW(keyname.as_mut_ptr(), prefixW.as_ptr());
    lstrcatW(keyname.as_mut_ptr(), clsidW.as_ptr());
    lstrcatW(keyname.as_mut_ptr(), postfix);

    open_classes_key(
        HKEY_CLASSES_ROOT,
        keyname.as_ptr(),
        KEY_READ,
        core::ptr::addr_of_mut!((*This).key),
    );

    *ret = core::ptr::addr_of_mut!((*This).IEnumGUID_iface);
    S_OK
}
