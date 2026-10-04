use crate::prelude::*;

extern "C" {
    fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_ole as *mut __wine_debug_channel };

static mut __wine_dbch_ole: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"ole\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
    }
};

#[repr(C)]
pub struct fieldstr {
    pub vt: VARENUM,
    pub varkind: VARKIND,
    pub offset: ULONG,
    pub name: BSTR,
}

pub type Pfieldstr = *mut fieldstr;

#[repr(C)]
pub struct IRecordInfoImpl {
    pub IRecordInfo_iface: IRecordInfo,
    pub ref_: LONG,
    pub guid: GUID,
    pub lib_index: UINT,
    pub n_vars: WORD,
    pub size: ULONG,
    pub name: BSTR,
    pub fields: *mut fieldstr,
    pub pTypeInfo: *mut ITypeInfo,
}

pub type PIRecordInfoImpl = *mut IRecordInfoImpl;

// oleauto.h:253-259 / :274-280 — `V_INT_PTR`/`V_UINT_PTR` are POINTER-WIDTH aliases:
//     #ifdef _WIN64   V_INT_PTR(A) = V_I8(A)   (llVal,  8 bytes)
//     #else           V_INT_PTR(A) = V_I4(A)   (lVal,   4 bytes)
// The machine translation hardcoded the _WIN64 spelling (llVal/ullVal) unconditionally,
// which on i386 reads/writes 8 bytes through a 4-byte field. Mirror the C's #ifdef.
#[cfg(target_pointer_width = "64")]
unsafe fn V_INT_PTR_ref(v: *mut VARIANT) -> *mut INT_PTR {
    V_I8_ref(v) as *mut INT_PTR
}
#[cfg(target_pointer_width = "32")]
unsafe fn V_INT_PTR_ref(v: *mut VARIANT) -> *mut INT_PTR {
    V_I4_ref(v) as *mut INT_PTR
}
#[cfg(target_pointer_width = "64")]
unsafe fn V_UINT_PTR_ref(v: *mut VARIANT) -> *mut UINT_PTR {
    V_UI8_ref(v) as *mut UINT_PTR
}
#[cfg(target_pointer_width = "32")]
unsafe fn V_UINT_PTR_ref(v: *mut VARIANT) -> *mut UINT_PTR {
    V_UI4_ref(v) as *mut UINT_PTR
}

unsafe extern "C" fn impl_from_IRecordInfo(iface: *mut IRecordInfo) -> *mut IRecordInfoImpl {
    return (iface as UINT_PTR).wrapping_sub(
        core::mem::offset_of!(IRecordInfoImpl, IRecordInfo_iface) as UINT_PTR
    ) as *mut IRecordInfoImpl;
}
unsafe extern "C" fn copy_to_variant(
    src: *mut core::ffi::c_void,
    pvar: *mut VARIANT,
    vt: VARENUM,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/recinfo.c\0"
                .as_ptr() as *const i8,
            b"copy_to_variant\0".as_ptr() as *const i8,
            62i32,
            b"%p %p %d\n\0".as_ptr() as *const i8,
            src,
            pvar,
            vt as u32,
        );
    }

    match vt as u32 {
        2 => {
            core::ptr::copy_nonoverlapping(
                src as *const SHORT,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.iVal),
                1,
            );
        }
        3 => {
            core::ptr::copy_nonoverlapping(
                src as *const LONG,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.lVal),
                1,
            );
        }
        4 => {
            core::ptr::copy_nonoverlapping(
                src as *const FLOAT,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.fltVal),
                1,
            );
        }
        5 => {
            core::ptr::copy_nonoverlapping(
                src as *const DOUBLE,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.dblVal),
                1,
            );
        }
        6 => {
            core::ptr::copy_nonoverlapping(
                src as *const CY,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.cyVal),
                1,
            );
        }
        7 => {
            core::ptr::copy_nonoverlapping(
                src as *const DATE,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.date),
                1,
            );
        }
        8 => {
            core::ptr::copy_nonoverlapping(
                src as *const BSTR,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.bstrVal),
                1,
            );
        }
        10 => {
            core::ptr::copy_nonoverlapping(
                src as *const SCODE,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.scode),
                1,
            );
        }
        11 => {
            core::ptr::copy_nonoverlapping(
                src as *const VARIANT_BOOL,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.boolVal),
                1,
            );
        }
        14 => {
            core::ptr::copy_nonoverlapping(
                src as *const DECIMAL,
                core::ptr::addr_of_mut!((*pvar).n1.decVal),
                1,
            );
        }
        16 => {
            core::ptr::copy_nonoverlapping(
                src as *const i8,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.cVal),
                1,
            );
        }
        17 => {
            core::ptr::copy_nonoverlapping(
                src as *const BYTE,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.bVal),
                1,
            );
        }
        18 => {
            core::ptr::copy_nonoverlapping(
                src as *const USHORT,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.uiVal),
                1,
            );
        }
        19 => {
            core::ptr::copy_nonoverlapping(
                src as *const ULONG,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.ulVal),
                1,
            );
        }
        20 => {
            core::ptr::copy_nonoverlapping(
                src as *const LONGLONG,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.llVal),
                1,
            );
        }
        21 => {
            core::ptr::copy_nonoverlapping(
                src as *const ULONGLONG,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.ullVal),
                1,
            );
        }
        22 => {
            core::ptr::copy_nonoverlapping(
                src as *const INT,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.intVal),
                1,
            );
        }
        23 => {
            core::ptr::copy_nonoverlapping(
                src as *const UINT,
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3.uintVal),
                1,
            );
        }
        37 => {
            core::ptr::copy_nonoverlapping(
                src as *const INT_PTR,
                V_INT_PTR_ref(pvar),
                1,
            );
        }
        38 => {
            core::ptr::copy_nonoverlapping(
                src as *const UINT_PTR,
                V_UINT_PTR_ref(pvar),
                1,
            );
        }
        _ => {
            if (*__wine_dbch___default).flags as i32
                & (1i32) << __WINE_DBCL_FIXME as i32
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/oleaut32/recinfo.c\0"
                        .as_ptr() as *const i8,
                    b"copy_to_variant\0".as_ptr() as *const i8,
                    91i32,
                    b"Not supported type: %d\n\0".as_ptr() as *const i8,
                    vt as u32,
                );
            }
            return E_NOTIMPL;
        }
    }

    (*pvar).n1.n2.vt = vt as VARTYPE;
    S_OK
}
unsafe extern "C" fn copy_from_variant(
    src: *mut VARIANT,
    dest: *mut core::ffi::c_void,
    vt: VARENUM,
) -> HRESULT {
    let mut var: VARIANT = core::mem::zeroed();
    let hres: HRESULT;

    TRACE!(
        b"(%p(%d) %p %d)\n\0".as_ptr() as *const i8,
        src,
        V_VT(src),
        dest,
        vt as u32,
    );

    hres = VariantChangeType(
        core::ptr::addr_of_mut!(var),
        src,
        0u16,
        vt as u16,
    );
    if FAILED(hres) {
        return hres;
    }

    // C: `memcpy(dest, &V_X(&var), sizeof(V_X(&var)))`. `var` is a LOCAL VARIANT *value*,
    // so the C takes the address of a field of it. The machine translation emitted
    // `(*var.n1.n2.n3).X`, dereferencing a struct/union value (E0614) — the correct
    // spelling of `&V_X(&var)` is the crate's `V_X_ref()` lvalue accessor.
    let pvar: *mut VARIANT = &raw mut var;

    match vt as u32 {
        2 => {
            core::ptr::copy_nonoverlapping(
                V_I2_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<SHORT>(),
            );
        }
        3 => {
            core::ptr::copy_nonoverlapping(
                V_I4_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<LONG>(),
            );
        }
        4 => {
            core::ptr::copy_nonoverlapping(
                V_R4_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<FLOAT>(),
            );
        }
        5 => {
            core::ptr::copy_nonoverlapping(
                V_R8_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<DOUBLE>(),
            );
        }
        6 => {
            core::ptr::copy_nonoverlapping(
                V_CY_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<CY>(),
            );
        }
        7 => {
            core::ptr::copy_nonoverlapping(
                V_DATE_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<DATE>(),
            );
        }
        8 => {
            core::ptr::copy_nonoverlapping(
                V_BSTR_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<BSTR>(),
            );
        }
        10 => {
            core::ptr::copy_nonoverlapping(
                V_ERROR_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<SCODE>(),
            );
        }
        11 => {
            core::ptr::copy_nonoverlapping(
                V_BOOL_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<VARIANT_BOOL>(),
            );
        }
        14 => {
            core::ptr::copy_nonoverlapping(
                V_DECIMAL_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<DECIMAL>(),
            );
        }
        16 => {
            core::ptr::copy_nonoverlapping(
                V_I1_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<::core::ffi::c_schar>(),
            );
        }
        17 => {
            core::ptr::copy_nonoverlapping(
                V_UI1_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<BYTE>(),
            );
        }
        18 => {
            core::ptr::copy_nonoverlapping(
                V_UI2_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<USHORT>(),
            );
        }
        19 => {
            core::ptr::copy_nonoverlapping(
                V_UI4_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<ULONG>(),
            );
        }
        20 => {
            core::ptr::copy_nonoverlapping(
                V_I8_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<LONGLONG>(),
            );
        }
        21 => {
            core::ptr::copy_nonoverlapping(
                V_UI8_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<ULONGLONG>(),
            );
        }
        22 => {
            core::ptr::copy_nonoverlapping(
                V_INT_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<INT>(),
            );
        }
        23 => {
            core::ptr::copy_nonoverlapping(
                V_UINT_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<UINT>(),
            );
        }
        37 => {
            core::ptr::copy_nonoverlapping(
                V_INT_PTR_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<INT_PTR>(),
            );
        }
        38 => {
            core::ptr::copy_nonoverlapping(
                V_UINT_PTR_ref(pvar) as *const core::ffi::c_void,
                dest,
                core::mem::size_of::<UINT_PTR>(),
            );
        }
        _ => {
            FIXME!(
                b"Not supported type: %d\n\0".as_ptr() as *const i8,
                V_VT(pvar) as i32,
            );
            return E_NOTIMPL;
        }
    }
    S_OK
}
stdmethod! {
fn IRecordInfoImpl_QueryInterface(
    iface: *mut IRecordInfo,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s %p)\n\0".as_ptr() as *const i8,
        iface,
        debugstr_guid(riid),
        ppvObject,
    );

    *ppvObject = core::ptr::null_mut();

    if IsEqualGUID(&IID_IUnknown as *const GUID, riid)
        || IsEqualGUID(&IID_IRecordInfo as *const GUID, riid)
    {
        *ppvObject = iface as *mut core::ffi::c_void;
        IRecordInfo_AddRef(iface);
        return S_OK;
    }

    FIXME!(
        b"Not supported interface: %s\n\0".as_ptr() as *const i8,
        debugstr_guid(riid),
    );
    return E_NOINTERFACE;
}
}
stdmethod! {
fn IRecordInfoImpl_AddRef(iface: *mut IRecordInfo) -> ULONG {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let ref_0: ULONG = InterlockedIncrement(&mut (*This).ref_) as ULONG;
    TRACE!("(%p) -> %d\n", This, ref_0);
    return ref_0;
}
}
stdmethod! {
fn IRecordInfoImpl_Release(iface: *mut IRecordInfo) -> ULONG {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let ref_count: ULONG = InterlockedDecrement(&mut (*This).ref_) as ULONG;

    TRACE!(
        b"(%p) -> %d\n\0".as_ptr() as *const i8,
        This,
        ref_count,
    );

    if ref_count == 0 {
        let mut i: i32 = 0;
        while i < (*This).n_vars as i32 {
            SysFreeString((*(*This).fields.offset(i as isize)).name);
            i += 1;
        }
        SysFreeString((*This).name);
        HeapFree(GetProcessHeap(), 0, (*This).fields as PVOID);
        ITypeInfo_Release((*This).pTypeInfo);
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    ref_count
}
}
stdmethod! {
fn IRecordInfoImpl_RecordInit(iface: *mut IRecordInfo, pvNew: PVOID) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/recinfo.c\0"
                as *const u8 as *const i8,
            b"IRecordInfoImpl_RecordInit\0" as *const u8 as *const i8,
            192 as i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const i8,
            This,
            pvNew,
        );
    }
    
    if pvNew.is_null() {
        return E_INVALIDARG;
    }
    
    memset(
        pvNew as *mut core::ffi::c_void,
        0 as i32,
        (*This).size as usize,
    );
    
    return S_OK;
}
}
stdmethod! {
fn IRecordInfoImpl_RecordClear(
    iface: *mut IRecordInfo,
    pvExisting: PVOID,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut i: i32;
    let mut var: PVOID;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pvExisting);

    if pvExisting.is_null() {
        return E_INVALIDARG;
    }

    i = 0;
    while i < (*This).n_vars as i32 {
        if (*(*This).fields.offset(i as isize)).varkind as u32 != VAR_PERINSTANCE as u32 {
            ERR!(b"varkind != VAR_PERINSTANCE\n\0".as_ptr() as *const i8);
        } else {
            var = (pvExisting as PBYTE).offset((*(*This).fields.offset(i as isize)).offset as isize) as PVOID;
            
            match (*(*This).fields.offset(i as isize)).vt as u32 {
                8 => {
                    SysFreeString(*(var as *mut BSTR));
                    *(var as *mut BSTR) = core::ptr::null_mut();
                }
                2 | 3 | 4 | 5 | 6 | 7 | 10 | 11 | 14 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 25 => {}
                37 | 38 => {
                    *(var as *mut *mut core::ffi::c_void) = core::ptr::null_mut();
                }
                27 => {
                    SafeArrayDestroy(var as *mut SAFEARRAY);
                }
                13 | 9 => {
                    let unk: *mut IUnknown = *(var as *mut *mut IUnknown);
                    if !unk.is_null() {
                        IUnknown_Release(unk);
                    }
                    *(var as *mut *mut core::ffi::c_void) = core::ptr::null_mut();
                }
                _ => {
                    FIXME!(b"Not supported vt = %d\n\0".as_ptr() as *const i8,
                          (*(*This).fields.offset(i as isize)).vt as u32);
                }
            }
        }
        i += 1;
    }

    S_OK
}
}
stdmethod! {
fn IRecordInfoImpl_RecordCopy(
    iface: *mut IRecordInfo,
    src_rec: *mut core::ffi::c_void,
    dest_rec: *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut hr: HRESULT = S_OK;
    let mut i: i32;

    TRACE!(
        b"(%p)->(%p %p)\n\0".as_ptr() as *const i8,
        This,
        src_rec,
        dest_rec,
    );

    if src_rec.is_null() || dest_rec.is_null() {
        return E_INVALIDARG;
    }

    IRecordInfo_RecordClear(iface, dest_rec);

    i = 0;
    while i < (*This).n_vars as i32 {
        let src: *mut core::ffi::c_void;
        let dest: *mut core::ffi::c_void;

        if (*(*This).fields.offset(i as isize)).varkind as u32 != VAR_PERINSTANCE as u32 {
            ERR!(b"varkind != VAR_PERINSTANCE\n\0".as_ptr() as *const i8);
            i += 1;
            continue;
        }

        // C: `((BYTE*)src_rec) + offset` -- BYTE*, not PBYTE*. Through `*mut PBYTE` the
        // offset stepped in POINTER-sized units, so every field past offset 0 was read
        // from (and the BSTR copied out of) 8x the right address on 64-bit.
        src = (src_rec as *mut BYTE)
            .offset((*(*This).fields.offset(i as isize)).offset as isize)
            as *mut core::ffi::c_void;
        dest = (dest_rec as *mut BYTE)
            .offset((*(*This).fields.offset(i as isize)).offset as isize)
            as *mut core::ffi::c_void;

        match (*(*This).fields.offset(i as isize)).vt as u32 {
            8 => {
                let src_str: BSTR = *(src as *mut BSTR);

                if !src_str.is_null() {
                    let str: BSTR = SysAllocString(*(src as *mut BSTR));
                    if str.is_null() {
                        hr = E_OUTOFMEMORY;
                    }

                    *(dest as *mut BSTR) = str;
                } else {
                    *(dest as *mut BSTR) = core::ptr::null_mut();
                }
            }
            13 | 9 => {
                let unk: *mut IUnknown = *(src as *mut *mut IUnknown);
                *(dest as *mut *mut IUnknown) = unk;
                if !unk.is_null() {
                    IUnknown_AddRef(unk);
                }
            }
            27 => {
                // C: `SafeArrayCopy(src, dest)` — `src`/`dest` are `void*` and convert
                // implicitly to `SAFEARRAY*` / `SAFEARRAY**`. Spell the conversions out.
                hr = SafeArrayCopy(src as *mut SAFEARRAY, dest as *mut *mut SAFEARRAY);
            }
            _ => {
                // get_type_size() takes a VARTYPE (u16); `fields[i].vt` is VARENUM (u32).
                // The C narrowed implicitly at the call.
                let len: i32 = get_type_size(
                    core::ptr::null_mut(),
                    (*(*This).fields.offset(i as isize)).vt as VARTYPE,
                ) as i32;
                memcpy(dest, src, len as usize);
            }
        }

        if FAILED(hr) {
            break;
        }

        i += 1;
    }

    if FAILED(hr) {
        IRecordInfo_RecordClear(iface, dest_rec);
    }

    return hr;
}
}
stdmethod! {
fn IRecordInfoImpl_GetGuid(
    iface: *mut IRecordInfo,
    pguid: *mut GUID,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pguid);
    
    if pguid.is_null() {
        return E_INVALIDARG;
    }
    
    *pguid = (*This).guid;
    S_OK
}
}
stdmethod! {
fn IRecordInfoImpl_GetName(
    iface: *mut IRecordInfo,
    pbstrName: *mut BSTR,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);

    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const i8,
        This,
        pbstrName,
    );

    if pbstrName.is_null() {
        return E_INVALIDARG;
    }

    *pbstrName = SysAllocString((*This).name as *const OLECHAR);
    return S_OK;
}
}
stdmethod! {
fn IRecordInfoImpl_GetSize(
    iface: *mut IRecordInfo,
    pcbSize: *mut ULONG,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pcbSize);

    if pcbSize.is_null() {
        return E_INVALIDARG;
    }

    *pcbSize = (*This).size;
    S_OK
}
}
stdmethod! {
fn IRecordInfoImpl_GetTypeInfo(
    iface: *mut IRecordInfo,
    ppTypeInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/recinfo.c\0"
                as *const u8 as *const i8,
            b"IRecordInfoImpl_GetTypeInfo\0" as *const u8 as *const i8,
            381i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const i8,
            This,
            ppTypeInfo,
        );
    }
    
    if ppTypeInfo.is_null() {
        return E_INVALIDARG;
    }
    
    ITypeInfo_AddRef((*This).pTypeInfo);
    *ppTypeInfo = (*This).pTypeInfo;
    
    S_OK
}
}
stdmethod! {
fn IRecordInfoImpl_GetField(
    iface: *mut IRecordInfo,
    pvData: PVOID,
    szFieldName: LPCOLESTR,
    pvarField: *mut VARIANT,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut i: i32 = 0;

    TRACE!(
        b"(%p)->(%p %s %p)\n\0".as_ptr() as *const i8,
        This,
        pvData,
        debugstr_w(szFieldName as *const WCHAR),
        pvarField,
    );

    if pvData.is_null() || szFieldName.is_null() || pvarField.is_null() {
        return E_INVALIDARG;
    }

    i = 0;
    while i < (*This).n_vars as i32 {
        if wcscmp(
            (*(*This).fields.offset(i as isize)).name as *const wchar_t,
            szFieldName as *const wchar_t,
        ) == 0
        {
            break;
        }
        i += 1;
    }
    if i == (*This).n_vars as i32 {
        return TYPE_E_FIELDNOTFOUND;
    }

    VariantClear(pvarField);
    return copy_to_variant(
        (pvData as PBYTE).offset((*(*This).fields.offset(i as isize)).offset as isize)
            as *mut core::ffi::c_void,
        pvarField,
        (*(*This).fields.offset(i as isize)).vt,
    );
}
}
stdmethod! {
fn IRecordInfoImpl_GetFieldNoCopy(
    iface: *mut IRecordInfo,
    pvData: PVOID,
    szFieldName: LPCOLESTR,
    pvarField: *mut VARIANT,
    ppvDataCArray: *mut PVOID,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut i: i32;

    TRACE!(
        b"(%p)->(%p %s %p %p)\n\0".as_ptr(),
        This,
        pvData,
        debugstr_w(szFieldName),
        pvarField,
        ppvDataCArray,
    );

    if pvData.is_null() || szFieldName.is_null() || pvarField.is_null() {
        return E_INVALIDARG;
    }

    i = 0;
    while i < (*This).n_vars as i32 {
        if wcscmp((*(*This).fields.offset(i as isize)).name, szFieldName) == 0 {
            break;
        }
        i += 1;
    }
    if i == (*This).n_vars as i32 {
        return TYPE_E_FIELDNOTFOUND;
    }

    VariantClear(pvarField);
    // C: `V_VT(pvarField) = VT_BYREF|This->fields[i].vt;` — VT_BYREF is a VARTYPE (u16);
    // `fields[i].vt` is VARENUM (u32) and narrows to VARTYPE at the assignment.
    (*pvarField).n1.n2.vt = VT_BYREF | (*(*This).fields.offset(i as isize)).vt as VARTYPE;
    (*pvarField).n1.n2.n3.byref =
        (pvData as PBYTE).offset((*(*This).fields.offset(i as isize)).offset as isize) as PVOID;
    *ppvDataCArray = core::ptr::null_mut();
    return S_OK;
}
}
stdmethod! {
fn IRecordInfoImpl_PutField(
    iface: *mut IRecordInfo,
    wFlags: ULONG,
    pvData: PVOID,
    szFieldName: LPCOLESTR,
    pvarField: *mut VARIANT,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut i: i32;

    TRACE!(
        b"(%p)->(%08x %p %s %p)\n\0".as_ptr() as *const i8,
        This,
        wFlags,
        pvData,
        debugstr_w(szFieldName as *const WCHAR),
        pvarField,
    );

    if pvData.is_null()
        || szFieldName.is_null()
        || pvarField.is_null()
        || (wFlags != INVOKE_PROPERTYPUTREF && wFlags != INVOKE_PROPERTYPUT)
    {
        return E_INVALIDARG;
    }

    if wFlags == INVOKE_PROPERTYPUTREF {
        FIXME!(b"wFlag == INVOKE_PROPERTYPUTREF not supported\n\0".as_ptr() as *const i8);
        return E_NOTIMPL;
    }

    i = 0;
    while i < (*This).n_vars as i32 {
        if wcscmp(
            (*(*This).fields.offset(i as isize)).name as *const wchar_t,
            szFieldName as *const wchar_t,
        ) == 0
        {
            break;
        }
        i += 1;
    }
    if i == (*This).n_vars as i32 {
        return TYPE_E_FIELDNOTFOUND;
    }

    return copy_from_variant(
        pvarField,
        (pvData as PBYTE).offset((*(*This).fields.offset(i as isize)).offset as isize)
            as *mut core::ffi::c_void,
        (*(*This).fields.offset(i as isize)).vt,
    );
}
}
stdmethod! {
fn IRecordInfoImpl_PutFieldNoCopy(
    iface: *mut IRecordInfo,
    wFlags: ULONG,
    pvData: PVOID,
    szFieldName: LPCOLESTR,
    pvarField: *mut VARIANT,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut i: i32;

    FIXME!(
        b"(%p)->(%08x %p %s %p) stub\n\0".as_ptr(),
        This,
        wFlags,
        pvData,
        debugstr_w(szFieldName as *const u16),
        pvarField,
    );

    if pvData.is_null()
        || szFieldName.is_null()
        || pvarField.is_null()
        || (wFlags != INVOKE_PROPERTYPUTREF && wFlags != INVOKE_PROPERTYPUT)
    {
        return E_INVALIDARG;
    }

    i = 0;
    while i < (*This).n_vars as i32 {
        if wcscmp((*(*This).fields.offset(i as isize)).name as *const u16, szFieldName as *const u16) == 0 {
            break;
        }
        i += 1;
    }

    if i == (*This).n_vars as i32 {
        return TYPE_E_FIELDNOTFOUND;
    }

    E_NOTIMPL
}
}
stdmethod! {
fn IRecordInfoImpl_GetFieldNames(
    iface: *mut IRecordInfo,
    pcNames: *mut ULONG,
    rgBstrNames: *mut BSTR,
) -> HRESULT {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut n: ULONG = (*This).n_vars as ULONG;
    let mut i: ULONG;

    TRACE!(b"(%p)->(%p %p)\n\0".as_ptr() as *const i8, This, pcNames, rgBstrNames);

    if pcNames.is_null() {
        return E_INVALIDARG;
    }

    if *pcNames < n {
        n = *pcNames;
    }

    if !rgBstrNames.is_null() {
        i = 0 as ULONG;
        while i < n {
            let ref mut fresh4 = *rgBstrNames.offset(i as isize);
            *fresh4 = SysAllocString((*(*This).fields.offset(i as isize)).name as *const OLECHAR);
            i = i.wrapping_add(1);
        }
    }

    *pcNames = n;
    return S_OK;
}
}
stdmethod! {
fn IRecordInfoImpl_IsMatchingType(
    iface: *mut IRecordInfo,
    info2: *mut IRecordInfo,
) -> BOOL {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let mut guid2: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };

    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const i8,
        This,
        info2,
    );

    IRecordInfo_GetGuid(info2, &mut guid2);
    // C returns BOOL (an int); IsEqualGUID() is `bool` in this crate.
    return IsEqualGUID(&(*This).guid, &guid2) as BOOL;
}
}
stdmethod! {
fn IRecordInfoImpl_RecordCreate(iface: *mut IRecordInfo) -> PVOID {
    let This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let record: *mut core::ffi::c_void;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    // Wine 10.0: records live in COM task memory, so a record created here can
    // be freed by any party that frees COM memory (and RecordDestroy below).
    record = CoTaskMemAlloc((*This).size as SIZE_T);
    IRecordInfo_RecordInit(iface, record as PVOID);
    TRACE!(b"created record at %p\n\0".as_ptr() as *const i8, record);
    record as PVOID
}
}
stdmethod! {
fn IRecordInfoImpl_RecordCreateCopy(
    iface: *mut IRecordInfo,
    pvSource: PVOID,
    ppvDest: *mut PVOID,
) -> HRESULT {
    let _This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);

    TRACE!(b"(%p)->(%p %p)\n\0".as_ptr() as *const i8, This, pvSource, ppvDest);

    if pvSource.is_null() || ppvDest.is_null() {
        return E_INVALIDARG;
    }

    *ppvDest = IRecordInfo_RecordCreate(iface);
    IRecordInfo_RecordCopy(iface, pvSource, *ppvDest)
}
}
stdmethod! {
fn IRecordInfoImpl_RecordDestroy(
    iface: *mut IRecordInfo,
    pvRecord: PVOID,
) -> HRESULT {
    let _This: *mut IRecordInfoImpl = impl_from_IRecordInfo(iface);
    let hres: HRESULT;

    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const ::core::ffi::c_char,
        This,
        pvRecord,
    );

    hres = IRecordInfo_RecordClear(iface, pvRecord);
    if FAILED(hres) {
        return hres;
    }

    CoTaskMemFree(pvRecord);
    return S_OK;
}
}

#[no_mangle]
pub unsafe extern "system" fn GetRecordInfoFromGuids(
    rGuidTypeLib: *const GUID,
    uVerMajor: ULONG,
    uVerMinor: ULONG,
    lcid: LCID,
    rGuidTypeInfo: *const GUID,
    ppRecInfo: *mut *mut IRecordInfo,
) -> HRESULT {
    let mut pTypeInfo: *mut ITypeInfo = core::ptr::null_mut::<ITypeInfo>();
    let mut pTypeLib: *mut ITypeLib = core::ptr::null_mut::<ITypeLib>();
    let mut hres: HRESULT = 0;
    
    TRACE!(
        b"(%p,%d,%d,%d,%s,%p)\n\0".as_ptr() as *const i8,
        rGuidTypeLib,
        uVerMajor,
        uVerMinor,
        lcid,
        debugstr_guid(rGuidTypeInfo),
        ppRecInfo,
    );

    // The crate exposes two ABI-identical shims of `GUID`/`ITypeLib` (com_sys vs olesys);
    // LoadRegTypeLib is typed against olesys' while this module (and the ITypeLib_*
    // accessors below) use com_sys'. Bridge with raw-pointer casts — same underlying
    // types, just a translation-surface split. Same pattern as oleaut.rs:748.
    hres = LoadRegTypeLib(
        rGuidTypeLib as *const crate::olesys::GUID,
        uVerMajor as WORD,
        uVerMinor as WORD,
        lcid,
        &raw mut pTypeLib as *mut *mut crate::olesys::ITypeLib,
    );
    
    if FAILED(hres) {
        WARN!(b"LoadRegTypeLib failed!\n\0".as_ptr() as *const i8);
        return hres;
    }

    hres = ITypeLib_GetTypeInfoOfGuid(pTypeLib, rGuidTypeInfo, &raw mut pTypeInfo);
    ITypeLib_Release(pTypeLib);
    
    if FAILED(hres) {
        WARN!(b"GetTypeInfoOfGuid failed!\n\0".as_ptr() as *const i8);
        return hres;
    }

    hres = GetRecordInfoFromTypeInfo(pTypeInfo, ppRecInfo);
    ITypeInfo_Release(pTypeInfo);
    return hres;
}

#[no_mangle]
pub unsafe extern "system" fn GetRecordInfoFromTypeInfo(
    pTI: *mut ITypeInfo,
    ppRecInfo: *mut *mut IRecordInfo,
) -> HRESULT {
    let mut hres: HRESULT;
    let mut typeattr: *mut TYPEATTR = core::ptr::null_mut();
    let ret: *mut IRecordInfoImpl;
    // C leaves this uninitialized; it is filled either by the GetRefTypeInfo out-param
    // (alias path) or by `pTypeInfo = pTI` (else path). rustc does not treat a write
    // through `&raw mut` as an initialization, so seed it — same as `typeattr` above.
    let mut pTypeInfo: *mut ITypeInfo = core::ptr::null_mut();
    let mut i: ::core::ffi::c_int;
    let guid: GUID;

    TRACE!(b"(%p %p)\n\0".as_ptr() as *const ::core::ffi::c_char, pTI, ppRecInfo);

    if pTI.is_null() || ppRecInfo.is_null() {
        return E_INVALIDARG;
    }

    hres = ITypeInfo_GetTypeAttr(pTI, &raw mut typeattr);
    if FAILED(hres) || typeattr.is_null() {
        WARN!(b"GetTypeAttr failed: %08x\n\0".as_ptr() as *const ::core::ffi::c_char, hres);
        return hres;
    }

    if (*typeattr).typekind as ::core::ffi::c_uint == TKIND_ALIAS as ::core::ffi::c_int as ::core::ffi::c_uint {
        hres = ITypeInfo_GetRefTypeInfo(
            pTI,
            (*typeattr).tdescAlias.__bindgen_anon_1.hreftype,
            &raw mut pTypeInfo,
        );
        guid = (*typeattr).guid;
        ITypeInfo_ReleaseTypeAttr(pTI, typeattr);
        if FAILED(hres) {
            WARN!(b"GetRefTypeInfo failed: %08x\n\0".as_ptr() as *const ::core::ffi::c_char, hres);
            return hres;
        }
        hres = ITypeInfo_GetTypeAttr(pTypeInfo, &raw mut typeattr);
        if FAILED(hres) {
            ITypeInfo_Release(pTypeInfo);
            WARN!(b"GetTypeAttr failed for referenced type: %08x\n\0".as_ptr() as *const ::core::ffi::c_char, hres);
            return hres;
        }
    } else {
        pTypeInfo = pTI;
        ITypeInfo_AddRef(pTypeInfo);
        guid = (*typeattr).guid;
    }

    if (*typeattr).typekind as ::core::ffi::c_uint != TKIND_RECORD as ::core::ffi::c_int as ::core::ffi::c_uint {
        WARN!(b"typekind != TKIND_RECORD\n\0".as_ptr() as *const ::core::ffi::c_char);
        ITypeInfo_ReleaseTypeAttr(pTypeInfo, typeattr);
        ITypeInfo_Release(pTypeInfo);
        return E_INVALIDARG;
    }

    // Zeroed, as Wine 10.0's calloc: a field whose GetVarDesc fails is skipped
    // below, and IRecordInfoImpl_Release SysFreeString's every fields[i].name,
    // so an unzeroed slot freed a garbage pointer.
    ret = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IRecordInfoImpl>() as SIZE_T,
    ) as *mut IRecordInfoImpl;
    // C: `ret->IRecordInfo_iface.lpVtbl = &IRecordInfoImplVtbl;` — the C field is
    // `CONST_VTBL IRecordInfoVtbl *`, but bindgen spells every COM `lpVtbl` as `*mut`.
    // The vtbl is never written through; the cast is the standard bindgen bridge.
    (*ret).IRecordInfo_iface.lpVtbl = &raw const IRecordInfoImplVtbl as *mut IRecordInfoVtbl;
    (*ret).ref_ = 1 as LONG;
    (*ret).pTypeInfo = pTypeInfo;
    (*ret).n_vars = (*typeattr).cVars;
    (*ret).size = (*typeattr).cbSizeInstance;
    ITypeInfo_ReleaseTypeAttr(pTypeInfo, typeattr);

    (*ret).guid = guid;

    hres = ITypeInfo_GetDocumentation(
        pTypeInfo,
        MEMBERID_NIL,
        &raw mut (*ret).name,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if FAILED(hres) {
        WARN!(b"ITypeInfo::GetDocumentation failed\n\0".as_ptr() as *const ::core::ffi::c_char);
        (*ret).name = core::ptr::null_mut();
    }

    (*ret).fields = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        ((*ret).n_vars as SIZE_T).wrapping_mul(core::mem::size_of::<fieldstr>() as SIZE_T),
    ) as *mut fieldstr;
    i = 0;
    while i < (*ret).n_vars as ::core::ffi::c_int {
        let mut vardesc: *mut VARDESC = core::ptr::null_mut();
        hres = ITypeInfo_GetVarDesc(pTypeInfo, i as UINT, &raw mut vardesc);
        if FAILED(hres) {
            WARN!(b"GetVarDesc failed\n\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            (*(*ret).fields.offset(i as isize)).vt = (*vardesc).elemdescVar.tdesc.vt as VARENUM;
            (*(*ret).fields.offset(i as isize)).varkind = (*vardesc).varkind;
            (*(*ret).fields.offset(i as isize)).offset = (*vardesc).__bindgen_anon_1.oInst;
            hres = ITypeInfo_GetDocumentation(
                pTypeInfo,
                (*vardesc).memid,
                &raw mut (*(*ret).fields.offset(i as isize)).name,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if FAILED(hres) {
                WARN!(b"GetDocumentation failed: %08x\n\0".as_ptr() as *const ::core::ffi::c_char, hres);
            }
            TRACE!(
                b"field=%s, offset=%d\n\0".as_ptr() as *const ::core::ffi::c_char,
                debugstr_w((*(*ret).fields.offset(i as isize)).name as *const WCHAR),
                (*(*ret).fields.offset(i as isize)).offset,
            );
            ITypeInfo_ReleaseVarDesc(pTypeInfo, vardesc);
        }
        i += 1;
    }

    *ppRecInfo = &raw mut (*ret).IRecordInfo_iface;

    S_OK
}

// COM vtable tables — generated from recinfo.c by vtbl_gen.py
/// recinfo.c:565 `static const IRecordInfoVtbl IRecordInfoImplVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static IRecordInfoImplVtbl: IRecordInfoVtbl = IRecordInfoVtbl {
    QueryInterface: Some(IRecordInfoImpl_QueryInterface),
    AddRef: Some(IRecordInfoImpl_AddRef),
    Release: Some(IRecordInfoImpl_Release),
    RecordInit: Some(IRecordInfoImpl_RecordInit),
    RecordClear: Some(IRecordInfoImpl_RecordClear),
    RecordCopy: Some(IRecordInfoImpl_RecordCopy),
    GetGuid: Some(IRecordInfoImpl_GetGuid),
    GetName: Some(IRecordInfoImpl_GetName),
    GetSize: Some(IRecordInfoImpl_GetSize),
    GetTypeInfo: Some(IRecordInfoImpl_GetTypeInfo),
    GetField: Some(IRecordInfoImpl_GetField),
    GetFieldNoCopy: Some(IRecordInfoImpl_GetFieldNoCopy),
    PutField: Some(IRecordInfoImpl_PutField),
    PutFieldNoCopy: Some(IRecordInfoImpl_PutFieldNoCopy),
    GetFieldNames: Some(IRecordInfoImpl_GetFieldNames),
    IsMatchingType: Some(IRecordInfoImpl_IsMatchingType),
    RecordCreate: Some(IRecordInfoImpl_RecordCreate),
    RecordCreateCopy: Some(IRecordInfoImpl_RecordCreateCopy),
    RecordDestroy: Some(IRecordInfoImpl_RecordDestroy),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2005 Jacek Caban
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <stdarg.h>
//
// #define COBJMACROS
// #define NONAMELESSUNION
//
// #include "windef.h"
// #include "winbase.h"
// #include "objbase.h"
// #include "oaidl.h"
// #include "oleauto.h"
// #include "variant.h"
//
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(ole);
//
// typedef struct {
//     enum VARENUM vt;
//     VARKIND varkind;
//     ULONG offset;
//     BSTR name;
// } fieldstr;
//
// typedef struct {
//     IRecordInfo IRecordInfo_iface;
//     LONG ref;
//
//     GUID guid;
//     UINT lib_index;
//     WORD n_vars;
//     ULONG size;
//     BSTR name;
//     fieldstr *fields;
//     ITypeInfo *pTypeInfo;
// } IRecordInfoImpl;
//
// static inline IRecordInfoImpl *impl_from_IRecordInfo(IRecordInfo *iface)
// {
//     return CONTAINING_RECORD(iface, IRecordInfoImpl, IRecordInfo_iface);
// }
//
// static HRESULT copy_to_variant(void *src, VARIANT *pvar, enum VARENUM vt)
// {
//     TRACE("%p %p %d\n", src, pvar, vt);
//
// #define CASE_COPY(x) \
//     case VT_ ## x: \
//         memcpy(&V_ ## x(pvar), src, sizeof(V_ ## x(pvar))); \
//         break 
//
//     switch(vt) {
//         CASE_COPY(I2);
//         CASE_COPY(I4);
//         CASE_COPY(R4);
//         CASE_COPY(R8);
//         CASE_COPY(CY);
//         CASE_COPY(DATE);
//         CASE_COPY(BSTR);
//         CASE_COPY(ERROR);
//         CASE_COPY(BOOL);
//         CASE_COPY(DECIMAL);
//         CASE_COPY(I1);
//         CASE_COPY(UI1);
//         CASE_COPY(UI2);
//         CASE_COPY(UI4);
//         CASE_COPY(I8);
//         CASE_COPY(UI8);
//         CASE_COPY(INT);
//         CASE_COPY(UINT);
//         CASE_COPY(INT_PTR);
//         CASE_COPY(UINT_PTR);
//     default:
//         FIXME("Not supported type: %d\n", vt);
//         return E_NOTIMPL;
//     };
// #undef CASE_COPY
//
//     V_VT(pvar) = vt;
//     return S_OK;
// }
//
// static HRESULT copy_from_variant(VARIANT *src, void *dest, enum VARENUM vt)
// {
//     VARIANT var;
//     HRESULT hres;
//
//     TRACE("(%p(%d) %p %d)\n", src, V_VT(src), dest, vt);
//
//     hres = VariantChangeType(&var, src, 0, vt);
//     if(FAILED(hres))
//         return hres;
//
// #define CASE_COPY(x) \
//     case VT_ ## x: \
//         memcpy(dest, &V_ ## x(&var), sizeof(V_ ## x(&var))); \
//         break
//
//     switch(vt) {
//         CASE_COPY(I2);
//         CASE_COPY(I4);
//         CASE_COPY(R4);
//         CASE_COPY(R8);
//         CASE_COPY(CY);
//         CASE_COPY(DATE);
//         CASE_COPY(BSTR);
//         CASE_COPY(ERROR);
//         CASE_COPY(BOOL);
//         CASE_COPY(DECIMAL);
//         CASE_COPY(I1);
//         CASE_COPY(UI1);
//         CASE_COPY(UI2);
//         CASE_COPY(UI4);
//         CASE_COPY(I8);
//         CASE_COPY(UI8);
//         CASE_COPY(INT);
//         CASE_COPY(UINT);
//         CASE_COPY(INT_PTR);
//         CASE_COPY(UINT_PTR);
//     default:
//         FIXME("Not supported type: %d\n", V_VT(&var));
//         return E_NOTIMPL;
//     };
// #undef CASE_COPY
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_QueryInterface(IRecordInfo *iface, REFIID riid,
//                                                 void **ppvObject)
// {
//     TRACE("(%p)->(%s %p)\n", iface, debugstr_guid(riid), ppvObject);
//
//     *ppvObject = NULL;
//
//     if(IsEqualGUID(&IID_IUnknown, riid) || IsEqualGUID(&IID_IRecordInfo, riid)) {
//        *ppvObject = iface;
//        IRecordInfo_AddRef(iface);
//        return S_OK;
//     }
//
//     FIXME("Not supported interface: %s\n", debugstr_guid(riid));
//     return E_NOINTERFACE;
// }
//
// static ULONG WINAPI IRecordInfoImpl_AddRef(IRecordInfo *iface)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     ULONG ref = InterlockedIncrement(&This->ref);
//     TRACE("(%p) -> %d\n", This, ref);
//     return ref;
// }
//
// static ULONG WINAPI IRecordInfoImpl_Release(IRecordInfo *iface)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     ULONG ref = InterlockedDecrement(&This->ref);
//
//     TRACE("(%p) -> %d\n", This, ref);
//
//     if(!ref) {
//         int i;
//         for(i=0; i<This->n_vars; i++)
//             SysFreeString(This->fields[i].name);
//         SysFreeString(This->name);
//         HeapFree(GetProcessHeap(), 0, This->fields);
//         ITypeInfo_Release(This->pTypeInfo);
//         HeapFree(GetProcessHeap(), 0, This);
//     }
//     return ref;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_RecordInit(IRecordInfo *iface, PVOID pvNew)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     TRACE("(%p)->(%p)\n", This, pvNew);
//
//     if(!pvNew)
//         return E_INVALIDARG;
//
//     memset(pvNew, 0, This->size);
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_RecordClear(IRecordInfo *iface, PVOID pvExisting)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     int i;
//     PVOID var;
//
//     TRACE("(%p)->(%p)\n", This, pvExisting);
//
//     if(!pvExisting)
//         return E_INVALIDARG;
//
//     for(i=0; i<This->n_vars; i++) {
//         if(This->fields[i].varkind != VAR_PERINSTANCE) {
//             ERR("varkind != VAR_PERINSTANCE\n");
//             continue;
//         }
//         var = ((PBYTE)pvExisting)+This->fields[i].offset;
//         switch(This->fields[i].vt) {
//             case VT_BSTR:
//                SysFreeString(*(BSTR*)var);
//                 *(BSTR*)var = NULL;
//                 break;
//             case VT_I2:
//             case VT_I4:
//             case VT_R4:
//             case VT_R8:
//             case VT_CY:
//             case VT_DATE:
//             case VT_ERROR:
//             case VT_BOOL:
//             case VT_DECIMAL:
//             case VT_I1:
//             case VT_UI1:
//             case VT_UI2:
//             case VT_UI4:
//             case VT_I8:
//             case VT_UI8:
//             case VT_INT:
//             case VT_UINT:
//             case VT_HRESULT:
//                 break;
//             case VT_INT_PTR:
//             case VT_UINT_PTR:
//                 *(void**)var = NULL;
//                 break;
//             case VT_SAFEARRAY:
//                 SafeArrayDestroy(var);
//                 break;
//             case VT_UNKNOWN:
//             case VT_DISPATCH:
//             {
//                 IUnknown *unk = *(IUnknown**)var;
//                 if (unk)
//                     IUnknown_Release(unk);
//                 *(void**)var = NULL;
//                 break;
//             }
//             default:
//                 FIXME("Not supported vt = %d\n", This->fields[i].vt);
//                 break;
//         }
//     }
//     
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_RecordCopy(IRecordInfo *iface, void *src_rec, void *dest_rec)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     HRESULT hr = S_OK;
//     int i;
//
//     TRACE("(%p)->(%p %p)\n", This, src_rec, dest_rec);
//
//     if(!src_rec || !dest_rec)
//         return E_INVALIDARG;
//
//     /* release already stored data */
//     IRecordInfo_RecordClear(iface, dest_rec);
//
//     for (i = 0; i < This->n_vars; i++)
//     {
//         void *src, *dest;
//
//         if (This->fields[i].varkind != VAR_PERINSTANCE) {
//             ERR("varkind != VAR_PERINSTANCE\n");
//             continue;
//         }
//
//         src  = ((BYTE*)src_rec) + This->fields[i].offset;
//         dest = ((BYTE*)dest_rec) + This->fields[i].offset;
//         switch (This->fields[i].vt)
//         {
//             case VT_BSTR:
//             {
//                 BSTR src_str = *(BSTR*)src;
//
//                 if (src_str)
//                 {
//                     BSTR str = SysAllocString(*(BSTR*)src);
//                     if (!str) hr = E_OUTOFMEMORY;
//
//                     *(BSTR*)dest = str;
//                 }
//                 else
//                     *(BSTR*)dest = NULL;
//                 break;
//             }
//             case VT_UNKNOWN:
//             case VT_DISPATCH:
//             {
//                 IUnknown *unk = *(IUnknown**)src;
//                 *(IUnknown**)dest = unk;
//                 if (unk) IUnknown_AddRef(unk);
//                 break;
//             }
//             case VT_SAFEARRAY:
//                 hr = SafeArrayCopy(src, dest);
//                 break;
//             default:
//             {
//                 /* copy directly for types that don't need deep copy */
//                 int len = get_type_size(NULL, This->fields[i].vt);
//                 memcpy(dest, src, len);
//                 break;
//             }
//         }
//
//         if (FAILED(hr)) break;
//     }
//
//     if (FAILED(hr))
//         IRecordInfo_RecordClear(iface, dest_rec);
//
//     return hr;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_GetGuid(IRecordInfo *iface, GUID *pguid)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//
//     TRACE("(%p)->(%p)\n", This, pguid);
//
//     if(!pguid)
//         return E_INVALIDARG;
//
//     *pguid = This->guid;
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_GetName(IRecordInfo *iface, BSTR *pbstrName)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//
//     TRACE("(%p)->(%p)\n", This, pbstrName);
//
//     if(!pbstrName)
//         return E_INVALIDARG;
//
//     *pbstrName = SysAllocString(This->name);
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_GetSize(IRecordInfo *iface, ULONG *pcbSize)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//
//     TRACE("(%p)->(%p)\n", This, pcbSize);
//
//     if(!pcbSize)
//         return E_INVALIDARG;
//     
//     *pcbSize = This->size;
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_GetTypeInfo(IRecordInfo *iface, ITypeInfo **ppTypeInfo)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//
//     TRACE("(%p)->(%p)\n", This, ppTypeInfo);
//
//     if(!ppTypeInfo)
//         return E_INVALIDARG;
//
//     ITypeInfo_AddRef(This->pTypeInfo);
//     *ppTypeInfo = This->pTypeInfo;
//
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_GetField(IRecordInfo *iface, PVOID pvData,
//                                                 LPCOLESTR szFieldName, VARIANT *pvarField)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     int i;
//
//     TRACE("(%p)->(%p %s %p)\n", This, pvData, debugstr_w(szFieldName), pvarField);
//
//     if(!pvData || !szFieldName || !pvarField)
//         return E_INVALIDARG;
//
//     for(i=0; i<This->n_vars; i++)
//         if(!wcscmp(This->fields[i].name, szFieldName))
//             break;
//     if(i == This->n_vars)
//         return TYPE_E_FIELDNOTFOUND;
//     
//     VariantClear(pvarField);
//     return copy_to_variant(((PBYTE)pvData)+This->fields[i].offset, pvarField,
//             This->fields[i].vt);
// }
//
// static HRESULT WINAPI IRecordInfoImpl_GetFieldNoCopy(IRecordInfo *iface, PVOID pvData,
//                             LPCOLESTR szFieldName, VARIANT *pvarField, PVOID *ppvDataCArray)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     int i;
//
//     TRACE("(%p)->(%p %s %p %p)\n", This, pvData, debugstr_w(szFieldName), pvarField, ppvDataCArray);
//
//     if(!pvData || !szFieldName || !pvarField)
//         return E_INVALIDARG;
//
//     for(i=0; i<This->n_vars; i++)
//         if(!wcscmp(This->fields[i].name, szFieldName))
//             break;
//     if(i == This->n_vars)
//         return TYPE_E_FIELDNOTFOUND;
//
//     VariantClear(pvarField);
//     V_VT(pvarField) = VT_BYREF|This->fields[i].vt;
//     V_BYREF(pvarField) = ((PBYTE)pvData)+This->fields[i].offset;
//     *ppvDataCArray = NULL;
//     return S_OK;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_PutField(IRecordInfo *iface, ULONG wFlags, PVOID pvData,
//                                             LPCOLESTR szFieldName, VARIANT *pvarField)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     int i;
//
//     TRACE("(%p)->(%08x %p %s %p)\n", This, wFlags, pvData, debugstr_w(szFieldName),
//                                     pvarField);
//
//     if(!pvData || !szFieldName || !pvarField
//             || (wFlags != INVOKE_PROPERTYPUTREF && wFlags != INVOKE_PROPERTYPUT))
//         return E_INVALIDARG;
//
//     if(wFlags == INVOKE_PROPERTYPUTREF) {
//         FIXME("wFlag == INVOKE_PROPERTYPUTREF not supported\n");
//         return E_NOTIMPL;
//     }
//
//     for(i=0; i<This->n_vars; i++)
//         if(!wcscmp(This->fields[i].name, szFieldName))
//             break;
//     if(i == This->n_vars)
//         return TYPE_E_FIELDNOTFOUND;
//
//     return copy_from_variant(pvarField, ((PBYTE)pvData)+This->fields[i].offset,
//             This->fields[i].vt);
// }
//
// static HRESULT WINAPI IRecordInfoImpl_PutFieldNoCopy(IRecordInfo *iface, ULONG wFlags,
//                 PVOID pvData, LPCOLESTR szFieldName, VARIANT *pvarField)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     int i;
//
//     FIXME("(%p)->(%08x %p %s %p) stub\n", This, wFlags, pvData, debugstr_w(szFieldName), pvarField);
//
//     if(!pvData || !szFieldName || !pvarField
//             || (wFlags != INVOKE_PROPERTYPUTREF && wFlags != INVOKE_PROPERTYPUT))
//         return E_INVALIDARG;
//
//     for(i=0; i<This->n_vars; i++)
//         if(!wcscmp(This->fields[i].name, szFieldName))
//             break;
//     if(i == This->n_vars)
//         return TYPE_E_FIELDNOTFOUND;
//
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_GetFieldNames(IRecordInfo *iface, ULONG *pcNames,
//                                                 BSTR *rgBstrNames)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     ULONG n = This->n_vars, i;
//
//     TRACE("(%p)->(%p %p)\n", This, pcNames, rgBstrNames);
//
//     if(!pcNames)
//         return E_INVALIDARG;
//
//     if(*pcNames < n)
//         n =  *pcNames;
//
//     if(rgBstrNames) {
//         for(i=0; i<n; i++)
//             rgBstrNames[i] = SysAllocString(This->fields[i].name);
//     }
//     
//     *pcNames = n;
//     return S_OK;
// }
//
// static BOOL WINAPI IRecordInfoImpl_IsMatchingType(IRecordInfo *iface, IRecordInfo *info2)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     GUID guid2;
//
//     TRACE( "(%p)->(%p)\n", This, info2 );
//
//     IRecordInfo_GetGuid( info2, &guid2 );
//     return IsEqualGUID( &This->guid, &guid2 );
// }
//
// static PVOID WINAPI IRecordInfoImpl_RecordCreate(IRecordInfo *iface)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     void *record;
//
//     TRACE("(%p)\n", This);
//
//     record = HeapAlloc(GetProcessHeap(), 0, This->size);
//     IRecordInfo_RecordInit(iface, record);
//     TRACE("created record at %p\n", record);
//     return record;
// }
//
// static HRESULT WINAPI IRecordInfoImpl_RecordCreateCopy(IRecordInfo *iface, PVOID pvSource,
//                                                     PVOID *ppvDest)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//
//     TRACE("(%p)->(%p %p)\n", This, pvSource, ppvDest);
//
//     if(!pvSource || !ppvDest)
//         return E_INVALIDARG;
//     
//     *ppvDest = IRecordInfo_RecordCreate(iface);
//     return IRecordInfo_RecordCopy(iface, pvSource, *ppvDest);
// }
//
// static HRESULT WINAPI IRecordInfoImpl_RecordDestroy(IRecordInfo *iface, PVOID pvRecord)
// {
//     IRecordInfoImpl *This = impl_from_IRecordInfo(iface);
//     HRESULT hres;
//
//     TRACE("(%p)->(%p)\n", This, pvRecord);
//
//     hres = IRecordInfo_RecordClear(iface, pvRecord);
//     if(FAILED(hres))
//         return hres;
//
//     if(!HeapFree(GetProcessHeap(), 0, pvRecord))
//         return E_INVALIDARG;
//
//     return S_OK;
// }
//
// static const IRecordInfoVtbl IRecordInfoImplVtbl = {
//     IRecordInfoImpl_QueryInterface,
//     IRecordInfoImpl_AddRef,
//     IRecordInfoImpl_Release,
//     IRecordInfoImpl_RecordInit,
//     IRecordInfoImpl_RecordClear,
//     IRecordInfoImpl_RecordCopy,
//     IRecordInfoImpl_GetGuid,
//     IRecordInfoImpl_GetName,
//     IRecordInfoImpl_GetSize,
//     IRecordInfoImpl_GetTypeInfo,
//     IRecordInfoImpl_GetField,
//     IRecordInfoImpl_GetFieldNoCopy,
//     IRecordInfoImpl_PutField,
//     IRecordInfoImpl_PutFieldNoCopy,
//     IRecordInfoImpl_GetFieldNames,
//     IRecordInfoImpl_IsMatchingType,
//     IRecordInfoImpl_RecordCreate,
//     IRecordInfoImpl_RecordCreateCopy,
//     IRecordInfoImpl_RecordDestroy
// };
//
// /******************************************************************************
//  *      GetRecordInfoFromGuids  [OLEAUT32.322]
//  *
//  * RETURNS
//  *  Success: S_OK
//  *  Failure: E_INVALIDARG, if any argument is invalid.
//  */
// HRESULT WINAPI GetRecordInfoFromGuids(REFGUID rGuidTypeLib, ULONG uVerMajor,
//                         ULONG uVerMinor, LCID lcid, REFGUID rGuidTypeInfo, IRecordInfo** ppRecInfo)
// {
//     ITypeInfo *pTypeInfo;
//     ITypeLib *pTypeLib;
//     HRESULT hres;
//     
//     TRACE("(%p,%d,%d,%d,%s,%p)\n", rGuidTypeLib, uVerMajor, uVerMinor,
//             lcid, debugstr_guid(rGuidTypeInfo), ppRecInfo);
//
//     hres = LoadRegTypeLib(rGuidTypeLib, uVerMajor, uVerMinor, lcid, &pTypeLib);
//     if(FAILED(hres)) {
//         WARN("LoadRegTypeLib failed!\n");
//         return hres;
//     }
//
//     hres = ITypeLib_GetTypeInfoOfGuid(pTypeLib, rGuidTypeInfo, &pTypeInfo);
//     ITypeLib_Release(pTypeLib);
//     if(FAILED(hres)) {
//         WARN("GetTypeInfoOfGuid failed!\n");
//         return hres;
//     }
//
//     hres = GetRecordInfoFromTypeInfo(pTypeInfo, ppRecInfo);
//     ITypeInfo_Release(pTypeInfo);
//     return hres;
// }
//
// /******************************************************************************
//  *      GetRecordInfoFromTypeInfo [OLEAUT32.332]
//  */
// HRESULT WINAPI GetRecordInfoFromTypeInfo(ITypeInfo* pTI, IRecordInfo** ppRecInfo) {
//     HRESULT hres;
//     TYPEATTR *typeattr;
//     IRecordInfoImpl *ret;
//     ITypeInfo *pTypeInfo;
//     int i;
//     GUID guid;
//
//     TRACE("(%p %p)\n", pTI, ppRecInfo);
//
//     if(!pTI || !ppRecInfo)
//         return E_INVALIDARG;
//     
//     hres = ITypeInfo_GetTypeAttr(pTI, &typeattr);
//     if(FAILED(hres) || !typeattr) {
//         WARN("GetTypeAttr failed: %08x\n", hres);
//         return hres;
//     }
//
//     if(typeattr->typekind == TKIND_ALIAS) {
//         hres = ITypeInfo_GetRefTypeInfo(pTI, typeattr->tdescAlias.u.hreftype, &pTypeInfo);
//         guid = typeattr->guid;
//         ITypeInfo_ReleaseTypeAttr(pTI, typeattr);
//         if(FAILED(hres)) {
//             WARN("GetRefTypeInfo failed: %08x\n", hres);
//             return hres;
//         }
//         hres = ITypeInfo_GetTypeAttr(pTypeInfo, &typeattr);
//         if(FAILED(hres)) {
//             ITypeInfo_Release(pTypeInfo);
//             WARN("GetTypeAttr failed for referenced type: %08x\n", hres);
//             return hres;
//         }
//     }else  {
//         pTypeInfo = pTI;
//         ITypeInfo_AddRef(pTypeInfo);
//         guid = typeattr->guid;
//     }
//
//     if(typeattr->typekind != TKIND_RECORD) {
//         WARN("typekind != TKIND_RECORD\n");
//         ITypeInfo_ReleaseTypeAttr(pTypeInfo, typeattr);
//         ITypeInfo_Release(pTypeInfo);
//         return E_INVALIDARG;
//     }
//
//     ret = HeapAlloc(GetProcessHeap(), 0, sizeof(*ret));
//     ret->IRecordInfo_iface.lpVtbl = &IRecordInfoImplVtbl;
//     ret->ref = 1;
//     ret->pTypeInfo = pTypeInfo;
//     ret->n_vars = typeattr->cVars;
//     ret->size = typeattr->cbSizeInstance;
//     ITypeInfo_ReleaseTypeAttr(pTypeInfo, typeattr);
//
//     ret->guid = guid;
//
//     /* NOTE: Windows implementation calls ITypeInfo::GetCantainingTypeLib and
//      *       ITypeLib::GetLibAttr, but we currently don't need this.
//      */
//
//     hres = ITypeInfo_GetDocumentation(pTypeInfo, MEMBERID_NIL, &ret->name, NULL, NULL, NULL);
//     if(FAILED(hres)) {
//         WARN("ITypeInfo::GetDocumentation failed\n");
//         ret->name = NULL;
//     }
//
//     ret->fields = HeapAlloc(GetProcessHeap(), 0, ret->n_vars*sizeof(fieldstr));
//     for(i = 0; i<ret->n_vars; i++) {
//         VARDESC *vardesc;
//         hres = ITypeInfo_GetVarDesc(pTypeInfo, i, &vardesc);
//         if(FAILED(hres)) {
//             WARN("GetVarDesc failed\n");
//             continue;
//         }
//         ret->fields[i].vt = vardesc->elemdescVar.tdesc.vt;
//         ret->fields[i].varkind = vardesc->varkind;
//         ret->fields[i].offset = vardesc->u.oInst;
//         hres = ITypeInfo_GetDocumentation(pTypeInfo, vardesc->memid, &ret->fields[i].name,
//                 NULL, NULL, NULL);
//         if(FAILED(hres))
//             WARN("GetDocumentation failed: %08x\n", hres);
//         TRACE("field=%s, offset=%d\n", debugstr_w(ret->fields[i].name), ret->fields[i].offset);
//         ITypeInfo_ReleaseVarDesc(pTypeInfo, vardesc);
//     }
//
//     *ppRecInfo = &ret->IRecordInfo_iface;
//
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
