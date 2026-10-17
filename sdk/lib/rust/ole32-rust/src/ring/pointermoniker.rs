use crate::prelude::*;

// Defined by sibling ring modules, but ALSO (wrongly) declared as externs in
// ole32_exports.rs — the prelude glob makes the bare names ambiguous, so bind explicitly.
use crate::ring::antimoniker::{create_anti_moniker, is_anti_moniker, CreateAntiMoniker};
use crate::ring::compositemoniker::CreateGenericComposite;

// ---- PointerMoniker: semantically ported to wine-10.0 (upstream 5f6b295cb90) ----
//
// Over the old translation: the moniker marshals BY VALUE through its own IMarshal (the
// wrapped object goes across as an IUnknown) instead of falling back to the standard
// marshaler; QueryInterface answers CLSID_PointerMoniker; IsEqual compares the wrapped
// pointers of OUR pointer monikers by vtable identity; ComposeWith cancels one order of an
// anti-moniker; and Enum/RelativePathTo/GetDisplayName/CommonPrefixWith return wine-10's
// codes. CreatePointerMoniker refuses a NULL out-pointer.

#[repr(C)]
pub struct PointerMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IMarshal_iface: IMarshal,
    pub ref_: LONG,
    /// the object this moniker points at
    pub pObject: *mut IUnknown,
}

pub type PPointerMonikerImpl = *mut PointerMonikerImpl;

static VT_PointerMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(PointerMonikerImpl_QueryInterface),
    AddRef: Some(PointerMonikerImpl_AddRef),
    Release: Some(PointerMonikerImpl_Release),
    GetClassID: Some(PointerMonikerImpl_GetClassID),
    IsDirty: Some(PointerMonikerImpl_IsDirty),
    Load: Some(PointerMonikerImpl_Load),
    Save: Some(PointerMonikerImpl_Save),
    GetSizeMax: Some(PointerMonikerImpl_GetSizeMax),
    BindToObject: Some(PointerMonikerImpl_BindToObject),
    BindToStorage: Some(PointerMonikerImpl_BindToStorage),
    Reduce: Some(PointerMonikerImpl_Reduce),
    ComposeWith: Some(PointerMonikerImpl_ComposeWith),
    Enum: Some(PointerMonikerImpl_Enum),
    IsEqual: Some(PointerMonikerImpl_IsEqual),
    Hash: Some(PointerMonikerImpl_Hash),
    IsRunning: Some(PointerMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(PointerMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(PointerMonikerImpl_Inverse),
    CommonPrefixWith: Some(PointerMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(PointerMonikerImpl_RelativePathTo),
    GetDisplayName: Some(PointerMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(PointerMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(PointerMonikerImpl_IsSystemMoniker),
};

static pointer_moniker_marshal_vtbl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: Some(pointer_moniker_marshal_QueryInterface),
    AddRef: Some(pointer_moniker_marshal_AddRef),
    Release: Some(pointer_moniker_marshal_Release),
    GetUnmarshalClass: Some(pointer_moniker_marshal_GetUnmarshalClass),
    GetMarshalSizeMax: Some(pointer_moniker_marshal_GetMarshalSizeMax),
    MarshalInterface: Some(pointer_moniker_marshal_MarshalInterface),
    UnmarshalInterface: Some(pointer_moniker_marshal_UnmarshalInterface),
    ReleaseMarshalData: Some(pointer_moniker_marshal_ReleaseMarshalData),
    DisconnectObject: Some(pointer_moniker_marshal_DisconnectObject),
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut PointerMonikerImpl {
    CONTAINING_RECORD!(iface, PointerMonikerImpl, IMoniker_iface)
}

unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut PointerMonikerImpl {
    CONTAINING_RECORD!(iface, PointerMonikerImpl, IMarshal_iface)
}

/// wine-10 `unsafe_impl_from_IMoniker`: OUR pointer moniker, identified by vtable.
unsafe fn unsafe_impl_from_IMoniker(iface: *mut IMoniker) -> *mut PointerMonikerImpl {
    if iface.is_null()
        || (*iface).lpVtbl as *const IMonikerVtbl != core::ptr::addr_of!(VT_PointerMonikerImpl)
    {
        return core::ptr::null_mut();
    }
    impl_from_IMoniker(iface)
}

stdmethod! {
fn PointerMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), iface, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
        || IsEqualGUID(core::ptr::addr_of!(CLSID_PointerMoniker), riid)
    {
        *ppvObject = iface as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*moniker).IMarshal_iface) as LPVOID;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn PointerMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let moniker = impl_from_IMoniker(iface);
    let refcount = InterlockedIncrement(core::ptr::addr_of_mut!((*moniker).ref_)) as ULONG;

    TRACE!(b"%p, refcount %u.\n\0".as_ptr(), iface, refcount);

    refcount
}
}

stdmethod! {
fn PointerMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let moniker = impl_from_IMoniker(iface);
    let refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*moniker).ref_)) as ULONG;

    TRACE!(b"%p, refcount %u.\n\0".as_ptr(), iface, refcount);

    if refcount == 0 {
        if !(*moniker).pObject.is_null() {
            IUnknown_Release((*moniker).pObject);
        }
        HeapFree(GetProcessHeap(), 0, moniker as LPVOID);
    }

    refcount
}
}

stdmethod! {
fn PointerMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_PointerMoniker;

    S_OK
}
}

stdmethod! {
fn PointerMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    S_FALSE
}
}

stdmethod! {
fn PointerMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), pStm);
    let _ = iface;

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    TRACE!(b"(%p, %d)\n\0".as_ptr(), pStm, fClearDirty);
    let _ = iface;

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    (*pcbSize).QuadPart = 0;

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, debugstr_guid(riid), ppvResult);

    if (*This).pObject.is_null() {
        return E_UNEXPECTED;
    }

    IUnknown_QueryInterface((*This).pObject, riid, ppvResult)
}
}

stdmethod! {
fn PointerMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, debugstr_guid(riid), ppvResult);

    if (*This).pObject.is_null() {
        return E_UNEXPECTED;
    }

    IUnknown_QueryInterface((*This).pObject, riid, ppvResult)
}
}

stdmethod! {
fn PointerMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%d,%p,%p)\n\0".as_ptr(), iface, pbc, dwReduceHowFar, ppmkToLeft, ppmkReduced);

    if ppmkReduced.is_null() {
        return E_POINTER;
    }

    PointerMonikerImpl_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

stdmethod! {
fn PointerMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    right: *mut IMoniker,
    only_if_not_generic: BOOL,
    result: *mut *mut IMoniker,
) -> HRESULT {
    let mut order: DWORD = 0;

    TRACE!(b"%p, %p, %d, %p.\n\0".as_ptr(), iface, right, only_if_not_generic, result);

    if result.is_null() || right.is_null() {
        return E_POINTER;
    }

    *result = core::ptr::null_mut();

    if is_anti_moniker(right, &mut order) {
        return if order > 1 {
            create_anti_moniker(order - 1, result)
        } else {
            S_OK
        };
    }

    if only_if_not_generic != 0 {
        MK_E_NEEDGENERIC
    } else {
        CreateGenericComposite(iface, right, result)
    }
}
}

stdmethod! {
fn PointerMonikerImpl_Enum(
    iface: *mut IMoniker,
    fForward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), iface, fForward, ppenumMoniker);

    if ppenumMoniker.is_null() {
        return E_POINTER;
    }

    *ppenumMoniker = core::ptr::null_mut();

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_IsEqual(iface: *mut IMoniker, other: *mut IMoniker) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p.\n\0".as_ptr(), iface, other);

    if other.is_null() {
        return E_INVALIDARG;
    }

    let other_moniker = unsafe_impl_from_IMoniker(other);
    if other_moniker.is_null() {
        return S_FALSE;
    }

    if (*moniker).pObject == (*other_moniker).pObject {
        S_OK
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn PointerMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pdwHash);

    if pdwHash.is_null() {
        return E_POINTER;
    }

    // PtrToUlong: the low 32 bits of the pointer.
    *pdwHash = (*This).pObject as usize as DWORD;

    S_OK
}
}

stdmethod! {
fn PointerMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pmkNewlyRunning);

    S_OK
}
}

stdmethod! {
fn PointerMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pAntiTime: *mut FILETIME,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pAntiTime);

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    CreateAntiMoniker(ppmk)
}
}

stdmethod! {
fn PointerMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    prefix: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%p, %p, %p.\n\0".as_ptr(), iface, other, prefix);

    if prefix.is_null() || other.is_null() {
        return E_INVALIDARG;
    }

    *prefix = core::ptr::null_mut();

    // wine-10 fixed the inverted test the old code carried (`!= S_OK` gave MK_S_US for
    // UNEQUAL monikers).
    if PointerMonikerImpl_IsEqual(iface, other) == S_OK {
        IMoniker_AddRef(iface);
        *prefix = iface;
        MK_S_US
    } else {
        MK_E_NOPREFIX
    }
}
}

stdmethod! {
fn PointerMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    result: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%p, %p, %p.\n\0".as_ptr(), iface, other, result);

    if result.is_null() {
        return E_INVALIDARG;
    }

    *result = core::ptr::null_mut();

    if !other.is_null() {
        E_NOTIMPL
    } else {
        E_INVALIDARG
    }
}
}

stdmethod! {
fn PointerMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    toleft: *mut IMoniker,
    name: *mut LPOLESTR,
) -> HRESULT {
    TRACE!(b"%p, %p, %p, %p.\n\0".as_ptr(), iface, pbc, toleft, name);

    if name.is_null() || pbc.is_null() {
        if !name.is_null() {
            *name = core::ptr::null_mut();
        }
        return E_INVALIDARG;
    }

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut pPDN: *mut IParseDisplayName = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pszDisplayName,
        pchEaten,
        ppmkOut,
    );

    if !pmkToLeft.is_null() {
        return MK_E_SYNTAX;
    }

    if (*This).pObject.is_null() {
        return E_UNEXPECTED;
    }

    let mut hr = IUnknown_QueryInterface(
        (*This).pObject,
        core::ptr::addr_of!(IID_IParseDisplayName),
        core::ptr::addr_of_mut!(pPDN) as *mut LPVOID,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = IParseDisplayName_ParseDisplayName(pPDN, pbc, pszDisplayName, pchEaten, ppmkOut);
    IParseDisplayName_Release(pPDN);

    hr
}
}

stdmethod! {
fn PointerMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_POINTERMONIKER;

    S_OK
}
}

// ---- IMarshal: by value, the wrapped object crosses as an IUnknown ----

stdmethod! {
fn pointer_moniker_marshal_QueryInterface(
    iface: *mut IMarshal,
    riid: *const IID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);

    TRACE!(b"(%s, %p)\n\0".as_ptr(), debugstr_guid(riid), ppv);

    IMoniker_QueryInterface(core::ptr::addr_of_mut!((*moniker).IMoniker_iface), riid, ppv)
}
}

stdmethod! {
fn pointer_moniker_marshal_AddRef(iface: *mut IMarshal) -> ULONG {
    let moniker = impl_from_IMarshal(iface);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*moniker).IMoniker_iface))
}
}

stdmethod! {
fn pointer_moniker_marshal_Release(iface: *mut IMarshal) -> ULONG {
    let moniker = impl_from_IMarshal(iface);

    IMoniker_Release(core::ptr::addr_of_mut!((*moniker).IMoniker_iface))
}
}

stdmethod! {
fn pointer_moniker_marshal_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: LPVOID,
    dwDestContext: DWORD,
    pvDestContext: LPVOID,
    mshlflags: DWORD,
    clsid: *mut CLSID,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);

    TRACE!(
        b"%s, %p, %x, %p, %x, %p.\n\0".as_ptr(),
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        clsid,
    );

    IMoniker_GetClassID(core::ptr::addr_of_mut!((*moniker).IMoniker_iface), clsid)
}
}

stdmethod! {
fn pointer_moniker_marshal_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: LPVOID,
    dwDestContext: DWORD,
    pvDestContext: LPVOID,
    mshlflags: DWORD,
    size: *mut DWORD,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);

    TRACE!(
        b"%s, %p, %d, %p, %#x, %p.\n\0".as_ptr(),
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        size,
    );

    CoGetMarshalSizeMax(
        size,
        core::ptr::addr_of!(IID_IUnknown),
        (*moniker).pObject,
        dwDestContext,
        pvDestContext,
        mshlflags,
    )
}
}

stdmethod! {
fn pointer_moniker_marshal_MarshalInterface(
    iface: *mut IMarshal,
    stream: *mut IStream,
    riid: *const IID,
    pv: LPVOID,
    dwDestContext: DWORD,
    pvDestContext: LPVOID,
    mshlflags: DWORD,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);

    TRACE!(
        b"%p, %s, %p, %x, %p, %x.\n\0".as_ptr(),
        stream,
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
    );

    CoMarshalInterface(
        (stream).cast(),
        (core::ptr::addr_of!(IID_IUnknown)).cast(),
        ((*moniker).pObject).cast(),
        dwDestContext,
        pvDestContext,
        mshlflags,
    )
}
}

stdmethod! {
fn pointer_moniker_marshal_UnmarshalInterface(
    iface: *mut IMarshal,
    stream: *mut IStream,
    riid: *const IID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);
    let mut object: *mut IUnknown = core::ptr::null_mut();

    TRACE!(b"%p, %p, %s, %p.\n\0".as_ptr(), iface, stream, debugstr_guid(riid), ppv);

    let hr = CoUnmarshalInterface(
        (stream).cast(),
        (core::ptr::addr_of!(IID_IUnknown)).cast(),
        core::ptr::addr_of_mut!(object) as *mut LPVOID,
    );
    if FAILED(hr) {
        ERR!(b"Couldn't unmarshal moniker, hr = %#x.\n\0".as_ptr(), hr);
        return hr;
    }

    if !(*moniker).pObject.is_null() {
        IUnknown_Release((*moniker).pObject);
    }
    (*moniker).pObject = object;

    IMoniker_QueryInterface(core::ptr::addr_of_mut!((*moniker).IMoniker_iface), riid, ppv)
}
}

stdmethod! {
fn pointer_moniker_marshal_ReleaseMarshalData(iface: *mut IMarshal, stream: *mut IStream) -> HRESULT {
    TRACE!(b"%p, %p.\n\0".as_ptr(), iface, stream);

    S_OK
}
}

stdmethod! {
fn pointer_moniker_marshal_DisconnectObject(iface: *mut IMarshal, reserved: DWORD) -> HRESULT {
    TRACE!(b"%p, %#x.\n\0".as_ptr(), iface, reserved);

    S_OK
}
}

/******************************************************************************
 *        CreatePointerMoniker	[OLE32.@]
 ******************************************************************************/
#[no_mangle]
pub unsafe extern "system" fn CreatePointerMoniker(
    object: *mut IUnknown,
    ret: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr(), object, ret);

    if ret.is_null() {
        return E_INVALIDARG;
    }

    let moniker = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<PointerMonikerImpl>() as usize,
    ) as *mut PointerMonikerImpl;
    if moniker.is_null() {
        *ret = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }

    (*moniker).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_PointerMonikerImpl) as *mut IMonikerVtbl;
    (*moniker).IMarshal_iface.lpVtbl =
        core::ptr::addr_of!(pointer_moniker_marshal_vtbl) as *mut IMarshalVtbl;
    (*moniker).ref_ = 1;
    (*moniker).pObject = object;
    if !(*moniker).pObject.is_null() {
        IUnknown_AddRef((*moniker).pObject);
    }

    *ret = core::ptr::addr_of_mut!((*moniker).IMoniker_iface);

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn PointerMoniker_CreateInstance(
    _iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut pMoniker: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), pUnk, debugstr_guid(riid), ppv);

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let hr = CreatePointerMoniker(core::ptr::null_mut(), &mut pMoniker);
    if FAILED(hr) {
        return hr;
    }

    let hr = IMoniker_QueryInterface(pMoniker, riid, ppv);
    /* wine-10: always drop the creation reference (the old code leaked it on success). */
    IMoniker_Release(pMoniker);

    hr
}

// ===========================================================================
// ObjrefMoniker -- pointermoniker.c's second half, from wine-10.0.
//
// Hand-ported 2026-10-04. The wine-10.0 ole32 sync (upstream 5f6b295cb90) reached this
// tree AFTER this file was translated, so the flipped C on disk grew the objref moniker
// and `CreateObjrefMoniker` while the Rust that replaced it never had them, and ole32.spec
// kept a stub for the export. Most methods are Wine's own E_NOTIMPL stubs; the real
// behaviour is the identity (QueryInterface/ClassID/Hash/IsSystemMoniker) and IMarshal,
// which marshals the wrapped object by OBJREF through CoMarshalInterface.
// ===========================================================================

#[repr(C)]
pub struct ObjrefMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IMarshal_iface: IMarshal,
    pub refcount: LONG,
    pub pObject: *mut IUnknown,
}

/// objidl.h `MKSYS_OBJREFMONIKER`.
const MKSYS_OBJREFMONIKER: DWORD = 8;

unsafe fn objref_impl_from_IMoniker(iface: *mut IMoniker) -> *mut ObjrefMonikerImpl {
    CONTAINING_RECORD!(iface, ObjrefMonikerImpl, IMoniker_iface)
}

unsafe fn objref_impl_from_IMarshal(iface: *mut IMarshal) -> *mut ObjrefMonikerImpl {
    CONTAINING_RECORD!(iface, ObjrefMonikerImpl, IMarshal_iface)
}

stdmethod! {
fn ObjrefMonikerImpl_QueryInterface(iface: *mut IMoniker, iid: *const IID, obj: *mut LPVOID) -> HRESULT {
    let moniker = objref_impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), iface, debugstr_guid(iid), obj);

    if obj.is_null() {
        return E_INVALIDARG;
    }

    *obj = core::ptr::null_mut();

    if IsEqualIID(iid, addr_of!(IID_IUnknown))
        || IsEqualIID(iid, addr_of!(IID_IPersist))
        || IsEqualIID(iid, addr_of!(IID_IPersistStream))
        || IsEqualIID(iid, addr_of!(IID_IMoniker))
        || IsEqualGUID(iid, addr_of!(CLSID_ObjrefMoniker))
        || IsEqualGUID(iid, addr_of!(CLSID_PointerMoniker))
    {
        *obj = iface as LPVOID;
    } else if IsEqualIID(iid, addr_of!(IID_IMarshal)) {
        *obj = addr_of_mut!((*moniker).IMarshal_iface) as LPVOID;
    } else {
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn ObjrefMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let moniker = objref_impl_from_IMoniker(iface);
    let refcount = InterlockedIncrement(addr_of_mut!((*moniker).refcount)) as ULONG;

    TRACE!(b"%p, refcount %lu\n\0".as_ptr(), iface, refcount);

    refcount
}
}

stdmethod! {
fn ObjrefMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let moniker = objref_impl_from_IMoniker(iface);
    let refcount = InterlockedDecrement(addr_of_mut!((*moniker).refcount)) as ULONG;

    TRACE!(b"%p, refcount %lu\n\0".as_ptr(), iface, refcount);

    if refcount == 0 {
        if !(*moniker).pObject.is_null() {
            IUnknown_Release((*moniker).pObject);
        }
        HeapFree(GetProcessHeap(), 0, moniker as LPVOID);
    }

    refcount
}
}

stdmethod! {
fn ObjrefMonikerImpl_GetClassID(iface: *mut IMoniker, clsid: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, clsid);

    if clsid.is_null() {
        return E_POINTER;
    }

    *clsid = CLSID_ObjrefMoniker;
    S_OK
}
}

stdmethod! {
fn ObjrefMonikerImpl_IsDirty(_iface: *mut IMoniker) -> HRESULT {
    FIXME!("(%p): stub\n", _iface);
    S_FALSE
}
}

stdmethod! {
fn ObjrefMonikerImpl_Load(_iface: *mut IMoniker, _stream: *mut IStream) -> HRESULT {
    FIXME!("(%p,%p): stub\n", _iface, _stream);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_Save(_iface: *mut IMoniker, _stream: *mut IStream, _dirty: BOOL) -> HRESULT {
    FIXME!("(%p,%p,%d): stub\n", _iface, _stream, _dirty);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_GetSizeMax(_iface: *mut IMoniker, _size: *mut ULARGE_INTEGER) -> HRESULT {
    FIXME!("(%p,%p): stub\n", _iface, _size);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_BindToObject(
    _iface: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _left: *mut IMoniker,
    _riid: *const IID,
    _result: *mut LPVOID,
) -> HRESULT {
    FIXME!("(%p,%p,%p,%s,%p): stub\n", _iface, _pbc, _left, debugstr_guid((_riid).cast()), _result);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_BindToStorage(
    _iface: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _left: *mut IMoniker,
    _riid: *const IID,
    _result: *mut LPVOID,
) -> HRESULT {
    FIXME!("(%p,%p,%p,%s,%p): stub\n", _iface, _pbc, _left, debugstr_guid((_riid).cast()), _result);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_Reduce(
    _iface: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _howfar: DWORD,
    _left: *mut *mut IMoniker,
    _reduced: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!("%p, %p, %ld, %p, %p: stub\n", _iface, _pbc, _howfar, _left, _reduced);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_ComposeWith(
    _iface: *mut IMoniker,
    _right: *mut IMoniker,
    _only_if_not_generic: BOOL,
    _result: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!("(%p,%p,%d,%p): stub\n", _iface, _right, _only_if_not_generic, _result);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_Enum(iface: *mut IMoniker, forward: BOOL, enummoniker: *mut *mut IEnumMoniker) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), iface, forward, enummoniker);

    if enummoniker.is_null() {
        return E_POINTER;
    }

    *enummoniker = core::ptr::null_mut();
    S_OK
}
}

stdmethod! {
fn ObjrefMonikerImpl_IsEqual(_iface: *mut IMoniker, _other: *mut IMoniker) -> HRESULT {
    FIXME!("(%p,%p): stub\n", _iface, _other);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_Hash(iface: *mut IMoniker, hash: *mut DWORD) -> HRESULT {
    let moniker = objref_impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, hash);

    if hash.is_null() {
        return E_POINTER;
    }

    *hash = PtrToUlong((*moniker).pObject as *const core::ffi::c_void);

    S_OK
}
}

stdmethod! {
fn ObjrefMonikerImpl_IsRunning(
    _iface: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _left: *mut IMoniker,
    _running: *mut IMoniker,
) -> HRESULT {
    FIXME!("(%p,%p,%p,%p): stub\n", _iface, _pbc, _left, _running);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_GetTimeOfLastChange(
    _iface: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _left: *mut IMoniker,
    _time: *mut FILETIME,
) -> HRESULT {
    FIXME!("(%p,%p,%p,%p): stub\n", _iface, _pbc, _left, _time);
    MK_E_UNAVAILABLE
}
}

stdmethod! {
fn ObjrefMonikerImpl_Inverse(_iface: *mut IMoniker, _moniker: *mut *mut IMoniker) -> HRESULT {
    FIXME!("(%p,%p): stub\n", _iface, _moniker);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_CommonPrefixWith(
    _iface: *mut IMoniker,
    _other: *mut IMoniker,
    _prefix: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!("(%p,%p,%p): stub\n", _iface, _other, _prefix);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_RelativePathTo(
    _iface: *mut IMoniker,
    _other: *mut IMoniker,
    _result: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!("(%p,%p,%p): stub\n", _iface, _other, _result);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_GetDisplayName(
    _iface: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _left: *mut IMoniker,
    _name: *mut LPOLESTR,
) -> HRESULT {
    FIXME!("(%p,%p,%p,%p): stub\n", _iface, _pbc, _left, _name);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_ParseDisplayName(
    _iface: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _left: *mut IMoniker,
    _name: LPOLESTR,
    _eaten: *mut ULONG,
    _out: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!("(%p,%p,%p,%p,%p,%p): stub\n", _iface, _pbc, _left, _name, _eaten, _out);
    E_NOTIMPL
}
}

stdmethod! {
fn ObjrefMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, mksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, mksys);

    if mksys.is_null() {
        return E_POINTER;
    }

    *mksys = MKSYS_OBJREFMONIKER;
    S_OK
}
}

static VT_ObjrefMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(ObjrefMonikerImpl_QueryInterface),
    AddRef: Some(ObjrefMonikerImpl_AddRef),
    Release: Some(ObjrefMonikerImpl_Release),
    GetClassID: Some(ObjrefMonikerImpl_GetClassID),
    IsDirty: Some(ObjrefMonikerImpl_IsDirty),
    Load: Some(ObjrefMonikerImpl_Load),
    Save: Some(ObjrefMonikerImpl_Save),
    GetSizeMax: Some(ObjrefMonikerImpl_GetSizeMax),
    BindToObject: Some(ObjrefMonikerImpl_BindToObject),
    BindToStorage: Some(ObjrefMonikerImpl_BindToStorage),
    Reduce: Some(ObjrefMonikerImpl_Reduce),
    ComposeWith: Some(ObjrefMonikerImpl_ComposeWith),
    Enum: Some(ObjrefMonikerImpl_Enum),
    IsEqual: Some(ObjrefMonikerImpl_IsEqual),
    Hash: Some(ObjrefMonikerImpl_Hash),
    IsRunning: Some(ObjrefMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(ObjrefMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(ObjrefMonikerImpl_Inverse),
    CommonPrefixWith: Some(ObjrefMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(ObjrefMonikerImpl_RelativePathTo),
    GetDisplayName: Some(ObjrefMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(ObjrefMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(ObjrefMonikerImpl_IsSystemMoniker),
};

stdmethod! {
fn objref_moniker_marshal_QueryInterface(iface: *mut IMarshal, riid: *const IID, ppv: *mut LPVOID) -> HRESULT {
    let moniker = objref_impl_from_IMarshal(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), iface, debugstr_guid(riid), ppv);

    IMoniker_QueryInterface(addr_of_mut!((*moniker).IMoniker_iface), riid, ppv)
}
}

stdmethod! {
fn objref_moniker_marshal_AddRef(iface: *mut IMarshal) -> ULONG {
    let moniker = objref_impl_from_IMarshal(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_AddRef(addr_of_mut!((*moniker).IMoniker_iface))
}
}

stdmethod! {
fn objref_moniker_marshal_Release(iface: *mut IMarshal) -> ULONG {
    let moniker = objref_impl_from_IMarshal(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_Release(addr_of_mut!((*moniker).IMoniker_iface))
}
}

stdmethod! {
fn objref_moniker_marshal_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    clsid: *mut CLSID,
) -> HRESULT {
    let moniker = objref_impl_from_IMarshal(iface);

    TRACE!(b"%p, %s, %p, %#lx, %p, %#lx, %p.\n\0".as_ptr(), iface, debugstr_guid(riid), pv,
        dwDestContext, pvDestContext, mshlflags, clsid);

    IMoniker_GetClassID(addr_of_mut!((*moniker).IMoniker_iface), clsid)
}
}

stdmethod! {
fn objref_moniker_marshal_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    size: *mut DWORD,
) -> HRESULT {
    let moniker = objref_impl_from_IMarshal(iface);

    TRACE!(b"%p, %s, %p, %#lx, %p, %#lx, %p.\n\0".as_ptr(), iface, debugstr_guid(riid), pv,
        dwDestContext, pvDestContext, mshlflags, size);

    CoGetMarshalSizeMax(size, addr_of!(IID_IUnknown), (*moniker).pObject, dwDestContext, pvDestContext, mshlflags)
}
}

stdmethod! {
fn objref_moniker_marshal_MarshalInterface(
    iface: *mut IMarshal,
    stream: *mut IStream,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
) -> HRESULT {
    let moniker = objref_impl_from_IMarshal(iface);

    TRACE!(b"%p, %s, %p, %#lx, %p, %#lx\n\0".as_ptr(), stream, debugstr_guid(riid), pv,
        dwDestContext, pvDestContext, mshlflags);

    CoMarshalInterface((stream).cast(), (addr_of!(IID_IUnknown)).cast(), ((*moniker).pObject).cast(), dwDestContext, pvDestContext, mshlflags)
}
}

stdmethod! {
fn objref_moniker_marshal_UnmarshalInterface(
    iface: *mut IMarshal,
    stream: *mut IStream,
    riid: *const IID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let moniker = objref_impl_from_IMarshal(iface);
    let mut object: *mut IUnknown = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%s,%p)\n\0".as_ptr(), iface, stream, debugstr_guid(riid), ppv);

    let hr = CoUnmarshalInterface((stream).cast(), (addr_of!(IID_IUnknown)).cast(), addr_of_mut!(object) as *mut LPVOID);
    if FAILED(hr) {
        ERR!(b"Couldn't unmarshal moniker, hr = %#lx.\n\0".as_ptr(), hr);
        return hr;
    }

    if !(*moniker).pObject.is_null() {
        IUnknown_Release((*moniker).pObject);
    }
    (*moniker).pObject = object;

    IMoniker_QueryInterface(addr_of_mut!((*moniker).IMoniker_iface), riid, ppv)
}
}

stdmethod! {
fn objref_moniker_marshal_ReleaseMarshalData(iface: *mut IMarshal, stream: *mut IStream) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, stream);
    S_OK
}
}

stdmethod! {
fn objref_moniker_marshal_DisconnectObject(iface: *mut IMarshal, reserved: DWORD) -> HRESULT {
    TRACE!(b"%p, %#lx.\n\0".as_ptr(), iface, reserved);
    S_OK
}
}

static objref_moniker_marshal_vtbl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: Some(objref_moniker_marshal_QueryInterface),
    AddRef: Some(objref_moniker_marshal_AddRef),
    Release: Some(objref_moniker_marshal_Release),
    GetUnmarshalClass: Some(objref_moniker_marshal_GetUnmarshalClass),
    GetMarshalSizeMax: Some(objref_moniker_marshal_GetMarshalSizeMax),
    MarshalInterface: Some(objref_moniker_marshal_MarshalInterface),
    UnmarshalInterface: Some(objref_moniker_marshal_UnmarshalInterface),
    ReleaseMarshalData: Some(objref_moniker_marshal_ReleaseMarshalData),
    DisconnectObject: Some(objref_moniker_marshal_DisconnectObject),
};

/***********************************************************************
 *           CreateObjrefMoniker (OLE32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn CreateObjrefMoniker(obj: *mut IUnknown, ret: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), obj, ret);

    if ret.is_null() {
        return E_INVALIDARG;
    }

    // calloc in the C; HEAP_ZERO_MEMORY is the same zeroed block from this file's allocator.
    let moniker = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<ObjrefMonikerImpl>() as usize)
        as *mut ObjrefMonikerImpl;
    if moniker.is_null() {
        *ret = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }

    (*moniker).IMoniker_iface.lpVtbl = addr_of!(VT_ObjrefMonikerImpl) as *mut IMonikerVtbl;
    (*moniker).IMarshal_iface.lpVtbl = addr_of!(objref_moniker_marshal_vtbl) as *mut IMarshalVtbl;
    (*moniker).refcount = 1;
    (*moniker).pObject = obj;
    if !(*moniker).pObject.is_null() {
        IUnknown_AddRef((*moniker).pObject);
    }

    *ret = addr_of_mut!((*moniker).IMoniker_iface);

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn ObjrefMoniker_CreateInstance(
    _iface: *mut IClassFactory,
    unk: *mut IUnknown,
    iid: REFIID,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut moniker: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), unk, debugstr_guid(iid), obj);

    *obj = core::ptr::null_mut();

    if !unk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let hr = CreateObjrefMoniker(core::ptr::null_mut(), &mut moniker);
    if FAILED(hr) {
        return hr;
    }

    let hr = IMoniker_QueryInterface(moniker, iid, obj);
    IMoniker_Release(moniker);

    hr
}
