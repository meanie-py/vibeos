use crate::prelude::*;

// Defined by sibling ring modules, but ALSO (wrongly) declared as externs in
// ole32_exports.rs — the prelude glob makes the bare names ambiguous, so bind explicitly.
use crate::ring::antimoniker::{create_anti_moniker, is_anti_moniker, CreateAntiMoniker};
use crate::ring::compositemoniker::{CreateGenericComposite, MonikerCommonPrefixWith};

extern "C" {
    // uuid.lib data symbol. Belongs in crate::iids; declared here until added there.
    static IID_IOleItemContainer: IID;
}

// Semantically ported to wine-10.0 (upstream 5f6b295cb90). Behaviour changes over the
// old translation: a NULL delimiter stays NULL (it was coerced to ""), Load reads both
// records through item_moniker_load_string_record (bounded, optional Unicode tail),
// binding takes a container lock registered on the bind context and derives the bind
// speed from it, IsEqual compares item names case-insensitively by vtable identity,
// IsRunning requires the container itself to be running, ComposeWith cancels one
// order of an anti-moniker, and ParseDisplayName parses the CALLER's string through
// the item's IParseDisplayName (it used to call through an unchecked pointer).

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ItemMonikerImpl {
    /// VTable relative to the IMoniker interface.
    pub IMoniker_iface: IMoniker,
    /// VTable relative to the IROTData interface.
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    /// item name identified by this ItemMoniker
    pub itemName: LPOLESTR,
    /// Delimiter string; NULL when created with no delimiter.
    pub itemDelimiter: LPOLESTR,
    /// custom marshaler
    pub pMarshal: *mut IUnknown,
}

pub type PItemMonikerImpl = *mut ItemMonikerImpl;

static VT_ItemMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(ItemMonikerImpl_QueryInterface),
    AddRef: Some(ItemMonikerImpl_AddRef),
    Release: Some(ItemMonikerImpl_Release),
    GetClassID: Some(ItemMonikerImpl_GetClassID),
    IsDirty: Some(ItemMonikerImpl_IsDirty),
    Load: Some(ItemMonikerImpl_Load),
    Save: Some(ItemMonikerImpl_Save),
    GetSizeMax: Some(ItemMonikerImpl_GetSizeMax),
    BindToObject: Some(ItemMonikerImpl_BindToObject),
    BindToStorage: Some(ItemMonikerImpl_BindToStorage),
    Reduce: Some(ItemMonikerImpl_Reduce),
    ComposeWith: Some(ItemMonikerImpl_ComposeWith),
    Enum: Some(ItemMonikerImpl_Enum),
    IsEqual: Some(ItemMonikerImpl_IsEqual),
    Hash: Some(ItemMonikerImpl_Hash),
    IsRunning: Some(ItemMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(ItemMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(ItemMonikerImpl_Inverse),
    CommonPrefixWith: Some(ItemMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(ItemMonikerImpl_RelativePathTo),
    GetDisplayName: Some(ItemMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(ItemMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(ItemMonikerImpl_IsSystemMoniker),
};

static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(ItemMonikerROTDataImpl_QueryInterface),
    AddRef: Some(ItemMonikerROTDataImpl_AddRef),
    Release: Some(ItemMonikerROTDataImpl_Release),
    GetComparisonData: Some(ItemMonikerROTDataImpl_GetComparisonData),
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut ItemMonikerImpl {
    CONTAINING_RECORD!(iface, ItemMonikerImpl, IMoniker_iface)
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut ItemMonikerImpl {
    CONTAINING_RECORD!(iface, ItemMonikerImpl, IROTData_iface)
}

/// wine-10 `unsafe_impl_from_IMoniker`: OUR item moniker, identified by vtable.
unsafe fn unsafe_impl_from_IMoniker(iface: *mut IMoniker) -> *mut ItemMonikerImpl {
    if iface.is_null()
        || (*iface).lpVtbl as *const IMonikerVtbl != core::ptr::addr_of!(VT_ItemMonikerImpl)
    {
        return core::ptr::null_mut();
    }
    impl_from_IMoniker(iface)
}

unsafe fn item_free(p: *mut WCHAR) {
    if !p.is_null() {
        HeapFree(GetProcessHeap(), 0, p as LPVOID);
    }
}

// ---- container lock: keeps the item container locked for the bind context's life ----

#[repr(C)]
struct container_lock {
    IUnknown_iface: IUnknown,
    refcount: LONG,
    container: *mut IOleItemContainer,
}

static container_lock_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(container_lock_QueryInterface),
    AddRef: Some(container_lock_AddRef),
    Release: Some(container_lock_Release),
};

unsafe fn impl_lock_from_IUnknown(iface: *mut IUnknown) -> *mut container_lock {
    CONTAINING_RECORD!(iface, container_lock, IUnknown_iface)
}

stdmethod! {
fn container_lock_QueryInterface(iface: *mut IUnknown, riid: *const IID, obj: *mut LPVOID) -> HRESULT {
    if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown)) {
        *obj = iface as LPVOID;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    WARN!(b"Unsupported interface %s.\n\0".as_ptr(), debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    E_NOINTERFACE
}
}

stdmethod! {
fn container_lock_AddRef(iface: *mut IUnknown) -> ULONG {
    let lock = impl_lock_from_IUnknown(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*lock).refcount)) as ULONG
}
}

stdmethod! {
fn container_lock_Release(iface: *mut IUnknown) -> ULONG {
    let lock = impl_lock_from_IUnknown(iface);
    let refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*lock).refcount)) as ULONG;

    if refcount == 0 {
        IOleItemContainer_LockContainer((*lock).container, 0);
        IOleItemContainer_Release((*lock).container);
        HeapFree(GetProcessHeap(), 0, lock as LPVOID);
    }

    refcount
}
}

unsafe fn set_container_lock(container: *mut IOleItemContainer, pbc: *mut IBindCtx) -> HRESULT {
    let lock = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<container_lock>() as usize,
    ) as *mut container_lock;
    if lock.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut hr = IOleItemContainer_LockContainer(container, 1);
    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, lock as LPVOID);
        return hr;
    }

    (*lock).IUnknown_iface.lpVtbl = core::ptr::addr_of!(container_lock_vtbl) as *mut IUnknownVtbl;
    (*lock).refcount = 1;
    (*lock).container = container;
    IOleItemContainer_AddRef((*lock).container);

    hr = IBindCtx_RegisterObjectBound(pbc, core::ptr::addr_of_mut!((*lock).IUnknown_iface));
    IUnknown_Release(core::ptr::addr_of_mut!((*lock).IUnknown_iface));
    hr
}

unsafe fn get_bind_speed_from_bindctx(pbc: *mut IBindCtx) -> DWORD {
    let mut bind_speed: DWORD = BINDSPEED_INDEFINITE as DWORD;
    let mut bind_opts: BIND_OPTS = core::mem::zeroed();

    bind_opts.cbStruct = core::mem::size_of::<BIND_OPTS>() as DWORD;
    if SUCCEEDED(IBindCtx_GetBindOptions(pbc, &mut bind_opts)) && bind_opts.dwTickCountDeadline != 0 {
        bind_speed = if bind_opts.dwTickCountDeadline < 2500 {
            BINDSPEED_IMMEDIATE as DWORD
        } else {
            BINDSPEED_MODERATE as DWORD
        };
    }

    bind_speed
}

stdmethod! {
fn ItemMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
        || IsEqualGUID(core::ptr::addr_of!(CLSID_ItemMoniker), riid)
    {
        *ppvObject = iface as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) {
        let mut hr: HRESULT = S_OK;
        if (*This).pMarshal.is_null() {
            hr = MonikerMarshal_Create(iface, core::ptr::addr_of_mut!((*This).pMarshal));
        }
        if hr != S_OK {
            return hr;
        }
        return IUnknown_QueryInterface((*This).pMarshal, riid, ppvObject);
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn ItemMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let moniker = impl_from_IMoniker(iface);
    let refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*moniker).ref_)) as ULONG;

    TRACE!(b"%p, refcount %u.\n\0".as_ptr(), iface, refcount);

    if refcount == 0 {
        if !(*moniker).pMarshal.is_null() {
            IUnknown_Release((*moniker).pMarshal);
        }
        item_free((*moniker).itemName);
        item_free((*moniker).itemDelimiter);
        HeapFree(GetProcessHeap(), 0, moniker as LPVOID);
    }

    refcount
}
}

stdmethod! {
fn ItemMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_ItemMoniker;

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    S_FALSE
}
}

/// wine-10 `item_moniker_load_string_record`: a DWORD byte length, then a
/// NUL-terminated ANSI string, optionally followed by an unterminated WCHAR string
/// filling the rest of the record (which wins when present). Length 0 means NULL.
/// Every read is bounded by the record length -- the old translation ran
/// MultiByteToWideChar(-1) over an unterminated buffer.
unsafe fn item_moniker_load_string_record(stream: *mut IStream, ret: *mut *mut WCHAR) -> HRESULT {
    let mut str_len: DWORD = 0;
    let mut read_len: ULONG = 0;
    let mut hr: HRESULT = S_OK;
    let str_: *mut WCHAR;

    IStream_Read(
        stream,
        core::ptr::addr_of_mut!(str_len) as LPVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        &mut read_len,
    );
    if read_len as usize != core::mem::size_of::<DWORD>() {
        return E_FAIL;
    }

    if str_len == 0 {
        item_free(*ret);
        *ret = core::ptr::null_mut();
        return S_OK;
    }

    let buffer = HeapAlloc(GetProcessHeap(), 0, str_len as usize) as *mut c_char;
    if buffer.is_null() {
        return E_OUTOFMEMORY;
    }

    IStream_Read(stream, buffer as LPVOID, str_len, &mut read_len);
    if read_len != str_len {
        HeapFree(GetProcessHeap(), 0, buffer as LPVOID);
        return E_FAIL;
    }

    'end: {
        /* Skip ansi buffer, it must be null terminated. */
        let mut i: DWORD = 0;
        while i < str_len && *buffer.add(i as usize) != 0 {
            i += 1;
        }

        /* C reads buffer[i] here even when i == str_len (one past the end); the
           in-bounds reading of that test is "no NUL found". */
        if i == str_len || *buffer.add(i as usize) != 0 {
            WARN!(b"Expected null terminated ansi name.\n\0".as_ptr());
            hr = E_FAIL;
            break 'end;
        }

        if i < str_len - 1 {
            let wlen = str_len - (i + 1);

            if wlen as usize % core::mem::size_of::<WCHAR>() != 0 {
                WARN!(b"Unexpected Unicode name length %u.\n\0".as_ptr(), wlen);
                hr = E_FAIL;
                break 'end;
            }

            str_ = HeapAlloc(
                GetProcessHeap(),
                0,
                (wlen as usize + core::mem::size_of::<WCHAR>()) as usize,
            ) as *mut WCHAR;
            if !str_.is_null() {
                core::ptr::copy_nonoverlapping(
                    buffer.add(i as usize + 1) as *const u8,
                    str_ as *mut u8,
                    wlen as usize,
                );
                *str_.add(wlen as usize / core::mem::size_of::<WCHAR>()) = 0;
            }
        } else {
            let lenW = MultiByteToWideChar(CP_ACP, 0, buffer, -1, core::ptr::null_mut(), 0);
            str_ = HeapAlloc(
                GetProcessHeap(),
                0,
                (lenW as usize * core::mem::size_of::<WCHAR>()) as usize,
            ) as *mut WCHAR;
            if !str_.is_null() {
                MultiByteToWideChar(CP_ACP, 0, buffer, -1, str_, lenW);
            }
        }

        if !str_.is_null() {
            item_free(*ret);
            *ret = str_;
        } else {
            hr = E_OUTOFMEMORY;
        }
    }

    HeapFree(GetProcessHeap(), 0, buffer as LPVOID);

    hr
}

stdmethod! {
fn ItemMonikerImpl_Load(iface: *mut IMoniker, stream: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p, %p)\n\0".as_ptr(), iface, stream);

    /* Delimiter and name use the same record structure: 4 bytes byte-length field,
       followed by string data. Data starts with single byte null-terminated string,
       WCHAR non-terminated string optionally follows. Length of WCHAR string is
       determined as a difference between total byte-length and single byte string
       length. */

    let mut hr = item_moniker_load_string_record(stream, core::ptr::addr_of_mut!((*This).itemDelimiter));
    if SUCCEEDED(hr) {
        hr = item_moniker_load_string_record(stream, core::ptr::addr_of_mut!((*This).itemName));
    }

    hr
}
}

/// Writes one Save record: DWORD ANSI byte length (NUL included), then the bytes.
unsafe fn item_moniker_save_string(stream: *mut IStream, s: *const WCHAR) -> HRESULT {
    let str_len: c_int = WideCharToMultiByte(
        CP_ACP,
        0,
        s,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    let buf = HeapAlloc(GetProcessHeap(), 0, str_len as usize) as *mut c_char;
    WideCharToMultiByte(
        CP_ACP,
        0,
        s,
        -1,
        buf,
        str_len,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    IStream_Write(
        stream,
        core::ptr::addr_of!(str_len) as *const core::ffi::c_void,
        core::mem::size_of::<c_int>() as ULONG,
        core::ptr::null_mut(),
    );
    let hr = IStream_Write(stream, buf as *const core::ffi::c_void, str_len as ULONG, core::ptr::null_mut());
    if !buf.is_null() {
        HeapFree(GetProcessHeap(), 0, buf as LPVOID);
    }
    hr
}

stdmethod! {
fn ItemMonikerImpl_Save(iface: *mut IMoniker, stream: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p, %p, %d)\n\0".as_ptr(), iface, stream, fClearDirty);

    /* data written by this function are : 1) DWORD : size of item delimiter string ('\0' included )
                                           2) String (type A): item delimiter string ('\0' included)
                                           3) DWORD : size of item name string ('\0' included)
                                           4) String (type A): item name string ('\0' included)
       A NULL delimiter is written as a zero length and no bytes. */
    if !(*This).itemDelimiter.is_null() {
        item_moniker_save_string(stream, (*This).itemDelimiter);
    } else {
        let str_len: c_int = 0;
        IStream_Write(
            stream,
            core::ptr::addr_of!(str_len) as *const core::ffi::c_void,
            core::mem::size_of::<c_int>() as ULONG,
            core::ptr::null_mut(),
        );
    }

    item_moniker_save_string(stream, (*This).itemName)
}
}

stdmethod! {
fn ItemMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    let nameLength = lstrlenW((*This).itemName) as DWORD + 1;

    /* for more details see ItemMonikerImpl_Save comments */
    let mut low: DWORD = (core::mem::size_of::<DWORD>() /* delimiter length */
        + core::mem::size_of::<DWORD>() /* item name length */) as DWORD
        + nameLength * 4 /* item name string */
        + 18; /* strange, but true */
    if !(*This).itemDelimiter.is_null() {
        low += (lstrlenW((*This).itemDelimiter) as DWORD + 1) * 4;
    }
    (*pcbSize).QuadPart = low as ULONGLONG;

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut container: *mut IOleItemContainer = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    if ppvResult.is_null() {
        return E_POINTER;
    }

    if pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    *ppvResult = core::ptr::null_mut();

    let mut hr = IMoniker_BindToObject(
        pmkToLeft,
        pbc,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IOleItemContainer),
        core::ptr::addr_of_mut!(container) as *mut LPVOID,
    );
    if SUCCEEDED(hr) {
        hr = set_container_lock(container, pbc);
        if FAILED(hr) {
            WARN!(b"Failed to lock container, hr %#x.\n\0".as_ptr(), hr);
        }

        hr = IOleItemContainer_GetObject(
            container,
            (*This).itemName,
            get_bind_speed_from_bindctx(pbc),
            pbc,
            riid,
            ppvResult,
        );
        IOleItemContainer_Release(container);
    }

    hr
}
}

stdmethod! {
fn ItemMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut container: *mut IOleItemContainer = core::ptr::null_mut();

    TRACE!(
        b"%p, %p, %p, %s, %p.\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    *ppvResult = core::ptr::null_mut();

    if pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    let mut hr = IMoniker_BindToObject(
        pmkToLeft,
        pbc,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IOleItemContainer),
        core::ptr::addr_of_mut!(container) as *mut LPVOID,
    );
    if SUCCEEDED(hr) {
        hr = set_container_lock(container, pbc);
        if FAILED(hr) {
            WARN!(b"Failed to lock container, hr %#x.\n\0".as_ptr(), hr);
        }

        hr = IOleItemContainer_GetObjectStorage(container, (*moniker).itemName, pbc, riid, ppvResult);
        IOleItemContainer_Release(container);
    }

    hr
}
}

stdmethod! {
fn ItemMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"%p, %p, %d, %p, %p.\n\0".as_ptr(),
        iface,
        pbc,
        dwReduceHowFar,
        ppmkToLeft,
        ppmkReduced,
    );

    if ppmkReduced.is_null() {
        return E_POINTER;
    }

    ItemMonikerImpl_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

stdmethod! {
fn ItemMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    right: *mut IMoniker,
    only_if_not_generic: BOOL,
    result: *mut *mut IMoniker,
) -> HRESULT {
    let mut order: DWORD = 0;

    TRACE!(b"%p, %p, %d, %p\n\0".as_ptr(), iface, right, only_if_not_generic, result);

    if result.is_null() || right.is_null() {
        return E_POINTER;
    }

    *result = core::ptr::null_mut();

    /* An anti-moniker of order N cancels this item and leaves order N-1. */
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
fn ItemMonikerImpl_Enum(
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
fn ItemMonikerImpl_IsEqual(iface: *mut IMoniker, other: *mut IMoniker) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p.\n\0".as_ptr(), iface, other);

    if other.is_null() {
        return E_INVALIDARG;
    }

    let other_moniker = unsafe_impl_from_IMoniker(other);
    if other_moniker.is_null() {
        return S_FALSE;
    }

    if crate::compobj::ffi::_wcsicmp((*moniker).itemName, (*other_moniker).itemName) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn ItemMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut h: DWORD = 0;
    let mut off: isize = 0;

    if pdwHash.is_null() {
        return E_POINTER;
    }

    let val = (*This).itemName;
    let len = lstrlenW(val);

    let mut i = len;
    while i > 0 {
        h = h.wrapping_mul(3) ^ (towupper(*val.offset(off) as u16) as DWORD);
        off += 1;
        i -= 1;
    }

    *pdwHash = h;

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut container: *mut IOleItemContainer = core::ptr::null_mut();
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pmkNewlyRunning);

    if pbc.is_null() {
        return E_INVALIDARG;
    }

    if pmkToLeft.is_null() {
        if !pmkNewlyRunning.is_null() {
            return IMoniker_IsEqual(iface, pmkNewlyRunning);
        }

        hr = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if SUCCEEDED(hr) {
            hr = IRunningObjectTable_IsRunning(rot, iface);
            IRunningObjectTable_Release(rot);
        }
    } else {
        /* Container itself must be running too. */
        hr = IMoniker_IsRunning(pmkToLeft, pbc, core::ptr::null_mut(), core::ptr::null_mut());
        if hr != S_OK {
            return hr;
        }

        hr = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IOleItemContainer),
            core::ptr::addr_of_mut!(container) as *mut LPVOID,
        );
        if SUCCEEDED(hr) {
            hr = IOleItemContainer_IsRunning(container, (*moniker).itemName);
            IOleItemContainer_Release(container);
        }
    }

    hr
}
}

stdmethod! {
fn ItemMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pItemTime: *mut FILETIME,
) -> HRESULT {
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut compositeMk: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pItemTime);

    if pItemTime.is_null() {
        return E_INVALIDARG;
    }

    /* If pmkToLeft is NULL, this method returns MK_E_NOTBINDABLE */
    if pmkToLeft.is_null() {
        return MK_E_NOTBINDABLE;
    }

    /* Otherwise, the method creates a composite of pmkToLeft and this moniker and uses
       the ROT to access the time of last change. If the object is not in the ROT, the
       method calls IMoniker::GetTimeOfLastChange on the pmkToLeft parameter. */
    let mut res = CreateGenericComposite(pmkToLeft, iface, &mut compositeMk);
    if FAILED(res) {
        return res;
    }

    res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
    if FAILED(res) {
        IMoniker_Release(compositeMk);
        return res;
    }

    if IRunningObjectTable_GetTimeOfLastChange(rot, compositeMk, pItemTime) != S_OK {
        res = IMoniker_GetTimeOfLastChange(pmkToLeft, pbc, core::ptr::null_mut(), pItemTime);
    }

    IMoniker_Release(compositeMk);

    res
}
}

stdmethod! {
fn ItemMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    CreateAntiMoniker(ppmk)
}
}

stdmethod! {
fn ItemMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    prefix: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%p, %p, %p\n\0".as_ptr(), iface, other, prefix);

    if IMoniker_IsEqual(iface, other) == S_OK {
        *prefix = iface;
        IMoniker_AddRef(iface);
        return MK_S_US;
    }

    MonikerCommonPrefixWith(iface, other, prefix)
}
}

stdmethod! {
fn ItemMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    result: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%p, %p, %p.\n\0".as_ptr(), iface, other, result);

    if other.is_null() || result.is_null() {
        return E_INVALIDARG;
    }

    *result = core::ptr::null_mut();

    MK_E_NOTBINDABLE
}
}

stdmethod! {
fn ItemMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, ppszDisplayName);

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    let mut size = lstrlenW((*This).itemName) as usize + 1;
    if !(*This).itemDelimiter.is_null() {
        size += lstrlenW((*This).itemDelimiter) as usize;
    }
    size *= core::mem::size_of::<WCHAR>();

    *ppszDisplayName = CoTaskMemAlloc(size as SIZE_T) as LPOLESTR;
    if (*ppszDisplayName).is_null() {
        return E_OUTOFMEMORY;
    }

    *(*ppszDisplayName) = 0;
    if !(*This).itemDelimiter.is_null() {
        lstrcatW(*ppszDisplayName, (*This).itemDelimiter);
    }
    lstrcatW(*ppszDisplayName, (*This).itemName);

    TRACE!(b"-- %s\n\0".as_ptr(), debugstr_w(*ppszDisplayName));

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    displayname: LPOLESTR,
    eaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut container: *mut IOleItemContainer = core::ptr::null_mut();
    let mut parser: *mut IParseDisplayName = core::ptr::null_mut();

    TRACE!(
        b"%p, %p, %p, %s, %p, %p.\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_w(displayname),
        eaten,
        ppmkOut,
    );

    if pmkToLeft.is_null() {
        return MK_E_SYNTAX;
    }

    let mut hr = IMoniker_BindToObject(
        pmkToLeft,
        pbc,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IOleItemContainer),
        core::ptr::addr_of_mut!(container) as *mut LPVOID,
    );
    if SUCCEEDED(hr) {
        hr = set_container_lock(container, pbc);
        if SUCCEEDED(hr) {
            hr = IOleItemContainer_GetObject(
                container,
                (*This).itemName,
                get_bind_speed_from_bindctx(pbc),
                pbc,
                core::ptr::addr_of!(IID_IParseDisplayName),
                core::ptr::addr_of_mut!(parser) as *mut LPVOID,
            );
            if SUCCEEDED(hr) {
                hr = IParseDisplayName_ParseDisplayName(parser, pbc, displayname, eaten, ppmkOut);
                IParseDisplayName_Release(parser);
            }
        }
        IOleItemContainer_Release(container);
    }

    hr
}
}

stdmethod! {
fn ItemMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_ITEMMONIKER;

    S_OK
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), iface, debugstr_guid(riid), ppvObject);

    ItemMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    ItemMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    ItemMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    buffer: *mut byte,
    max_len: ULONG,
    data_len: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"%p, %p, %u, %p.\n\0".as_ptr(), iface, buffer, max_len, data_len);

    let name_len = lstrlenW((*This).itemName) as usize;
    let delim_len = if !(*This).itemDelimiter.is_null() && *(*This).itemDelimiter != 0 {
        lstrlenW((*This).itemDelimiter) as usize
    } else {
        0
    };

    *data_len = (core::mem::size_of::<CLSID>()
        + core::mem::size_of::<WCHAR>()
        + (delim_len + name_len) * core::mem::size_of::<WCHAR>()) as ULONG;
    if max_len < *data_len {
        return E_OUTOFMEMORY;
    }

    /* write CLSID */
    memcpy(
        buffer as *mut core::ffi::c_void,
        core::ptr::addr_of!(CLSID_ItemMoniker) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );
    let mut ptrW = buffer.add(core::mem::size_of::<CLSID>()) as *mut WCHAR;

    /* write delimiter, upper-cased (the whole string, not just its first char) */
    for i in 0..delim_len {
        ptrW.write_unaligned(towupper(*(*This).itemDelimiter.add(i) as u16) as WCHAR);
        ptrW = ptrW.add(1);
    }

    /* write name */
    for i in 0..name_len {
        ptrW.write_unaligned(towupper(*(*This).itemName.add(i) as u16) as WCHAR);
        ptrW = ptrW.add(1);
    }
    ptrW.write_unaligned(0);

    S_OK
}
}

/******************************************************************************
 *        CreateItemMoniker	[OLE32.@]
 ******************************************************************************/
#[no_mangle]
pub unsafe extern "system" fn CreateItemMoniker(
    delimiter: LPCOLESTR,
    name: LPCOLESTR,
    ret: *mut *mut IMoniker,
) -> HRESULT {
    let hr: HRESULT;

    TRACE!(b"%s, %s, %p.\n\0".as_ptr(), debugstr_w(delimiter), debugstr_w(name), ret);

    let moniker = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<ItemMonikerImpl>() as usize,
    ) as *mut ItemMonikerImpl;
    if moniker.is_null() {
        return E_OUTOFMEMORY;
    }

    (*moniker).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_ItemMonikerImpl) as *mut IMonikerVtbl;
    (*moniker).IROTData_iface.lpVtbl =
        core::ptr::addr_of!(VT_ROTDataImpl) as *mut IROTDataVtbl;
    (*moniker).ref_ = 1;

    'failed: {
        let mut str_len = (lstrlenW(name) as usize + 1) * core::mem::size_of::<WCHAR>();
        (*moniker).itemName = HeapAlloc(GetProcessHeap(), 0, str_len as usize) as *mut WCHAR;
        if (*moniker).itemName.is_null() {
            hr = E_OUTOFMEMORY;
            break 'failed;
        }
        core::ptr::copy_nonoverlapping(name as *const u8, (*moniker).itemName as *mut u8, str_len);

        if !delimiter.is_null() {
            str_len = (lstrlenW(delimiter) as usize + 1) * core::mem::size_of::<WCHAR>();
            (*moniker).itemDelimiter =
                HeapAlloc(GetProcessHeap(), 0, str_len as usize) as *mut WCHAR;
            if (*moniker).itemDelimiter.is_null() {
                hr = E_OUTOFMEMORY;
                break 'failed;
            }
            core::ptr::copy_nonoverlapping(
                delimiter as *const u8,
                (*moniker).itemDelimiter as *mut u8,
                str_len,
            );
        }

        *ret = core::ptr::addr_of_mut!((*moniker).IMoniker_iface);

        return S_OK;
    }

    IMoniker_Release(core::ptr::addr_of_mut!((*moniker).IMoniker_iface));
    hr
}

#[no_mangle]
pub unsafe extern "system" fn ItemMoniker_CreateInstance(
    _iface: *mut IClassFactory,
    outer: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    static wszEmpty: [WCHAR; 1] = [0];
    let mut moniker: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), outer, debugstr_guid(riid), ppv);

    *ppv = core::ptr::null_mut();

    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let hr = CreateItemMoniker(wszEmpty.as_ptr(), wszEmpty.as_ptr(), &mut moniker);
    if FAILED(hr) {
        return hr;
    }

    let hr = IMoniker_QueryInterface(moniker, riid, ppv);
    IMoniker_Release(moniker);

    hr
}
