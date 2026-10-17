//! CompositeMoniker implementation — translation of dll/win32/ole32/compositemoniker.c
//!
//! A generic composite moniker (IMoniker + IROTData + IMarshal) as a binary node over
//! its left and right halves (wine-10.0), plus the EnumMonikerImpl (IEnumMoniker) that
//! walks a snapshot of its leaves in either direction.
use crate::prelude::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Local bits the shared surface does not (yet) carry.
//
// OleLoadFromStream / OleSaveToStream are ole32's own exports, still implemented in C
// (storage32.c) — they belong in crate::ole32_exports' "still C" block. Declared
// privately here (WINAPI ⇒ `extern "system"`, never `extern "C"`) so they cannot
// collide with a future shared declaration.
// ---------------------------------------------------------------------------
extern "system" {
    fn OleLoadFromStream(pStm: *mut IStream, iidInterface: *const IID, ppvObj: *mut LPVOID)
        -> HRESULT;
    fn OleSaveToStream(pPStm: *mut IPersistStream, pStm: *mut IStream) -> HRESULT;
}

// ---------------------------------------------------------------------------
// Semantically ported to wine-10.0 (upstream 5f6b295cb90). The old translation kept
// the components in a growable table and cancelled anti-monikers by walking it, which
// underflowed its index once every component cancelled (tabLastIndex 0 -> -1). wine-10
// makes a composite a binary node -- `left`, `right`, and `comp_count` leaves below
// it -- and creates one through moniker_simplify_composition, which composes the
// rightmost leaf of the left tree with the leftmost leaf of the right tree (non-
// generically) until they stop combining. A composition that cancels to nothing
// yields NULL, not an empty composite.
// ---------------------------------------------------------------------------

/// CompositeMoniker data structure (`ref` -> `ref_`, a Rust keyword).
#[repr(C)]
pub struct CompositeMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub IMarshal_iface: IMarshal,
    pub ref_: LONG,
    pub left: *mut IMoniker,
    pub right: *mut IMoniker,
    /// Number of non-composite leaves under this node; 0 = uninitialized (activation).
    pub comp_count: u32,
}

/// EnumMoniker data structure: a snapshot of the components, each holding a reference.
#[repr(C)]
pub struct EnumMonikerImpl {
    pub IEnumMoniker_iface: IEnumMoniker,
    pub ref_: LONG,
    pub monikers: *mut *mut IMoniker,
    pub count: u32,
    pub pos: u32,
}

#[inline]
unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut CompositeMonikerImpl {
    CONTAINING_RECORD!(iface, CompositeMonikerImpl, IMoniker_iface)
}

/// OUR composite, identified by vtable (never by IsSystemMoniker).
unsafe fn unsafe_impl_from_IMoniker(iface: *mut IMoniker) -> *mut CompositeMonikerImpl {
    if iface.is_null()
        || (*iface).lpVtbl as *const IMonikerVtbl
            != core::ptr::addr_of!(VT_CompositeMonikerImpl)
    {
        return core::ptr::null_mut();
    }
    impl_from_IMoniker(iface)
}

#[inline]
unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut CompositeMonikerImpl {
    CONTAINING_RECORD!(iface, CompositeMonikerImpl, IROTData_iface)
}

#[inline]
unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut CompositeMonikerImpl {
    CONTAINING_RECORD!(iface, CompositeMonikerImpl, IMarshal_iface)
}

#[inline]
unsafe fn impl_from_IEnumMoniker(iface: *mut IEnumMoniker) -> *mut EnumMonikerImpl {
    CONTAINING_RECORD!(iface, EnumMonikerImpl, IEnumMoniker_iface)
}

unsafe fn heap_calloc(n: usize, size: usize) -> *mut c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (n.max(1) * size) as usize) as *mut c_void
}

unsafe fn heap_free(p: *mut c_void) {
    if !p.is_null() {
        HeapFree(GetProcessHeap(), 0, p);
    }
}

// ---------------------------------------------------------------------------
// IMoniker
// ---------------------------------------------------------------------------

stdmethod! {
fn CompositeMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, This,
           debugstr_guid(riid as *const c_void), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
    {
        *ppvObject = iface as *mut c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as *mut c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IMarshal_iface) as *mut c_void;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn CompositeMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn CompositeMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let moniker = impl_from_IMoniker(iface);
    let refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*moniker).ref_)) as ULONG;

    TRACE!(b"%p, refcount %u\n\0".as_ptr() as *const c_char, iface, refcount);

    if refcount == 0 {
        if !(*moniker).left.is_null() {
            IMoniker_Release((*moniker).left);
        }
        if !(*moniker).right.is_null() {
            IMoniker_Release((*moniker).right);
        }
        heap_free(moniker as *mut c_void);
    }

    refcount
}
}

stdmethod! {
fn CompositeMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_CompositeMoniker;

    S_OK
}
}

stdmethod! {
fn CompositeMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    S_FALSE
}
}

stdmethod! {
fn CompositeMonikerImpl_Load(iface: *mut IMoniker, stream: *mut IStream) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut last: *mut IMoniker = core::ptr::null_mut();
    let mut m: *mut IMoniker = core::ptr::null_mut();
    let mut c: *mut IMoniker = core::ptr::null_mut();
    let mut count: DWORD = 0;

    TRACE!(b"%p, %p\n\0".as_ptr() as *const c_char, iface, stream);

    if (*moniker).comp_count != 0 {
        return E_UNEXPECTED;
    }

    let mut hr = IStream_Read(
        stream,
        core::ptr::addr_of_mut!(count) as *mut c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    if hr != S_OK {
        WARN!(b"Failed to read component count, hr %#x.\n\0".as_ptr() as *const c_char, hr);
        return hr;
    }

    if count < 2 {
        WARN!(b"Unexpected component count %u.\n\0".as_ptr() as *const c_char, count);
        return E_UNEXPECTED;
    }

    hr = OleLoadFromStream(stream, core::ptr::addr_of!(IID_IMoniker),
                           core::ptr::addr_of_mut!(last) as *mut LPVOID);
    if FAILED(hr) {
        return hr;
    }

    for i in 1..count - 1 {
        hr = OleLoadFromStream(stream, core::ptr::addr_of!(IID_IMoniker),
                               core::ptr::addr_of_mut!(m) as *mut LPVOID);
        if FAILED(hr) {
            WARN!(b"Failed to initialize component %u, hr %#x.\n\0".as_ptr() as *const c_char,
                  i, hr);
            IMoniker_Release(last);
            return hr;
        }
        hr = CreateGenericComposite(last, m, &mut c);
        IMoniker_Release(last);
        IMoniker_Release(m);
        if FAILED(hr) {
            return hr;
        }
        last = c;
    }

    hr = OleLoadFromStream(stream, core::ptr::addr_of!(IID_IMoniker),
                           core::ptr::addr_of_mut!(m) as *mut LPVOID);
    if FAILED(hr) {
        IMoniker_Release(last);
        return hr;
    }

    (*moniker).left = last;
    (*moniker).right = m;
    (*moniker).comp_count = count;

    hr
}
}

unsafe fn composite_save_components(moniker: *mut IMoniker, stream: *mut IStream) -> HRESULT {
    let comp_moniker = unsafe_impl_from_IMoniker(moniker);

    if !comp_moniker.is_null() {
        let mut hr = composite_save_components((*comp_moniker).left, stream);
        if SUCCEEDED(hr) {
            hr = composite_save_components((*comp_moniker).right, stream);
        }
        hr
    } else {
        OleSaveToStream(moniker as *mut IPersistStream, stream)
    }
}

stdmethod! {
fn CompositeMonikerImpl_Save(iface: *mut IMoniker, stream: *mut IStream, clear_dirty: BOOL) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p, %d\n\0".as_ptr() as *const c_char, iface, stream, clear_dirty);

    if (*moniker).comp_count == 0 {
        return E_UNEXPECTED;
    }

    let hr = IStream_Write(
        stream,
        core::ptr::addr_of!((*moniker).comp_count) as *const c_void,
        core::mem::size_of::<u32>() as ULONG,
        core::ptr::null_mut(),
    );
    if FAILED(hr) {
        return hr;
    }

    composite_save_components(iface, stream)
}
}

stdmethod! {
fn CompositeMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    let mut enumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let mut ptmpSize: ULARGE_INTEGER = core::mem::zeroed();

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    /* The sizeMax of this object is calculated by calling GetSizeMax on each moniker
       within this object then summing all returned values. */
    (*pcbSize).QuadPart = core::mem::size_of::<DWORD>() as ULONGLONG;

    IMoniker_Enum(iface, TRUE as BOOL, &mut enumMk);

    while IEnumMoniker_Next(enumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        IMoniker_GetSizeMax(pmk, &mut ptmpSize);
        IMoniker_Release(pmk);
        (*pcbSize).QuadPart += ptmpSize.QuadPart + core::mem::size_of::<CLSID>() as ULONGLONG;
    }

    IEnumMoniker_Release(enumMk);

    S_OK
}
}

/// wine-10 compose_with: try a non-generic composition first, else a generic one.
unsafe fn compose_with(left: *mut IMoniker, right: *mut IMoniker, c: *mut *mut IMoniker) -> HRESULT {
    let hr = IMoniker_ComposeWith(left, right, TRUE as BOOL, c);
    if FAILED(hr) && hr != MK_E_NEEDGENERIC {
        return hr;
    }
    CreateGenericComposite(left, right, c)
}

stdmethod! {
fn CompositeMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    toleft: *mut IMoniker,
    riid: *const IID,
    result: *mut *mut c_void,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut left: *mut IMoniker = core::ptr::null_mut();
    let mut rightmost: *mut IMoniker = core::ptr::null_mut();
    let mut c: *mut IMoniker = core::ptr::null_mut();
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut object: *mut IUnknown = core::ptr::null_mut();

    TRACE!(b"%p, %p, %p, %s, %p.\n\0".as_ptr() as *const c_char, iface, pbc, toleft,
           debugstr_guid(riid as *const c_void), result);

    if result.is_null() {
        return E_POINTER;
    }

    *result = core::ptr::null_mut();

    if toleft.is_null() {
        let mut hr = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if SUCCEEDED(hr) {
            hr = IRunningObjectTable_GetObject(rot, iface, &mut object);
            IRunningObjectTable_Release(rot);
            if FAILED(hr) {
                return E_INVALIDARG;
            }

            hr = IUnknown_QueryInterface(object, riid, result);
            IUnknown_Release(object);
        }

        return hr;
    }

    /* Try to bind rightmost component with (toleft, composite->left) composite at its
       left side */
    let mut hr = composite_get_rightmost(moniker, &mut left, &mut rightmost);
    if FAILED(hr) {
        return hr;
    }

    hr = compose_with(toleft, left, &mut c);
    IMoniker_Release(left);

    if SUCCEEDED(hr) {
        hr = IMoniker_BindToObject(rightmost, pbc, c, riid, result);
        IMoniker_Release(c);
    }

    IMoniker_Release(rightmost);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    toleft: *mut IMoniker,
    riid: *const IID,
    result: *mut *mut c_void,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut left: *mut IMoniker = core::ptr::null_mut();
    let mut rightmost: *mut IMoniker = core::ptr::null_mut();
    let mut composed_left: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"%p, %p, %p, %s, %p.\n\0".as_ptr() as *const c_char, iface, pbc, toleft,
           debugstr_guid(riid as *const c_void), result);

    *result = core::ptr::null_mut();

    let mut hr = composite_get_rightmost(moniker, &mut left, &mut rightmost);
    if FAILED(hr) {
        return hr;
    }

    if !toleft.is_null() {
        hr = compose_with(toleft, left, &mut composed_left);
    } else {
        composed_left = left;
        IMoniker_AddRef(composed_left);
    }

    if SUCCEEDED(hr) {
        hr = IMoniker_BindToStorage(rightmost, pbc, composed_left, riid, result);
        IMoniker_Release(composed_left);
    }

    IMoniker_Release(rightmost);
    IMoniker_Release(left);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    howfar: DWORD,
    toleft: *mut *mut IMoniker,
    reduced: *mut *mut IMoniker,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut reduced_left: *mut IMoniker = core::ptr::null_mut();
    let mut reduced_right: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"%p, %p, %u, %p, %p.\n\0".as_ptr() as *const c_char, iface, pbc, howfar, toleft,
           reduced);

    if pbc.is_null() || reduced.is_null() {
        return E_INVALIDARG;
    }

    let mut hr = IMoniker_Reduce((*moniker).left, pbc, howfar, core::ptr::null_mut(),
                                 &mut reduced_left);
    if FAILED(hr) {
        return hr;
    }

    let mut m: *mut IMoniker = (*moniker).left;
    hr = IMoniker_Reduce((*moniker).right, pbc, howfar, &mut m, &mut reduced_right);
    if FAILED(hr) {
        IMoniker_Release(reduced_left);
        return hr;
    }

    let was_reduced = reduced_left != (*moniker).left || reduced_right != (*moniker).right;
    if was_reduced {
        hr = CreateGenericComposite(reduced_left, reduced_right, reduced);
    } else {
        *reduced = iface;
        IMoniker_AddRef(*reduced);
    }

    IMoniker_Release(reduced_left);
    IMoniker_Release(reduced_right);

    if was_reduced {
        hr
    } else {
        MK_S_REDUCED_TO_SELF
    }
}
}

stdmethod! {
fn CompositeMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    right: *mut IMoniker,
    only_if_not_generic: BOOL,
    composite: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%p, %p, %d, %p.\n\0".as_ptr() as *const c_char, iface, right,
           only_if_not_generic, composite);

    *composite = core::ptr::null_mut();

    if only_if_not_generic != 0 {
        MK_E_NEEDGENERIC
    } else {
        CreateGenericComposite(iface, right, composite)
    }
}
}

unsafe fn composite_get_components(
    moniker: *mut IMoniker,
    components: *mut *mut IMoniker,
    index: *mut u32,
) {
    let comp_moniker = unsafe_impl_from_IMoniker(moniker);

    if !comp_moniker.is_null() {
        composite_get_components((*comp_moniker).left, components, index);
        composite_get_components((*comp_moniker).right, components, index);
    } else {
        *components.add(*index as usize) = moniker;
        *index += 1;
    }
}

/// The leaves of `iface` left to right (borrowed, NOT AddRef'd) in a heap array the
/// caller frees with heap_free.
unsafe fn composite_get_components_alloc(
    iface: *mut IMoniker,
    count: *mut u32,
    components: *mut *mut *mut IMoniker,
) -> HRESULT {
    let moniker = unsafe_impl_from_IMoniker(iface);

    *count = if !moniker.is_null() { (*moniker).comp_count } else { 1 };

    *components =
        heap_calloc(*count as usize, core::mem::size_of::<*mut IMoniker>()) as *mut *mut IMoniker;
    if (*components).is_null() {
        return E_OUTOFMEMORY;
    }

    let mut index: u32 = 0;
    composite_get_components(iface, *components, &mut index);

    S_OK
}

stdmethod! {
fn CompositeMonikerImpl_Enum(
    iface: *mut IMoniker,
    forward: BOOL,
    ret_enum: *mut *mut IEnumMoniker,
) -> HRESULT {
    let mut monikers: *mut *mut IMoniker = core::ptr::null_mut();
    let mut count: u32 = 0;

    TRACE!(b"%p, %d, %p\n\0".as_ptr() as *const c_char, iface, forward, ret_enum);

    if ret_enum.is_null() {
        return E_INVALIDARG;
    }

    let mut hr = composite_get_components_alloc(iface, &mut count, &mut monikers);
    if FAILED(hr) {
        return hr;
    }

    hr = create_enumerator(monikers, count, forward, ret_enum);
    heap_free(monikers as *mut c_void);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_IsEqual(iface: *mut IMoniker, other: *mut IMoniker) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut components: *mut *mut IMoniker = core::ptr::null_mut();
    let mut other_components: *mut *mut IMoniker = core::ptr::null_mut();
    let mut count: u32 = 0;

    TRACE!(b"%p, %p.\n\0".as_ptr() as *const c_char, iface, other);

    if other.is_null() {
        return E_INVALIDARG;
    }

    let other_moniker = unsafe_impl_from_IMoniker(other);
    if other_moniker.is_null() {
        return S_FALSE;
    }

    if (*moniker).comp_count != (*other_moniker).comp_count {
        return S_FALSE;
    }

    let mut hr = composite_get_components_alloc(iface, &mut count, &mut components);
    if FAILED(hr) {
        return hr;
    }
    hr = composite_get_components_alloc(other, &mut count, &mut other_components);
    if FAILED(hr) {
        heap_free(components as *mut c_void);
        return hr;
    }

    /* C: `(hr = IMoniker_IsEqual(...) != S_OK)` -- the misplaced parenthesis stores
       the comparison, 0 or 1, which happens to be S_OK or S_FALSE. Same result here,
       written as meant. */
    for i in 0..(*moniker).comp_count as usize {
        hr = if IMoniker_IsEqual(*components.add(i), *other_components.add(i)) != S_OK {
            S_FALSE
        } else {
            S_OK
        };
        if hr != S_OK {
            break;
        }
    }

    heap_free(other_components as *mut c_void);
    heap_free(components as *mut c_void);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_Hash(iface: *mut IMoniker, hash: *mut DWORD) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut left_hash: DWORD = 0;
    let mut right_hash: DWORD = 0;

    TRACE!(b"%p, %p\n\0".as_ptr() as *const c_char, iface, hash);

    if hash.is_null() {
        return E_POINTER;
    }

    if (*moniker).comp_count == 0 {
        return E_UNEXPECTED;
    }

    *hash = 0;

    let mut hr = IMoniker_Hash((*moniker).left, &mut left_hash);
    if FAILED(hr) {
        return hr;
    }
    hr = IMoniker_Hash((*moniker).right, &mut right_hash);
    if FAILED(hr) {
        return hr;
    }

    *hash = left_hash ^ right_hash;

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    toleft: *mut IMoniker,
    newly_running: *mut IMoniker,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut c: *mut IMoniker = core::ptr::null_mut();
    let mut left: *mut IMoniker = core::ptr::null_mut();
    let mut rightmost: *mut IMoniker = core::ptr::null_mut();
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();

    TRACE!(b"%p, %p, %p, %p.\n\0".as_ptr() as *const c_char, iface, pbc, toleft,
           newly_running);

    if pbc.is_null() {
        return E_INVALIDARG;
    }

    if !toleft.is_null() {
        let mut hr = CreateGenericComposite(toleft, iface, &mut c);
        if SUCCEEDED(hr) {
            hr = IMoniker_IsRunning(c, pbc, core::ptr::null_mut(), newly_running);
            IMoniker_Release(c);
        }

        return hr;
    }

    if !newly_running.is_null() {
        return IMoniker_IsEqual(iface, newly_running);
    }

    let mut hr = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
    if FAILED(hr) {
        return hr;
    }

    hr = IRunningObjectTable_IsRunning(rot, iface);
    IRunningObjectTable_Release(rot);
    if hr == S_OK {
        return S_OK;
    }

    hr = composite_get_rightmost(moniker, &mut left, &mut rightmost);
    if FAILED(hr) {
        return hr;
    }

    hr = IMoniker_IsRunning(rightmost, pbc, left, core::ptr::null_mut());

    IMoniker_Release(left);
    IMoniker_Release(rightmost);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    toleft: *mut IMoniker,
    changetime: *mut FILETIME,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut left: *mut IMoniker = core::ptr::null_mut();
    let mut rightmost: *mut IMoniker = core::ptr::null_mut();
    let mut composed_left: *mut IMoniker = core::ptr::null_mut();
    let mut running: *mut IMoniker = core::ptr::null_mut();
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();

    TRACE!(b"%p, %p, %p, %p.\n\0".as_ptr() as *const c_char, iface, pbc, toleft, changetime);

    if changetime.is_null() || pbc.is_null() {
        return E_INVALIDARG;
    }

    let mut hr = composite_get_rightmost(moniker, &mut left, &mut rightmost);
    if FAILED(hr) {
        return hr;
    }

    if !toleft.is_null() {
        /* Compose (toleft, left) and check that against rightmost */
        hr = compose_with(toleft, left, &mut composed_left);
        if SUCCEEDED(hr) && !composed_left.is_null() {
            hr = compose_with(composed_left, rightmost, &mut running);
        }
    } else {
        composed_left = left;
        IMoniker_AddRef(composed_left);
        running = iface;
        IMoniker_AddRef(running);
    }

    if SUCCEEDED(hr) {
        hr = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if SUCCEEDED(hr) {
            if IRunningObjectTable_GetTimeOfLastChange(rot, running, changetime) != S_OK {
                hr = IMoniker_GetTimeOfLastChange(rightmost, pbc, composed_left, changetime);
            }
            IRunningObjectTable_Release(rot);
        }
    }

    if !composed_left.is_null() {
        IMoniker_Release(composed_left);
    }
    if !running.is_null() {
        IMoniker_Release(running);
    }
    IMoniker_Release(rightmost);
    IMoniker_Release(left);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_Inverse(iface: *mut IMoniker, inverse: *mut *mut IMoniker) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut right_inverted: *mut IMoniker = core::ptr::null_mut();
    let mut left_inverted: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"%p, %p.\n\0".as_ptr() as *const c_char, iface, inverse);

    if inverse.is_null() {
        return E_INVALIDARG;
    }

    *inverse = core::ptr::null_mut();

    let mut hr = IMoniker_Inverse((*moniker).right, &mut right_inverted);
    if FAILED(hr) {
        return hr;
    }
    hr = IMoniker_Inverse((*moniker).left, &mut left_inverted);
    if FAILED(hr) {
        IMoniker_Release(right_inverted);
        return hr;
    }

    hr = CreateGenericComposite(right_inverted, left_inverted, inverse);

    IMoniker_Release(left_inverted);
    IMoniker_Release(right_inverted);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    prefix: *mut *mut IMoniker,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut count: u32 = 0;
    let mut prefix_len: u32 = 0;
    let mut leftmost: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(b"%p, %p, %p\n\0".as_ptr() as *const c_char, iface, other, prefix);

    /* If the other moniker is a composite, this method compares the components of each
       composite from left to right. The returned common prefix moniker might also be a
       composite moniker, depending on how many of the leftmost components were common
       to both monikers. */

    if !prefix.is_null() {
        *prefix = core::ptr::null_mut();
    }

    if other.is_null() || prefix.is_null() {
        return E_INVALIDARG;
    }

    let other_moniker = unsafe_impl_from_IMoniker(other);
    if !other_moniker.is_null() {
        let mut components: *mut *mut IMoniker = core::ptr::null_mut();
        let mut other_components: *mut *mut IMoniker = core::ptr::null_mut();
        let mut c: *mut IMoniker = core::ptr::null_mut();

        hr = composite_get_components_alloc(iface, &mut count, &mut components);
        if FAILED(hr) {
            return hr;
        }
        hr = composite_get_components_alloc(other, &mut count, &mut other_components);
        if FAILED(hr) {
            heap_free(components as *mut c_void);
            return hr;
        }

        count = (*moniker).comp_count.min((*other_moniker).comp_count);
        let prefix_components =
            heap_calloc(count as usize, core::mem::size_of::<*mut IMoniker>()) as *mut *mut IMoniker;
        if prefix_components.is_null() {
            heap_free(components as *mut c_void);
            heap_free(other_components as *mut c_void);
            return E_OUTOFMEMORY;
        }

        /* Collect prefix components */
        for i in 0..count as usize {
            let mut p: *mut IMoniker = core::ptr::null_mut();

            hr = IMoniker_CommonPrefixWith(*components.add(i), *other_components.add(i), &mut p);
            if FAILED(hr) {
                break;
            }
            *prefix_components.add(prefix_len as usize) = p;
            prefix_len += 1;
            /* S_OK means that prefix was found and is neither of tested monikers */
            if hr == S_OK {
                break;
            }
        }

        heap_free(components as *mut c_void);
        heap_free(other_components as *mut c_void);

        if prefix_len == 0 {
            heap_free(prefix_components as *mut c_void);
            return MK_E_NOPREFIX;
        }

        let mut last = *prefix_components;
        for i in 1..prefix_len as usize {
            hr = CreateGenericComposite(last, *prefix_components.add(i), &mut c);
            IMoniker_Release(last);
            IMoniker_Release(*prefix_components.add(i));
            if FAILED(hr) {
                break;
            }
            last = c;
        }
        heap_free(prefix_components as *mut c_void);

        if SUCCEEDED(hr) {
            *prefix = last;
            hr = if IMoniker_IsEqual(iface, *prefix) == S_OK {
                MK_S_US
            } else if prefix_len < count {
                S_OK
            } else if prefix_len == (*moniker).comp_count {
                MK_S_ME
            } else {
                MK_S_HIM
            };
        }

        return hr;
    }

    /* For non-composite, compare to leftmost component */
    hr = composite_get_leftmost(moniker, &mut leftmost);
    if SUCCEEDED(hr) {
        hr = IMoniker_IsEqual(leftmost, other);
        if hr == S_OK {
            *prefix = leftmost;
            IMoniker_AddRef(*prefix);
        }

        hr = if hr == S_OK { MK_S_HIM } else { MK_E_NOPREFIX };
        IMoniker_Release(leftmost);
    }

    hr
}
}

unsafe fn composite_compose_components(
    comp: *mut *mut IMoniker,
    count: u32,
    ret: *mut *mut IMoniker,
) -> HRESULT {
    let mut c: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    let mut last = *comp;
    IMoniker_AddRef(last);

    for i in 1..count as usize {
        hr = CreateGenericComposite(last, *comp.add(i), &mut c);
        IMoniker_Release(last);
        if FAILED(hr) {
            break;
        }
        last = c;
    }

    *ret = if SUCCEEDED(hr) { last } else { core::ptr::null_mut() };

    hr
}

stdmethod! {
fn CompositeMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    relpath: *mut *mut IMoniker,
) -> HRESULT {
    let mut this_count: u32 = 0;
    let mut other_count: u32 = 0;
    let mut prefix_len: u32 = 0;
    let mut inv: *mut IMoniker = core::ptr::null_mut();
    let mut tail: *mut IMoniker = core::ptr::null_mut();
    let mut other_tail: *mut IMoniker = core::ptr::null_mut();
    let mut rel: *mut IMoniker = core::ptr::null_mut();
    let mut components: *mut *mut IMoniker = core::ptr::null_mut();
    let mut other_components: *mut *mut IMoniker = core::ptr::null_mut();
    let mut start: u32 = 0;
    let mut other_start: u32 = 0;

    TRACE!(b"%p, %p, %p.\n\0".as_ptr() as *const c_char, iface, other, relpath);

    if relpath.is_null() {
        return E_INVALIDARG;
    }

    *relpath = core::ptr::null_mut();

    let mut hr = composite_get_components_alloc(iface, &mut this_count, &mut components);
    if FAILED(hr) {
        return hr;
    }
    hr = composite_get_components_alloc(other, &mut other_count, &mut other_components);
    if FAILED(hr) {
        heap_free(components as *mut c_void);
        return hr;
    }

    /* Skip common prefix of equal components */
    let count = other_count.min(this_count);
    while IMoniker_IsEqual(*components.add(prefix_len as usize),
                           *other_components.add(prefix_len as usize)) == S_OK
    {
        prefix_len += 1;
        if prefix_len == count {
            break;
        }
    }

    if prefix_len != 0 {
        this_count -= prefix_len;
        other_count -= prefix_len;
        other_start += prefix_len;
        start += prefix_len;
    } else {
        /* Replace first component of the other tail with relative path */
        hr = IMoniker_RelativePathTo(*components, *other_components, &mut rel);
        if SUCCEEDED(hr) {
            *other_components = rel;
        }

        this_count -= 1;
        start += 1;
    }

    /* Invert left side tail */
    if this_count != 0 && SUCCEEDED(hr) {
        hr = composite_compose_components(components.add(start as usize), this_count, &mut tail);
        if SUCCEEDED(hr) {
            hr = IMoniker_Inverse(tail, &mut inv);
            IMoniker_Release(tail);
            tail = inv;
        }
    }

    if other_count != 0 && SUCCEEDED(hr) {
        hr = composite_compose_components(other_components.add(other_start as usize),
                                          other_count, &mut other_tail);
    }

    if !tail.is_null() || !other_tail.is_null() {
        hr = CreateGenericComposite(tail, other_tail, relpath);
    } else if SUCCEEDED(hr) {
        *relpath = other;
        IMoniker_AddRef(*relpath);
        hr = MK_S_HIM;
    }

    if !rel.is_null() {
        IMoniker_Release(rel);
    }
    if !tail.is_null() {
        IMoniker_Release(tail);
    }
    if !other_tail.is_null() {
        IMoniker_Release(other_tail);
    }

    heap_free(other_components as *mut c_void);
    heap_free(components as *mut c_void);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    displayname: *mut LPOLESTR,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut left_name: *mut WCHAR = core::ptr::null_mut();
    let mut right_name: *mut WCHAR = core::ptr::null_mut();

    TRACE!(b"%p, %p, %p, %p\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           displayname);

    if pbc.is_null() || displayname.is_null() || (*moniker).comp_count == 0 {
        return E_INVALIDARG;
    }

    let mut hr = IMoniker_GetDisplayName((*moniker).left, pbc, core::ptr::null_mut(),
                                         &mut left_name);
    if FAILED(hr) {
        return hr;
    }
    hr = IMoniker_GetDisplayName((*moniker).right, pbc, core::ptr::null_mut(), &mut right_name);
    if FAILED(hr) {
        CoTaskMemFree(left_name as *mut c_void);
        return hr;
    }

    *displayname = CoTaskMemAlloc(
        ((lstrlenW(left_name) + lstrlenW(right_name) + 1) as usize
            * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPOLESTR;
    if (*displayname).is_null() {
        CoTaskMemFree(left_name as *mut c_void);
        CoTaskMemFree(right_name as *mut c_void);
        return E_OUTOFMEMORY;
    }

    *(*displayname) = 0;
    lstrcatW(*displayname, left_name);
    lstrcatW(*displayname, right_name);

    CoTaskMemFree(left_name as *mut c_void);
    CoTaskMemFree(right_name as *mut c_void);

    S_OK
}
}

stdmethod! {
fn CompositeMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    name: LPOLESTR,
    eaten: *mut ULONG,
    result: *mut *mut IMoniker,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut left: *mut IMoniker = core::ptr::null_mut();
    let mut rightmost: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"%p, %p, %p, %s, %p, %p.\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           debugstr_w(name), eaten, result);

    if pbc.is_null() {
        return E_INVALIDARG;
    }

    let mut hr = composite_get_rightmost(moniker, &mut left, &mut rightmost);
    if FAILED(hr) {
        return hr;
    }

    /* Let rightmost component parse the name, using what's left of the composite as a
       left side. */
    hr = IMoniker_ParseDisplayName(rightmost, pbc, left, name, eaten, result);

    IMoniker_Release(left);
    IMoniker_Release(rightmost);

    hr
}
}

stdmethod! {
fn CompositeMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_GENERICCOMPOSITE;

    S_OK
}
}

// ---------------------------------------------------------------------------
// IROTData
// ---------------------------------------------------------------------------

stdmethod! {
fn CompositeMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, iface,
           debugstr_guid(riid as *const c_void), ppvObject);

    CompositeMonikerImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IMoniker_iface), riid,
                                        ppvObject)
}
}

stdmethod! {
fn CompositeMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn CompositeMonikerROTDataImpl_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    IMoniker_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

unsafe fn composite_get_moniker_comparison_data(
    moniker: *mut IMoniker,
    data: *mut BYTE,
    max_len: ULONG,
    ret_len: *mut ULONG,
) -> HRESULT {
    let mut rot_data: *mut IROTData = core::ptr::null_mut();

    let mut hr = IMoniker_QueryInterface(moniker, core::ptr::addr_of!(IID_IROTData),
                                         core::ptr::addr_of_mut!(rot_data) as *mut *mut c_void);
    if FAILED(hr) {
        WARN!(b"Failed to get IROTData for component moniker, hr %#x.\n\0".as_ptr()
              as *const c_char, hr);
        return hr;
    }

    hr = IROTData_GetComparisonData(rot_data, data, max_len, ret_len);
    IROTData_Release(rot_data);

    hr
}

stdmethod! {
fn CompositeMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    data: *mut BYTE,
    max_len: ULONG,
    ret_len: *mut ULONG,
) -> HRESULT {
    let moniker = impl_from_IROTData(iface);
    let mut len: ULONG;
    let mut data = data;
    let mut max_len = max_len;

    TRACE!(b"%p, %p, %u, %p\n\0".as_ptr() as *const c_char, iface, data, max_len, ret_len);

    if (*moniker).comp_count == 0 {
        return E_UNEXPECTED;
    }

    /* Get required size first */
    *ret_len = core::mem::size_of::<CLSID>() as ULONG;

    len = 0;
    let mut hr = composite_get_moniker_comparison_data((*moniker).left, core::ptr::null_mut(), 0,
                                                       &mut len);
    if SUCCEEDED(hr) || hr == E_OUTOFMEMORY {
        *ret_len += len;
    } else {
        WARN!(b"Failed to get comparison data length for left component, hr %#x.\n\0".as_ptr()
              as *const c_char, hr);
        return hr;
    }

    len = 0;
    hr = composite_get_moniker_comparison_data((*moniker).right, core::ptr::null_mut(), 0,
                                               &mut len);
    if SUCCEEDED(hr) || hr == E_OUTOFMEMORY {
        *ret_len += len;
    } else {
        WARN!(b"Failed to get comparison data length for right component, hr %#x.\n\0".as_ptr()
              as *const c_char, hr);
        return hr;
    }

    if max_len < *ret_len {
        return E_OUTOFMEMORY;
    }

    memcpy(data as *mut c_void, core::ptr::addr_of!(CLSID_CompositeMoniker) as *const c_void,
           core::mem::size_of::<CLSID>());
    data = data.add(core::mem::size_of::<CLSID>());
    max_len -= core::mem::size_of::<CLSID>() as ULONG;
    hr = composite_get_moniker_comparison_data((*moniker).left, data, max_len, &mut len);
    if FAILED(hr) {
        WARN!(b"Failed to get comparison data for left component, hr %#x.\n\0".as_ptr()
              as *const c_char, hr);
        return hr;
    }
    data = data.add(len as usize);
    max_len -= len;
    hr = composite_get_moniker_comparison_data((*moniker).right, data, max_len, &mut len);
    if FAILED(hr) {
        WARN!(b"Failed to get comparison data for right component, hr %#x.\n\0".as_ptr()
              as *const c_char, hr);
        return hr;
    }

    S_OK
}
}

// ---------------------------------------------------------------------------
// IMarshal: by value -- the two halves are marshalled as IMonikers in turn.
// ---------------------------------------------------------------------------

stdmethod! {
fn CompositeMonikerMarshalImpl_QueryInterface(
    iface: *mut IMarshal,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"(%s, %p)\n\0".as_ptr() as *const c_char, debugstr_guid(riid as *const c_void), ppv);

    CompositeMonikerImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IMoniker_iface), riid, ppv)
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_AddRef(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"\n\0".as_ptr() as *const c_char);

    CompositeMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_Release(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"\n\0".as_ptr() as *const c_char);

    CompositeMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pCid: *mut CLSID,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr() as *const c_char,
           debugstr_guid(riid as *const c_void), pv, dwDestContext, pvDestContext, mshlflags,
           pCid);

    IMoniker_GetClassID(core::ptr::addr_of_mut!((*This).IMoniker_iface), pCid)
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pSize: *mut DWORD,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);
    let mut size: ULONG = 0;

    TRACE!(b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr() as *const c_char,
           debugstr_guid(riid as *const c_void), pv, dwDestContext, pvDestContext, mshlflags,
           pSize);

    if (*moniker).comp_count == 0 {
        return E_UNEXPECTED;
    }

    *pSize = 0x10; /* to match native */

    let mut hr = CoGetMarshalSizeMax(&mut size, core::ptr::addr_of!(IID_IMoniker),
                                     (*moniker).left as *mut IUnknown, dwDestContext,
                                     pvDestContext, mshlflags);
    if FAILED(hr) {
        return hr;
    }
    *pSize += size;

    hr = CoGetMarshalSizeMax(&mut size, core::ptr::addr_of!(IID_IMoniker),
                             (*moniker).right as *mut IUnknown, dwDestContext, pvDestContext,
                             mshlflags);
    if FAILED(hr) {
        return hr;
    }
    *pSize += size;

    hr
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_MarshalInterface(
    iface: *mut IMarshal,
    stream: *mut IStream,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    flags: DWORD,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);

    TRACE!(b"%p, %p, %s, %p, %x, %p, %#x\n\0".as_ptr() as *const c_char, iface, stream,
           debugstr_guid(riid as *const c_void), pv, dwDestContext, pvDestContext, flags);

    if (*moniker).comp_count == 0 {
        return E_UNEXPECTED;
    }

    let mut hr = CoMarshalInterface((stream).cast(), (core::ptr::addr_of!(IID_IMoniker)).cast(),
                                    ((*moniker).left as *mut IUnknown).cast(), dwDestContext,
                                    pvDestContext, flags);
    if FAILED(hr) {
        WARN!(b"Failed to marshal left component, hr %#x.\n\0".as_ptr() as *const c_char, hr);
        return hr;
    }

    hr = CoMarshalInterface((stream).cast(), (core::ptr::addr_of!(IID_IMoniker)).cast(),
                            ((*moniker).right as *mut IUnknown).cast(), dwDestContext, pvDestContext,
                            flags);
    if FAILED(hr) {
        WARN!(b"Failed to marshal right component, hr %#x.\n\0".as_ptr() as *const c_char, hr);
    }

    hr
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_UnmarshalInterface(
    iface: *mut IMarshal,
    stream: *mut IStream,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let moniker = impl_from_IMarshal(iface);

    TRACE!(b"%p, %p, %s, %p\n\0".as_ptr() as *const c_char, iface, stream,
           debugstr_guid(riid as *const c_void), ppv);

    if !(*moniker).left.is_null() {
        IMoniker_Release((*moniker).left);
        (*moniker).left = core::ptr::null_mut();
    }

    if !(*moniker).right.is_null() {
        IMoniker_Release((*moniker).right);
        (*moniker).right = core::ptr::null_mut();
    }

    let mut hr = CoUnmarshalInterface((stream).cast(), (core::ptr::addr_of!(IID_IMoniker)).cast(),
                                      core::ptr::addr_of_mut!((*moniker).left) as *mut LPVOID);
    if FAILED(hr) {
        WARN!(b"Failed to unmarshal left moniker, hr %#x.\n\0".as_ptr() as *const c_char, hr);
        return hr;
    }

    hr = CoUnmarshalInterface((stream).cast(), (core::ptr::addr_of!(IID_IMoniker)).cast(),
                              core::ptr::addr_of_mut!((*moniker).right) as *mut LPVOID);
    if FAILED(hr) {
        WARN!(b"Failed to unmarshal right moniker, hr %#x.\n\0".as_ptr() as *const c_char, hr);
        return hr;
    }

    IMoniker_QueryInterface(core::ptr::addr_of_mut!((*moniker).IMoniker_iface), riid, ppv)
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_ReleaseMarshalData(iface: *mut IMarshal, pStm: *mut IStream) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, pStm);
    let _ = iface;
    /* can't release a state-based marshal as nothing on server side to release */
    S_OK
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_DisconnectObject(iface: *mut IMarshal, dwReserved: DWORD) -> HRESULT {
    TRACE!(b"(0x%x)\n\0".as_ptr() as *const c_char, dwReserved);
    let _ = iface;
    /* can't disconnect a state-based marshal as nothing on server side to disconnect from */
    S_OK
}
}

// ---------------------------------------------------------------------------
// IEnumMoniker
// ---------------------------------------------------------------------------

stdmethod! {
fn EnumMonikerImpl_QueryInterface(
    iface: *mut IEnumMoniker,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, iface,
           debugstr_guid(riid as *const c_void), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IEnumMoniker), riid)
    {
        *ppvObject = iface as *mut c_void;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IEnumMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn EnumMonikerImpl_AddRef(iface: *mut IEnumMoniker) -> ULONG {
    let This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn EnumMonikerImpl_Release(iface: *mut IEnumMoniker) -> ULONG {
    let e = impl_from_IEnumMoniker(iface);
    let refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*e).ref_)) as ULONG;

    TRACE!(b"%p, refcount %u.\n\0".as_ptr() as *const c_char, iface, refcount);

    if refcount == 0 {
        for i in 0..(*e).count as usize {
            IMoniker_Release(*(*e).monikers.add(i));
        }
        heap_free((*e).monikers as *mut c_void);
        heap_free(e as *mut c_void);
    }

    refcount
}
}

stdmethod! {
fn EnumMonikerImpl_Next(
    iface: *mut IEnumMoniker,
    count: ULONG,
    m: *mut *mut IMoniker,
    fetched: *mut ULONG,
) -> HRESULT {
    let e = impl_from_IEnumMoniker(iface);
    let mut i: ULONG = 0;

    TRACE!(b"%p, %u, %p, %p.\n\0".as_ptr() as *const c_char, iface, count, m, fetched);

    if m.is_null() {
        return E_INVALIDARG;
    }

    *m = core::ptr::null_mut();

    /* retrieve the requested number of moniker from the current position */
    while (*e).pos < (*e).count && i < count {
        *m.add(i as usize) = *(*e).monikers.add((*e).pos as usize);
        (*e).pos += 1;
        IMoniker_AddRef(*m.add(i as usize));
        i += 1;
    }

    if !fetched.is_null() {
        *fetched = i;
    }

    if i == count { S_OK } else { S_FALSE }
}
}

stdmethod! {
fn EnumMonikerImpl_Skip(iface: *mut IEnumMoniker, count: ULONG) -> HRESULT {
    let e = impl_from_IEnumMoniker(iface);

    TRACE!(b"%p, %u.\n\0".as_ptr() as *const c_char, iface, count);

    if count == 0 {
        return S_OK;
    }

    if (*e).pos as u64 + count as u64 >= (*e).count as u64 {
        return S_FALSE;
    }

    (*e).pos += count;

    S_OK
}
}

stdmethod! {
fn EnumMonikerImpl_Reset(iface: *mut IEnumMoniker) -> HRESULT {
    let e = impl_from_IEnumMoniker(iface);

    TRACE!(b"%p.\n\0".as_ptr() as *const c_char, iface);

    (*e).pos = 0;

    S_OK
}
}

stdmethod! {
fn EnumMonikerImpl_Clone(iface: *mut IEnumMoniker, ret: *mut *mut IEnumMoniker) -> HRESULT {
    TRACE!(b"%p, %p.\n\0".as_ptr() as *const c_char, iface, ret);

    if ret.is_null() {
        return E_INVALIDARG;
    }

    *ret = core::ptr::null_mut();

    E_NOTIMPL
}
}

pub static VT_EnumMonikerImpl: IEnumMonikerVtbl = IEnumMonikerVtbl {
    QueryInterface: Some(EnumMonikerImpl_QueryInterface),
    AddRef: Some(EnumMonikerImpl_AddRef),
    Release: Some(EnumMonikerImpl_Release),
    Next: Some(EnumMonikerImpl_Next),
    Skip: Some(EnumMonikerImpl_Skip),
    Reset: Some(EnumMonikerImpl_Reset),
    Clone: Some(EnumMonikerImpl_Clone),
};

/// An enumerator over `count` components, each AddRef'd, reversed when !forward.
unsafe fn create_enumerator(
    components: *mut *mut IMoniker,
    count: u32,
    forward: BOOL,
    ret: *mut *mut IEnumMoniker,
) -> HRESULT {
    let object =
        heap_calloc(1, core::mem::size_of::<EnumMonikerImpl>()) as *mut EnumMonikerImpl;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IEnumMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_EnumMonikerImpl) as *mut IEnumMonikerVtbl;
    (*object).ref_ = 1;
    (*object).count = count;

    (*object).monikers =
        heap_calloc(count as usize, core::mem::size_of::<*mut IMoniker>()) as *mut *mut IMoniker;
    if (*object).monikers.is_null() {
        heap_free(object as *mut c_void);
        return E_OUTOFMEMORY;
    }

    for i in 0..count as usize {
        let src = if forward != 0 { i } else { count as usize - i - 1 };
        *(*object).monikers.add(i) = *components.add(src);
        IMoniker_AddRef(*(*object).monikers.add(i));
    }

    *ret = core::ptr::addr_of_mut!((*object).IEnumMoniker_iface);

    S_OK
}

// ---------------------------------------------------------------------------
// vtables
// ---------------------------------------------------------------------------

/// Virtual function table for the CompositeMonikerImpl class which includes IPersist,
/// IPersistStream and IMoniker functions.
pub static VT_CompositeMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(CompositeMonikerImpl_QueryInterface),
    AddRef: Some(CompositeMonikerImpl_AddRef),
    Release: Some(CompositeMonikerImpl_Release),
    GetClassID: Some(CompositeMonikerImpl_GetClassID),
    IsDirty: Some(CompositeMonikerImpl_IsDirty),
    Load: Some(CompositeMonikerImpl_Load),
    Save: Some(CompositeMonikerImpl_Save),
    GetSizeMax: Some(CompositeMonikerImpl_GetSizeMax),
    BindToObject: Some(CompositeMonikerImpl_BindToObject),
    BindToStorage: Some(CompositeMonikerImpl_BindToStorage),
    Reduce: Some(CompositeMonikerImpl_Reduce),
    ComposeWith: Some(CompositeMonikerImpl_ComposeWith),
    Enum: Some(CompositeMonikerImpl_Enum),
    IsEqual: Some(CompositeMonikerImpl_IsEqual),
    Hash: Some(CompositeMonikerImpl_Hash),
    IsRunning: Some(CompositeMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(CompositeMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(CompositeMonikerImpl_Inverse),
    CommonPrefixWith: Some(CompositeMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(CompositeMonikerImpl_RelativePathTo),
    GetDisplayName: Some(CompositeMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(CompositeMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(CompositeMonikerImpl_IsSystemMoniker),
};

/// Virtual function table for the IROTData class.
pub static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(CompositeMonikerROTDataImpl_QueryInterface),
    AddRef: Some(CompositeMonikerROTDataImpl_AddRef),
    Release: Some(CompositeMonikerROTDataImpl_Release),
    GetComparisonData: Some(CompositeMonikerROTDataImpl_GetComparisonData),
};

pub static VT_MarshalImpl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: Some(CompositeMonikerMarshalImpl_QueryInterface),
    AddRef: Some(CompositeMonikerMarshalImpl_AddRef),
    Release: Some(CompositeMonikerMarshalImpl_Release),
    GetUnmarshalClass: Some(CompositeMonikerMarshalImpl_GetUnmarshalClass),
    GetMarshalSizeMax: Some(CompositeMonikerMarshalImpl_GetMarshalSizeMax),
    MarshalInterface: Some(CompositeMonikerMarshalImpl_MarshalInterface),
    UnmarshalInterface: Some(CompositeMonikerMarshalImpl_UnmarshalInterface),
    ReleaseMarshalData: Some(CompositeMonikerMarshalImpl_ReleaseMarshalData),
    DisconnectObject: Some(CompositeMonikerMarshalImpl_DisconnectObject),
};

// ---------------------------------------------------------------------------
// The component tree: composites flattened to a binary tree of leaves, so the
// simplifier can cancel the rightmost leaf of one side against the leftmost leaf of
// the other. A leaf holds a reference to its moniker; an inner node holds none.
// ---------------------------------------------------------------------------

#[repr(C)]
struct comp_node {
    moniker: *mut IMoniker,
    parent: *mut comp_node,
    left: *mut comp_node,
    right: *mut comp_node,
}

unsafe fn moniker_get_tree_representation(
    moniker: *mut IMoniker,
    parent: *mut comp_node,
    ret: *mut *mut comp_node,
) -> HRESULT {
    let node = heap_calloc(1, core::mem::size_of::<comp_node>()) as *mut comp_node;
    if node.is_null() {
        return E_OUTOFMEMORY;
    }
    (*node).parent = parent;

    let comp_moniker = unsafe_impl_from_IMoniker(moniker);
    if !comp_moniker.is_null() {
        moniker_get_tree_representation((*comp_moniker).left, node, &mut (*node).left);
        moniker_get_tree_representation((*comp_moniker).right, node, &mut (*node).right);
    } else {
        (*node).moniker = moniker;
        IMoniker_AddRef((*node).moniker);
    }

    *ret = node;

    S_OK
}

unsafe fn moniker_tree_get_rightmost(mut root: *mut comp_node) -> *mut comp_node {
    if (*root).left.is_null() && (*root).right.is_null() {
        return if !(*root).moniker.is_null() { root } else { core::ptr::null_mut() };
    }
    while !(*root).right.is_null() {
        root = (*root).right;
    }
    root
}

unsafe fn moniker_tree_get_leftmost(mut root: *mut comp_node) -> *mut comp_node {
    if (*root).left.is_null() && (*root).right.is_null() {
        return if !(*root).moniker.is_null() { root } else { core::ptr::null_mut() };
    }
    while !(*root).left.is_null() {
        root = (*root).left;
    }
    root
}

unsafe fn moniker_tree_node_release(node: *mut comp_node) {
    if !(*node).moniker.is_null() {
        IMoniker_Release((*node).moniker);
    }
    heap_free(node as *mut c_void);
}

/// Releases the whole tree. Divergence from wine-10, whose moniker_tree_release frees
/// only the root and its two children: every node below depth one leaked on each
/// composition of a three-or-more-leaf composite (each leaf still holding its
/// moniker reference).
unsafe fn moniker_tree_release(node: *mut comp_node) {
    if !(*node).left.is_null() {
        moniker_tree_release((*node).left);
    }
    if !(*node).right.is_null() {
        moniker_tree_release((*node).right);
    }
    moniker_tree_node_release(node);
}

unsafe fn moniker_tree_replace_node(node: *mut comp_node, replace_with: *mut comp_node) {
    if !(*node).parent.is_null() {
        let parent = (*node).parent;
        if (*parent).left == node {
            (*parent).left = replace_with;
        } else {
            (*parent).right = replace_with;
        }
        (*replace_with).parent = parent;
        /* Divergence from wine-10: `node` is now unreachable (an inner node, so it
           holds no reference) and wine leaks it. Its children are the replacement and
           the leaf the caller frees next, so free just this node. */
        heap_free(node as *mut c_void);
    } else if !(*replace_with).moniker.is_null() {
        /* Replacing root with non-composite */
        (*node).moniker = (*replace_with).moniker;
        IMoniker_AddRef((*node).moniker);
        (*node).left = core::ptr::null_mut();
        (*node).right = core::ptr::null_mut();
        moniker_tree_node_release(replace_with);
    } else {
        /* Attaching composite branches to the root */
        (*node).left = (*replace_with).left;
        (*node).right = (*replace_with).right;
        /* The adopted branches still name replace_with as their parent. */
        if !(*node).left.is_null() {
            (*(*node).left).parent = node;
        }
        if !(*node).right.is_null() {
            (*(*node).right).parent = node;
        }
        moniker_tree_node_release(replace_with);
    }
}

/// Removes leaf `node`; `left` says it is the left tree's rightmost leaf (so its
/// sibling is its parent's left child) rather than the right tree's leftmost leaf.
unsafe fn moniker_tree_discard(node: *mut comp_node, left: bool) {
    if !(*node).parent.is_null() {
        let parent = (*node).parent;
        moniker_tree_replace_node(parent, if left { (*parent).left } else { (*parent).right });
        moniker_tree_node_release(node);
    } else {
        IMoniker_Release((*node).moniker);
        (*node).moniker = core::ptr::null_mut();
    }
}

unsafe fn moniker_create_from_tree(
    root: *const comp_node,
    count: *mut u32,
    moniker: *mut *mut IMoniker,
) -> HRESULT {
    let mut left_moniker: *mut IMoniker = core::ptr::null_mut();
    let mut right_moniker: *mut IMoniker = core::ptr::null_mut();

    *moniker = core::ptr::null_mut();

    /* Non-composite node */
    if (*root).left.is_null() && (*root).right.is_null() {
        *count += 1;
        *moniker = (*root).moniker;
        if !(*moniker).is_null() {
            IMoniker_AddRef(*moniker);
        }
        return S_OK;
    }

    let mut hr = moniker_create_from_tree((*root).left, count, &mut left_moniker);
    if FAILED(hr) {
        return hr;
    }
    hr = moniker_create_from_tree((*root).right, count, &mut right_moniker);
    if FAILED(hr) {
        IMoniker_Release(left_moniker);
        return hr;
    }

    hr = CreateGenericComposite(left_moniker, right_moniker, moniker);
    IMoniker_Release(left_moniker);
    IMoniker_Release(right_moniker);
    hr
}

unsafe fn moniker_get_tree_comp_count(root: *const comp_node, count: *mut u32) {
    if (*root).left.is_null() && (*root).right.is_null() {
        *count += 1;
        return;
    }

    moniker_get_tree_comp_count((*root).left, count);
    moniker_get_tree_comp_count((*root).right, count);
}

/// Splits a composite into its rightmost leaf and everything to the left of it, both
/// returned with a reference.
unsafe fn composite_get_rightmost(
    composite: *mut CompositeMonikerImpl,
    left: *mut *mut IMoniker,
    rightmost: *mut *mut IMoniker,
) -> HRESULT {
    let mut root: *mut comp_node = core::ptr::null_mut();
    let mut count: u32 = 0;

    /* Shortcut for trivial case when right component is non-composite */
    if unsafe_impl_from_IMoniker((*composite).right).is_null() {
        *left = (*composite).left;
        IMoniker_AddRef(*left);
        *rightmost = (*composite).right;
        IMoniker_AddRef(*rightmost);
        return S_OK;
    }

    *left = core::ptr::null_mut();
    *rightmost = core::ptr::null_mut();

    let mut hr = moniker_get_tree_representation(
        core::ptr::addr_of_mut!((*composite).IMoniker_iface),
        core::ptr::null_mut(),
        &mut root,
    );
    if FAILED(hr) {
        return hr;
    }

    let node = moniker_tree_get_rightmost(root);
    if node.is_null() {
        WARN!(b"Couldn't get right most component.\n\0".as_ptr() as *const c_char);
        moniker_tree_release(root);
        return E_FAIL;
    }

    *rightmost = (*node).moniker;
    IMoniker_AddRef(*rightmost);
    moniker_tree_discard(node, true);

    hr = moniker_create_from_tree(root, &mut count, left);
    moniker_tree_release(root);
    if FAILED(hr) {
        IMoniker_Release(*rightmost);
        *rightmost = core::ptr::null_mut();
    }

    hr
}

unsafe fn composite_get_leftmost(
    composite: *mut CompositeMonikerImpl,
    leftmost: *mut *mut IMoniker,
) -> HRESULT {
    let mut root: *mut comp_node = core::ptr::null_mut();

    if unsafe_impl_from_IMoniker((*composite).left).is_null() {
        *leftmost = (*composite).left;
        IMoniker_AddRef(*leftmost);
        return S_OK;
    }

    let hr = moniker_get_tree_representation(
        core::ptr::addr_of_mut!((*composite).IMoniker_iface),
        core::ptr::null_mut(),
        &mut root,
    );
    if FAILED(hr) {
        return hr;
    }

    let node = moniker_tree_get_leftmost(root);
    if node.is_null() {
        WARN!(b"Couldn't get left most component.\n\0".as_ptr() as *const c_char);
        moniker_tree_release(root);
        return E_FAIL;
    }

    *leftmost = (*node).moniker;
    IMoniker_AddRef(*leftmost);

    moniker_tree_release(root);

    S_OK
}

/// Composes the facing leaves of `left` and `right` non-generically until they stop
/// combining; returns the simplified sides (either may come back NULL, cancelled)
/// and the total leaf count.
unsafe fn moniker_simplify_composition(
    left: *mut IMoniker,
    right: *mut IMoniker,
    count: *mut u32,
    new_left: *mut *mut IMoniker,
    new_right: *mut *mut IMoniker,
) -> HRESULT {
    let mut left_tree: *mut comp_node = core::ptr::null_mut();
    let mut right_tree: *mut comp_node = core::ptr::null_mut();
    let mut modified: u32 = 0;
    let mut hr: HRESULT = S_OK;

    *count = 0;
    *new_left = core::ptr::null_mut();
    *new_right = core::ptr::null_mut();

    if FAILED(moniker_get_tree_representation(left, core::ptr::null_mut(), &mut left_tree)) {
        return E_OUTOFMEMORY;
    }
    if FAILED(moniker_get_tree_representation(right, core::ptr::null_mut(), &mut right_tree)) {
        moniker_tree_release(left_tree);
        return E_OUTOFMEMORY;
    }

    /* Simplify by composing trees together, in a non-generic way. */
    loop {
        let l = moniker_tree_get_rightmost(left_tree);
        if l.is_null() {
            break;
        }
        let r = moniker_tree_get_leftmost(right_tree);
        if r.is_null() {
            break;
        }

        let mut c: *mut IMoniker = core::ptr::null_mut();
        if FAILED(IMoniker_ComposeWith((*l).moniker, (*r).moniker, TRUE as BOOL, &mut c)) {
            break;
        }
        modified += 1;

        if !c.is_null() {
            /* Replace with composed moniker on the left side */
            IMoniker_Release((*l).moniker);
            (*l).moniker = c;
        } else {
            moniker_tree_discard(l, true);
        }
        moniker_tree_discard(r, false);
    }

    if modified == 0 {
        *new_left = left;
        IMoniker_AddRef(*new_left);
        *new_right = right;
        IMoniker_AddRef(*new_right);

        moniker_get_tree_comp_count(left_tree, count);
        moniker_get_tree_comp_count(right_tree, count);
    } else {
        hr = moniker_create_from_tree(left_tree, count, new_left);
        if SUCCEEDED(hr) {
            hr = moniker_create_from_tree(right_tree, count, new_right);
        }
    }

    moniker_tree_release(left_tree);
    moniker_tree_release(right_tree);

    if FAILED(hr) {
        if !(*new_left).is_null() {
            IMoniker_Release(*new_left);
        }
        if !(*new_right).is_null() {
            IMoniker_Release(*new_right);
        }
        *new_left = core::ptr::null_mut();
        *new_right = core::ptr::null_mut();
    }

    hr
}

unsafe fn create_composite(
    left: *mut IMoniker,
    right: *mut IMoniker,
    moniker: *mut *mut IMoniker,
) -> HRESULT {
    let mut new_left: *mut IMoniker = core::ptr::null_mut();
    let mut new_right: *mut IMoniker = core::ptr::null_mut();

    *moniker = core::ptr::null_mut();

    let object =
        heap_calloc(1, core::mem::size_of::<CompositeMonikerImpl>()) as *mut CompositeMonikerImpl;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_CompositeMonikerImpl) as *mut IMonikerVtbl;
    (*object).IROTData_iface.lpVtbl = core::ptr::addr_of!(VT_ROTDataImpl) as *mut IROTDataVtbl;
    (*object).IMarshal_iface.lpVtbl = core::ptr::addr_of!(VT_MarshalImpl) as *mut IMarshalVtbl;
    (*object).ref_ = 1;

    /* Uninitialized moniker created by object activation */
    if left.is_null() && right.is_null() {
        *moniker = core::ptr::addr_of_mut!((*object).IMoniker_iface);
        return S_OK;
    }

    let hr = moniker_simplify_composition(left, right, &mut (*object).comp_count, &mut new_left,
                                          &mut new_right);
    if FAILED(hr) {
        IMoniker_Release(core::ptr::addr_of_mut!((*object).IMoniker_iface));
        return hr;
    }

    if new_left.is_null() || new_right.is_null() {
        *moniker = if !new_left.is_null() { new_left } else { new_right };
        IMoniker_Release(core::ptr::addr_of_mut!((*object).IMoniker_iface));
        return S_OK;
    }

    (*object).left = new_left;
    (*object).right = new_right;

    *moniker = core::ptr::addr_of_mut!((*object).IMoniker_iface);

    S_OK
}

/// CreateGenericComposite [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CreateGenericComposite(
    left: *mut IMoniker,
    right: *mut IMoniker,
    composite: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%p, %p, %p\n\0".as_ptr() as *const c_char, left, right, composite);

    if composite.is_null() {
        return E_POINTER;
    }

    if left.is_null() && !right.is_null() {
        *composite = right;
        IMoniker_AddRef(*composite);
        return S_OK;
    } else if !left.is_null() && right.is_null() {
        *composite = left;
        IMoniker_AddRef(*composite);
        return S_OK;
    } else if left.is_null() && right.is_null() {
        return S_OK;
    }

    create_composite(left, right, composite)
}

/// MonikerCommonPrefixWith [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn MonikerCommonPrefixWith(
    pmkThis: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkCommon: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!(b"(),stub!\n\0".as_ptr() as *const c_char);
    let _ = (pmkThis, pmkOther, ppmkCommon);

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn CompositeMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut pMoniker: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr() as *const c_char, pUnk,
           debugstr_guid(riid as *const c_void), ppv);

    let _ = iface;

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let mut hr = create_composite(core::ptr::null_mut(), core::ptr::null_mut(), &mut pMoniker);
    if SUCCEEDED(hr) {
        hr = IMoniker_QueryInterface(pMoniker, riid, ppv);
        IMoniker_Release(pMoniker);
    }

    hr
}
