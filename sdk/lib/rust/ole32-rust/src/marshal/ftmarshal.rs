//! ftmarshal.rs — translation of dll/win32/ole32/ftmarshal.c (the
//! free-threaded marshaler, CoCreateFreeThreadedMarshaler).
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_unsafe,
    unused_mut,
    unused_assignments,
    static_mut_refs,
    clippy::all
)]

use super::types::*;

#[repr(C)]
struct FTMarshalImpl {
    IUnknown_inner: IUnknown,
    IMarshal_iface: IMarshal,
    outer_unk: *mut IUnknown,
    ref_: LONG,
}

#[inline]
unsafe fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut FTMarshalImpl {
    containing_record!(iface, FTMarshalImpl, IUnknown_inner)
}

#[inline]
unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut FTMarshalImpl {
    containing_record!(iface, FTMarshalImpl, IMarshal_iface)
}

/* inner IUnknown to handle aggregation */
unsafe extern "system" fn IiFTMUnknown_fnQueryInterface(
    iface: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IUnknown(iface);

    TRACE!("\n");
    *ppv = core::ptr::null_mut();

    if IsEqualIID(&IID_IUnknown, riid) {
        *ppv = core::ptr::addr_of_mut!((*This).IUnknown_inner) as *mut c_void;
    } else if IsEqualIID(&IID_IMarshal, riid) {
        *ppv = core::ptr::addr_of_mut!((*This).IMarshal_iface) as *mut c_void;
    } else {
        FIXME!("No interface for %s.\n");
        return E_NOINTERFACE;
    }
    ((*ppv) as *mut IUnknown).AddRef();
    S_OK
}

unsafe extern "system" fn IiFTMUnknown_fnAddRef(iface: *mut IUnknown) -> ULONG {
    let This = impl_from_IUnknown(iface);

    TRACE!("\n");
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}

unsafe extern "system" fn IiFTMUnknown_fnRelease(iface: *mut IUnknown) -> ULONG {
    let This = impl_from_IUnknown(iface);

    TRACE!("\n");
    if InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) != 0 {
        return (*This).ref_ as ULONG;
    }
    HeapFree(GetProcessHeap(), 0, This as PVOID);
    0
}

static iunkvt: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: IiFTMUnknown_fnQueryInterface,
    AddRef: IiFTMUnknown_fnAddRef,
    Release: IiFTMUnknown_fnRelease,
};

unsafe extern "system" fn FTMarshalImpl_QueryInterface(
    iface: LPMARSHAL,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!("(%p)->(%s,%p)\n");
    (*This).outer_unk.QueryInterface(riid, ppv)
}

unsafe extern "system" fn FTMarshalImpl_AddRef(iface: LPMARSHAL) -> ULONG {
    let This = impl_from_IMarshal(iface);

    TRACE!("\n");
    (*This).outer_unk.AddRef()
}

unsafe extern "system" fn FTMarshalImpl_Release(iface: LPMARSHAL) -> ULONG {
    let This = impl_from_IMarshal(iface);

    TRACE!("\n");
    (*This).outer_unk.Release()
}

unsafe extern "system" fn FTMarshalImpl_GetUnmarshalClass(
    iface: LPMARSHAL,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pCid: *mut CLSID,
) -> HRESULT {
    TRACE!("(%s, %p, 0x%x, %p, 0x%x, %p)\n");
    if dwDestContext == MSHCTX_INPROC || dwDestContext == MSHCTX_CROSSCTX {
        *pCid = CLSID_InProcFreeMarshaler;
    } else {
        *pCid = CLSID_StdMarshal;
    }
    S_OK
}

unsafe extern "system" fn FTMarshalImpl_GetMarshalSizeMax(
    iface: LPMARSHAL,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pSize: *mut DWORD,
) -> HRESULT {
    let mut pMarshal: *mut IMarshal = core::ptr::null_mut();

    TRACE!("(%s, %p, 0x%x, %p, 0x%x, %p)\n");

    /* if the marshalling happens inside the same process the interface pointer
       is copied between the apartments */
    if dwDestContext == MSHCTX_INPROC || dwDestContext == MSHCTX_CROSSCTX {
        // wine-10: flags + union { UINT64 value; IUnknown *object; } + GUID = 28 bytes on
        // every arch, which is what MarshalInterface writes (a 32-bit pointer is padded
        // by a zero DWORD). Counting pv AND the pad overstated 64-bit as 32.
        *pSize = (core::mem::size_of::<DWORD>()          /* mshlflags */
            + core::mem::size_of::<u64>()                /* union ftmarshaler_data */
            + core::mem::size_of::<GUID>()) as DWORD; /* GUID */
        return S_OK;
    }

    /* use the standard marshaller to handle all other cases */
    CoGetStandardMarshal(riid, pv as *mut IUnknown, dwDestContext, pvDestContext, mshlflags, &mut pMarshal);
    let hres = pMarshal.GetMarshalSizeMax(riid, pv, dwDestContext, pvDestContext, mshlflags, pSize);
    pMarshal.Release();
    hres
}

unsafe extern "system" fn FTMarshalImpl_MarshalInterface(
    iface: LPMARSHAL,
    pStm: *mut IStream,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
) -> HRESULT {
    let mut pMarshal: *mut IMarshal = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!("(%p, %s, %p, 0x%x, %p, 0x%x)\n");

    /* if the marshalling happens inside the same process the interface pointer
       is copied between the apartments */
    if dwDestContext == MSHCTX_INPROC || dwDestContext == MSHCTX_CROSSCTX {
        let mut object: *mut c_void = core::ptr::null_mut();
        let constant: DWORD = 0;
        let unknown_guid: GUID = GUID_NULL;

        hres = (pv as *mut IUnknown).QueryInterface(riid, &mut object);
        if FAILED(hres) {
            return hres;
        }

        /* don't hold a reference to table-weak marshaled interfaces */
        if mshlflags & MSHLFLAGS_TABLEWEAK != 0 {
            (object as *mut IUnknown).Release();
        }

        hres = pStm.Write(
            core::ptr::addr_of!(mshlflags) as *const c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::null_mut(),
        );
        if hres != S_OK {
            return STG_E_MEDIUMFULL;
        }

        hres = pStm.Write(
            core::ptr::addr_of!(object) as *const c_void,
            core::mem::size_of::<*mut c_void>() as ULONG,
            core::ptr::null_mut(),
        );
        if hres != S_OK {
            return STG_E_MEDIUMFULL;
        }

        if core::mem::size_of::<*mut c_void>() == core::mem::size_of::<DWORD>() {
            hres = pStm.Write(
                core::ptr::addr_of!(constant) as *const c_void,
                core::mem::size_of::<DWORD>() as ULONG,
                core::ptr::null_mut(),
            );
            if hres != S_OK {
                return STG_E_MEDIUMFULL;
            }
        }

        hres = pStm.Write(
            core::ptr::addr_of!(unknown_guid) as *const c_void,
            core::mem::size_of::<GUID>() as ULONG,
            core::ptr::null_mut(),
        );
        if hres != S_OK {
            return STG_E_MEDIUMFULL;
        }

        return S_OK;
    }

    /* use the standard marshaler to handle all other cases */
    CoGetStandardMarshal(riid, pv as *mut IUnknown, dwDestContext, pvDestContext, mshlflags, &mut pMarshal);
    hres = pMarshal.MarshalInterface(pStm, riid, pv, dwDestContext, pvDestContext, mshlflags);
    pMarshal.Release();
    hres
}

unsafe extern "system" fn FTMarshalImpl_UnmarshalInterface(
    iface: LPMARSHAL,
    pStm: *mut IStream,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut mshlflags: DWORD = 0;
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut constant: DWORD = 0;
    let mut unknown_guid: GUID = GUID_NULL;
    let mut hres: HRESULT;

    TRACE!("(%p, %s, %p)\n");

    hres = pStm.Read(
        core::ptr::addr_of_mut!(mshlflags) as *mut c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    if hres != S_OK {
        return STG_E_READFAULT;
    }

    hres = pStm.Read(
        core::ptr::addr_of_mut!(object) as *mut c_void,
        core::mem::size_of::<*mut IUnknown>() as ULONG,
        core::ptr::null_mut(),
    );
    if hres != S_OK {
        return STG_E_READFAULT;
    }

    if core::mem::size_of::<*mut IUnknown>() == core::mem::size_of::<DWORD>() {
        hres = pStm.Read(
            core::ptr::addr_of_mut!(constant) as *mut c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::null_mut(),
        );
        if hres != S_OK {
            return STG_E_READFAULT;
        }
        if constant != 0 {
            FIXME!("constant is 0x%x instead of 0\n");
        }
    }

    hres = pStm.Read(
        core::ptr::addr_of_mut!(unknown_guid) as *mut c_void,
        core::mem::size_of::<GUID>() as ULONG,
        core::ptr::null_mut(),
    );
    if hres != S_OK {
        return STG_E_READFAULT;
    }

    hres = object.QueryInterface(riid, ppv);
    if mshlflags & (MSHLFLAGS_TABLEWEAK | MSHLFLAGS_TABLESTRONG) == 0 {
        object.Release();
    }
    hres
}

unsafe extern "system" fn FTMarshalImpl_ReleaseMarshalData(iface: LPMARSHAL, pStm: *mut IStream) -> HRESULT {
    let mut mshlflags: DWORD = 0;
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut constant: DWORD = 0;
    let mut unknown_guid: GUID = GUID_NULL;
    let mut hres: HRESULT;

    TRACE!("(%p)\n");

    hres = pStm.Read(
        core::ptr::addr_of_mut!(mshlflags) as *mut c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    if hres != S_OK {
        return STG_E_READFAULT;
    }

    hres = pStm.Read(
        core::ptr::addr_of_mut!(object) as *mut c_void,
        core::mem::size_of::<*mut IUnknown>() as ULONG,
        core::ptr::null_mut(),
    );
    if hres != S_OK {
        return STG_E_READFAULT;
    }

    if core::mem::size_of::<*mut IUnknown>() == core::mem::size_of::<DWORD>() {
        hres = pStm.Read(
            core::ptr::addr_of_mut!(constant) as *mut c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::null_mut(),
        );
        if hres != S_OK {
            return STG_E_READFAULT;
        }
        if constant != 0 {
            FIXME!("constant is 0x%x instead of 0\n");
        }
    }

    hres = pStm.Read(
        core::ptr::addr_of_mut!(unknown_guid) as *mut c_void,
        core::mem::size_of::<GUID>() as ULONG,
        core::ptr::null_mut(),
    );
    if hres != S_OK {
        return STG_E_READFAULT;
    }

    object.Release();
    S_OK
}

unsafe extern "system" fn FTMarshalImpl_DisconnectObject(iface: LPMARSHAL, dwReserved: DWORD) -> HRESULT {
    TRACE!("()\n");
    /* nothing to do */
    S_OK
}

static ftmvtbl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: FTMarshalImpl_QueryInterface,
    AddRef: FTMarshalImpl_AddRef,
    Release: FTMarshalImpl_Release,
    GetUnmarshalClass: FTMarshalImpl_GetUnmarshalClass,
    GetMarshalSizeMax: FTMarshalImpl_GetMarshalSizeMax,
    MarshalInterface: FTMarshalImpl_MarshalInterface,
    UnmarshalInterface: FTMarshalImpl_UnmarshalInterface,
    ReleaseMarshalData: FTMarshalImpl_ReleaseMarshalData,
    DisconnectObject: FTMarshalImpl_DisconnectObject,
};

/***********************************************************************
 *          CoCreateFreeThreadedMarshaler [OLE32.@]
 *
 * Creates a free-threaded marshaler.
 *
 * NOTES
 *  Objects that ensure their state is maintained consistent when used by
 *  multiple threads and reference no single-threaded objects are known as
 *  free-threaded. The free-threaded marshaler enables these objects to be
 *  efficiently marshaled within the same process, by not creating proxies
 *  (as they aren't needed for the object to be safely used), whilst still
 *  allowing the object to be used in inter-process and inter-machine
 *  contexts.
 */
#[no_mangle]
pub unsafe extern "system" fn CoCreateFreeThreadedMarshaler(
    punkOuter: *mut IUnknown,
    ppunkMarshal: *mut *mut IUnknown,
) -> HRESULT {
    TRACE!("(%p %p)\n");

    let ftm = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<FTMarshalImpl>())
        as *mut FTMarshalImpl;
    if ftm.is_null() {
        return E_OUTOFMEMORY;
    }

    (*ftm).IUnknown_inner.lpVtbl = &iunkvt;
    (*ftm).IMarshal_iface.lpVtbl = &ftmvtbl;
    (*ftm).ref_ = 1;
    (*ftm).outer_unk = if !punkOuter.is_null() {
        punkOuter
    } else {
        core::ptr::addr_of_mut!((*ftm).IUnknown_inner)
    };

    *ppunkMarshal = core::ptr::addr_of_mut!((*ftm).IUnknown_inner);
    S_OK
}

unsafe extern "system" fn FTMarshalCF_QueryInterface(
    iface: *mut IClassFactory,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    *ppv = core::ptr::null_mut();
    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IClassFactory) {
        *ppv = iface as *mut c_void;
        iface.AddRef();
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe extern "system" fn FTMarshalCF_AddRef(iface: *mut IClassFactory) -> ULONG {
    2 /* non-heap based object */
}

unsafe extern "system" fn FTMarshalCF_Release(iface: *mut IClassFactory) -> ULONG {
    1 /* non-heap based object */
}

unsafe extern "system" fn FTMarshalCF_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut pUnknown: *mut IUnknown = core::ptr::null_mut();

    TRACE!("(%p, %s, %p)\n");

    *ppv = core::ptr::null_mut();

    let mut hr = CoCreateFreeThreadedMarshaler(pUnk, &mut pUnknown);

    if SUCCEEDED(hr) {
        hr = pUnknown.QueryInterface(riid, ppv);
        pUnknown.Release();
    }

    hr
}

unsafe extern "system" fn FTMarshalCF_LockServer(iface: *mut IClassFactory, fLock: BOOL) -> HRESULT {
    FIXME!("(%d), stub!\n");
    S_OK
}

static FTMarshalCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: FTMarshalCF_QueryInterface,
    AddRef: FTMarshalCF_AddRef,
    Release: FTMarshalCF_Release,
    CreateInstance: FTMarshalCF_CreateInstance,
    LockServer: FTMarshalCF_LockServer,
};
/// C: `static const IClassFactoryVtbl *FTMarshalCF = &FTMarshalCFVtbl;` — the
/// ADDRESS of this pointer is the IClassFactory object.
static FTMarshalCF: ComStatic<IClassFactory> = ComStatic(IClassFactory {
    lpVtbl: &FTMarshalCFVtbl,
});

/// (compobj_private.h: HRESULT FTMarshalCF_Create(REFIID, LPVOID*))
#[no_mangle]
pub unsafe extern "C" fn FTMarshalCF_Create(riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    let cf = core::ptr::addr_of!(FTMarshalCF.0) as *mut IClassFactory;
    cf.QueryInterface(riid, ppv as *mut *mut c_void)
}
