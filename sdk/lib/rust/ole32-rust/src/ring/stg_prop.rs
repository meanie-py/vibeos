use crate::prelude::*;
use core::ffi::c_void;

/// com-sys spells the COM vtbl slots `unsafe extern "stdcall" fn` on i386 and
/// `unsafe extern "C" fn` on amd64/arm64 — i.e. exactly what STDMETHODCALLTYPE
/// compiles to per arch. `extern "system"` *lowers* to the same ABI but rustc keeps
/// it a DISTINCT fn type that will not unify with the bindgen slot type, so the vtbl
/// impls must spell the ABI the way com-sys does. (Same macro as hglobalstream.rs.)
macro_rules! stdmethod {
    ($(
        $(#[$attr:meta])*
        unsafe fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty $body:block
    )*) => {$(
        $(#[$attr])*
        unsafe extern "system" fn $name($($arg: $ty),*) -> $ret $body
    )*};
}

// ---- file-scope preamble (translated) ----
pub const PROPSETHDR_BYTEORDER_MAGIC: u16 = 0xfffe;
pub const PROPSETHDR_OSVER_KIND_WIN16: u32 = 0;
pub const PROPSETHDR_OSVER_KIND_MAC: u32 = 1;
pub const PROPSETHDR_OSVER_KIND_WIN32: u32 = 2;
pub const CP_UNICODE: u32 = 1200;
pub const MAX_VERSION_0_PROP_NAME_LENGTH: u32 = 256;
pub const CFTAG_WINDOWS: i32 = -1;
pub const CFTAG_MACINTOSH: i32 = -2;
pub const CFTAG_FMTID: i32 = -3;
pub const CFTAG_NODATA: i32 = 0;

/// propidl.idl:67 `const PROPID PID_DICTIONARY = 0x00000000;`
/// (build-amd64/sdk/include/psdk/propidl.h:111 → `#define PID_DICTIONARY (0x0)`).
/// NOT `pub`: this belongs in `crate::win_consts` next to PID_CODEPAGE/PID_LOCALE —
/// see the report. Keeping it module-private avoids a future glob-import clash.

/// wtypes.idl:776 `VT_BYREF = 0x4000` (build-amd64/sdk/include/psdk/wtypes.h:718).
/// Missing from `crate::win_consts` (which does carry VT_VECTOR/VT_ARRAY) — see the
/// report. Module-private for the same reason as PID_DICTIONARY above.

#[repr(C)]
pub struct PROPERTYSETHEADER {
    pub wByteOrder: u16,
    pub wFormat: u16,
    pub dwOSVer: u32,
    pub clsid: CLSID,
    pub reserved: u32,
}

#[repr(C)]
pub struct FORMATIDOFFSET {
    pub fmtid: FMTID,
    pub dwOffset: u32,
}

#[repr(C)]
pub struct PROPERTYSECTIONHEADER {
    pub cbSection: u32,
    pub cProperties: u32,
}

#[repr(C)]
pub struct PROPERTYIDOFFSET {
    pub propid: u32,
    pub dwOffset: u32,
}

#[repr(C)]
pub struct PropertyStorage_impl {
    pub IPropertyStorage_iface: IPropertyStorage,
    pub ref_: i32,
    pub cs: CRITICAL_SECTION,
    pub stm: *mut IStream,
    pub dirty: u8,
    pub fmtid: FMTID,
    pub clsid: CLSID,
    pub format: u16,
    pub originatorOS: u32,
    pub grfFlags: u32,
    pub grfMode: u32,
    pub codePage: u32,
    pub locale: u32,
    pub highestProp: u32,
    pub name_to_propid: *mut dictionary,
    pub propid_to_name: *mut dictionary,
    pub propid_to_prop: *mut dictionary,
}

pub type PPropertyStorage_impl = *mut PropertyStorage_impl;

#[repr(C)]
pub struct DictionaryClosure {
    pub hr: HRESULT,
    pub bytesWritten: u32,
}

pub const SECTIONHEADER_OFFSET: usize =
    core::mem::size_of::<PROPERTYSETHEADER>() + core::mem::size_of::<FORMATIDOFFSET>();

#[repr(C)]
pub struct PropertyClosure {
    pub hr: HRESULT,
    pub propNum: u32,
    pub sectionOffset: *mut u32,
}

/// stg_prop.c:2612 — `{ 5,'S','u','m','m','a','r','y','I','n','f','o','r','m','a','t','i','o','n',0 }`
pub static szSummaryInfo: [u16; 20] = [
    5, b'S' as u16, b'u' as u16, b'm' as u16, b'm' as u16, b'a' as u16, b'r' as u16, b'y' as u16,
    b'I' as u16, b'n' as u16, b'f' as u16, b'o' as u16, b'r' as u16, b'm' as u16, b'a' as u16,
    b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0,
];

/// stg_prop.c:2614 — `{ 5,'D','o','c','u','m','e','n','t','S','u','m','m','a','r','y','I','n','f','o','r','m','a','t','i','o','n',0 }`
pub static szDocSummaryInfo: [u16; 28] = [
    5, b'D' as u16, b'o' as u16, b'c' as u16, b'u' as u16, b'm' as u16, b'e' as u16, b'n' as u16,
    b't' as u16, b'S' as u16, b'u' as u16, b'm' as u16, b'm' as u16, b'a' as u16, b'r' as u16,
    b'y' as u16, b'I' as u16, b'n' as u16, b'f' as u16, b'o' as u16, b'r' as u16, b'm' as u16,
    b'a' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0,
];

pub const BITS_PER_BYTE: u32 = 8;
pub const CHARMASK: u32 = 0x1f;
pub const BITS_IN_CHARMASK: u32 = 5;
pub const NUM_ALPHA_CHARS: u32 = 26;

unsafe fn impl_from_IPropertySetStorage(iface: *mut IPropertySetStorage) -> *mut StorageImpl {
    (iface as *mut u8).offset(
        -(core::mem::offset_of!(StorageImpl, base) as isize
            + core::mem::offset_of!(StorageBaseImpl, IPropertySetStorage_iface) as isize),
    ) as *mut StorageImpl
}

unsafe fn impl_from_IPropertyStorage(iface: *mut IPropertyStorage) -> *mut PropertyStorage_impl {
    (iface as *mut u8)
        .offset(-(core::mem::offset_of!(PropertyStorage_impl, IPropertyStorage_iface) as isize))
        as *mut PropertyStorage_impl
}

/// `This->cs.DebugInfo->Spare[0] = x` — `crate::win32::CRITICAL_SECTION` models
/// DebugInfo as an opaque `*mut c_void`, and `ring::ifs::CRITICAL_SECTION_DEBUG`
/// models the trailing ULONG_PTR union through its `WineDebugString` arm, which is
/// exactly the arm `Spare[0]` aliases (see the note on CRITICAL_SECTION_DEBUG).
#[inline]
unsafe fn set_critsec_debug_name(cs: *mut CRITICAL_SECTION, name: *const c_char) {
    let dbg = (*cs).DebugInfo as *mut CRITICAL_SECTION_DEBUG;
    if !dbg.is_null() {
        (*dbg).WineDebugString = name;
    }
}

stdmethod! {

unsafe fn IPropertyStorage_fnQueryInterface(
    iface: *mut IPropertyStorage,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let _This = impl_from_IPropertyStorage(iface);

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr() as LPCSTR,
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualGUID(&IID_IUnknown, riid) || IsEqualGUID(&IID_IPropertyStorage, riid) {
        IPropertyStorage_AddRef(iface);
        *ppvObject = iface as *mut c_void;
        return S_OK;
    }

    E_NOINTERFACE
}

unsafe fn IPropertyStorage_fnAddRef(iface: *mut IPropertyStorage) -> ULONG {
    let This = impl_from_IPropertyStorage(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}

unsafe fn IPropertyStorage_fnRelease(iface: *mut IPropertyStorage) -> ULONG {
    let This = impl_from_IPropertyStorage(iface);
    let ref_count: ULONG;

    ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;
    if ref_count == 0 {
        TRACE!(b"Destroying %p\n\0".as_ptr() as LPCSTR, This);
        if (*This).dirty != 0 {
            IPropertyStorage_Commit(iface, STGC_DEFAULT);
        }
        IStream_Release((*This).stm);
        set_critsec_debug_name(core::ptr::addr_of_mut!((*This).cs), core::ptr::null());
        DeleteCriticalSection(core::ptr::addr_of_mut!((*This).cs));
        PropertyStorage_DestroyDictionaries(This);
        HeapFree(GetProcessHeap(), 0, This as *mut c_void);
    }
    ref_count
}

} // stdmethod!

unsafe fn PropertyStorage_FindProperty(
    This: *mut PropertyStorage_impl,
    propid: DWORD,
) -> *mut PROPVARIANT {
    let mut ret: *mut PROPVARIANT = core::ptr::null_mut();

    dictionary_find(
        (*This).propid_to_prop,
        UlongToPtr(propid),
        core::ptr::addr_of_mut!(ret) as *mut *mut c_void,
    );
    TRACE!(b"returning %p\n\0".as_ptr() as LPCSTR, ret);
    ret
}

/// Returns NULL if name is NULL.
unsafe fn PropertyStorage_FindPropertyByName(
    This: *mut PropertyStorage_impl,
    name: LPCWSTR,
) -> *mut PROPVARIANT {
    let mut ret: *mut PROPVARIANT = core::ptr::null_mut();
    let mut propid: *mut c_void = core::ptr::null_mut();

    if name.is_null() {
        return core::ptr::null_mut();
    }
    if (*This).codePage == CP_UNICODE {
        if dictionary_find(
            (*This).name_to_propid,
            name as *const c_void,
            core::ptr::addr_of_mut!(propid),
        ) != 0
        {
            ret = PropertyStorage_FindProperty(This, PtrToUlong(propid));
        }
    } else {
        let mut ansiName: LPSTR = core::ptr::null_mut();
        let hr: HRESULT = PropertyStorage_StringCopy(
            name as LPCSTR,
            CP_UNICODE,
            core::ptr::addr_of_mut!(ansiName),
            (*This).codePage,
        );

        if SUCCEEDED(hr) {
            if dictionary_find(
                (*This).name_to_propid,
                ansiName as *const c_void,
                core::ptr::addr_of_mut!(propid),
            ) != 0
            {
                ret = PropertyStorage_FindProperty(This, PtrToUlong(propid));
            }
            CoTaskMemFree(ansiName as *mut c_void);
        }
    }
    TRACE!(b"returning %p\n\0".as_ptr() as LPCSTR, ret);
    ret
}

unsafe fn PropertyStorage_FindPropertyNameById(
    This: *mut PropertyStorage_impl,
    propid: DWORD,
) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();

    dictionary_find(
        (*This).propid_to_name,
        UlongToPtr(propid),
        core::ptr::addr_of_mut!(ret) as *mut *mut c_void,
    );
    TRACE!(b"returning %p\n\0".as_ptr() as LPCSTR, ret);
    ret
}

stdmethod! {

unsafe fn IPropertyStorage_fnReadMultiple(
    iface: *mut IPropertyStorage,
    cpspec: ULONG,
    rgpspec: *const PROPSPEC,
    rgpropvar: *mut PROPVARIANT,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: ULONG;

    TRACE!(b"(%p, %d, %p, %p)\n\0".as_ptr() as LPCSTR, iface, cpspec, rgpspec, rgpropvar);

    if cpspec == 0 {
        return S_FALSE;
    }
    if rgpspec.is_null() || rgpropvar.is_null() {
        return E_INVALIDARG;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    i = 0;
    while i < cpspec {
        PropVariantInit(rgpropvar.add(i as usize));
        if (*rgpspec.add(i as usize)).ulKind == PRSPEC_LPWSTR {
            let prop = PropertyStorage_FindPropertyByName(
                This,
                (*rgpspec.add(i as usize)).__bindgen_anon_1.lpwstr,
            );

            if !prop.is_null() {
                PropertyStorage_PropVariantCopy(
                    rgpropvar.add(i as usize),
                    prop,
                    GetACP(),
                    (*This).codePage,
                );
            }
        } else {
            match (*rgpspec.add(i as usize)).__bindgen_anon_1.propid {
                PID_CODEPAGE => {
                    (*rgpropvar.add(i as usize)).vt = VT_I2 as VARTYPE;
                    (*rgpropvar.add(i as usize)).__bindgen_anon_1.iVal = (*This).codePage as i16;
                }
                PID_LOCALE => {
                    (*rgpropvar.add(i as usize)).vt = VT_I4 as VARTYPE;
                    (*rgpropvar.add(i as usize)).__bindgen_anon_1.lVal = (*This).locale as i32;
                }
                _ => {
                    let prop = PropertyStorage_FindProperty(
                        This,
                        (*rgpspec.add(i as usize)).__bindgen_anon_1.propid,
                    );

                    if !prop.is_null() {
                        PropertyStorage_PropVariantCopy(
                            rgpropvar.add(i as usize),
                            prop,
                            GetACP(),
                            (*This).codePage,
                        );
                    } else {
                        hr = S_FALSE;
                    }
                }
            }
        }
        i += 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

} // stdmethod!

unsafe fn PropertyStorage_StringCopy(
    src: LPCSTR,
    srcCP: LCID,
    dst: *mut LPSTR,
    dstCP: LCID,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut len: i32;

    TRACE!(
        b"%s, %p, %d, %d\n\0".as_ptr() as LPCSTR,
        if srcCP == CP_UNICODE { debugstr_w(src as LPCWSTR) } else { debugstr_a(src) },
        dst,
        dstCP,
        srcCP,
    );
    assert(!src.is_null());
    assert(!dst.is_null());
    *dst = core::ptr::null_mut();

    if dstCP == srcCP {
        let len_size: usize;

        if dstCP == CP_UNICODE {
            len_size = ((lstrlenW(src as LPCWSTR) + 1) * core::mem::size_of::<WCHAR>() as i32) as usize;
        } else {
            len_size = strlen(src) + 1;
        }
        // UPSTREAM BUG (stg_prop.c:374): `CoTaskMemAlloc(len * sizeof(WCHAR))` — len is
        // already a BYTE count on both arms here, so this over-allocates 2x. Harmless
        // (the memcpy below copies only `len` bytes); preserved bug-for-bug.
        *dst = CoTaskMemAlloc((len_size * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPSTR;
        if (*dst).is_null() {
            hr = STG_E_INSUFFICIENTMEMORY;
        } else {
            memcpy(*dst as *mut c_void, src as *const c_void, len_size);
        }
    } else {
        if dstCP == CP_UNICODE {
            len = MultiByteToWideChar(srcCP, 0, src, -1, core::ptr::null_mut(), 0);
            *dst = CoTaskMemAlloc((len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPSTR;
            if (*dst).is_null() {
                hr = STG_E_INSUFFICIENTMEMORY;
            } else {
                MultiByteToWideChar(srcCP, 0, src, -1, *dst as LPWSTR, len);
            }
        } else {
            let mut wideStr: LPCWSTR = core::ptr::null();
            let mut wideStr_tmp: LPWSTR = core::ptr::null_mut();

            if srcCP == CP_UNICODE {
                wideStr = src as LPCWSTR;
            } else {
                len = MultiByteToWideChar(srcCP, 0, src, -1, core::ptr::null_mut(), 0);
                wideStr_tmp = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
                ) as LPWSTR;
                if !wideStr_tmp.is_null() {
                    MultiByteToWideChar(srcCP, 0, src, -1, wideStr_tmp, len);
                    wideStr = wideStr_tmp;
                } else {
                    hr = STG_E_INSUFFICIENTMEMORY;
                }
            }
            if SUCCEEDED(hr) {
                len = WideCharToMultiByte(
                    dstCP,
                    0,
                    wideStr,
                    -1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                *dst = CoTaskMemAlloc(len as SIZE_T) as LPSTR;
                if (*dst).is_null() {
                    hr = STG_E_INSUFFICIENTMEMORY;
                } else {
                    let mut defCharUsed: BOOL = 0;

                    if WideCharToMultiByte(
                        dstCP,
                        0,
                        wideStr,
                        -1,
                        *dst,
                        len,
                        core::ptr::null(),
                        core::ptr::addr_of_mut!(defCharUsed),
                    ) == 0
                        || defCharUsed != 0
                    {
                        CoTaskMemFree(*dst as *mut c_void);
                        *dst = core::ptr::null_mut();
                        hr = HRESULT_FROM_WIN32(ERROR_NO_UNICODE_TRANSLATION);
                    }
                }
            }
            HeapFree(GetProcessHeap(), 0, wideStr_tmp as *mut c_void);
        }
    }
    TRACE!(
        b"returning 0x%08x (%s)\n\0".as_ptr() as LPCSTR,
        hr,
        if dstCP == CP_UNICODE { debugstr_w(*dst as LPCWSTR) } else { debugstr_a(*dst) },
    );
    hr
}

unsafe fn PropertyStorage_PropVariantCopy(
    prop: *mut PROPVARIANT,
    propvar: *const PROPVARIANT,
    targetCP: LCID,
    srcCP: LCID,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    assert(!prop.is_null());
    assert(!propvar.is_null());
    if (*propvar).vt == VT_LPSTR as VARTYPE {
        hr = PropertyStorage_StringCopy(
            (*propvar).__bindgen_anon_1.pszVal,
            srcCP,
            core::ptr::addr_of_mut!((*prop).__bindgen_anon_1.pszVal),
            targetCP,
        );
        if SUCCEEDED(hr) {
            (*prop).vt = VT_LPSTR as VARTYPE;
        }
    } else if (*propvar).vt == VT_BSTR as VARTYPE {
        // wine-10: by length, so a BSTR with embedded NULs survives (PropVariantCopy
        // routes it through PropSysAllocString, which stops at the first NUL).
        (*prop).__bindgen_anon_1.bstrVal = SysAllocStringLen(
            (*propvar).__bindgen_anon_1.bstrVal,
            SysStringLen((*propvar).__bindgen_anon_1.bstrVal),
        );
        if !(*prop).__bindgen_anon_1.bstrVal.is_null() {
            (*prop).vt = VT_BSTR as VARTYPE;
        } else {
            hr = E_OUTOFMEMORY;
        }
    } else {
        // wine-10: the copy's own result is the result (it was discarded, and a failed
        // copy was then stored as if it had succeeded).
        hr = PropVariantCopy(prop, propvar);
    }
    hr
}

/// Stores the property with id propid and value propvar into this property storage.
/// As a side effect, may change This->format to 1 if the type of propvar is a
/// version 1-only property.
unsafe fn PropertyStorage_StorePropWithId(
    This: *mut PropertyStorage_impl,
    propid: PROPID,
    propvar: *const PROPVARIANT,
    lcid: LCID,
) -> HRESULT {
    let hr: HRESULT;
    let prop: *mut PROPVARIANT = PropertyStorage_FindProperty(This, propid);

    assert(!propvar.is_null());
    if ((*propvar).vt & VT_BYREF as VARTYPE) != 0 || ((*propvar).vt & VT_ARRAY as VARTYPE) != 0 {
        (*This).format = 1;
    }
    match (*propvar).vt as DWORD {
        // VT_DECIMAL | VT_I1 | VT_INT | VT_UINT | VT_VECTOR|VT_I1
        VT_DECIMAL | VT_I1 | VT_INT | VT_UINT | 0x1010 => {
            (*This).format = 1;
        }
        _ => {}
    }
    TRACE!(
        b"Setting 0x%08x to type %d\n\0".as_ptr() as LPCSTR,
        propid,
        (*propvar).vt,
    );
    if !prop.is_null() {
        PropVariantClear(prop);
        hr = PropertyStorage_PropVariantCopy(prop, propvar, (*This).codePage, lcid);
    } else {
        let prop = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<PROPVARIANT>() as SIZE_T,
        ) as *mut PROPVARIANT;
        if !prop.is_null() {
            hr = PropertyStorage_PropVariantCopy(prop, propvar, (*This).codePage, lcid);
            if SUCCEEDED(hr) {
                dictionary_insert(
                    (*This).propid_to_prop,
                    UlongToPtr(propid),
                    prop as *mut c_void,
                );
                if propid > (*This).highestProp {
                    (*This).highestProp = propid;
                }
            } else {
                HeapFree(GetProcessHeap(), 0, prop as *mut c_void);
            }
        } else {
            hr = STG_E_INSUFFICIENTMEMORY;
        }
    }
    hr
}

/// Adds the name srcName to the name dictionaries, mapped to property ID id.
unsafe fn PropertyStorage_StoreNameWithId(
    This: *mut PropertyStorage_impl,
    srcName: LPCSTR,
    cp: LCID,
    id: PROPID,
) -> HRESULT {
    let mut name: LPSTR = core::ptr::null_mut();
    let hr: HRESULT;

    assert(!srcName.is_null());

    hr = PropertyStorage_StringCopy(srcName, cp, core::ptr::addr_of_mut!(name), (*This).codePage);
    if SUCCEEDED(hr) {
        if (*This).codePage == CP_UNICODE {
            if lstrlenW(name as LPCWSTR) >= MAX_VERSION_0_PROP_NAME_LENGTH as i32 {
                (*This).format = 1;
            }
        } else {
            if strlen(name) >= MAX_VERSION_0_PROP_NAME_LENGTH as usize {
                (*This).format = 1;
            }
        }
        TRACE!(
            b"Adding prop name %s, propid %d\n\0".as_ptr() as LPCSTR,
            if (*This).codePage == CP_UNICODE {
                debugstr_w(name as LPCWSTR)
            } else {
                debugstr_a(name)
            },
            id,
        );
        dictionary_insert((*This).name_to_propid, name as *const c_void, UlongToPtr(id));
        dictionary_insert((*This).propid_to_name, UlongToPtr(id), name as *mut c_void);
    }
    hr
}

stdmethod! {

unsafe fn IPropertyStorage_fnWriteMultiple(
    iface: *mut IPropertyStorage,
    cpspec: ULONG,
    rgpspec: *const PROPSPEC,
    rgpropvar: *const PROPVARIANT,
    propidNameFirst: PROPID,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!(b"(%p, %d, %p, %p)\n\0".as_ptr() as LPCSTR, iface, cpspec, rgpspec, rgpropvar);

    if cpspec != 0 && (rgpspec.is_null() || rgpropvar.is_null()) {
        return E_INVALIDARG;
    }
    if ((*This).grfMode & STGM_READWRITE) == 0 {
        return STG_E_ACCESSDENIED;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    (*This).dirty = 1u8;
    (*This).originatorOS =
        MAKELONG!(LOWORD!(GetVersion()), PROPSETHDR_OSVER_KIND_WIN32) as DWORD;

    let mut i: ULONG = 0;
    while i < cpspec {
        if (*rgpspec.add(i as usize)).ulKind == PRSPEC_LPWSTR {
            let prop = PropertyStorage_FindPropertyByName(
                This,
                (*rgpspec.add(i as usize)).__bindgen_anon_1.lpwstr,
            );

            if !prop.is_null() {
                PropVariantCopy(prop, rgpropvar.add(i as usize));
            } else {
                // Note that I don't do the special cases here that I do below,
                // because naming the special PIDs isn't supported.
                if propidNameFirst < PID_FIRST_USABLE || propidNameFirst >= PID_MIN_READONLY {
                    hr = STG_E_INVALIDPARAMETER;
                } else {
                    let nextId: PROPID = max(propidNameFirst, (*This).highestProp + 1);

                    hr = PropertyStorage_StoreNameWithId(
                        This,
                        (*rgpspec.add(i as usize)).__bindgen_anon_1.lpwstr as LPCSTR,
                        CP_UNICODE,
                        nextId,
                    );
                    if SUCCEEDED(hr) {
                        hr = PropertyStorage_StorePropWithId(
                            This,
                            nextId,
                            rgpropvar.add(i as usize),
                            GetACP(),
                        );
                    }
                }
            }
        } else {
            match (*rgpspec.add(i as usize)).__bindgen_anon_1.propid {
                PID_DICTIONARY => {
                    // Can't set the dictionary
                    hr = STG_E_INVALIDPARAMETER;
                }
                PID_CODEPAGE => {
                    // Can only set the code page if nothing else has been set
                    if dictionary_num_entries((*This).propid_to_prop) == 0
                        && (*rgpropvar.add(i as usize)).vt == VT_I2 as VARTYPE
                    {
                        (*This).codePage =
                            (*rgpropvar.add(i as usize)).__bindgen_anon_1.iVal as u16 as UINT;
                        if (*This).codePage == CP_UNICODE {
                            (*This).grfFlags &= !PROPSETFLAG_ANSI;
                        } else {
                            (*This).grfFlags |= PROPSETFLAG_ANSI;
                        }
                    } else {
                        hr = STG_E_INVALIDPARAMETER;
                    }
                }
                PID_LOCALE => {
                    // Can only set the locale if nothing else has been set
                    if dictionary_num_entries((*This).propid_to_prop) == 0
                        && (*rgpropvar.add(i as usize)).vt == VT_I4 as VARTYPE
                    {
                        (*This).locale =
                            (*rgpropvar.add(i as usize)).__bindgen_anon_1.lVal as LCID;
                    } else {
                        hr = STG_E_INVALIDPARAMETER;
                    }
                }
                PID_ILLEGAL => {
                    // silently ignore like MSDN says
                }
                _ => {
                    if (*rgpspec.add(i as usize)).__bindgen_anon_1.propid >= PID_MIN_READONLY {
                        hr = STG_E_INVALIDPARAMETER;
                    } else {
                        hr = PropertyStorage_StorePropWithId(
                            This,
                            (*rgpspec.add(i as usize)).__bindgen_anon_1.propid,
                            rgpropvar.add(i as usize),
                            GetACP(),
                        );
                    }
                }
            }
        }
        i += 1;
    }

    if ((*This).grfFlags & PROPSETFLAG_UNBUFFERED) != 0 {
        IPropertyStorage_Commit(iface, STGC_DEFAULT);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

unsafe fn IPropertyStorage_fnDeleteMultiple(
    iface: *mut IPropertyStorage,
    cpspec: ULONG,
    rgpspec: *const PROPSPEC,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p, %d, %p)\n\0".as_ptr() as LPCSTR, iface, cpspec, rgpspec);

    if cpspec != 0 && rgpspec.is_null() {
        return E_INVALIDARG;
    }
    if ((*This).grfMode & STGM_READWRITE) == 0 {
        return STG_E_ACCESSDENIED;
    }
    hr = S_OK;
    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    (*This).dirty = 1u8;
    for i in 0..cpspec {
        if (*rgpspec.add(i as usize)).ulKind == PRSPEC_LPWSTR {
            let mut propid: *mut c_void = core::ptr::null_mut();

            if dictionary_find(
                (*This).name_to_propid,
                (*rgpspec.add(i as usize)).__bindgen_anon_1.lpwstr as *const c_void,
                core::ptr::addr_of_mut!(propid),
            ) != 0
            {
                dictionary_remove((*This).propid_to_prop, propid);
            }
        } else {
            if (*rgpspec.add(i as usize)).__bindgen_anon_1.propid >= PID_FIRST_USABLE
                && (*rgpspec.add(i as usize)).__bindgen_anon_1.propid < PID_MIN_READONLY
            {
                dictionary_remove(
                    (*This).propid_to_prop,
                    UlongToPtr((*rgpspec.add(i as usize)).__bindgen_anon_1.propid),
                );
            } else {
                hr = STG_E_INVALIDPARAMETER;
            }
        }
    }
    if ((*This).grfFlags & PROPSETFLAG_UNBUFFERED) != 0 {
        IPropertyStorage_Commit(iface, STGC_DEFAULT);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

unsafe fn IPropertyStorage_fnReadPropertyNames(
    iface: *mut IPropertyStorage,
    cpropid: ULONG,
    rgpropid: *const PROPID,
    rglpwstrName: *mut LPOLESTR,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let mut hr: HRESULT = S_FALSE;

    TRACE!(
        b"(%p, %d, %p, %p)\n\0".as_ptr() as LPCSTR,
        iface,
        cpropid,
        rgpropid,
        rglpwstrName,
    );

    if cpropid != 0 && (rgpropid.is_null() || rglpwstrName.is_null()) {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));

    let mut i: ULONG = 0;
    while i < cpropid && SUCCEEDED(hr) {
        let name = PropertyStorage_FindPropertyNameById(This, *rgpropid.add(i as usize));

        if !name.is_null() {
            let len = lstrlenW(name) as usize;

            hr = S_OK;
            *rglpwstrName.add(i as usize) =
                CoTaskMemAlloc(((len + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPOLESTR;
            if !(*rglpwstrName.add(i as usize)).is_null() {
                memcpy(
                    *rglpwstrName.add(i as usize) as *mut c_void,
                    name as *const c_void,
                    (len + 1) * core::mem::size_of::<WCHAR>(),
                );
            } else {
                hr = STG_E_INSUFFICIENTMEMORY;
            }
        } else {
            *rglpwstrName.add(i as usize) = core::ptr::null_mut();
        }

        i += 1;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

unsafe fn IPropertyStorage_fnWritePropertyNames(
    iface: *mut IPropertyStorage,
    cpropid: ULONG,
    rgpropid: *const PROPID,
    rglpwstrName: *const LPOLESTR,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p, %d, %p, %p)\n\0".as_ptr() as LPCSTR, iface, cpropid, rgpropid, rglpwstrName);

    if cpropid != 0 && (rgpropid.is_null() || rglpwstrName.is_null()) {
        return E_INVALIDARG;
    }
    if ((*This).grfMode & STGM_READWRITE) == 0 {
        return STG_E_ACCESSDENIED;
    }
    hr = S_OK;
    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    (*This).dirty = 1u8;
    let mut i: ULONG = 0;
    while SUCCEEDED(hr) && i < cpropid {
        if *rgpropid.add(i as usize) != PID_ILLEGAL {
            hr = PropertyStorage_StoreNameWithId(
                This,
                *rglpwstrName.add(i as usize) as LPCSTR,
                CP_UNICODE,
                *rgpropid.add(i as usize),
            );
        }
        i += 1;
    }
    if ((*This).grfFlags & PROPSETFLAG_UNBUFFERED) != 0 {
        IPropertyStorage_Commit(iface, STGC_DEFAULT);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

unsafe fn IPropertyStorage_fnDeletePropertyNames(
    iface: *mut IPropertyStorage,
    cpropid: ULONG,
    rgpropid: *const PROPID,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let hr: HRESULT;

    TRACE!(b"(%p, %d, %p)\n\0".as_ptr() as LPCSTR, iface, cpropid, rgpropid);

    if cpropid != 0 && rgpropid.is_null() {
        return E_INVALIDARG;
    }
    if ((*This).grfMode & STGM_READWRITE) == 0 {
        return STG_E_ACCESSDENIED;
    }
    hr = S_OK;
    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    (*This).dirty = 1u8;
    for i in 0..cpropid {
        let mut name: LPWSTR = core::ptr::null_mut();

        if dictionary_find(
            (*This).propid_to_name,
            UlongToPtr(*rgpropid.add(i as usize)),
            core::ptr::addr_of_mut!(name) as *mut *mut c_void,
        ) != 0
        {
            dictionary_remove((*This).propid_to_name, UlongToPtr(*rgpropid.add(i as usize)));
            dictionary_remove((*This).name_to_propid, name as *const c_void);
        }
    }
    if ((*This).grfFlags & PROPSETFLAG_UNBUFFERED) != 0 {
        IPropertyStorage_Commit(iface, STGC_DEFAULT);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

unsafe fn IPropertyStorage_fnCommit(
    iface: *mut IPropertyStorage,
    grfCommitFlags: DWORD,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let hr: HRESULT;

    TRACE!(b"(%p, 0x%08x)\n\0".as_ptr() as LPCSTR, iface, grfCommitFlags);

    if ((*This).grfMode & STGM_READWRITE) == 0 {
        return STG_E_ACCESSDENIED;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    if (*This).dirty != 0 {
        hr = PropertyStorage_WriteToStream(This);
    } else {
        hr = S_OK;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

unsafe fn IPropertyStorage_fnRevert(iface: *mut IPropertyStorage) -> HRESULT {
    let mut hr: HRESULT;
    let This: *mut PropertyStorage_impl = impl_from_IPropertyStorage(iface);

    TRACE!(b"%p\n\0".as_ptr() as LPCSTR, iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    if (*This).dirty != 0 {
        PropertyStorage_DestroyDictionaries(This);
        hr = PropertyStorage_CreateDictionaries(This);
        if SUCCEEDED(hr) {
            hr = PropertyStorage_ReadFromStream(This);
        }
    } else {
        hr = S_OK;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    hr
}

unsafe fn IPropertyStorage_fnEnum(
    iface: *mut IPropertyStorage,
    ppenum: *mut *mut IEnumSTATPROPSTG,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    create_EnumSTATPROPSTG(This, ppenum)
}

unsafe fn IPropertyStorage_fnSetTimes(
    _iface: *mut IPropertyStorage,
    _pctime: *const FILETIME,
    _patime: *const FILETIME,
    _pmtime: *const FILETIME,
) -> HRESULT {
    FIXME!(b"\n\0".as_ptr() as LPCSTR);
    E_NOTIMPL
}

unsafe fn IPropertyStorage_fnSetClass(
    iface: *mut IPropertyStorage,
    clsid: *const CLSID,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);

    TRACE!(b"%p, %s\n\0".as_ptr() as LPCSTR, iface, debugstr_guid(clsid));

    if clsid.is_null() {
        return E_INVALIDARG;
    }
    if ((*This).grfMode & STGM_READWRITE) == 0 {
        return STG_E_ACCESSDENIED;
    }
    (*This).clsid = *clsid;
    (*This).dirty = 1u8;
    if ((*This).grfFlags & PROPSETFLAG_UNBUFFERED) != 0 {
        IPropertyStorage_Commit(iface, STGC_DEFAULT);
    }
    S_OK
}

unsafe fn IPropertyStorage_fnStat(
    iface: *mut IPropertyStorage,
    statpsstg: *mut STATPROPSETSTG,
) -> HRESULT {
    let This = impl_from_IPropertyStorage(iface);
    let mut stat: STATSTG = core::mem::zeroed();
    let hr: HRESULT;

    TRACE!(b"%p, %p\n\0".as_ptr() as LPCSTR, iface, statpsstg);

    if statpsstg.is_null() {
        return E_INVALIDARG;
    }

    hr = IStream_Stat((*This).stm, core::ptr::addr_of_mut!(stat), STATFLAG_NONAME);
    if SUCCEEDED(hr) {
        (*statpsstg).fmtid = (*This).fmtid;
        (*statpsstg).clsid = (*This).clsid;
        (*statpsstg).grfFlags = (*This).grfFlags;
        (*statpsstg).mtime = stat.mtime;
        (*statpsstg).ctime = stat.ctime;
        (*statpsstg).atime = stat.atime;
        (*statpsstg).dwOSVersion = (*This).originatorOS;
    }
    hr
}

} // stdmethod!

// ---------------------------------------------------------------------------
// dictionary callbacks — `extern "C"`, the ABI of dictionary.h's comparefunc /
// destroyfunc / enumeratefunc function-pointer typedefs.
// ---------------------------------------------------------------------------

unsafe extern "C" fn PropertyStorage_PropNameCompare(
    a: *const c_void,
    b: *const c_void,
    extra: *mut c_void,
) -> c_int {
    let This = extra as *mut PropertyStorage_impl;

    if (*This).codePage == CP_UNICODE {
        TRACE!(
            b"(%s, %s)\n\0".as_ptr() as LPCSTR,
            debugstr_w(a as LPCWSTR),
            debugstr_w(b as LPCWSTR),
        );
        if (*This).grfFlags & PROPSETFLAG_CASE_SENSITIVE != 0 {
            wcscmp(a as *const u16, b as *const u16)
        } else {
            lstrcmpiW(a as *const u16, b as *const u16)
        }
    } else {
        TRACE!(
            b"(%s, %s)\n\0".as_ptr() as LPCSTR,
            debugstr_a(a as LPCSTR),
            debugstr_a(b as LPCSTR),
        );
        if (*This).grfFlags & PROPSETFLAG_CASE_SENSITIVE != 0 {
            lstrcmpA(a as *const c_char, b as *const c_char)
        } else {
            lstrcmpiA(a as *const c_char, b as *const c_char)
        }
    }
}

unsafe extern "C" fn PropertyStorage_PropNameDestroy(
    k: *mut c_void,
    _d: *mut c_void,
    _extra: *mut c_void,
) {
    CoTaskMemFree(k);
}

unsafe extern "C" fn PropertyStorage_PropCompare(
    a: *const c_void,
    b: *const c_void,
    _extra: *mut c_void,
) -> c_int {
    TRACE!(b"(%d, %d)\n\0".as_ptr() as LPCSTR, PtrToUlong(a), PtrToUlong(b));
    // C: `return PtrToUlong(a) - PtrToUlong(b);` — unsigned wraparound, then the
    // implicit ULONG->int conversion. `wrapping_sub` reproduces it exactly (a plain
    // `-` would panic in a debug build).
    PtrToUlong(a).wrapping_sub(PtrToUlong(b)) as c_int
}

unsafe extern "C" fn PropertyStorage_PropertyDestroy(
    _k: *mut c_void,
    d: *mut c_void,
    _extra: *mut c_void,
) {
    PropVariantClear(d as *mut PROPVARIANT);
    HeapFree(GetProcessHeap(), 0, d);
}

/// stg_prop.c:975 — a no-op on little-endian (`#define PropertyStorage_ByteSwapString(s,l)`);
/// `lendian16toh` is the identity on all three of our LE targets.
unsafe fn PropertyStorage_ByteSwapString(str: LPWSTR, len: usize) {
    for i in 0..len {
        *str.add(i) = lendian16toh(*str.add(i));
    }
}

// ---- wine-10.0 bounded reads (upstream 5f6b295cb90) ----
//
// The old translation read property sets through a bare pointer: the dictionary wrote
// `*ptr.add(cbEntry - 1) = 0` with no bound (a hostile or truncated stream corrupted the
// heap), and every property read could run past the section. wine-10 threads a
// `read_buffer { data, size }` through every read and fails STG_E_READFAULT instead.

/// wine-10 `struct read_buffer`.
#[repr(C)]
struct read_buffer {
    data: *mut BYTE,
    size: usize,
}

/// `((len + align) & ~align)` -- stg_prop.c's ALIGNED_LENGTH.
#[inline]
fn ALIGNED_LENGTH(len: usize, align: usize) -> usize {
    (len.wrapping_add(align)) & !align
}

unsafe fn buffer_test_offset(buffer: *const read_buffer, offset: usize, len: usize) -> HRESULT {
    if len > (*buffer).size || offset > (*buffer).size - len {
        STG_E_READFAULT
    } else {
        S_OK
    }
}

unsafe fn buffer_read_uint64(
    buffer: *const read_buffer,
    offset: usize,
    data: *mut ULARGE_INTEGER,
) -> HRESULT {
    let hr = buffer_test_offset(buffer, offset, core::mem::size_of::<ULARGE_INTEGER>());
    if SUCCEEDED(hr) {
        StorageUtl_ReadULargeInteger((*buffer).data.add(offset), 0, data);
    }
    hr
}

unsafe fn buffer_read_dword(buffer: *const read_buffer, offset: usize, data: *mut DWORD) -> HRESULT {
    let hr = buffer_test_offset(buffer, offset, core::mem::size_of::<DWORD>());
    if SUCCEEDED(hr) {
        StorageUtl_ReadDWord((*buffer).data.add(offset), 0, data);
    }
    hr
}

unsafe fn buffer_read_word(buffer: *const read_buffer, offset: usize, data: *mut WORD) -> HRESULT {
    let hr = buffer_test_offset(buffer, offset, core::mem::size_of::<WORD>());
    if SUCCEEDED(hr) {
        StorageUtl_ReadWord((*buffer).data.add(offset), 0, data);
    }
    hr
}

unsafe fn buffer_read_byte(buffer: *const read_buffer, offset: usize, data: *mut BYTE) -> HRESULT {
    let hr = buffer_test_offset(buffer, offset, core::mem::size_of::<BYTE>());
    if SUCCEEDED(hr) {
        *data = *(*buffer).data.add(offset);
    }
    hr
}

unsafe fn buffer_read_len(
    buffer: *const read_buffer,
    offset: usize,
    dest: *mut c_void,
    len: usize,
) -> HRESULT {
    let hr = buffer_test_offset(buffer, offset, len);
    if SUCCEEDED(hr) {
        core::ptr::copy_nonoverlapping((*buffer).data.add(offset), dest as *mut u8, len);
    }
    hr
}

/// Reads the dictionary from `buffer` at `offset`, interpreting the entries according to
/// This->codePage and This->locale (wine-10.0).
unsafe fn PropertyStorage_ReadDictionary(
    This: *mut PropertyStorage_impl,
    buffer: *const read_buffer,
    mut offset: usize,
) -> HRESULT {
    let mut numEntries: DWORD = 0;

    assert(!(*This).name_to_propid.is_null());
    assert(!(*This).propid_to_name.is_null());

    let mut hr = buffer_read_dword(buffer, offset, &mut numEntries);
    if FAILED(hr) {
        return hr;
    }
    TRACE!(b"Reading %d entries:\n\0".as_ptr() as LPCSTR, numEntries);
    offset += core::mem::size_of::<DWORD>();

    let mut i: DWORD = 0;
    while SUCCEEDED(hr) && i < numEntries {
        let mut propid: PROPID = 0;
        let mut cbEntry: DWORD = 0;
        let mut ch: WCHAR = 0;

        hr = buffer_read_dword(buffer, offset, &mut propid);
        if SUCCEEDED(hr) {
            hr = buffer_read_dword(buffer, offset + core::mem::size_of::<PROPID>(), &mut cbEntry);
        }
        if FAILED(hr) {
            break;
        }

        offset += core::mem::size_of::<PROPID>() + core::mem::size_of::<DWORD>();

        let span = if (*This).codePage == CP_UNICODE {
            ALIGNED_LENGTH(cbEntry as usize * core::mem::size_of::<WCHAR>(), 3)
        } else {
            cbEntry as usize
        };
        hr = buffer_test_offset(buffer, offset, span);
        if FAILED(hr) {
            WARN!(b"Broken name length for entry %d.\n\0".as_ptr() as LPCSTR, i);
            return hr;
        }

        // Make sure the source string is NULL-terminated -- CHECKED, not forced: wine-10
        // rejects an unterminated name instead of writing a NUL into the caller's data.
        // (C size_t arithmetic: cbEntry == 0 wraps to a probe that fails its bounds test,
        // leaving ch == 0.)
        if (*This).codePage != CP_UNICODE {
            let mut b: BYTE = 0;
            buffer_read_byte(buffer, offset.wrapping_add(cbEntry as usize).wrapping_sub(1), &mut b);
            ch = b as WCHAR;
        } else {
            buffer_read_word(
                buffer,
                offset.wrapping_add(
                    (cbEntry as usize).wrapping_sub(1).wrapping_mul(core::mem::size_of::<WCHAR>()),
                ),
                &mut ch,
            );
        }

        if ch != 0 {
            WARN!(b"Dictionary entry name %d is not null-terminated.\n\0".as_ptr() as LPCSTR, i);
            return E_FAIL;
        }

        TRACE!(
            b"Reading entry with ID 0x%08x, %d chars\n\0".as_ptr() as LPCSTR,
            propid,
            cbEntry,
        );

        hr = PropertyStorage_StoreNameWithId(
            This,
            (*buffer).data.add(offset) as LPCSTR,
            (*This).codePage,
            propid,
        );

        // Unicode entries are padded to DWORD boundaries
        let advance = if (*This).codePage == CP_UNICODE {
            ALIGNED_LENGTH(cbEntry as usize * core::mem::size_of::<WCHAR>(), 3)
        } else {
            cbEntry as usize
        };
        offset += advance;

        i += 1;
    }

    hr
}

// `__thiscall_wrapper` in the C: `__stdcall` on i386, `__cdecl` elsewhere — which is
// precisely what rustc's `extern "system"` lowers to per target.
unsafe extern "system" fn Allocate_CoTaskMemAlloc(
    _this: *mut c_void,
    size: ULONG,
) -> *mut c_void {
    CoTaskMemAlloc(size as SIZE_T)
}

type PropAllocator = unsafe extern "system" fn(*mut c_void, ULONG) -> *mut c_void;

/// wine-10 `propertystorage_read_scalar`: one non-vector value of `(*prop).vt` at `offset`.
unsafe fn propertystorage_read_scalar(
    prop: *mut PROPVARIANT,
    buffer: *const read_buffer,
    mut offset: usize,
    codepage: UINT,
    allocate: PropAllocator,
    allocate_data: *mut c_void,
) -> HRESULT {
    let u = core::ptr::addr_of_mut!((*prop).__bindgen_anon_1);
    let mut hr: HRESULT;

    assert(((*prop).vt as DWORD & (VT_ARRAY | VT_VECTOR)) == 0);

    match (*prop).vt as DWORD {
        VT_EMPTY | VT_NULL => hr = S_OK,
        VT_I1 => {
            hr = buffer_read_byte(buffer, offset, core::ptr::addr_of_mut!((*u).cVal) as *mut BYTE);
        }
        VT_UI1 => hr = buffer_read_byte(buffer, offset, core::ptr::addr_of_mut!((*u).bVal) as *mut BYTE),
        VT_BOOL => {
            hr = buffer_read_word(buffer, offset, core::ptr::addr_of_mut!((*u).boolVal) as *mut WORD);
        }
        VT_I2 => hr = buffer_read_word(buffer, offset, core::ptr::addr_of_mut!((*u).iVal) as *mut WORD),
        VT_UI2 => hr = buffer_read_word(buffer, offset, core::ptr::addr_of_mut!((*u).uiVal) as *mut WORD),
        VT_INT | VT_I4 => {
            hr = buffer_read_dword(buffer, offset, core::ptr::addr_of_mut!((*u).lVal) as *mut DWORD);
        }
        VT_UINT | VT_UI4 => {
            hr = buffer_read_dword(buffer, offset, core::ptr::addr_of_mut!((*u).ulVal) as *mut DWORD);
        }
        VT_I8 => {
            hr = buffer_read_uint64(
                buffer,
                offset,
                core::ptr::addr_of_mut!((*u).hVal) as *mut ULARGE_INTEGER,
            );
        }
        VT_UI8 => {
            hr = buffer_read_uint64(
                buffer,
                offset,
                core::ptr::addr_of_mut!((*u).uhVal) as *mut ULARGE_INTEGER,
            );
        }
        VT_R8 => {
            hr = buffer_read_len(
                buffer,
                offset,
                core::ptr::addr_of_mut!((*u).dblVal) as *mut c_void,
                core::mem::size_of::<f64>(),
            );
        }
        VT_LPSTR => {
            let mut count: DWORD = 0;

            hr = buffer_read_dword(buffer, offset, &mut count);
            if FAILED(hr) {
                return hr;
            }
            offset += core::mem::size_of::<DWORD>();

            if codepage == CP_UNICODE && count % core::mem::size_of::<WCHAR>() as DWORD != 0 {
                WARN!(b"Unicode string has odd number of bytes\n\0".as_ptr() as LPCSTR);
                hr = STG_E_INVALIDHEADER;
            } else {
                (*u).pszVal = allocate(allocate_data, count) as LPSTR;
                if !(*u).pszVal.is_null() {
                    hr = buffer_read_len(buffer, offset, (*u).pszVal as *mut c_void, count as usize);
                    if FAILED(hr) {
                        return hr;
                    }

                    // This is stored in the code page specified in codepage. Don't convert
                    // it, the caller will just store it as-is. Make sure it's
                    // NULL-terminated.
                    if codepage == CP_UNICODE {
                        *((*u).pszVal as *mut WCHAR)
                            .add((count as usize / core::mem::size_of::<WCHAR>()).wrapping_sub(1)) = 0;
                    } else {
                        *(*u).pszVal.add((count as usize).wrapping_sub(1)) = 0;
                    }
                } else {
                    hr = STG_E_INSUFFICIENTMEMORY;
                }
            }
        }
        VT_BSTR => {
            let mut count: DWORD = 0;
            let wcount: DWORD;

            hr = buffer_read_dword(buffer, offset, &mut count);
            if FAILED(hr) {
                return hr;
            }
            offset += core::mem::size_of::<DWORD>();

            if codepage == CP_UNICODE && count % core::mem::size_of::<WCHAR>() as DWORD != 0 {
                WARN!(b"Unicode string has odd number of bytes\n\0".as_ptr() as LPCSTR);
                hr = STG_E_INVALIDHEADER;
            } else {
                if codepage == CP_UNICODE {
                    wcount = count / core::mem::size_of::<WCHAR>() as DWORD;
                } else {
                    hr = buffer_test_offset(buffer, offset, count as usize);
                    if FAILED(hr) {
                        return hr;
                    }
                    wcount = MultiByteToWideChar(
                        codepage,
                        0,
                        (*buffer).data.add(offset) as LPCSTR,
                        count as c_int,
                        core::ptr::null_mut(),
                        0,
                    ) as DWORD;
                }

                // FIXME: use allocator?
                (*u).bstrVal = SysAllocStringLen(core::ptr::null(), wcount as UINT);
                if !(*u).bstrVal.is_null() {
                    if codepage == CP_UNICODE {
                        hr = buffer_read_len(buffer, offset, (*u).bstrVal as *mut c_void, count as usize);
                    } else {
                        MultiByteToWideChar(
                            codepage,
                            0,
                            (*buffer).data.add(offset) as LPCSTR,
                            count as c_int,
                            (*u).bstrVal,
                            wcount as c_int,
                        );
                    }
                    *(*u).bstrVal.add((wcount as usize).wrapping_sub(1)) = 0;
                } else {
                    hr = STG_E_INSUFFICIENTMEMORY;
                }
            }
        }
        VT_BLOB => {
            let mut count: DWORD = 0;

            hr = buffer_read_dword(buffer, offset, &mut count);
            if FAILED(hr) {
                return hr;
            }
            offset += core::mem::size_of::<DWORD>();
            (*u).blob.cbSize = count;
            (*u).blob.pBlobData = allocate(allocate_data, count) as *mut BYTE;
            if !(*u).blob.pBlobData.is_null() {
                hr = buffer_read_len(buffer, offset, (*u).blob.pBlobData as *mut c_void, count as usize);
            } else {
                hr = STG_E_INSUFFICIENTMEMORY;
            }
        }
        VT_LPWSTR => {
            let mut count: DWORD = 0;

            hr = buffer_read_dword(buffer, offset, &mut count);
            if FAILED(hr) {
                return hr;
            }
            offset += core::mem::size_of::<DWORD>();
            (*u).pwszVal =
                allocate(allocate_data, count * core::mem::size_of::<WCHAR>() as ULONG) as LPWSTR;
            if !(*u).pwszVal.is_null() {
                hr = buffer_read_len(
                    buffer,
                    offset,
                    (*u).pwszVal as *mut c_void,
                    count as usize * core::mem::size_of::<WCHAR>(),
                );
                if SUCCEEDED(hr) {
                    // make sure string is NULL-terminated
                    *(*u).pwszVal.add((count as usize).wrapping_sub(1)) = 0;
                    PropertyStorage_ByteSwapString((*u).pwszVal, count as usize);
                }
            } else {
                hr = STG_E_INSUFFICIENTMEMORY;
            }
        }
        VT_FILETIME => {
            hr = buffer_read_uint64(
                buffer,
                offset,
                core::ptr::addr_of_mut!((*u).filetime) as *mut ULARGE_INTEGER,
            );
        }
        VT_CF => {
            let mut len: DWORD = 0;
            let mut tag: DWORD = 0;

            hr = buffer_read_dword(buffer, offset, &mut len);
            if SUCCEEDED(hr) {
                hr = buffer_read_dword(buffer, offset + core::mem::size_of::<DWORD>(), &mut tag);
            }
            if FAILED(hr) {
                return hr;
            }
            offset += 2 * core::mem::size_of::<DWORD>();

            if len > 8 {
                len -= 8;
                (*u).pclipdata =
                    allocate(allocate_data, core::mem::size_of::<CLIPDATA>() as ULONG) as *mut CLIPDATA;
                (*(*u).pclipdata).cbSize = len;
                (*(*u).pclipdata).ulClipFmt = tag as LONG;
                // C: `len - sizeof(prop->pclipdata->ulClipFmt)` -- ulClipFmt is a LONG.
                let clip_len = len - core::mem::size_of::<LONG>() as DWORD;
                (*(*u).pclipdata).pClipData = allocate(allocate_data, clip_len) as *mut BYTE;
                hr = buffer_read_len(
                    buffer,
                    offset,
                    (*(*u).pclipdata).pClipData as *mut c_void,
                    clip_len as usize,
                );
            } else {
                hr = STG_E_INVALIDPARAMETER;
            }
        }
        VT_CLSID => {
            // wine-10: new. The old translation fell to the default arm, so a VT_CLSID
            // property this crate's own writer emits was dropped on reload.
            (*u).puuid = allocate(allocate_data, core::mem::size_of::<CLSID>() as ULONG) as *mut CLSID;
            if (*u).puuid.is_null() {
                return STG_E_INSUFFICIENTMEMORY;
            }
            hr = buffer_test_offset(buffer, offset, core::mem::size_of::<CLSID>());
            if SUCCEEDED(hr) {
                StorageUtl_ReadGUID((*buffer).data.add(offset), 0, (*u).puuid);
            }
        }
        _ => {
            FIXME!(b"unsupported type %d\n\0".as_ptr() as LPCSTR, (*prop).vt as core::ffi::c_uint);
            hr = STG_E_INVALIDPARAMETER;
        }
    }

    hr
}

/// wine-10 `propertystorage_get_elemsize`: the element size of a fixed-size VT_VECTOR.
unsafe fn propertystorage_get_elemsize(prop: *const PROPVARIANT) -> usize {
    if ((*prop).vt as DWORD & VT_VECTOR) == 0 {
        return 0;
    }

    match (*prop).vt as DWORD & !VT_VECTOR {
        VT_I1 | VT_UI1 => 1,
        VT_I2 | VT_UI2 | VT_BOOL => 2,
        VT_I4 | VT_UI4 | VT_R4 | VT_ERROR => 4,
        VT_I8 | VT_UI8 | VT_R8 | VT_CY | VT_DATE | VT_FILETIME => 8,
        VT_CLSID => core::mem::size_of::<CLSID>(),
        VT_VARIANT => core::mem::size_of::<PROPVARIANT>(),
        _ => {
            FIXME!(b"Unhandled type %#x.\n\0".as_ptr() as LPCSTR, (*prop).vt as core::ffi::c_uint);
            0
        }
    }
}

/// wine-10 `PropertyStorage_ReadProperty`: a type DWORD at `offset`, then the value --
/// a scalar, or a VT_VECTOR of fixed-size scalars (variable-length vectors and VT_ARRAY
/// stay unsupported, as in Wine).
unsafe fn PropertyStorage_ReadProperty(
    prop: *mut PROPVARIANT,
    buffer: *const read_buffer,
    mut offset: usize,
    codepage: UINT,
    allocate: PropAllocator,
    allocate_data: *mut c_void,
) -> HRESULT {
    let mut vt: DWORD = 0;

    assert(!prop.is_null());
    assert(!(*buffer).data.is_null());

    let mut hr = buffer_read_dword(buffer, offset, &mut vt);
    if FAILED(hr) {
        return hr;
    }

    offset += core::mem::size_of::<DWORD>();
    (*prop).vt = vt as VARTYPE;

    if ((*prop).vt as DWORD & VT_VECTOR) != 0 {
        let mut count: DWORD = 0;

        match (*prop).vt as DWORD & !VT_VECTOR {
            VT_BSTR | VT_VARIANT | VT_LPSTR | VT_LPWSTR | VT_CF => {
                FIXME!(b"Vector with variable length elements are not supported.\n\0".as_ptr() as LPCSTR);
                return STG_E_INVALIDPARAMETER;
            }
            _ => {}
        }

        hr = buffer_read_dword(buffer, offset, &mut count);
        if SUCCEEDED(hr) {
            let elemsize = propertystorage_get_elemsize(prop);
            let mut elem: PROPVARIANT = core::mem::zeroed();

            offset += core::mem::size_of::<DWORD>();

            let elems = allocate(allocate_data, (elemsize * count as usize) as ULONG) as *mut u8;
            (*prop).__bindgen_anon_1.capropvar.pElems = elems as *mut PROPVARIANT;
            if !elems.is_null() {
                (*prop).__bindgen_anon_1.capropvar.cElems = count;
                elem.vt = (*prop).vt & !(VT_VECTOR as VARTYPE);

                for i in 0..count as usize {
                    hr = propertystorage_read_scalar(
                        &mut elem,
                        buffer,
                        offset + i * elemsize,
                        codepage,
                        allocate,
                        allocate_data,
                    );
                    if SUCCEEDED(hr) {
                        // Divergence from wine-10, whose `memcpy(&prop->capropvar.pElems[i],
                        // &elem.lVal, elemsize)` indexes pElems as a PROPVARIANT* -- a stride
                        // of sizeof(PROPVARIANT) into a buffer allocated elemsize * count, so
                        // every fixed-size vector of 2+ elements narrower than a PROPVARIANT
                        // overran the heap. The element arrays are typed per member (CAL is
                        // LONG*), so element i lives at pElems + i * elemsize.
                        core::ptr::copy_nonoverlapping(
                            core::ptr::addr_of!(elem.__bindgen_anon_1) as *const u8,
                            elems.add(i * elemsize),
                            elemsize,
                        );
                    }
                }
            } else {
                hr = STG_E_INSUFFICIENTMEMORY;
            }
        }
    } else if ((*prop).vt as DWORD & VT_ARRAY) != 0 {
        FIXME!(b"VT_ARRAY properties are not supported.\n\0".as_ptr() as LPCSTR);
        hr = STG_E_INVALIDPARAMETER;
    } else {
        hr = propertystorage_read_scalar(prop, buffer, offset, codepage, allocate, allocate_data);
    }

    hr
}

unsafe fn PropertyStorage_ReadHeaderFromStream(
    stm: *mut IStream,
    hdr: *mut PROPERTYSETHEADER,
) -> HRESULT {
    let mut buf: [u8; core::mem::size_of::<PROPERTYSETHEADER>()] =
        [0; core::mem::size_of::<PROPERTYSETHEADER>()];
    let mut count: ULONG = 0;
    let mut hr: HRESULT;

    assert(!stm.is_null());
    assert(!hdr.is_null());

    hr = IStream_Read(
        stm,
        buf.as_mut_ptr() as *mut c_void,
        core::mem::size_of_val(&buf) as ULONG,
        core::ptr::addr_of_mut!(count),
    );

    if SUCCEEDED(hr) {
        if count != core::mem::size_of_val(&buf) as ULONG {
            WARN!(b"read only %d\n\0".as_ptr() as LPCSTR, count);
            hr = STG_E_INVALIDHEADER;
        } else {
            StorageUtl_ReadWord(
                buf.as_ptr(),
                core::mem::offset_of!(PROPERTYSETHEADER, wByteOrder) as ULONG,
                core::ptr::addr_of_mut!((*hdr).wByteOrder),
            );
            StorageUtl_ReadWord(
                buf.as_ptr(),
                core::mem::offset_of!(PROPERTYSETHEADER, wFormat) as ULONG,
                core::ptr::addr_of_mut!((*hdr).wFormat),
            );
            StorageUtl_ReadDWord(
                buf.as_ptr(),
                core::mem::offset_of!(PROPERTYSETHEADER, dwOSVer) as ULONG,
                core::ptr::addr_of_mut!((*hdr).dwOSVer),
            );
            StorageUtl_ReadGUID(
                buf.as_ptr(),
                core::mem::offset_of!(PROPERTYSETHEADER, clsid) as ULONG,
                core::ptr::addr_of_mut!((*hdr).clsid),
            );
            StorageUtl_ReadDWord(
                buf.as_ptr(),
                core::mem::offset_of!(PROPERTYSETHEADER, reserved) as ULONG,
                core::ptr::addr_of_mut!((*hdr).reserved),
            );
        }
    }

    TRACE!(b"returning 0x%08x\n\0".as_ptr() as LPCSTR, hr);
    hr
}

unsafe fn PropertyStorage_ReadFmtIdOffsetFromStream(
    stm: *mut IStream,
    fmt: *mut FORMATIDOFFSET,
) -> HRESULT {
    let mut buf: [u8; core::mem::size_of::<FORMATIDOFFSET>()] =
        [0; core::mem::size_of::<FORMATIDOFFSET>()];
    let mut count: ULONG = 0;
    let mut hr: HRESULT;

    assert(!stm.is_null());
    assert(!fmt.is_null());

    hr = IStream_Read(
        stm,
        buf.as_mut_ptr() as *mut c_void,
        core::mem::size_of_val(&buf) as ULONG,
        core::ptr::addr_of_mut!(count),
    );

    if SUCCEEDED(hr) {
        if count != core::mem::size_of_val(&buf) as ULONG {
            WARN!(b"read only %d\n\0".as_ptr() as LPCSTR, count);
            hr = STG_E_INVALIDHEADER;
        } else {
            StorageUtl_ReadGUID(
                buf.as_ptr(),
                core::mem::offset_of!(FORMATIDOFFSET, fmtid) as ULONG,
                core::ptr::addr_of_mut!((*fmt).fmtid),
            );
            StorageUtl_ReadDWord(
                buf.as_ptr(),
                core::mem::offset_of!(FORMATIDOFFSET, dwOffset) as ULONG,
                core::ptr::addr_of_mut!((*fmt).dwOffset),
            );
        }
    }

    TRACE!(b"returning 0x%08x\n\0".as_ptr() as LPCSTR, hr);
    hr
}

unsafe fn PropertyStorage_ReadSectionHeaderFromStream(
    stm: *mut IStream,
    hdr: *mut PROPERTYSECTIONHEADER,
) -> HRESULT {
    let mut buf: [u8; core::mem::size_of::<PROPERTYSECTIONHEADER>()] =
        [0; core::mem::size_of::<PROPERTYSECTIONHEADER>()];
    let mut count: ULONG = 0;
    let mut hr: HRESULT;

    assert(!stm.is_null());
    assert(!hdr.is_null());
    hr = IStream_Read(
        stm,
        buf.as_mut_ptr() as *mut c_void,
        core::mem::size_of_val(&buf) as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if SUCCEEDED(hr) {
        if count != core::mem::size_of_val(&buf) as ULONG {
            WARN!(b"read only %d\n\0".as_ptr() as LPCSTR, count);
            hr = STG_E_INVALIDHEADER;
        } else {
            StorageUtl_ReadDWord(
                buf.as_ptr(),
                core::mem::offset_of!(PROPERTYSECTIONHEADER, cbSection) as ULONG,
                core::ptr::addr_of_mut!((*hdr).cbSection),
            );
            StorageUtl_ReadDWord(
                buf.as_ptr(),
                core::mem::offset_of!(PROPERTYSECTIONHEADER, cProperties) as ULONG,
                core::ptr::addr_of_mut!((*hdr).cProperties),
            );
        }
    }
    TRACE!(b"returning 0x%08x\n\0".as_ptr() as LPCSTR, hr);
    hr
}

unsafe fn PropertyStorage_ReadFromStream(This: *mut PropertyStorage_impl) -> HRESULT {
    let mut hdr: PROPERTYSETHEADER = core::mem::zeroed();
    let mut fmtOffset: FORMATIDOFFSET = core::mem::zeroed();
    let mut sectionHdr: PROPERTYSECTIONHEADER = core::mem::zeroed();
    let mut seek: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG;
    let mut stat: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut buf: *mut BYTE = core::ptr::null_mut();
    let mut read_buf = read_buffer { data: core::ptr::null_mut(), size: 0 };
    let mut count: ULONG = 0;
    let mut dictOffset: DWORD = 0;

    (*This).dirty = 0u8;
    (*This).highestProp = 0;

    // The C's `goto end` chain, spelled as a labeled block (the exit path below the
    // block is the C's `end:` label).
    'end: {
        hr = IStream_Stat((*This).stm, core::ptr::addr_of_mut!(stat), STATFLAG_NONAME);
        if FAILED(hr) {
            break 'end;
        }
        if stat.cbSize.u.HighPart != 0 {
            WARN!(b"stream too big\n\0".as_ptr() as LPCSTR);
            // maximum size varies, but it can't be this big
            hr = STG_E_INVALIDHEADER;
            break 'end;
        }
        if stat.cbSize.u.LowPart == 0 {
            // empty stream is okay
            hr = S_OK;
            break 'end;
        } else if (stat.cbSize.u.LowPart as usize)
            < core::mem::size_of::<PROPERTYSETHEADER>() + core::mem::size_of::<FORMATIDOFFSET>()
        {
            WARN!(b"stream too small\n\0".as_ptr() as LPCSTR);
            hr = STG_E_INVALIDHEADER;
            break 'end;
        }
        seek.QuadPart = 0;
        hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
        if FAILED(hr) {
            break 'end;
        }
        // UPSTREAM QUIRK (stg_prop.c:1380): the C does NOT check this hr — a failed
        // header read just leaves `hdr` zeroed, and the magic check below then remaps
        // the failure to STG_E_INVALIDHEADER. Kept as-is (hence the "never read" warn).
        PropertyStorage_ReadHeaderFromStream((*This).stm, core::ptr::addr_of_mut!(hdr));
        // I've only seen reserved == 1, but the article says I shouldn't disallow
        // higher values.
        if hdr.wByteOrder != PROPSETHDR_BYTEORDER_MAGIC || hdr.reserved < 1 {
            WARN!(b"bad magic in prop set header\n\0".as_ptr() as LPCSTR);
            hr = STG_E_INVALIDHEADER;
            break 'end;
        }
        if hdr.wFormat != 0 && hdr.wFormat != 1 {
            WARN!(b"bad format version %d\n\0".as_ptr() as LPCSTR, hdr.wFormat);
            hr = STG_E_INVALIDHEADER;
            break 'end;
        }
        (*This).format = hdr.wFormat;
        (*This).clsid = hdr.clsid;
        (*This).originatorOS = hdr.dwOSVer;
        if PROPSETHDR_OSVER_KIND(hdr.dwOSVer) == PROPSETHDR_OSVER_KIND_MAC {
            WARN!(
                b"File comes from a Mac, strings will probably be screwed up\n\0".as_ptr()
                    as LPCSTR
            );
        }
        hr = PropertyStorage_ReadFmtIdOffsetFromStream(
            (*This).stm,
            core::ptr::addr_of_mut!(fmtOffset),
        );
        if FAILED(hr) {
            break 'end;
        }
        if fmtOffset.dwOffset > stat.cbSize.u.LowPart {
            WARN!(
                b"invalid offset %d (stream length is %d)\n\0".as_ptr() as LPCSTR,
                fmtOffset.dwOffset,
                stat.cbSize.u.LowPart,
            );
            hr = STG_E_INVALIDHEADER;
            break 'end;
        }
        // wine-10: seek to the section the format-ID entry points at. The old code read
        // the section header from wherever the FMTIDOFFSET read left the stream, which is
        // only right when the section directly follows -- not for DocSummaryInformation
        // (two FMTIDOFFSETs), where cbSection came back as garbage and the allocation
        // below failed STG_E_INSUFFICIENTMEMORY.
        seek.QuadPart = fmtOffset.dwOffset as LONGLONG;
        hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
        if FAILED(hr) {
            break 'end;
        }
        // wackiness alert: if the format ID is FMTID_DocSummaryInformation, there
        // follows not one, but two sections.  Reading the second isn't implemented yet.
        hr = PropertyStorage_ReadSectionHeaderFromStream(
            (*This).stm,
            core::ptr::addr_of_mut!(sectionHdr),
        );
        if FAILED(hr) {
            break 'end;
        }
        // The section size includes the section header, so check it
        if (sectionHdr.cbSection as usize) < core::mem::size_of::<PROPERTYSECTIONHEADER>() {
            WARN!(
                b"section header too small, got %d\n\0".as_ptr() as LPCSTR,
                sectionHdr.cbSection,
            );
            hr = STG_E_INVALIDHEADER;
            break 'end;
        }
        buf = HeapAlloc(
            GetProcessHeap(),
            0,
            (sectionHdr.cbSection - core::mem::size_of::<PROPERTYSECTIONHEADER>() as DWORD)
                as SIZE_T,
        ) as *mut BYTE;
        if buf.is_null() {
            hr = STG_E_INSUFFICIENTMEMORY;
            break 'end;
        }
        read_buf.data = buf;
        read_buf.size =
            (sectionHdr.cbSection as usize) - core::mem::size_of::<PROPERTYSECTIONHEADER>();
        hr = IStream_Read(
            (*This).stm,
            buf as *mut c_void,
            read_buf.size as ULONG,
            core::ptr::addr_of_mut!(count),
        );
        if FAILED(hr) {
            break 'end;
        }
        TRACE!(b"Reading %d properties:\n\0".as_ptr() as LPCSTR, sectionHdr.cProperties);
        i = 0;
        while SUCCEEDED(hr) && i < sectionHdr.cProperties {
            let idOffset = buf.add(i as usize * core::mem::size_of::<PROPERTYIDOFFSET>())
                as *mut PROPERTYIDOFFSET;

            if (*idOffset).dwOffset < core::mem::size_of::<PROPERTYSECTIONHEADER>() as DWORD
                || (*idOffset).dwOffset
                    > sectionHdr.cbSection - core::mem::size_of::<DWORD>() as DWORD
            {
                hr = STG_E_INVALIDPOINTER;
            } else {
                if (*idOffset).propid >= PID_FIRST_USABLE
                    && (*idOffset).propid < PID_MIN_READONLY
                    && (*idOffset).propid > (*This).highestProp
                {
                    (*This).highestProp = (*idOffset).propid;
                }
                if (*idOffset).propid == PID_DICTIONARY {
                    // Don't read the dictionary yet, its entries depend on the code
                    // page. Just store the offset so we know to read it later.
                    dictOffset = (*idOffset).dwOffset;
                    TRACE!(b"Dictionary offset is %d\n\0".as_ptr() as LPCSTR, dictOffset);
                } else {
                    let mut prop: PROPVARIANT = core::mem::zeroed();

                    PropVariantInit(core::ptr::addr_of_mut!(prop));
                    if SUCCEEDED(PropertyStorage_ReadProperty(
                        core::ptr::addr_of_mut!(prop),
                        &read_buf,
                        ((*idOffset).dwOffset
                            - core::mem::size_of::<PROPERTYSECTIONHEADER>() as DWORD)
                            as usize,
                        (*This).codePage,
                        Allocate_CoTaskMemAlloc,
                        core::ptr::null_mut(),
                    )) {
                        TRACE!(
                            b"Read property with ID 0x%08x, type %d\n\0".as_ptr() as LPCSTR,
                            (*idOffset).propid,
                            prop.vt,
                        );
                        match (*idOffset).propid {
                            PID_CODEPAGE => {
                                if prop.vt == VT_I2 as VARTYPE {
                                    // (USHORT) first, as wine-10: a negative i16 (CP_UTF8 = 65001)
                                    // must not sign-extend to 0xFFFFFDE9.
                                    (*This).codePage = prop.__bindgen_anon_1.iVal as u16 as UINT;
                                }
                            }
                            PID_LOCALE => {
                                if prop.vt == VT_I4 as VARTYPE {
                                    (*This).locale = prop.__bindgen_anon_1.lVal as LCID;
                                }
                            }
                            PID_BEHAVIOR => {
                                if prop.vt == VT_I4 as VARTYPE
                                    && prop.__bindgen_anon_1.lVal != 0
                                {
                                    (*This).grfFlags |= PROPSETFLAG_CASE_SENSITIVE;
                                }
                                // The format should already be 1, but just in case
                                (*This).format = 1;
                            }
                            _ => {
                                hr = PropertyStorage_StorePropWithId(
                                    This,
                                    (*idOffset).propid,
                                    core::ptr::addr_of!(prop),
                                    (*This).codePage,
                                );
                            }
                        }
                    }
                    PropVariantClear(core::ptr::addr_of_mut!(prop));
                }
            }
            i += 1;
        }
        if (*This).codePage == 0 {
            // default to Unicode unless told not to, as specified on msdn
            if (*This).grfFlags & PROPSETFLAG_ANSI != 0 {
                (*This).codePage = GetACP();
            } else {
                (*This).codePage = CP_UNICODE;
            }
        }
        if (*This).locale == 0 {
            (*This).locale = LOCALE_SYSTEM_DEFAULT;
        }
        TRACE!(
            b"Code page is %d, locale is %d\n\0".as_ptr() as LPCSTR,
            (*This).codePage,
            (*This).locale,
        );
        if dictOffset != 0 {
            hr = PropertyStorage_ReadDictionary(
                This,
                &read_buf,
                (dictOffset - core::mem::size_of::<PROPERTYSECTIONHEADER>() as DWORD) as usize,
            );
        }
    }

    // end:
    HeapFree(GetProcessHeap(), 0, buf as *mut c_void);
    if FAILED(hr) {
        dictionary_destroy((*This).name_to_propid);
        (*This).name_to_propid = core::ptr::null_mut();
        dictionary_destroy((*This).propid_to_name);
        (*This).propid_to_name = core::ptr::null_mut();
        dictionary_destroy((*This).propid_to_prop);
        (*This).propid_to_prop = core::ptr::null_mut();
    }
    hr
}

unsafe fn PropertyStorage_MakeHeader(
    This: *mut PropertyStorage_impl,
    hdr: *mut PROPERTYSETHEADER,
) {
    assert(!hdr.is_null());
    StorageUtl_WriteWord(
        core::ptr::addr_of_mut!((*hdr).wByteOrder) as LPBYTE,
        0,
        PROPSETHDR_BYTEORDER_MAGIC,
    );
    StorageUtl_WriteWord(
        core::ptr::addr_of_mut!((*hdr).wFormat) as LPBYTE,
        0,
        (*This).format,
    );
    StorageUtl_WriteDWord(
        core::ptr::addr_of_mut!((*hdr).dwOSVer) as LPBYTE,
        0,
        (*This).originatorOS,
    );
    StorageUtl_WriteGUID(
        core::ptr::addr_of_mut!((*hdr).clsid) as LPBYTE,
        0,
        core::ptr::addr_of!((*This).clsid),
    );
    StorageUtl_WriteDWord(
        core::ptr::addr_of_mut!((*hdr).reserved) as LPBYTE,
        0,
        1,
    );
}

unsafe fn PropertyStorage_MakeFmtIdOffset(
    This: *mut PropertyStorage_impl,
    fmtOffset: *mut FORMATIDOFFSET,
) {
    assert(!fmtOffset.is_null());
    StorageUtl_WriteGUID(
        fmtOffset as LPBYTE,
        0,
        core::ptr::addr_of!((*This).fmtid),
    );
    StorageUtl_WriteDWord(
        fmtOffset as LPBYTE,
        core::mem::offset_of!(FORMATIDOFFSET, dwOffset) as ULONG,
        (core::mem::size_of::<PROPERTYSETHEADER>() + core::mem::size_of::<FORMATIDOFFSET>())
            as DWORD,
    );
}

unsafe fn PropertyStorage_MakeSectionHdr(
    cbSection: DWORD,
    numProps: DWORD,
    hdr: *mut PROPERTYSECTIONHEADER,
) {
    assert(!hdr.is_null());
    StorageUtl_WriteDWord(hdr as LPBYTE, 0, cbSection);
    StorageUtl_WriteDWord(
        hdr as LPBYTE,
        core::mem::offset_of!(PROPERTYSECTIONHEADER, cProperties) as ULONG,
        numProps,
    );
}

unsafe fn PropertyStorage_MakePropertyIdOffset(
    propid: DWORD,
    dwOffset: DWORD,
    propIdOffset: *mut PROPERTYIDOFFSET,
) {
    assert(!propIdOffset.is_null());
    StorageUtl_WriteDWord(propIdOffset as LPBYTE, 0, propid);
    StorageUtl_WriteDWord(
        propIdOffset as LPBYTE,
        core::mem::offset_of!(PROPERTYIDOFFSET, dwOffset) as ULONG,
        dwOffset,
    );
}

unsafe fn PropertStorage_WriteWStringToStream(
    stm: *mut IStream,
    str: LPCWSTR,
    len: DWORD,
    written: *mut DWORD,
) -> HRESULT {
    // Little-endian only (all three targets): the C's `#else` arm.
    IStream_Write(stm, str as *const c_void, len, written)
}

unsafe extern "C" fn PropertyStorage_DictionaryWriter(
    key: *const c_void,
    value: *const c_void,
    extra: *mut c_void,
    closure: *mut c_void,
) -> BOOL {
    let This = extra as *mut PropertyStorage_impl;
    let c = closure as *mut DictionaryClosure;
    let mut propid: DWORD = 0;
    let mut count: ULONG = 0;

    assert(!key.is_null());
    assert(!closure.is_null());
    StorageUtl_WriteDWord(
        core::ptr::addr_of_mut!(propid) as LPBYTE,
        0,
        PtrToUlong(value),
    );
    (*c).hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(propid) as *const c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED((*c).hr) {
        return SUCCEEDED((*c).hr) as BOOL;
    }
    (*c).bytesWritten = (*c)
        .bytesWritten
        .wrapping_add(core::mem::size_of::<DWORD>() as DWORD);
    if (*This).codePage == CP_UNICODE {
        let mut keyLen: DWORD = 0;
        let pad: DWORD = 0;
        // The on-disk entry length of a CP_UNICODE dictionary is a CHARACTER count:
        // PropertyStorage_ReadDictionary (same file) reads it as one and doubles it to
        // bytes. The pre-wine-10 C wrote a BYTE count here and then added
        // `keyLen * sizeof(WCHAR)` to bytesWritten, so a dictionary this wrote could not be
        // read back (the reader skipped twice the string) and every property offset after
        // it in the section was shifted by the doubled count. Write the character count,
        // and account the string, its padding and the section offset in bytes -- what
        // wine-10.0 does, keeping the old reader-matching padding rule (pad only when the
        // byte length is not already DWORD-aligned; wine-10 pads a full DWORD when it is).
        let cchKey: DWORD = (lstrlenW(key as LPCWSTR) + 1) as DWORD;
        let cbKey: DWORD = cchKey * core::mem::size_of::<WCHAR>() as DWORD;

        StorageUtl_WriteDWord(core::ptr::addr_of_mut!(keyLen) as LPBYTE, 0, cchKey);
        (*c).hr = IStream_Write(
            (*This).stm,
            core::ptr::addr_of!(keyLen) as *const c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::addr_of_mut!(count),
        );
        if FAILED((*c).hr) {
            return SUCCEEDED((*c).hr) as BOOL;
        }
        (*c).bytesWritten = (*c)
            .bytesWritten
            .wrapping_add(core::mem::size_of::<DWORD>() as DWORD);
        (*c).hr = PropertStorage_WriteWStringToStream(
            (*This).stm,
            key as LPCWSTR,
            cbKey,
            core::ptr::addr_of_mut!(count),
        );
        if FAILED((*c).hr) {
            return SUCCEEDED((*c).hr) as BOOL;
        }
        (*c).bytesWritten = (*c).bytesWritten.wrapping_add(cbKey);
        if cbKey % core::mem::size_of::<DWORD>() as DWORD != 0 {
            let cbPad = core::mem::size_of::<DWORD>() as DWORD
                - cbKey % core::mem::size_of::<DWORD>() as DWORD;
            (*c).hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(pad) as *const c_void,
                cbPad as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            if FAILED((*c).hr) {
                return SUCCEEDED((*c).hr) as BOOL;
            }
            (*c).bytesWritten = (*c).bytesWritten.wrapping_add(cbPad);
        }
    } else {
        let mut keyLen: DWORD = 0;

        StorageUtl_WriteDWord(
            core::ptr::addr_of_mut!(keyLen) as LPBYTE,
            0,
            (strlen(key as LPCSTR) + 1) as DWORD,
        );
        (*c).hr = IStream_Write(
            (*This).stm,
            core::ptr::addr_of!(keyLen) as *const c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::addr_of_mut!(count),
        );
        if FAILED((*c).hr) {
            return SUCCEEDED((*c).hr) as BOOL;
        }
        (*c).bytesWritten = (*c)
            .bytesWritten
            .wrapping_add(core::mem::size_of::<DWORD>() as DWORD);
        (*c).hr = IStream_Write(
            (*This).stm,
            key,
            keyLen as ULONG,
            core::ptr::addr_of_mut!(count),
        );
        if FAILED((*c).hr) {
            return SUCCEEDED((*c).hr) as BOOL;
        }
        (*c).bytesWritten = (*c).bytesWritten.wrapping_add(keyLen);
    }
    SUCCEEDED((*c).hr) as BOOL
}

/// Writes the dictionary to the stream.  Assumes without checking that the dictionary
/// isn't empty.
unsafe fn PropertyStorage_WriteDictionaryToStream(
    This: *mut PropertyStorage_impl,
    sectionOffset: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut seek: LARGE_INTEGER = core::mem::zeroed();
    let mut propIdOffset: PROPERTYIDOFFSET = core::mem::zeroed();
    let mut count: ULONG = 0;
    let mut dwTemp: DWORD = 0;
    let mut closure: DictionaryClosure = core::mem::zeroed();

    assert(!sectionOffset.is_null());

    // The dictionary's always the first property written, so seek to its spot.
    seek.QuadPart =
        (SECTIONHEADER_OFFSET + core::mem::size_of::<PROPERTYSECTIONHEADER>()) as LONGLONG;
    hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }

    PropertyStorage_MakePropertyIdOffset(
        PID_DICTIONARY,
        *sectionOffset,
        core::ptr::addr_of_mut!(propIdOffset),
    );
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(propIdOffset) as *const c_void,
        core::mem::size_of::<PROPERTYIDOFFSET>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED(hr) {
        return hr;
    }

    seek.QuadPart = (SECTIONHEADER_OFFSET as u64 + *sectionOffset as u64) as LONGLONG;
    hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }

    StorageUtl_WriteDWord(
        core::ptr::addr_of_mut!(dwTemp) as LPBYTE,
        0,
        dictionary_num_entries((*This).name_to_propid),
    );
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(dwTemp) as *const c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED(hr) {
        return hr;
    }
    *sectionOffset = (*sectionOffset).wrapping_add(core::mem::size_of::<DWORD>() as DWORD);

    closure.hr = S_OK;
    closure.bytesWritten = 0;
    dictionary_enumerate(
        (*This).name_to_propid,
        Some(PropertyStorage_DictionaryWriter),
        core::ptr::addr_of_mut!(closure) as *mut c_void,
    );
    hr = closure.hr;
    if FAILED(hr) {
        return hr;
    }
    *sectionOffset = (*sectionOffset).wrapping_add(closure.bytesWritten);
    if closure.bytesWritten % core::mem::size_of::<DWORD>() as DWORD != 0 {
        let padding: DWORD = (core::mem::size_of::<DWORD>() as DWORD)
            .wrapping_sub(closure.bytesWritten % core::mem::size_of::<DWORD>() as DWORD);
        TRACE!(b"adding %d bytes of padding\n\0".as_ptr() as LPCSTR, padding);
        *sectionOffset = (*sectionOffset).wrapping_add(padding);
    }

    hr
}

unsafe fn PropertyStorage_WritePropertyToStream(
    This: *mut PropertyStorage_impl,
    propNum: DWORD,
    propid: DWORD,
    var: *const PROPVARIANT,
    sectionOffset: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut seek: LARGE_INTEGER = core::mem::zeroed();
    let mut propIdOffset: PROPERTYIDOFFSET = core::mem::zeroed();
    let mut count: ULONG = 0;
    let mut dwType: DWORD = 0;
    let mut bytesWritten: DWORD = 0;

    assert(!var.is_null());
    assert(!sectionOffset.is_null());

    TRACE!(
        b"%p, %d, 0x%08x, (%d), (%d)\n\0".as_ptr() as LPCSTR,
        This,
        propNum,
        propid,
        (*var).vt,
        *sectionOffset,
    );

    seek.QuadPart = (SECTIONHEADER_OFFSET as u64
        + core::mem::size_of::<PROPERTYSECTIONHEADER>() as u64
        + propNum as u64 * core::mem::size_of::<PROPERTYIDOFFSET>() as u64)
        as LONGLONG;
    hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }
    PropertyStorage_MakePropertyIdOffset(
        propid,
        *sectionOffset,
        core::ptr::addr_of_mut!(propIdOffset),
    );
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(propIdOffset) as *const c_void,
        core::mem::size_of::<PROPERTYIDOFFSET>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED(hr) {
        return hr;
    }

    seek.QuadPart = (SECTIONHEADER_OFFSET as u64 + *sectionOffset as u64) as LONGLONG;
    hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }
    StorageUtl_WriteDWord(
        core::ptr::addr_of_mut!(dwType) as LPBYTE,
        0,
        (*var).vt as DWORD,
    );
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(dwType) as *const c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED(hr) {
        return hr;
    }
    *sectionOffset += core::mem::size_of::<DWORD>() as DWORD;

    match (*var).vt as DWORD {
        VT_EMPTY | VT_NULL => {
            bytesWritten = 0;
        }
        VT_I1 | VT_UI1 => {
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!((*var).__bindgen_anon_1.cVal) as *const c_void,
                core::mem::size_of::<c_char>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count;
        }
        VT_I2 | VT_UI2 => {
            let mut wTemp: WORD = 0;

            StorageUtl_WriteWord(
                core::ptr::addr_of_mut!(wTemp) as LPBYTE,
                0,
                (*var).__bindgen_anon_1.iVal as WORD,
            );
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(wTemp) as *const c_void,
                core::mem::size_of::<WORD>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count;
        }
        VT_I4 | VT_UI4 => {
            let mut dwTemp: DWORD = 0;

            StorageUtl_WriteDWord(
                core::ptr::addr_of_mut!(dwTemp) as LPBYTE,
                0,
                (*var).__bindgen_anon_1.lVal as DWORD,
            );
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(dwTemp) as *const c_void,
                core::mem::size_of::<DWORD>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count;
        }
        VT_I8 | VT_UI8 => {
            // wine-10: new. The old translation fell to the default arm, so WriteMultiple
            // accepted these and Commit then failed STG_E_INVALIDPARAMETER mid-write.
            let mut ularge: ULARGE_INTEGER = core::mem::zeroed();

            StorageUtl_WriteULargeInteger(
                core::ptr::addr_of_mut!(ularge) as LPBYTE,
                0,
                core::ptr::addr_of!((*var).__bindgen_anon_1.uhVal),
            );
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(ularge) as *const c_void,
                core::mem::size_of::<ULARGE_INTEGER>() as ULONG,
                core::ptr::addr_of_mut!(bytesWritten),
            );
        }
        VT_LPSTR => {
            let len: DWORD;
            let mut dwTemp: DWORD = 0;

            if (*This).codePage == CP_UNICODE {
                len = ((lstrlenW((*var).__bindgen_anon_1.pwszVal) + 1) as DWORD)
                    * core::mem::size_of::<WCHAR>() as DWORD;
            } else {
                len = (lstrlenA((*var).__bindgen_anon_1.pszVal) + 1) as DWORD;
            }
            StorageUtl_WriteDWord(core::ptr::addr_of_mut!(dwTemp) as LPBYTE, 0, len);
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(dwTemp) as *const c_void,
                core::mem::size_of::<DWORD>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            if FAILED(hr) {
                return hr;
            }
            hr = IStream_Write(
                (*This).stm,
                (*var).__bindgen_anon_1.pszVal as *const c_void,
                len,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count + core::mem::size_of::<DWORD>() as DWORD;
        }
        VT_BSTR => {
            // wine-10: new (see VT_I8). Unicode sets store the BSTR bytes plus its NUL; ANSI
            // sets convert to the set's code page, NUL included.
            let mut dwTemp: DWORD = 0;
            let len: DWORD;

            if (*This).codePage == CP_UNICODE {
                len = SysStringByteLen((*var).__bindgen_anon_1.bstrVal)
                    + core::mem::size_of::<WCHAR>() as DWORD;
                StorageUtl_WriteDWord(core::ptr::addr_of_mut!(dwTemp) as LPBYTE, 0, len);
                hr = IStream_Write(
                    (*This).stm,
                    core::ptr::addr_of!(dwTemp) as *const c_void,
                    core::mem::size_of::<DWORD>() as ULONG,
                    core::ptr::addr_of_mut!(count),
                );
                if SUCCEEDED(hr) {
                    hr = IStream_Write(
                        (*This).stm,
                        (*var).__bindgen_anon_1.bstrVal as *const c_void,
                        len,
                        core::ptr::addr_of_mut!(count),
                    );
                }
            } else {
                let bstr = (*var).__bindgen_anon_1.bstrVal;
                let cch = SysStringLen(bstr) as c_int;

                len = WideCharToMultiByte(
                    (*This).codePage,
                    0,
                    bstr,
                    cch + 1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                ) as DWORD;

                // Zeroed: wine-10 converts only the SysStringLen characters (not the NUL
                // its length counted) into an uninitialised buffer, so the last byte it
                // writes to the stream is heap garbage. Here it is the terminator.
                let str_ = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len as SIZE_T) as LPSTR;
                if str_.is_null() {
                    return E_OUTOFMEMORY;
                }

                WideCharToMultiByte(
                    (*This).codePage,
                    0,
                    bstr,
                    cch,
                    str_,
                    len as c_int,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                StorageUtl_WriteDWord(core::ptr::addr_of_mut!(dwTemp) as LPBYTE, 0, len);
                hr = IStream_Write(
                    (*This).stm,
                    core::ptr::addr_of!(dwTemp) as *const c_void,
                    core::mem::size_of::<DWORD>() as ULONG,
                    core::ptr::addr_of_mut!(count),
                );
                if SUCCEEDED(hr) {
                    hr = IStream_Write(
                        (*This).stm,
                        str_ as *const c_void,
                        len,
                        core::ptr::addr_of_mut!(count),
                    );
                }
                HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
            }

            bytesWritten = count + core::mem::size_of::<DWORD>() as DWORD;
        }
        VT_BLOB => {
            // wine-10: new (see VT_I8).
            let mut dwTemp: DWORD = 0;

            StorageUtl_WriteDWord(
                core::ptr::addr_of_mut!(dwTemp) as LPBYTE,
                0,
                (*var).__bindgen_anon_1.blob.cbSize,
            );
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(dwTemp) as *const c_void,
                core::mem::size_of::<DWORD>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            if FAILED(hr) {
                return hr;
            }
            hr = IStream_Write(
                (*This).stm,
                (*var).__bindgen_anon_1.blob.pBlobData as *const c_void,
                (*var).__bindgen_anon_1.blob.cbSize,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count + core::mem::size_of::<DWORD>() as DWORD;
        }
        VT_LPWSTR => {
            let len: DWORD = (lstrlenW((*var).__bindgen_anon_1.pwszVal) + 1) as DWORD;
            let mut dwTemp: DWORD = 0;

            StorageUtl_WriteDWord(core::ptr::addr_of_mut!(dwTemp) as LPBYTE, 0, len);
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(dwTemp) as *const c_void,
                core::mem::size_of::<DWORD>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            if FAILED(hr) {
                return hr;
            }
            hr = IStream_Write(
                (*This).stm,
                (*var).__bindgen_anon_1.pwszVal as *const c_void,
                len * core::mem::size_of::<WCHAR>() as DWORD,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count + core::mem::size_of::<DWORD>() as DWORD;
        }
        VT_FILETIME => {
            let mut temp: FILETIME = core::mem::zeroed();

            StorageUtl_WriteULargeInteger(
                core::ptr::addr_of_mut!(temp) as LPBYTE,
                0,
                core::ptr::addr_of!((*var).__bindgen_anon_1.filetime) as *const ULARGE_INTEGER,
            );
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(temp) as *const c_void,
                core::mem::size_of::<FILETIME>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count;
        }
        VT_CF => {
            let mut cf_hdr: [DWORD; 2] = [0; 2];
            let len: DWORD = (*(*var).__bindgen_anon_1.pclipdata).cbSize;

            StorageUtl_WriteDWord(
                core::ptr::addr_of_mut!(cf_hdr[0]) as LPBYTE,
                0,
                len + 8,
            );
            StorageUtl_WriteDWord(
                core::ptr::addr_of_mut!(cf_hdr[1]) as LPBYTE,
                0,
                (*(*var).__bindgen_anon_1.pclipdata).ulClipFmt as DWORD,
            );
            hr = IStream_Write(
                (*This).stm,
                cf_hdr.as_ptr() as *const c_void,
                core::mem::size_of_val(&cf_hdr) as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            if FAILED(hr) {
                return hr;
            }
            hr = IStream_Write(
                (*This).stm,
                (*(*var).__bindgen_anon_1.pclipdata).pClipData as *const c_void,
                len - core::mem::size_of::<LONG>() as DWORD,
                core::ptr::addr_of_mut!(count),
            );
            if FAILED(hr) {
                return hr;
            }
            bytesWritten = count + core::mem::size_of_val(&cf_hdr) as DWORD;
        }
        VT_CLSID => {
            let mut temp: CLSID = core::mem::zeroed();

            StorageUtl_WriteGUID(
                core::ptr::addr_of_mut!(temp) as LPBYTE,
                0,
                (*var).__bindgen_anon_1.puuid,
            );
            hr = IStream_Write(
                (*This).stm,
                core::ptr::addr_of!(temp) as *const c_void,
                core::mem::size_of::<CLSID>() as ULONG,
                core::ptr::addr_of_mut!(count),
            );
            bytesWritten = count;
        }
        _ => {
            FIXME!(b"unsupported type: %d\n\0".as_ptr() as LPCSTR, (*var).vt as core::ffi::c_uint);
            return STG_E_INVALIDPARAMETER;
        }
    }

    if SUCCEEDED(hr) {
        *sectionOffset += bytesWritten;
        if bytesWritten % core::mem::size_of::<DWORD>() as DWORD != 0 {
            let padding = core::mem::size_of::<DWORD>() as DWORD
                - bytesWritten % core::mem::size_of::<DWORD>() as DWORD;
            TRACE!(b"adding %d bytes of padding\n\0".as_ptr() as LPCSTR, padding);
            *sectionOffset += padding;
        }
    }

    hr
}

unsafe extern "C" fn PropertyStorage_PropertiesWriter(
    key: *const c_void,
    value: *const c_void,
    extra: *mut c_void,
    closure: *mut c_void,
) -> BOOL {
    let This = extra as *mut PropertyStorage_impl;
    let c = closure as *mut PropertyClosure;

    assert(!key.is_null());
    assert(!value.is_null());
    assert(!extra.is_null());
    assert(!closure.is_null());

    // C: `PropertyStorage_WritePropertyToStream(This, c->propNum++, ...)` — the
    // post-increment passes the OLD value and bumps the counter unconditionally.
    (*c).hr = PropertyStorage_WritePropertyToStream(
        This,
        (*c).propNum,
        PtrToUlong(key),
        value as *const PROPVARIANT,
        (*c).sectionOffset,
    );
    (*c).propNum = (*c).propNum.wrapping_add(1);

    SUCCEEDED((*c).hr) as BOOL
}

unsafe fn PropertyStorage_WritePropertiesToStream(
    This: *mut PropertyStorage_impl,
    startingPropNum: DWORD,
    sectionOffset: *mut DWORD,
) -> HRESULT {
    let mut closure = PropertyClosure {
        hr: S_OK,
        propNum: startingPropNum,
        sectionOffset,
    };

    assert(!sectionOffset.is_null());

    dictionary_enumerate(
        (*This).propid_to_prop,
        Some(PropertyStorage_PropertiesWriter),
        core::ptr::addr_of_mut!(closure) as *mut c_void,
    );

    closure.hr
}

unsafe fn PropertyStorage_WriteHeadersToStream(This: *mut PropertyStorage_impl) -> HRESULT {
    let mut hr: HRESULT;
    let mut count: ULONG = 0;
    let seek: LARGE_INTEGER = core::mem::zeroed();
    let mut hdr: PROPERTYSETHEADER = core::mem::zeroed();
    let mut fmtOffset: FORMATIDOFFSET = core::mem::zeroed();

    hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }
    PropertyStorage_MakeHeader(This, core::ptr::addr_of_mut!(hdr));
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(hdr) as *const c_void,
        core::mem::size_of::<PROPERTYSETHEADER>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED(hr) {
        return hr;
    }
    if count != core::mem::size_of::<PROPERTYSETHEADER>() as ULONG {
        hr = STG_E_WRITEFAULT;
        return hr;
    }

    PropertyStorage_MakeFmtIdOffset(This, core::ptr::addr_of_mut!(fmtOffset));
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(fmtOffset) as *const c_void,
        core::mem::size_of::<FORMATIDOFFSET>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED(hr) {
        return hr;
    }
    if count != core::mem::size_of::<FORMATIDOFFSET>() as ULONG {
        hr = STG_E_WRITEFAULT;
        return hr;
    }
    hr = S_OK;

    hr
}

unsafe fn PropertyStorage_WriteToStream(This: *mut PropertyStorage_impl) -> HRESULT {
    let mut sectionHdr: PROPERTYSECTIONHEADER = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut count: ULONG = 0;
    let mut seek: LARGE_INTEGER = core::mem::zeroed();
    let mut numProps: DWORD;
    let mut prop: DWORD;
    let mut sectionOffset: DWORD;
    let mut dwTemp: DWORD = 0;
    let mut var: PROPVARIANT = core::mem::zeroed();

    PropertyStorage_WriteHeadersToStream(This);

    // Count properties.  Always at least one property, the code page
    numProps = 1;
    if dictionary_num_entries((*This).name_to_propid) != 0 {
        numProps += 1;
    }
    if (*This).locale != LOCALE_SYSTEM_DEFAULT {
        numProps += 1;
    }
    if ((*This).grfFlags & PROPSETFLAG_CASE_SENSITIVE) != 0 {
        numProps += 1;
    }
    numProps += dictionary_num_entries((*This).propid_to_prop);

    // Write section header with 0 bytes right now, adjusted after writing properties.
    PropertyStorage_MakeSectionHdr(0, numProps, core::ptr::addr_of_mut!(sectionHdr));
    seek.QuadPart = SECTIONHEADER_OFFSET as LONGLONG;
    hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(sectionHdr) as *const c_void,
        core::mem::size_of::<PROPERTYSECTIONHEADER>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );
    if FAILED(hr) {
        return hr;
    }

    prop = 0;
    sectionOffset = core::mem::size_of::<PROPERTYSECTIONHEADER>() as DWORD
        + numProps * core::mem::size_of::<PROPERTYIDOFFSET>() as DWORD;

    if dictionary_num_entries((*This).name_to_propid) != 0 {
        prop += 1;
        hr = PropertyStorage_WriteDictionaryToStream(
            This,
            core::ptr::addr_of_mut!(sectionOffset),
        );
        if FAILED(hr) {
            return hr;
        }
    }

    PropVariantInit(core::ptr::addr_of_mut!(var));

    var.vt = VT_I2 as VARTYPE;
    var.__bindgen_anon_1.iVal = (*This).codePage as i16;
    hr = PropertyStorage_WritePropertyToStream(
        This,
        prop,
        PID_CODEPAGE,
        core::ptr::addr_of!(var),
        core::ptr::addr_of_mut!(sectionOffset),
    );
    prop += 1;
    if FAILED(hr) {
        return hr;
    }

    if (*This).locale != LOCALE_SYSTEM_DEFAULT {
        var.vt = VT_I4 as VARTYPE;
        var.__bindgen_anon_1.lVal = (*This).locale as LONG;
        hr = PropertyStorage_WritePropertyToStream(
            This,
            prop,
            PID_LOCALE,
            core::ptr::addr_of!(var),
            core::ptr::addr_of_mut!(sectionOffset),
        );
        prop += 1;
        if FAILED(hr) {
            return hr;
        }
    }

    if ((*This).grfFlags & PROPSETFLAG_CASE_SENSITIVE) != 0 {
        var.vt = VT_I4 as VARTYPE;
        var.__bindgen_anon_1.lVal = 1;
        hr = PropertyStorage_WritePropertyToStream(
            This,
            prop,
            PID_BEHAVIOR,
            core::ptr::addr_of!(var),
            core::ptr::addr_of_mut!(sectionOffset),
        );
        prop += 1;
        if FAILED(hr) {
            return hr;
        }
    }

    hr = PropertyStorage_WritePropertiesToStream(
        This,
        prop,
        core::ptr::addr_of_mut!(sectionOffset),
    );
    if FAILED(hr) {
        return hr;
    }

    // Now write the byte count of the section
    seek.QuadPart = SECTIONHEADER_OFFSET as LONGLONG;
    hr = IStream_Seek((*This).stm, seek, STREAM_SEEK_SET, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }
    StorageUtl_WriteDWord(core::ptr::addr_of_mut!(dwTemp) as LPBYTE, 0, sectionOffset);
    hr = IStream_Write(
        (*This).stm,
        core::ptr::addr_of!(dwTemp) as *const c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(count),
    );

    hr
}

unsafe fn PropertyStorage_DestroyDictionaries(This: *mut PropertyStorage_impl) {
    dictionary_destroy((*This).name_to_propid);
    (*This).name_to_propid = core::ptr::null_mut();
    dictionary_destroy((*This).propid_to_name);
    (*This).propid_to_name = core::ptr::null_mut();
    dictionary_destroy((*This).propid_to_prop);
    (*This).propid_to_prop = core::ptr::null_mut();
}

unsafe fn PropertyStorage_CreateDictionaries(This: *mut PropertyStorage_impl) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    'end: {
        (*This).name_to_propid = dictionary_create(
            Some(PropertyStorage_PropNameCompare),
            Some(PropertyStorage_PropNameDestroy),
            This as *mut c_void,
        );
        if (*This).name_to_propid.is_null() {
            hr = STG_E_INSUFFICIENTMEMORY;
            break 'end;
        }
        (*This).propid_to_name = dictionary_create(
            Some(PropertyStorage_PropCompare),
            None,
            This as *mut c_void,
        );
        if (*This).propid_to_name.is_null() {
            hr = STG_E_INSUFFICIENTMEMORY;
            break 'end;
        }
        (*This).propid_to_prop = dictionary_create(
            Some(PropertyStorage_PropCompare),
            Some(PropertyStorage_PropertyDestroy),
            This as *mut c_void,
        );
        if (*This).propid_to_prop.is_null() {
            hr = STG_E_INSUFFICIENTMEMORY;
            break 'end;
        }
    }

    // end:
    if FAILED(hr) {
        PropertyStorage_DestroyDictionaries(This);
    }
    hr
}

unsafe fn PropertyStorage_BaseConstruct(
    stm: *mut IStream,
    rfmtid: *const FMTID,
    grfMode: DWORD,
    pps: *mut *mut PropertyStorage_impl,
) -> HRESULT {
    let hr: HRESULT;

    assert(!pps.is_null());
    assert(!rfmtid.is_null());

    *pps = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<PropertyStorage_impl>() as SIZE_T,
    ) as *mut PropertyStorage_impl;

    if (*pps).is_null() {
        return E_OUTOFMEMORY;
    }

    (**pps).IPropertyStorage_iface.lpVtbl =
        &IPropertyStorage_Vtbl as *const IPropertyStorageVtbl as *mut IPropertyStorageVtbl;
    (**pps).ref_ = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((**pps).cs));
    set_critsec_debug_name(
        core::ptr::addr_of_mut!((**pps).cs),
        b"stg_prop.c: PropertyStorage_impl.cs\0".as_ptr() as *const c_char,
    );
    (**pps).stm = stm;
    (**pps).fmtid = *rfmtid;
    (**pps).grfMode = grfMode;

    hr = PropertyStorage_CreateDictionaries(*pps);
    if FAILED(hr) {
        set_critsec_debug_name(core::ptr::addr_of_mut!((**pps).cs), core::ptr::null());
        DeleteCriticalSection(core::ptr::addr_of_mut!((**pps).cs));
        HeapFree(GetProcessHeap(), 0, *pps as *mut c_void);
        *pps = core::ptr::null_mut();
    } else {
        IStream_AddRef(stm);
    }

    hr
}

unsafe fn PropertyStorage_ConstructFromStream(
    stm: *mut IStream,
    rfmtid: *const FMTID,
    grfMode: DWORD,
    pps: *mut *mut IPropertyStorage,
) -> HRESULT {
    let mut ps: *mut PropertyStorage_impl = core::ptr::null_mut();
    let mut hr: HRESULT;

    assert(!pps.is_null());
    hr = PropertyStorage_BaseConstruct(stm, rfmtid, grfMode, core::ptr::addr_of_mut!(ps));
    if SUCCEEDED(hr) {
        hr = PropertyStorage_ReadFromStream(ps);
        if SUCCEEDED(hr) {
            *pps = core::ptr::addr_of_mut!((*ps).IPropertyStorage_iface);
            TRACE!(b"PropertyStorage %p constructed\n\0".as_ptr() as LPCSTR, ps);
            hr = S_OK;
        } else {
            IPropertyStorage_Release(core::ptr::addr_of_mut!((*ps).IPropertyStorage_iface));
        }
    }
    hr
}

unsafe fn PropertyStorage_ConstructEmpty(
    stm: *mut IStream,
    rfmtid: *const FMTID,
    grfFlags: DWORD,
    grfMode: DWORD,
    pps: *mut *mut IPropertyStorage,
) -> HRESULT {
    let mut ps: *mut PropertyStorage_impl = core::ptr::null_mut();
    let mut hr: HRESULT;

    assert(!pps.is_null());
    hr = PropertyStorage_BaseConstruct(stm, rfmtid, grfMode, core::ptr::addr_of_mut!(ps));
    if SUCCEEDED(hr) {
        (*ps).format = 0;
        (*ps).grfFlags = grfFlags;
        if (*ps).grfFlags & PROPSETFLAG_CASE_SENSITIVE != 0 {
            (*ps).format = 1;
        }
        // default to Unicode unless told not to, as specified on msdn
        if (*ps).grfFlags & PROPSETFLAG_ANSI != 0 {
            (*ps).codePage = GetACP();
        } else {
            (*ps).codePage = CP_UNICODE;
        }
        (*ps).locale = LOCALE_SYSTEM_DEFAULT;
        TRACE!(
            b"Code page is %d, locale is %d\n\0".as_ptr() as LPCSTR,
            (*ps).codePage,
            (*ps).locale,
        );
        *pps = core::ptr::addr_of_mut!((*ps).IPropertyStorage_iface);
        TRACE!(b"PropertyStorage %p constructed\n\0".as_ptr() as LPCSTR, ps);
        hr = S_OK;
    }
    hr
}

// ---------------------------------------------------------------------------
// Implementation of IPropertySetStorage
// ---------------------------------------------------------------------------

stdmethod! {

unsafe fn IPropertySetStorage_fnQueryInterface(
    ppstg: *mut IPropertySetStorage,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IPropertySetStorage(ppstg);
    IStorage_QueryInterface(
        core::ptr::addr_of_mut!((*This).base.IStorage_iface),
        riid,
        ppvObject,
    )
}

unsafe fn IPropertySetStorage_fnAddRef(ppstg: *mut IPropertySetStorage) -> ULONG {
    let This = impl_from_IPropertySetStorage(ppstg);
    IStorage_AddRef(core::ptr::addr_of_mut!((*This).base.IStorage_iface))
}

unsafe fn IPropertySetStorage_fnRelease(ppstg: *mut IPropertySetStorage) -> ULONG {
    let This = impl_from_IPropertySetStorage(ppstg);
    IStorage_Release(core::ptr::addr_of_mut!((*This).base.IStorage_iface))
}

unsafe fn IPropertySetStorage_fnCreate(
    ppstg: *mut IPropertySetStorage,
    rfmtid: *const FMTID,
    _pclsid: *const CLSID,
    grfFlags: DWORD,
    grfMode: DWORD,
    ppprstg: *mut *mut IPropertyStorage,
) -> HRESULT {
    let This = impl_from_IPropertySetStorage(ppstg);
    let mut name: [WCHAR; CCH_MAX_PROPSTG_NAME as usize + 1] =
        [0; CCH_MAX_PROPSTG_NAME as usize + 1];
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut r: HRESULT;

    TRACE!(
        b"%p %s %08x %08x %p\n\0".as_ptr() as LPCSTR,
        This,
        debugstr_guid(rfmtid),
        grfFlags,
        grfMode,
        ppprstg,
    );

    'end: {
        // be picky
        if grfMode != (STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE) {
            r = STG_E_INVALIDFLAG;
            break 'end;
        }

        if rfmtid.is_null() {
            r = E_INVALIDARG;
            break 'end;
        }

        // FIXME: if (grfFlags & PROPSETFLAG_NONSIMPLE), we need to create a storage,
        // not a stream.  For now, disallow it.
        if (grfFlags & PROPSETFLAG_NONSIMPLE) != 0 {
            FIXME!(b"PROPSETFLAG_NONSIMPLE not supported\n\0".as_ptr() as LPCSTR);
            r = STG_E_INVALIDFLAG;
            break 'end;
        }

        r = FmtIdToPropStgName(rfmtid, name.as_mut_ptr());
        if FAILED(r) {
            break 'end;
        }

        r = IStorage_CreateStream(
            core::ptr::addr_of_mut!((*This).base.IStorage_iface),
            name.as_ptr(),
            grfMode,
            0,
            0,
            core::ptr::addr_of_mut!(stm),
        );
        if FAILED(r) {
            break 'end;
        }

        r = PropertyStorage_ConstructEmpty(stm, rfmtid, grfFlags, grfMode, ppprstg);

        IStream_Release(stm);
    }

    // end:
    TRACE!(b"returning 0x%08x\n\0".as_ptr() as LPCSTR, r);
    r
}

unsafe fn IPropertySetStorage_fnOpen(
    ppstg: *mut IPropertySetStorage,
    rfmtid: *const FMTID,
    grfMode: DWORD,
    ppprstg: *mut *mut IPropertyStorage,
) -> HRESULT {
    let This = impl_from_IPropertySetStorage(ppstg);
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut name: [WCHAR; CCH_MAX_PROPSTG_NAME as usize + 1] =
        [0; CCH_MAX_PROPSTG_NAME as usize + 1];
    let mut r: HRESULT;

    TRACE!(
        b"%p %s %08x %p\n\0".as_ptr() as LPCSTR,
        This,
        debugstr_guid(rfmtid),
        grfMode,
        ppprstg,
    );

    'end: {
        // be picky
        if grfMode != (STGM_READWRITE | STGM_SHARE_EXCLUSIVE)
            && grfMode != (STGM_READ | STGM_SHARE_EXCLUSIVE)
        {
            r = STG_E_INVALIDFLAG;
            break 'end;
        }

        if rfmtid.is_null() {
            r = E_INVALIDARG;
            break 'end;
        }

        r = FmtIdToPropStgName(rfmtid, name.as_mut_ptr());
        if FAILED(r) {
            break 'end;
        }

        r = IStorage_OpenStream(
            core::ptr::addr_of_mut!((*This).base.IStorage_iface),
            name.as_ptr(),
            core::ptr::null_mut(),
            grfMode,
            0,
            core::ptr::addr_of_mut!(stm),
        );
        if FAILED(r) {
            break 'end;
        }

        r = PropertyStorage_ConstructFromStream(stm, rfmtid, grfMode, ppprstg);

        IStream_Release(stm);
    }

    // end:
    TRACE!(b"returning 0x%08x\n\0".as_ptr() as LPCSTR, r);
    r
}

unsafe fn IPropertySetStorage_fnDelete(
    ppstg: *mut IPropertySetStorage,
    rfmtid: *const FMTID,
) -> HRESULT {
    let This = impl_from_IPropertySetStorage(ppstg);
    let mut name: [WCHAR; CCH_MAX_PROPSTG_NAME as usize + 1] =
        [0; CCH_MAX_PROPSTG_NAME as usize + 1];
    let r: HRESULT;

    TRACE!(b"%p %s\n\0".as_ptr() as LPCSTR, This, debugstr_guid(rfmtid));

    if rfmtid.is_null() {
        return E_INVALIDARG;
    }

    r = FmtIdToPropStgName(rfmtid, name.as_mut_ptr());
    if FAILED(r) {
        return r;
    }

    IStorage_DestroyElement(
        core::ptr::addr_of_mut!((*This).base.IStorage_iface),
        name.as_ptr(),
    )
}

unsafe fn IPropertySetStorage_fnEnum(
    ppstg: *mut IPropertySetStorage,
    ppenum: *mut *mut IEnumSTATPROPSETSTG,
) -> HRESULT {
    let This = impl_from_IPropertySetStorage(ppstg);
    // wine-10: a NULL out-pointer is E_INVALIDARG, not a store through NULL
    // (Wine's stg_prop test calls Enum(NULL)).
    if ppenum.is_null() {
        return E_INVALIDARG;
    }
    create_EnumSTATPROPSETSTG(This, ppenum)
}

} // stdmethod!

// ---------------------------------------------------------------------------
// Implement IEnumSTATPROPSETSTG using enumx
// ---------------------------------------------------------------------------

stdmethod! {

unsafe fn IEnumSTATPROPSETSTG_fnQueryInterface(
    iface: *mut IEnumSTATPROPSETSTG,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    enumx_QueryInterface(iface as *mut enumx_impl, riid, ppvObject)
}

unsafe fn IEnumSTATPROPSETSTG_fnAddRef(iface: *mut IEnumSTATPROPSETSTG) -> ULONG {
    enumx_AddRef(iface as *mut enumx_impl)
}

unsafe fn IEnumSTATPROPSETSTG_fnRelease(iface: *mut IEnumSTATPROPSETSTG) -> ULONG {
    enumx_Release(iface as *mut enumx_impl)
}

unsafe fn IEnumSTATPROPSETSTG_fnNext(
    iface: *mut IEnumSTATPROPSETSTG,
    celt: ULONG,
    rgelt: *mut STATPROPSETSTG,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    enumx_Next(
        iface as *mut enumx_impl,
        celt,
        rgelt as *mut c_void,
        pceltFetched,
    )
}

unsafe fn IEnumSTATPROPSETSTG_fnSkip(
    iface: *mut IEnumSTATPROPSETSTG,
    celt: ULONG,
) -> HRESULT {
    enumx_Skip(iface as *mut enumx_impl, celt)
}

unsafe fn IEnumSTATPROPSETSTG_fnReset(iface: *mut IEnumSTATPROPSETSTG) -> HRESULT {
    enumx_Reset(iface as *mut enumx_impl)
}

unsafe fn IEnumSTATPROPSETSTG_fnClone(
    iface: *mut IEnumSTATPROPSETSTG,
    ppenum: *mut *mut IEnumSTATPROPSETSTG,
) -> HRESULT {
    enumx_Clone(iface as *mut enumx_impl, ppenum as *mut *mut enumx_impl)
}

} // stdmethod!

unsafe fn create_EnumSTATPROPSETSTG(
    This: *mut StorageImpl,
    ppenum: *mut *mut IEnumSTATPROPSETSTG,
) -> HRESULT {
    let stg = core::ptr::addr_of_mut!((*This).base.IStorage_iface);
    let mut penum: *mut IEnumSTATSTG = core::ptr::null_mut();
    let mut stat: STATSTG = core::mem::zeroed();
    let mut count: ULONG;
    let mut r: HRESULT;
    let mut statpss: STATPROPSETSTG = core::mem::zeroed();
    let enumx: *mut enumx_impl;

    TRACE!(b"%p %p\n\0".as_ptr() as LPCSTR, This, ppenum);

    enumx = enumx_allocate(
        core::ptr::addr_of!(IID_IEnumSTATPROPSETSTG),
        &IEnumSTATPROPSETSTG_Vtbl as *const IEnumSTATPROPSETSTGVtbl as *const c_void,
        core::mem::size_of::<STATPROPSETSTG>() as ULONG,
        core::ptr::addr_of_mut!((*This).base.IStorage_iface) as *mut IUnknown,
        None,
    );

    // add all the property set elements into a list
    r = IStorage_EnumElements(
        stg,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(penum),
    );
    if FAILED(r) {
        enumx_Release(enumx);
        return E_OUTOFMEMORY;
    }

    loop {
        count = 0;
        r = IEnumSTATSTG_Next(
            penum,
            1,
            core::ptr::addr_of_mut!(stat),
            core::ptr::addr_of_mut!(count),
        );
        if FAILED(r) {
            break;
        }
        if count == 0 {
            break;
        }
        if stat.pwcsName.is_null() {
            continue;
        }
        if *stat.pwcsName == 5 && stat.type_ == STGTY_STREAM {
            PropStgNameToFmtId(stat.pwcsName, core::ptr::addr_of_mut!(statpss.fmtid));
            TRACE!(
                b"adding %s (%s)\n\0".as_ptr() as LPCSTR,
                debugstr_w(stat.pwcsName),
                debugstr_guid(core::ptr::addr_of!(statpss.fmtid)),
            );
            statpss.mtime = stat.mtime;
            statpss.atime = stat.atime;
            statpss.ctime = stat.ctime;
            statpss.grfFlags = stat.grfMode;
            statpss.clsid = stat.clsid;
            enumx_add_element(enumx, core::ptr::addr_of!(statpss) as *const c_void);
        }
        CoTaskMemFree(stat.pwcsName as *mut c_void);
    }
    IEnumSTATSTG_Release(penum);

    *ppenum = enumx as *mut IEnumSTATPROPSETSTG;

    S_OK
}

// ---------------------------------------------------------------------------
// Implement IEnumSTATPROPSTG using enumx
// ---------------------------------------------------------------------------

stdmethod! {

unsafe fn IEnumSTATPROPSTG_fnQueryInterface(
    iface: *mut IEnumSTATPROPSTG,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    enumx_QueryInterface(iface as *mut enumx_impl, riid, ppvObject)
}

unsafe fn IEnumSTATPROPSTG_fnAddRef(iface: *mut IEnumSTATPROPSTG) -> ULONG {
    enumx_AddRef(iface as *mut enumx_impl)
}

unsafe fn IEnumSTATPROPSTG_fnRelease(iface: *mut IEnumSTATPROPSTG) -> ULONG {
    enumx_Release(iface as *mut enumx_impl)
}

unsafe fn IEnumSTATPROPSTG_fnNext(
    iface: *mut IEnumSTATPROPSTG,
    celt: ULONG,
    rgelt: *mut STATPROPSTG,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    enumx_Next(
        iface as *mut enumx_impl,
        celt,
        rgelt as *mut c_void,
        pceltFetched,
    )
}

unsafe fn IEnumSTATPROPSTG_fnSkip(iface: *mut IEnumSTATPROPSTG, celt: ULONG) -> HRESULT {
    enumx_Skip(iface as *mut enumx_impl, celt)
}

unsafe fn IEnumSTATPROPSTG_fnReset(iface: *mut IEnumSTATPROPSTG) -> HRESULT {
    enumx_Reset(iface as *mut enumx_impl)
}

unsafe fn IEnumSTATPROPSTG_fnClone(
    iface: *mut IEnumSTATPROPSTG,
    ppenum: *mut *mut IEnumSTATPROPSTG,
) -> HRESULT {
    enumx_Clone(iface as *mut enumx_impl, ppenum as *mut *mut enumx_impl)
}

} // stdmethod!

unsafe extern "C" fn prop_enum_copy_cb(
    parent: *mut IUnknown,
    orig: *mut c_void,
    dest: *mut c_void,
) {
    let storage = impl_from_IPropertyStorage(parent as *mut IPropertyStorage);
    let src_prop = orig as *mut STATPROPSTG;
    let dest_prop = dest as *mut STATPROPSTG;
    let mut name: LPWSTR = core::ptr::null_mut();

    (*dest_prop).propid = (*src_prop).propid;
    (*dest_prop).vt = (*src_prop).vt;
    (*dest_prop).lpwstrName = core::ptr::null_mut();

    if dictionary_find(
        (*storage).propid_to_name,
        UlongToPtr((*src_prop).propid),
        core::ptr::addr_of_mut!(name) as *mut *mut c_void,
    ) != 0
    {
        let size: DWORD =
            ((lstrlenW(name) + 1) * core::mem::size_of::<WCHAR>() as i32) as DWORD;

        (*dest_prop).lpwstrName = CoTaskMemAlloc(size as SIZE_T) as LPOLESTR;
        if (*dest_prop).lpwstrName.is_null() {
            return;
        }
        memcpy(
            (*dest_prop).lpwstrName as *mut c_void,
            name as *const c_void,
            size as usize,
        );
    }
}

unsafe extern "C" fn prop_enum_stat(
    k: *const c_void,
    v: *const c_void,
    _extra: *mut c_void,
    arg: *mut c_void,
) -> BOOL {
    let enumx = arg as *mut enumx_impl;
    let propid: PROPID = PtrToUlong(k);
    let prop = v as *const PROPVARIANT;
    let mut stat: STATPROPSTG = core::mem::zeroed();

    stat.lpwstrName = core::ptr::null_mut();
    stat.propid = propid;
    stat.vt = (*prop).vt;

    enumx_add_element(enumx, core::ptr::addr_of!(stat) as *const c_void);

    1
}

unsafe fn create_EnumSTATPROPSTG(
    This: *mut PropertyStorage_impl,
    ppenum: *mut *mut IEnumSTATPROPSTG,
) -> HRESULT {
    TRACE!(b"%p %p\n\0".as_ptr() as LPCSTR, This, ppenum);

    let enumx = enumx_allocate(
        core::ptr::addr_of!(IID_IEnumSTATPROPSTG),
        &IEnumSTATPROPSTG_Vtbl as *const IEnumSTATPROPSTGVtbl as *const c_void,
        core::mem::size_of::<STATPROPSTG>() as ULONG,
        core::ptr::addr_of_mut!((*This).IPropertyStorage_iface) as *mut IUnknown,
        Some(prop_enum_copy_cb),
    );

    dictionary_enumerate(
        (*This).propid_to_prop,
        Some(prop_enum_stat),
        enumx as *mut c_void,
    );

    *ppenum = enumx as *mut IEnumSTATPROPSTG;

    S_OK
}

// ---------------------------------------------------------------------------
// vtables (stg_prop.c:2557-2607)
// ---------------------------------------------------------------------------

/// stg_prop.c:2557 — NOT `static` in the C: storage32.c declares it `extern` and
/// installs it into every StorageBaseImpl.
pub static IPropertySetStorage_Vtbl: IPropertySetStorageVtbl = IPropertySetStorageVtbl {
    QueryInterface: Some(IPropertySetStorage_fnQueryInterface),
    AddRef: Some(IPropertySetStorage_fnAddRef),
    Release: Some(IPropertySetStorage_fnRelease),
    Create: Some(IPropertySetStorage_fnCreate),
    Open: Some(IPropertySetStorage_fnOpen),
    Delete: Some(IPropertySetStorage_fnDelete),
    Enum: Some(IPropertySetStorage_fnEnum),
};

/// stg_prop.c:2568
static IPropertyStorage_Vtbl: IPropertyStorageVtbl = IPropertyStorageVtbl {
    QueryInterface: Some(IPropertyStorage_fnQueryInterface),
    AddRef: Some(IPropertyStorage_fnAddRef),
    Release: Some(IPropertyStorage_fnRelease),
    ReadMultiple: Some(IPropertyStorage_fnReadMultiple),
    WriteMultiple: Some(IPropertyStorage_fnWriteMultiple),
    DeleteMultiple: Some(IPropertyStorage_fnDeleteMultiple),
    ReadPropertyNames: Some(IPropertyStorage_fnReadPropertyNames),
    WritePropertyNames: Some(IPropertyStorage_fnWritePropertyNames),
    DeletePropertyNames: Some(IPropertyStorage_fnDeletePropertyNames),
    Commit: Some(IPropertyStorage_fnCommit),
    Revert: Some(IPropertyStorage_fnRevert),
    Enum: Some(IPropertyStorage_fnEnum),
    SetTimes: Some(IPropertyStorage_fnSetTimes),
    SetClass: Some(IPropertyStorage_fnSetClass),
    Stat: Some(IPropertyStorage_fnStat),
};

/// stg_prop.c:2587
static IEnumSTATPROPSETSTG_Vtbl: IEnumSTATPROPSETSTGVtbl = IEnumSTATPROPSETSTGVtbl {
    QueryInterface: Some(IEnumSTATPROPSETSTG_fnQueryInterface),
    AddRef: Some(IEnumSTATPROPSETSTG_fnAddRef),
    Release: Some(IEnumSTATPROPSETSTG_fnRelease),
    Next: Some(IEnumSTATPROPSETSTG_fnNext),
    Skip: Some(IEnumSTATPROPSETSTG_fnSkip),
    Reset: Some(IEnumSTATPROPSETSTG_fnReset),
    Clone: Some(IEnumSTATPROPSETSTG_fnClone),
};

/// stg_prop.c:2598
static IEnumSTATPROPSTG_Vtbl: IEnumSTATPROPSTGVtbl = IEnumSTATPROPSTGVtbl {
    QueryInterface: Some(IEnumSTATPROPSTG_fnQueryInterface),
    AddRef: Some(IEnumSTATPROPSTG_fnAddRef),
    Release: Some(IEnumSTATPROPSTG_fnRelease),
    Next: Some(IEnumSTATPROPSTG_fnNext),
    Skip: Some(IEnumSTATPROPSTG_fnSkip),
    Reset: Some(IEnumSTATPROPSTG_fnReset),
    Clone: Some(IEnumSTATPROPSTG_fnClone),
};

// ---------------------------------------------------------------------------
// Format ID <-> name conversion
// ---------------------------------------------------------------------------

/// FmtIdToPropStgName [ole32.@]
///
/// `str` must be at least CCH_MAX_PROPSTG_NAME characters in length.
#[no_mangle]
pub unsafe extern "system" fn FmtIdToPropStgName(rfmtid: *const FMTID, str: LPOLESTR) -> HRESULT {
    static fmtMap: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz012345";

    TRACE!(b"%s, %p\n\0".as_ptr() as LPCSTR, debugstr_guid(rfmtid), str);

    if rfmtid.is_null() {
        return E_INVALIDARG;
    }
    if str.is_null() {
        return E_INVALIDARG;
    }

    if IsEqualGUID(&FMTID_SummaryInformation, rfmtid) {
        lstrcpyW(str, szSummaryInfo.as_ptr());
    } else if IsEqualGUID(&FMTID_DocSummaryInformation, rfmtid) {
        lstrcpyW(str, szDocSummaryInfo.as_ptr());
    } else if IsEqualGUID(&FMTID_UserDefinedProperties, rfmtid) {
        lstrcpyW(str, szDocSummaryInfo.as_ptr());
    } else {
        let fmtptr_start = rfmtid as *const BYTE;
        let fmtptr_end = fmtptr_start.add(core::mem::size_of::<FMTID>());
        let mut fmtptr = fmtptr_start;
        let mut pstr = str;
        let mut bitsRemaining: ULONG = BITS_PER_BYTE;

        *pstr = 5;
        pstr = pstr.add(1);

        while fmtptr < fmtptr_end {
            let mut i: ULONG = (*fmtptr as ULONG) >> (BITS_PER_BYTE - bitsRemaining);

            if bitsRemaining >= BITS_IN_CHARMASK {
                *pstr = fmtMap[(i & CHARMASK) as usize] as WCHAR;
                if bitsRemaining == BITS_PER_BYTE
                    && *pstr >= b'a' as WCHAR
                    && *pstr <= b'z' as WCHAR
                {
                    // C: `*pstr += 'A' - 'a';` — i.e. subtract 32.
                    *pstr = (*pstr).wrapping_sub(b'a' as WCHAR - b'A' as WCHAR);
                }
                pstr = pstr.add(1);
                bitsRemaining -= BITS_IN_CHARMASK;
                if bitsRemaining == 0 {
                    fmtptr = fmtptr.add(1);
                    bitsRemaining = BITS_PER_BYTE;
                }
            } else {
                fmtptr = fmtptr.add(1);
                if fmtptr < fmtptr_end {
                    i |= (*fmtptr as ULONG) << bitsRemaining;
                }
                *pstr = fmtMap[(i & CHARMASK) as usize] as WCHAR;
                pstr = pstr.add(1);
                bitsRemaining += BITS_PER_BYTE - BITS_IN_CHARMASK;
            }
        }
        *pstr = 0;
    }
    TRACE!(b"returning %s\n\0".as_ptr() as LPCSTR, debugstr_w(str));
    S_OK
}

/// PropStgNameToFmtId [ole32.@]
#[no_mangle]
pub unsafe extern "system" fn PropStgNameToFmtId(str: LPOLESTR, rfmtid: *mut FMTID) -> HRESULT {
    let mut hr: HRESULT = STG_E_INVALIDNAME;

    TRACE!(b"%s, %p\n\0".as_ptr() as LPCSTR, debugstr_w(str), rfmtid);

    if rfmtid.is_null() {
        return E_INVALIDARG;
    }
    if str.is_null() {
        return STG_E_INVALIDNAME;
    }

    if lstrcmpiW(str, szDocSummaryInfo.as_ptr()) == 0 {
        *rfmtid = FMTID_DocSummaryInformation;
        hr = S_OK;
    } else if lstrcmpiW(str, szSummaryInfo.as_ptr()) == 0 {
        *rfmtid = FMTID_SummaryInformation;
        hr = S_OK;
    } else {
        let mut bits: ULONG;
        // C: `BYTE *fmtptr = (BYTE *)rfmtid - 1;` (pre-incremented on first use)
        let mut fmtptr: *mut BYTE = (rfmtid as *mut BYTE).offset(-1);
        let mut pstr: LPCWSTR = str;

        memset(rfmtid as *mut c_void, 0, core::mem::size_of::<FMTID>());
        bits = 0;
        'end: {
            while bits < core::mem::size_of::<FMTID>() as ULONG * BITS_PER_BYTE {
                let bitsUsed: ULONG = bits % BITS_PER_BYTE;
                let bitsStored: ULONG;
                let mut wc: WCHAR;

                if bitsUsed == 0 {
                    fmtptr = fmtptr.offset(1);
                }
                pstr = pstr.offset(1);
                wc = (*pstr).wrapping_sub(b'A' as WCHAR);
                if wc > NUM_ALPHA_CHARS as WCHAR {
                    wc = wc.wrapping_add((b'A' as WCHAR).wrapping_sub(b'a' as WCHAR));
                    if wc > NUM_ALPHA_CHARS as WCHAR {
                        wc = wc.wrapping_add(
                            (b'a' as WCHAR)
                                .wrapping_sub(b'0' as WCHAR)
                                .wrapping_add(NUM_ALPHA_CHARS as WCHAR),
                        );
                        if wc > CHARMASK as WCHAR {
                            WARN!(b"invalid character (%d)\n\0".as_ptr() as LPCSTR, *pstr);
                            break 'end;
                        }
                    }
                }
                *fmtptr |= ((wc as ULONG) << bitsUsed) as BYTE;
                bitsStored = min(BITS_PER_BYTE - bitsUsed, BITS_IN_CHARMASK);
                if bitsStored < BITS_IN_CHARMASK {
                    wc >>= BITS_PER_BYTE - bitsUsed;
                    if bits + bitsStored
                        == core::mem::size_of::<FMTID>() as ULONG * BITS_PER_BYTE
                    {
                        if wc != 0 {
                            WARN!(b"extra bits\n\0".as_ptr() as LPCSTR);
                            break 'end;
                        }
                        break;
                    }
                    fmtptr = fmtptr.offset(1);
                    *fmtptr |= wc as BYTE;
                }

                bits += BITS_IN_CHARMASK;
            }
            hr = S_OK;
        }
    }

    // end:
    hr
}

/// `Allocate_PMemoryAllocator` — call through the PMemoryAllocator's first vtbl slot.
///
/// PMemoryAllocator is a **C++** object handed to us by the caller, so its `Allocate` slot
/// uses the platform C++ `this` convention. That differs by arch, and stg_prop.c has two
/// bodies for exactly this reason:
///
///   * non-i386 (stg_prop.c:2787): the slot is `__cdecl` with `this` as an ordinary first
///     argument, so a normal indirect call is right.
///   * i386 (stg_prop.c:2774-2783): the slot is `__thiscall` — `this` in **ecx**, callee
///     cleans the stack — while our own entry is `__stdcall` with `this` on the stack. The C
///     bridges that with a `DEFINE_STDCALL_WRAPPER` asm thunk: pop the return address, pop
///     `this` straight into ecx, push the return address back, load the vtbl, and tail-jump
///     slot 0 with `cbSize` still sitting on the stack.
///
/// Rust has no stable `extern "thiscall"`, so the i386 half is that same thunk, written as a
/// naked function (stable since 1.88). Calling the slot with the C ABI instead — passing
/// `this` on the stack where the callee expects ecx — would corrupt the stack on every call.
#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn Allocate_PMemoryAllocator(
    this: *mut c_void,
    cbSize: ULONG,
) -> *mut c_void {
    let fn_ptr: unsafe extern "C" fn(*mut c_void, ULONG) -> *mut c_void =
        **(this as *mut *mut unsafe extern "C" fn(*mut c_void, ULONG) -> *mut c_void);
    fn_ptr(this, cbSize)
}

/// i386: the `DEFINE_STDCALL_WRAPPER(0, Allocate_PMemoryAllocator, 8)` thunk, instruction
/// for instruction. See the doc comment above.
#[cfg(target_arch = "x86")]
#[unsafe(naked)]
unsafe extern "system" fn Allocate_PMemoryAllocator(
    this: *mut c_void,
    cbSize: ULONG,
) -> *mut c_void {
    core::arch::naked_asm!(
        "pop eax",             // return address
        "pop ecx",             // `this` -> ecx, the __thiscall register
        "push eax",            // put the return address back
        "mov eax, [ecx]",      // eax = this->vtbl
        "jmp dword ptr [eax]", // tail-jump vtbl slot 0; cbSize is still on the stack
    )
}

#[no_mangle]
pub unsafe extern "system" fn StgConvertPropertyToVariant(
    prop: *const SERIALIZEDPROPERTYVALUE,
    CodePage: USHORT,
    pvar: *mut PROPVARIANT,
    pma: *mut c_void,
) -> BOOLEAN {
    // wine-10: an unbounded buffer -- the caller vouches for the serialized value.
    let read_buf = read_buffer { data: prop as *mut BYTE, size: !0usize };
    let hr = PropertyStorage_ReadProperty(
        pvar,
        &read_buf,
        0,
        CodePage as UINT,
        Allocate_PMemoryAllocator,
        pma,
    );

    if FAILED(hr) {
        FIXME!(b"should raise C++ exception on failure\n\0".as_ptr() as LPCSTR);
        PropVariantInit(pvar);
    }

    0u8
}

/// stg_prop.c:2811 — a FIXME stub in the C too (returns NULL unconditionally).
/// rustconv dropped it entirely; restored here so the Rust TU covers the whole C TU.
#[no_mangle]
pub unsafe extern "system" fn StgConvertVariantToProperty(
    pvar: *const PROPVARIANT,
    CodePage: USHORT,
    pprop: *mut SERIALIZEDPROPERTYVALUE,
    pcb: *mut ULONG,
    pid: PROPID,
    fReserved: BOOLEAN,
    pcIndirect: *mut ULONG,
) -> *mut SERIALIZEDPROPERTYVALUE {
    FIXME!(
        b"%p,%d,%p,%p,%d,%d,%p\n\0".as_ptr() as LPCSTR,
        pvar,
        CodePage as core::ffi::c_uint,
        pprop,
        pcb,
        pid,
        fReserved as core::ffi::c_uint,
        pcIndirect,
    );

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn StgCreatePropStg(
    unk: *mut IUnknown,
    fmt: *const FMTID,
    _clsid: *const CLSID,
    flags: DWORD,
    reserved: DWORD,
    prop_stg: *mut *mut IPropertyStorage,
) -> HRESULT {
    let mut r: HRESULT;

    TRACE!(
        b"%p %s %s %08x %d %p\n\0".as_ptr() as LPCSTR,
        unk,
        debugstr_guid(fmt),
        debugstr_guid(clsid),
        flags,
        reserved,
        prop_stg,
    );

    'end: {
        if fmt.is_null() || reserved != 0 {
            r = E_INVALIDARG;
            break 'end;
        }

        if (flags & PROPSETFLAG_NONSIMPLE) != 0 {
            let mut stg: *mut IStorage = core::ptr::null_mut();

            r = IUnknown_QueryInterface(
                unk,
                core::ptr::addr_of!(IID_IStorage),
                core::ptr::addr_of_mut!(stg) as *mut *mut c_void,
            );
            if FAILED(r) {
                break 'end;
            }

            // FIXME: if (flags & PROPSETFLAG_NONSIMPLE), we need to create a storage,
            // not a stream.  For now, disallow it.
            FIXME!(b"PROPSETFLAG_NONSIMPLE not supported\n\0".as_ptr() as LPCSTR);
            IStorage_Release(stg);
            r = STG_E_INVALIDFLAG;
        } else {
            let mut stm: *mut IStream = core::ptr::null_mut();

            r = IUnknown_QueryInterface(
                unk,
                core::ptr::addr_of!(IID_IStream),
                core::ptr::addr_of_mut!(stm) as *mut *mut c_void,
            );
            if FAILED(r) {
                break 'end;
            }

            r = PropertyStorage_ConstructEmpty(
                stm,
                fmt,
                flags,
                STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
                prop_stg,
            );

            IStream_Release(stm);
        }
    }

    // end:
    TRACE!(b"returning 0x%08x\n\0".as_ptr() as LPCSTR, r);
    r
}

#[no_mangle]
pub unsafe extern "system" fn StgOpenPropStg(
    unk: *mut IUnknown,
    fmt: *const FMTID,
    flags: DWORD,
    reserved: DWORD,
    prop_stg: *mut *mut IPropertyStorage,
) -> HRESULT {
    let mut r: HRESULT;

    TRACE!(
        b"%p %s %08x %d %p\n\0".as_ptr() as LPCSTR,
        unk,
        debugstr_guid(fmt),
        flags,
        reserved,
        prop_stg,
    );

    'end: {
        if fmt.is_null() || reserved != 0 {
            r = E_INVALIDARG;
            break 'end;
        }

        if (flags & PROPSETFLAG_NONSIMPLE) != 0 {
            let mut stg: *mut IStorage = core::ptr::null_mut();

            r = IUnknown_QueryInterface(
                unk,
                core::ptr::addr_of!(IID_IStorage),
                core::ptr::addr_of_mut!(stg) as *mut *mut c_void,
            );
            if FAILED(r) {
                break 'end;
            }

            // FIXME: if (flags & PROPSETFLAG_NONSIMPLE), we need to open a storage,
            // not a stream.  For now, disallow it.
            FIXME!(b"PROPSETFLAG_NONSIMPLE not supported\n\0".as_ptr() as LPCSTR);
            IStorage_Release(stg);
            r = STG_E_INVALIDFLAG;
        } else {
            let mut stm: *mut IStream = core::ptr::null_mut();

            r = IUnknown_QueryInterface(
                unk,
                core::ptr::addr_of!(IID_IStream),
                core::ptr::addr_of_mut!(stm) as *mut *mut c_void,
            );
            if FAILED(r) {
                break 'end;
            }

            r = PropertyStorage_ConstructFromStream(
                stm,
                fmt,
                STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
                prop_stg,
            );

            IStream_Release(stm);
        }
    }

    // end:
    TRACE!(b"returning 0x%08x\n\0".as_ptr() as LPCSTR, r);
    r
}
