use crate::prelude::*;

// These two are DEFINED by sibling ring modules. They are ALSO (wrongly) declared as
// externs in ole32_exports.rs, so the prelude glob makes the bare name ambiguous —
// name them explicitly so we bind to the Rust definition.
use crate::ring::antimoniker::{is_anti_moniker, CreateAntiMoniker};
use crate::ring::compositemoniker::{CreateGenericComposite, MonikerCommonPrefixWith};

extern "system" {
    // compobj.c (Rust, #[no_mangle]) — an ole32 export, so WINAPI/stdcall.
    // Belongs in crate::ole32_exports; declared here until it is added there.
    fn CoGetClassObject(
        rclsid: REFCLSID,
        dwClsContext: DWORD,
        pvReserved: LPVOID,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
}

extern "C" {
    // uuid.lib data symbol. Belongs in crate::iids; declared here until added there.
    static IID_IClassActivator: GUID;
}

// Semantically ported to wine-10.0 (upstream 5f6b295cb90). A class moniker now
// keeps whatever followed the CLSID in its display name ("clsid:<guid>;extra:")
// as `data`, persisted after the header; IsEqual/CommonPrefixWith compare CLSIDs
// by vtable identity instead of display names; ParseDisplayName binds the class
// object's IParseDisplayName instead of returning E_NOTIMPL.

// ---- file-scope preamble (translated) ----
pub const CHARS_IN_GUID: i32 = 39;

/// The persisted header: what Load/Save read and write before the data bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassMonikerHeader {
    pub clsid: CLSID,
    /// Byte length of `data` INCLUDING its NUL, or 0 when there is none.
    pub data_len: DWORD,
}

#[repr(C)]
pub struct ClassMoniker {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub header: ClassMonikerHeader,
    pub data: *mut WCHAR,
    pub pMarshal: *mut IUnknown,
}

const _: () = assert!(core::mem::size_of::<ClassMonikerHeader>() == 20);

pub type PClassMoniker = *mut ClassMoniker;

static ClassMonikerVtbl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(ClassMoniker_QueryInterface),
    AddRef: Some(ClassMoniker_AddRef),
    Release: Some(ClassMoniker_Release),
    GetClassID: Some(ClassMoniker_GetClassID),
    IsDirty: Some(ClassMoniker_IsDirty),
    Load: Some(ClassMoniker_Load),
    Save: Some(ClassMoniker_Save),
    GetSizeMax: Some(ClassMoniker_GetSizeMax),
    BindToObject: Some(ClassMoniker_BindToObject),
    BindToStorage: Some(ClassMoniker_BindToStorage),
    Reduce: Some(ClassMoniker_Reduce),
    ComposeWith: Some(ClassMoniker_ComposeWith),
    Enum: Some(ClassMoniker_Enum),
    IsEqual: Some(ClassMoniker_IsEqual),
    Hash: Some(ClassMoniker_Hash),
    IsRunning: Some(ClassMoniker_IsRunning),
    GetTimeOfLastChange: Some(ClassMoniker_GetTimeOfLastChange),
    Inverse: Some(ClassMoniker_Inverse),
    CommonPrefixWith: Some(ClassMoniker_CommonPrefixWith),
    RelativePathTo: Some(ClassMoniker_RelativePathTo),
    GetDisplayName: Some(ClassMoniker_GetDisplayName),
    ParseDisplayName: Some(ClassMoniker_ParseDisplayName),
    IsSystemMoniker: Some(ClassMoniker_IsSystemMoniker),
};

static ROTDataVtbl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(ClassMonikerROTData_QueryInterface),
    AddRef: Some(ClassMonikerROTData_AddRef),
    Release: Some(ClassMonikerROTData_Release),
    GetComparisonData: Some(ClassMonikerROTData_GetComparisonData),
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut ClassMoniker {
    CONTAINING_RECORD!(iface, ClassMoniker, IMoniker_iface)
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut ClassMoniker {
    CONTAINING_RECORD!(iface, ClassMoniker, IROTData_iface)
}

/// wine-10 `unsafe_impl_from_IMoniker`: OUR class moniker, identified by vtable.
unsafe fn unsafe_impl_from_IMoniker(iface: *mut IMoniker) -> *mut ClassMoniker {
    if iface.is_null()
        || (*iface).lpVtbl as *const IMonikerVtbl != core::ptr::addr_of!(ClassMonikerVtbl)
    {
        return core::ptr::null_mut();
    }
    impl_from_IMoniker(iface)
}

stdmethod! {
fn ClassMoniker_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    /* Perform a sanity check on the parameters. */
    if ppvObject.is_null() {
        return E_POINTER;
    }

    /* Initialize the return parameter */
    *ppvObject = core::ptr::null_mut();

    /* Compare the riid with the interface IDs implemented by this object. */
    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
        || IsEqualGUID(core::ptr::addr_of!(CLSID_ClassMoniker), riid)
    {
        *ppvObject = iface as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) {
        let mut hr = S_OK;
        if (*This).pMarshal.is_null() {
            hr = MonikerMarshal_Create(iface, core::ptr::addr_of_mut!((*This).pMarshal));
        }
        if hr != S_OK {
            return hr;
        }
        return IUnknown_QueryInterface((*This).pMarshal, riid, ppvObject);
    }

    /* Check that we obtained an interface. */
    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    /* QueryInterface always increases the reference count by one when successful */
    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn ClassMoniker_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn ClassMoniker_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references to it */
    if ref_ == 0 {
        if !(*This).pMarshal.is_null() {
            IUnknown_Release((*This).pMarshal);
        }
        if !(*This).data.is_null() {
            HeapFree(GetProcessHeap(), 0, (*This).data as LPVOID);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}
}

stdmethod! {
fn ClassMoniker_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p),stub!\n\0".as_ptr(), iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_ClassMoniker;

    S_OK
}
}

stdmethod! {
fn ClassMoniker_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    S_FALSE
}
}

stdmethod! {
fn ClassMoniker_Load(iface: *mut IMoniker, stream: *mut IStream) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut length: ULONG = 0;
    let header_size = core::mem::size_of::<ClassMonikerHeader>() as ULONG;

    TRACE!(b"%p, %p\n\0".as_ptr(), iface, stream);

    let mut hr = IStream_Read(
        stream,
        core::ptr::addr_of_mut!((*moniker).header) as LPVOID,
        header_size,
        &mut length,
    );
    if hr != S_OK || length != header_size {
        return STG_E_READFAULT;
    }

    if (*moniker).header.data_len != 0 {
        if !(*moniker).data.is_null() {
            HeapFree(GetProcessHeap(), 0, (*moniker).data as LPVOID);
        }
        (*moniker).data =
            HeapAlloc(GetProcessHeap(), 0, (*moniker).header.data_len as SIZE_T) as *mut WCHAR;
        if (*moniker).data.is_null() {
            WARN!(
                b"Failed to allocate moniker data of size %u.\n\0".as_ptr(),
                (*moniker).header.data_len,
            );
            (*moniker).header.data_len = 0;
            return E_OUTOFMEMORY;
        }
        hr = IStream_Read(
            stream,
            (*moniker).data as LPVOID,
            (*moniker).header.data_len,
            &mut length,
        );
        if hr != S_OK || length != (*moniker).header.data_len {
            return STG_E_READFAULT;
        }
    }

    S_OK
}
}

stdmethod! {
fn ClassMoniker_Save(iface: *mut IMoniker, stream: *mut IStream, clear_dirty: BOOL) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p, %d\n\0".as_ptr(), iface, stream, clear_dirty);

    let mut hr = IStream_Write(
        stream,
        core::ptr::addr_of!((*moniker).header) as *const core::ffi::c_void,
        core::mem::size_of::<ClassMonikerHeader>() as ULONG,
        core::ptr::null_mut(),
    );

    if SUCCEEDED(hr) && (*moniker).header.data_len != 0 {
        hr = IStream_Write(
            stream,
            (*moniker).data as *const core::ffi::c_void,
            (*moniker).header.data_len,
            core::ptr::null_mut(),
        );
    }

    hr
}
}

stdmethod! {
fn ClassMoniker_GetSizeMax(iface: *mut IMoniker, size: *mut ULARGE_INTEGER) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p\n\0".as_ptr(), iface, size);

    (*size).QuadPart = (core::mem::size_of::<ClassMonikerHeader>() as ULONGLONG)
        + (*moniker).header.data_len as ULONGLONG;

    S_OK
}
}

stdmethod! {
fn ClassMoniker_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut bindopts: BIND_OPTS2 = core::mem::zeroed();
    let mut pActivator: *mut IClassActivator = core::ptr::null_mut();

    TRACE!(
        b"(%p, %p, %s, %p)\n\0".as_ptr(),
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    bindopts.cbStruct = core::mem::size_of::<BIND_OPTS2>() as DWORD;
    IBindCtx_GetBindOptions(pbc, core::ptr::addr_of_mut!(bindopts) as *mut BIND_OPTS);

    if pmkToLeft.is_null() {
        CoGetClassObject(
            core::ptr::addr_of!((*This).header.clsid),
            bindopts.dwClassContext,
            core::ptr::null_mut(),
            riid,
            ppvResult,
        )
    } else {
        let mut hr = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IClassActivator),
            core::ptr::addr_of_mut!(pActivator) as *mut LPVOID,
        );
        if FAILED(hr) {
            return hr;
        }

        hr = IClassActivator_GetClassObject(
            pActivator,
            core::ptr::addr_of!((*This).header.clsid),
            bindopts.dwClassContext,
            bindopts.locale,
            riid,
            ppvResult,
        );

        IClassActivator_Release(pActivator);

        hr
    }
}
}

stdmethod! {
fn ClassMoniker_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, %s, %p)\n\0".as_ptr(),
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    IMoniker_BindToObject(iface, pbc, pmkToLeft, riid, ppvResult)
}
}

stdmethod! {
fn ClassMoniker_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%d,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        dwReduceHowFar,
        ppmkToLeft,
        ppmkReduced,
    );

    if ppmkReduced.is_null() {
        return E_POINTER;
    }

    IMoniker_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

stdmethod! {
fn ClassMoniker_ComposeWith(
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

    /* Any anti-moniker, of any order, cancels a class moniker to nothing. */
    if is_anti_moniker(right, &mut order) {
        return S_OK;
    }

    if only_if_not_generic != 0 {
        MK_E_NEEDGENERIC
    } else {
        CreateGenericComposite(iface, right, result)
    }
}
}

stdmethod! {
fn ClassMoniker_Enum(
    iface: *mut IMoniker,
    fForward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), iface, fForward, ppenumMoniker);

    if ppenumMoniker.is_null() {
        return E_POINTER;
    }

    *ppenumMoniker = core::ptr::null_mut();

    S_OK
}
}

stdmethod! {
fn ClassMoniker_IsEqual(iface: *mut IMoniker, other: *mut IMoniker) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p.\n\0".as_ptr(), iface, other);

    if other.is_null() {
        return E_INVALIDARG;
    }

    let other_moniker = unsafe_impl_from_IMoniker(other);
    if other_moniker.is_null() {
        return S_FALSE;
    }

    if IsEqualGUID(
        core::ptr::addr_of!((*moniker).header.clsid),
        core::ptr::addr_of!((*other_moniker).header.clsid),
    ) {
        S_OK
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn ClassMoniker_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), pdwHash);

    *pdwHash = (*This).header.clsid.Data1;

    S_OK
}
}

stdmethod! {
fn ClassMoniker_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), pbc, pmkToLeft, pmkNewlyRunning);

    /* as in native */
    E_NOTIMPL
}
}

stdmethod! {
fn ClassMoniker_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pItemTime: *mut FILETIME,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), pbc, pmkToLeft, pItemTime);

    MK_E_UNAVAILABLE
}
}

stdmethod! {
fn ClassMoniker_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    CreateAntiMoniker(ppmk)
}
}

stdmethod! {
fn ClassMoniker_CommonPrefixWith(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    prefix: *mut *mut IMoniker,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p, %p\n\0".as_ptr(), iface, other, prefix);

    *prefix = core::ptr::null_mut();

    let other_moniker = unsafe_impl_from_IMoniker(other);

    if !other_moniker.is_null() {
        if !IsEqualGUID(
            core::ptr::addr_of!((*moniker).header.clsid),
            core::ptr::addr_of!((*other_moniker).header.clsid),
        ) {
            return MK_E_NOPREFIX;
        }

        *prefix = iface;
        IMoniker_AddRef(iface);

        return MK_S_US;
    }

    MonikerCommonPrefixWith(iface, other, prefix)
}
}

stdmethod! {
fn ClassMoniker_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr(), pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    MK_E_NOTBINDABLE
}
}

stdmethod! {
fn ClassMoniker_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    name: *mut LPOLESTR,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    /* "clsid:" + 36 GUID chars + ":" + NUL fits in CHARS_IN_GUID + 5 */
    let name_len: usize = CHARS_IN_GUID as usize + 5;
    let mut guid_str: [WCHAR; 39] = [0; 39];

    TRACE!(b"%p, %p, %p, %p.\n\0".as_ptr(), iface, pbc, pmkToLeft, name);

    if name.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    let out = CoTaskMemAlloc(
        (name_len * core::mem::size_of::<WCHAR>() + (*moniker).header.data_len as usize) as SIZE_T,
    ) as *mut WCHAR;
    *name = out;
    if out.is_null() {
        return E_OUTOFMEMORY;
    }

    /* C: swprintf "clsid:%08X-%04X-%04X-%02X%02X-%02X..." -- StringFromGUID2 is the
       same upper-case digits inside "{...}", so take the 36 chars between the braces. */
    StringFromGUID2(
        core::ptr::addr_of!((*moniker).header.clsid),
        guid_str.as_mut_ptr(),
        CHARS_IN_GUID as c_int,
    );
    let prefix = [b'c', b'l', b's', b'i', b'd', b':'];
    let mut n = 0usize;
    for c in prefix {
        *out.add(n) = c as WCHAR;
        n += 1;
    }
    for i in 0..36 {
        *out.add(n) = guid_str[1 + i];
        n += 1;
    }
    *out.add(n) = 0;

    if (*moniker).header.data_len != 0 {
        lstrcatW(out, (*moniker).data);
    }
    let colon: [WCHAR; 2] = [b':' as WCHAR, 0];
    lstrcatW(out, colon.as_ptr());

    TRACE!(b"Returning %s\n\0".as_ptr(), debugstr_w(*name));

    S_OK
}
}

stdmethod! {
fn ClassMoniker_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    display_name: LPOLESTR,
    eaten: *mut ULONG,
    result: *mut *mut IMoniker,
) -> HRESULT {
    let mut parser: *mut IParseDisplayName = core::ptr::null_mut();

    TRACE!(
        b"%p, %p, %p, %s, %p, %p\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_w(display_name),
        eaten,
        result,
    );

    let mut hr = IMoniker_BindToObject(
        iface,
        pbc,
        pmkToLeft,
        core::ptr::addr_of!(IID_IParseDisplayName),
        core::ptr::addr_of_mut!(parser) as *mut LPVOID,
    );
    if SUCCEEDED(hr) {
        hr = IParseDisplayName_ParseDisplayName(parser, pbc, display_name, eaten, result);
        IParseDisplayName_Release(parser);
    }

    hr
}
}

stdmethod! {
fn ClassMoniker_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_CLASSMONIKER;

    S_OK
}
}

stdmethod! {
fn ClassMonikerROTData_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), iface, debugstr_guid(riid), ppvObject);

    IMoniker_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}
}

stdmethod! {
fn ClassMonikerROTData_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ClassMonikerROTData_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ClassMonikerROTData_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut byte,
    cbMax: ULONG,
    pcbData: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p, %u, %p)\n\0".as_ptr(), pbData, cbMax, pcbData);

    *pcbData = (2 * core::mem::size_of::<CLSID>()) as ULONG;
    if cbMax < *pcbData {
        return E_OUTOFMEMORY;
    }

    /* write CLSID of the moniker */
    memcpy(
        pbData as *mut core::ffi::c_void,
        core::ptr::addr_of!(CLSID_ClassMoniker) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );
    /* write CLSID the moniker represents */
    memcpy(
        pbData.add(core::mem::size_of::<CLSID>()) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*This).header.clsid) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );

    S_OK
}
}

/// wine-10 `create_class_moniker`: refcount 1, with `data_len` WCHARs of trailing
/// display-name data (stored NUL-terminated) when there is any.
unsafe fn create_class_moniker(
    clsid: *const CLSID,
    data: *const WCHAR,
    data_len: usize,
    moniker: *mut *mut IMoniker,
) -> HRESULT {
    let object = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<ClassMoniker>() as SIZE_T,
    ) as *mut ClassMoniker;

    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(ClassMonikerVtbl) as *mut IMonikerVtbl;
    (*object).IROTData_iface.lpVtbl = core::ptr::addr_of!(ROTDataVtbl) as *mut IROTDataVtbl;
    (*object).ref_ = 1;
    (*object).header.clsid = *clsid;
    if data_len != 0 {
        (*object).header.data_len = ((data_len + 1) * core::mem::size_of::<WCHAR>()) as DWORD;

        (*object).data =
            HeapAlloc(GetProcessHeap(), 0, (*object).header.data_len as SIZE_T) as *mut WCHAR;
        if (*object).data.is_null() {
            IMoniker_Release(core::ptr::addr_of_mut!((*object).IMoniker_iface));
            return E_OUTOFMEMORY;
        }
        core::ptr::copy_nonoverlapping(data, (*object).data, data_len);
        *(*object).data.add(data_len) = 0;
    }

    *moniker = core::ptr::addr_of_mut!((*object).IMoniker_iface);

    S_OK
}

/******************************************************************************
 *        CreateClassMoniker	[OLE32.@]
 ******************************************************************************/
#[no_mangle]
pub unsafe extern "system" fn CreateClassMoniker(
    rclsid: REFCLSID,
    moniker: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%s, %p\n\0".as_ptr(), debugstr_guid(rclsid), moniker);

    create_class_moniker(rclsid, core::ptr::null(), 0, moniker)
}

/// wine-10: "clsid:" prefix (any case), a 36-char GUID optionally in braces, then
/// anything up to an optional terminating ':' kept as the moniker's data. The old
/// parser demanded the GUID fill the whole span, so "clsid:<guid>;extra data:"
/// was MK_E_SYNTAX; MkParseDisplayName then left *ppmk NULL and the caller crashed.
#[no_mangle]
pub unsafe extern "C" fn ClassMoniker_CreateFromDisplayName(
    _pbc: *mut IBindCtx,
    display_name: LPCOLESTR,
    eaten: LPDWORD,
    moniker: *mut *mut IMoniker,
) -> HRESULT {
    let prefix: [WCHAR; 7] = [
        b'c' as WCHAR,
        b'l' as WCHAR,
        b's' as WCHAR,
        b'i' as WCHAR,
        b'd' as WCHAR,
        b':' as WCHAR,
        0,
    ];
    let mut clsid: CLSID = core::mem::zeroed();
    let mut s: LPCWSTR = display_name;

    /* Skip prefix */
    if crate::crt::_wcsnicmp(s, prefix.as_ptr(), 6) != 0 {
        return MK_E_SYNTAX;
    }
    s = s.add(6);

    /* Terminating marker is optional */
    let mut end: LPCWSTR = wcschr(s, b':' as WCHAR);
    if end.is_null() {
        end = s.add(lstrlenW(s) as usize);
    }

    if end.offset_from(s) < 36 {
        return MK_E_SYNTAX;
    }

    let has_braces = *s == b'{' as WCHAR;
    if has_braces {
        s = s.add(1);
    }

    /* C: UuidFromStringW on the 36 chars -- validate the shape, then decode. */
    for i in 0..36isize {
        let c = *s.offset(i);
        if i == 8 || i == 13 || i == 18 || i == 23 {
            if c != b'-' as WCHAR {
                return MK_E_SYNTAX;
            }
        } else if !((c >= b'0' as WCHAR && c <= b'9' as WCHAR)
            || (c >= b'a' as WCHAR && c <= b'f' as WCHAR)
            || (c >= b'A' as WCHAR && c <= b'F' as WCHAR))
        {
            WARN!(b"Failed to parse clsid string.\n\0".as_ptr());
            return MK_E_SYNTAX;
        }
    }
    let nib = |n: isize| -> u32 {
        let c = *s.offset(n) as u32;
        match c {
            0x30..=0x39 => c - 0x30,
            0x41..=0x46 => c - 0x41 + 10,
            _ => c - 0x61 + 10,
        }
    };
    let byte = |n: isize| -> u8 { ((nib(n) << 4) | nib(n + 1)) as u8 };

    clsid.Data1 = (nib(0) << 28)
        | (nib(1) << 24)
        | (nib(2) << 20)
        | (nib(3) << 16)
        | (nib(4) << 12)
        | (nib(5) << 8)
        | (nib(6) << 4)
        | nib(7);
    clsid.Data2 = ((nib(9) << 12) | (nib(10) << 8) | (nib(11) << 4) | nib(12)) as u16;
    clsid.Data3 = ((nib(14) << 12) | (nib(15) << 8) | (nib(16) << 4) | nib(17)) as u16;
    clsid.Data4[0] = byte(19);
    clsid.Data4[1] = byte(21);
    clsid.Data4[2] = byte(24);
    clsid.Data4[3] = byte(26);
    clsid.Data4[4] = byte(28);
    clsid.Data4[5] = byte(30);
    clsid.Data4[6] = byte(32);
    clsid.Data4[7] = byte(34);

    s = s.add(36);
    if has_braces {
        if *s != b'}' as WCHAR {
            return MK_E_SYNTAX;
        }
        s = s.add(1);
    }

    /* Consume terminal marker. A brace consumed past `end` (an unbraced 36-char span
       that merely began with '{') makes this negative; C's int len would too, and
       its `len ? s : NULL` then passes a negative length -- treat that as syntax. */
    let len = end.offset_from(s);
    if len < 0 {
        return MK_E_SYNTAX;
    }
    if *end == b':' as WCHAR {
        end = end.add(1);
    }

    let hr = create_class_moniker(
        &clsid,
        if len != 0 { s } else { core::ptr::null() },
        len as usize,
        moniker,
    );
    if SUCCEEDED(hr) {
        /* character count, NOT byte count -- C did WCHAR* - WCHAR* */
        *eaten = end.offset_from(display_name) as DWORD;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn ClassMoniker_CreateInstance(
    _iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut pmk: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), pUnk, debugstr_guid(riid), ppv);

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let hr = CreateClassMoniker(core::ptr::addr_of!(CLSID_NULL), &mut pmk);
    if FAILED(hr) {
        return hr;
    }

    let hr = IMoniker_QueryInterface(pmk, riid, ppv);
    IMoniker_Release(pmk);

    hr
}
