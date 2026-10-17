use crate::prelude::*;

// ---- file-scope preamble (translated) ----
pub const INITIAL_SINKS: u32 = 10;

#[repr(C)]
pub struct EnumSTATDATA {
    pub IEnumSTATDATA_iface: IEnumSTATDATA,
    pub ref_: LONG,
    pub index: ULONG,
    pub num_of_elems: DWORD,
    pub statdata: *mut STATDATA,
    pub holder: *mut IUnknown,
}
pub type PEnumSTATDATA = *mut EnumSTATDATA;

pub static EnumSTATDATA_VTable: IEnumSTATDATAVtbl = IEnumSTATDATAVtbl {
    QueryInterface: Some(EnumSTATDATA_QueryInterface),
    AddRef: Some(EnumSTATDATA_AddRef),
    Release: Some(EnumSTATDATA_Release),
    Next: Some(EnumSTATDATA_Next),
    Skip: Some(EnumSTATDATA_Skip),
    Reset: Some(EnumSTATDATA_Reset),
    Clone: Some(EnumSTATDATA_Clone),
};

#[repr(C)]
pub struct OleAdviseHolderImpl {
    pub IOleAdviseHolder_iface: IOleAdviseHolder,
    pub ref_: LONG,
    pub max_cons: DWORD,
    pub connections: *mut STATDATA,
}
pub type POleAdviseHolderImpl = *mut OleAdviseHolderImpl;

pub static oahvt: IOleAdviseHolderVtbl = IOleAdviseHolderVtbl {
    QueryInterface: Some(OleAdviseHolderImpl_QueryInterface),
    AddRef: Some(OleAdviseHolderImpl_AddRef),
    Release: Some(OleAdviseHolderImpl_Release),
    Advise: Some(OleAdviseHolderImpl_Advise),
    Unadvise: Some(OleAdviseHolderImpl_Unadvise),
    EnumAdvise: Some(OleAdviseHolderImpl_EnumAdvise),
    SendOnRename: Some(OleAdviseHolderImpl_SendOnRename),
    SendOnSave: Some(OleAdviseHolderImpl_SendOnSave),
    SendOnClose: Some(OleAdviseHolderImpl_SendOnClose),
};

#[repr(C)]
pub struct DataAdviseHolder {
    pub IDataAdviseHolder_iface: IDataAdviseHolder,
    pub ref_: LONG,
    pub maxCons: DWORD,
    pub connections: *mut STATDATA,
    pub remote_connections: *mut DWORD,
    pub delegate: *mut IDataObject,
}
pub type PDataAdviseHolder = *mut DataAdviseHolder;

/* this connection has also has been advised to the delegate data object */
pub const WINE_ADVF_REMOTE: u32 = 0x80000000;

pub static DataAdviseHolderImpl_VTable: IDataAdviseHolderVtbl = IDataAdviseHolderVtbl {
    QueryInterface: Some(DataAdviseHolder_QueryInterface),
    AddRef: Some(DataAdviseHolder_AddRef),
    Release: Some(DataAdviseHolder_Release),
    Advise: Some(DataAdviseHolder_Advise),
    Unadvise: Some(DataAdviseHolder_Unadvise),
    EnumAdvise: Some(DataAdviseHolder_EnumAdvise),
    SendOnDataChange: Some(DataAdviseHolder_SendOnDataChange),
};

unsafe fn release_statdata(data: *mut STATDATA) {
    CoTaskMemFree((*data).formatetc.ptd as LPVOID);
    (*data).formatetc.ptd = core::ptr::null_mut();

    if !(*data).pAdvSink.is_null() {
        IAdviseSink_Release((*data).pAdvSink);
        (*data).pAdvSink = core::ptr::null_mut();
    }
}

unsafe fn copy_statdata(dst: *mut STATDATA, src: *const STATDATA) -> HRESULT {
    let hr: HRESULT;

    hr = copy_formatetc(
        core::ptr::addr_of_mut!((*dst).formatetc),
        core::ptr::addr_of!((*src).formatetc),
    );
    if FAILED(hr) {
        return hr;
    }
    (*dst).advf = (*src).advf;
    (*dst).pAdvSink = (*src).pAdvSink;
    if !(*dst).pAdvSink.is_null() {
        IAdviseSink_AddRef((*dst).pAdvSink);
    }
    (*dst).dwConnection = (*src).dwConnection;
    S_OK
}

/**************************************************************************
 *  EnumSTATDATA Implementation
 */

unsafe fn impl_from_IEnumSTATDATA(iface: *mut IEnumSTATDATA) -> *mut EnumSTATDATA {
    CONTAINING_RECORD!(iface, EnumSTATDATA, IEnumSTATDATA_iface)
}

stdmethod! {
fn EnumSTATDATA_QueryInterface(
    iface: *mut IEnumSTATDATA,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    TRACE!(b"(%s, %p)\n\0".as_ptr() as *const i8, debugstr_guid(riid), ppv);
    if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualIID(riid, core::ptr::addr_of!(IID_IEnumSTATDATA))
    {
        IEnumSTATDATA_AddRef(iface);
        *ppv = iface as *mut c_void;
        return S_OK;
    }
    E_NOINTERFACE
}
}

stdmethod! {
fn EnumSTATDATA_AddRef(iface: *mut IEnumSTATDATA) -> ULONG {
    let This = impl_from_IEnumSTATDATA(iface);
    TRACE!(b"()\n\0".as_ptr() as *const i8);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn EnumSTATDATA_Release(iface: *mut IEnumSTATDATA) -> ULONG {
    let This = impl_from_IEnumSTATDATA(iface);
    let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));
    TRACE!(b"()\n\0".as_ptr() as *const i8);
    if refs == 0 {
        let mut i: DWORD = 0;
        while i < (*This).num_of_elems {
            release_statdata((*This).statdata.add(i as usize));
            i += 1;
        }
        HeapFree(GetProcessHeap(), 0, (*This).statdata as LPVOID);
        if !(*This).holder.is_null() {
            IUnknown_Release((*This).holder);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    refs as ULONG
}
}

stdmethod! {
fn EnumSTATDATA_Next(
    iface: *mut IEnumSTATDATA,
    num: ULONG,
    data: LPSTATDATA,
    fetched: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IEnumSTATDATA(iface);
    let mut count: DWORD = 0;
    let mut hr: HRESULT = S_OK;
    let mut num = num;

    TRACE!(b"(%d, %p, %p)\n\0".as_ptr() as *const i8, num, data, fetched);

    while num > 0 {
        num -= 1;
        if (*This).index >= (*This).num_of_elems {
            hr = S_FALSE;
            break;
        }

        copy_statdata(
            data.add(count as usize),
            (*This).statdata.add((*This).index as usize),
        );

        count += 1;
        (*This).index += 1;
    }

    if !fetched.is_null() {
        *fetched = count;
    }

    hr
}
}

stdmethod! {
fn EnumSTATDATA_Skip(iface: *mut IEnumSTATDATA, num: ULONG) -> HRESULT {
    let This = impl_from_IEnumSTATDATA(iface);

    TRACE!(b"(%d)\n\0".as_ptr() as *const i8, num);

    if (*This).index + num >= (*This).num_of_elems {
        (*This).index = (*This).num_of_elems;
        return S_FALSE;
    }

    (*This).index += num;
    S_OK
}
}

stdmethod! {
fn EnumSTATDATA_Reset(iface: *mut IEnumSTATDATA) -> HRESULT {
    let This = impl_from_IEnumSTATDATA(iface);

    TRACE!(b"()\n\0".as_ptr() as *const i8);

    (*This).index = 0;
    S_OK
}
}

stdmethod! {
fn EnumSTATDATA_Clone(iface: *mut IEnumSTATDATA, ppenum: *mut *mut IEnumSTATDATA) -> HRESULT {
    let This = impl_from_IEnumSTATDATA(iface);

    EnumSTATDATA_Construct(
        (*This).holder,
        (*This).index,
        (*This).num_of_elems,
        (*This).statdata,
        TRUE as BOOL,
        ppenum,
    )
}
}

#[no_mangle]
pub unsafe extern "C" fn EnumSTATDATA_Construct(
    holder: *mut IUnknown,
    index: ULONG,
    array_len: DWORD,
    data: *mut STATDATA,
    copy: BOOL,
    ppenum: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumSTATDATA>() as usize,
    ) as *mut EnumSTATDATA;
    let mut i: DWORD;
    let count: DWORD;

    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IEnumSTATDATA_iface.lpVtbl =
        core::ptr::addr_of!(EnumSTATDATA_VTable) as *mut IEnumSTATDATAVtbl;
    (*This).ref_ = 1;
    (*This).index = index;

    if copy != 0 {
        (*This).statdata = HeapAlloc(
            GetProcessHeap(),
            0,
            (array_len as usize * core::mem::size_of::<STATDATA>()) as usize,
        ) as *mut STATDATA;
        if (*This).statdata.is_null() {
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
            return E_OUTOFMEMORY;
        }

        let mut n: DWORD = 0;
        i = 0;
        while i < array_len {
            if !(*data.add(i as usize)).pAdvSink.is_null() {
                copy_statdata((*This).statdata.add(n as usize), data.add(i as usize));
                n += 1;
            }
            i += 1;
        }
        count = n;
    } else {
        (*This).statdata = data;
        count = array_len;
    }

    (*This).num_of_elems = count;
    (*This).holder = holder;
    if !holder.is_null() {
        IUnknown_AddRef(holder);
    }
    *ppenum = core::ptr::addr_of_mut!((*This).IEnumSTATDATA_iface);
    S_OK
}

/**************************************************************************
 *  OleAdviseHolder Implementation
 */

unsafe fn impl_from_IOleAdviseHolder(iface: *mut IOleAdviseHolder) -> *mut OleAdviseHolderImpl {
    CONTAINING_RECORD!(iface, OleAdviseHolderImpl, IOleAdviseHolder_iface)
}

unsafe fn OleAdviseHolderImpl_Destructor(This: *mut OleAdviseHolderImpl) {
    let mut index: DWORD;
    TRACE!(b"%p\n\0".as_ptr() as *const i8, This);

    index = 0;
    while index < (*This).max_cons {
        if !(*(*This).connections.add(index as usize)).pAdvSink.is_null() {
            release_statdata((*This).connections.add(index as usize));
        }
        index += 1;
    }

    HeapFree(GetProcessHeap(), 0, (*This).connections as LPVOID);
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

stdmethod! {
fn OleAdviseHolderImpl_QueryInterface(
    iface: *mut IOleAdviseHolder,
    iid: *const IID,
    obj: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IOleAdviseHolder(iface);
    TRACE!(
        b"(%p)->(%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(iid),
        obj,
    );

    if obj.is_null() {
        return E_POINTER;
    }

    *obj = core::ptr::null_mut();

    if IsEqualIID(iid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualIID(iid, core::ptr::addr_of!(IID_IOleAdviseHolder))
    {
        *obj = core::ptr::addr_of_mut!((*This).IOleAdviseHolder_iface) as *mut c_void;
    }

    if (*obj).is_null() {
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);

    S_OK
}
}

stdmethod! {
fn OleAdviseHolderImpl_AddRef(iface: *mut IOleAdviseHolder) -> ULONG {
    let This = impl_from_IOleAdviseHolder(iface);
    let ref_ = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    TRACE!(b"(%p)->(ref=%d)\n\0".as_ptr() as *const i8, This, ref_ - 1);

    ref_ as ULONG
}
}

stdmethod! {
fn OleAdviseHolderImpl_Release(iface: *mut IOleAdviseHolder) -> ULONG {
    let This = impl_from_IOleAdviseHolder(iface);
    let ref_: ULONG;

    TRACE!(b"(%p)->(ref=%d)\n\0".as_ptr() as *const i8, This, (*This).ref_);

    ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    if ref_ == 0 {
        OleAdviseHolderImpl_Destructor(This);
    }

    ref_
}
}

stdmethod! {
fn OleAdviseHolderImpl_Advise(
    iface: *mut IOleAdviseHolder,
    pAdvise: *mut IAdviseSink,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    let This = impl_from_IOleAdviseHolder(iface);
    let empty_fmtetc = FORMATETC {
        cfFormat: 0,
        ptd: core::ptr::null_mut(),
        dwAspect: 0,
        lindex: -1,
        tymed: 0,
    };

    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8,
        This,
        pAdvise,
        pdwConnection,
    );

    if pdwConnection.is_null() {
        return E_POINTER;
    }

    *pdwConnection = 0;

    let mut index: DWORD = 0;
    while index < (*This).max_cons {
        if (*(*This).connections.add(index as usize)).pAdvSink.is_null() {
            break;
        }
        index += 1;
    }

    if index == (*This).max_cons {
        (*This).max_cons += INITIAL_SINKS;
        (*This).connections = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).connections as LPVOID,
            ((*This).max_cons as usize * core::mem::size_of::<STATDATA>()) as SIZE_T,
        ) as *mut STATDATA;
    }

    let new_conn = STATDATA {
        formatetc: empty_fmtetc,
        advf: 0,
        pAdvSink: pAdvise,
        /* 0 is not a valid cookie, so increment the index */
        dwConnection: index + 1,
    };

    copy_statdata((*This).connections.add(index as usize), &new_conn);

    *pdwConnection = new_conn.dwConnection;

    S_OK
}
}

stdmethod! {
fn OleAdviseHolderImpl_Unadvise(
    iface: *mut IOleAdviseHolder,
    dwConnection: DWORD,
) -> HRESULT {
    let This = impl_from_IOleAdviseHolder(iface);
    let index: DWORD;

    TRACE!(
        b"(%p)->(%u)\n\0".as_ptr() as *const i8,
        This,
        dwConnection,
    );

    /* The connection number is 1 more than the index, see OleAdviseHolder_Advise */
    index = dwConnection.wrapping_sub(1);

    if index >= (*This).max_cons
        || (*(*This).connections.add(index as usize)).pAdvSink.is_null()
    {
        return OLE_E_NOCONNECTION;
    }

    release_statdata((*This).connections.add(index as usize));

    S_OK
}
}

stdmethod! {
fn OleAdviseHolderImpl_EnumAdvise(
    iface: *mut IOleAdviseHolder,
    enum_advise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    let This = impl_from_IOleAdviseHolder(iface);
    let mut unk: *mut IUnknown = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, enum_advise);

    IOleAdviseHolder_QueryInterface(
        iface,
        core::ptr::addr_of!(IID_IUnknown),
        core::ptr::addr_of_mut!(unk) as *mut *mut c_void,
    );
    hr = EnumSTATDATA_Construct(
        unk,
        0,
        (*This).max_cons,
        (*This).connections,
        TRUE as BOOL,
        enum_advise,
    );
    IUnknown_Release(unk);
    hr
}
}

stdmethod! {
fn OleAdviseHolderImpl_SendOnRename(iface: *mut IOleAdviseHolder, pmk: *mut IMoniker) -> HRESULT {
    let mut pEnum: *mut IEnumSTATDATA = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, iface, pmk);

    hr = IOleAdviseHolder_EnumAdvise(iface, core::ptr::addr_of_mut!(pEnum));
    if SUCCEEDED(hr) {
        let mut statdata: STATDATA = core::mem::zeroed();
        while IEnumSTATDATA_Next(
            pEnum,
            1,
            core::ptr::addr_of_mut!(statdata),
            core::ptr::null_mut(),
        ) == S_OK
        {
            IAdviseSink_OnRename(statdata.pAdvSink, pmk);

            IAdviseSink_Release(statdata.pAdvSink);
        }
        IEnumSTATDATA_Release(pEnum);
    }

    hr
}
}

stdmethod! {
fn OleAdviseHolderImpl_SendOnSave(iface: *mut IOleAdviseHolder) -> HRESULT {
    let mut pEnum: *mut IEnumSTATDATA = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(b"(%p)->()\n\0".as_ptr() as *const i8, iface);

    hr = IOleAdviseHolder_EnumAdvise(iface, core::ptr::addr_of_mut!(pEnum));
    if SUCCEEDED(hr) {
        let mut statdata: STATDATA = core::mem::zeroed();
        while IEnumSTATDATA_Next(
            pEnum,
            1,
            core::ptr::addr_of_mut!(statdata),
            core::ptr::null_mut(),
        ) == S_OK
        {
            IAdviseSink_OnSave(statdata.pAdvSink);

            IAdviseSink_Release(statdata.pAdvSink);
        }
        IEnumSTATDATA_Release(pEnum);
    }

    hr
}
}

stdmethod! {
fn OleAdviseHolderImpl_SendOnClose(iface: *mut IOleAdviseHolder) -> HRESULT {
    let mut pEnum: *mut IEnumSTATDATA = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(b"(%p)->()\n\0".as_ptr() as *const i8, iface);

    hr = IOleAdviseHolder_EnumAdvise(iface, core::ptr::addr_of_mut!(pEnum));
    if SUCCEEDED(hr) {
        let mut statdata: STATDATA = core::mem::zeroed();
        while IEnumSTATDATA_Next(
            pEnum,
            1,
            core::ptr::addr_of_mut!(statdata),
            core::ptr::null_mut(),
        ) == S_OK
        {
            IAdviseSink_OnClose(statdata.pAdvSink);

            IAdviseSink_Release(statdata.pAdvSink);
        }
        IEnumSTATDATA_Release(pEnum);
    }

    hr
}
}

/**************************************************************************
 *  OleAdviseHolderImpl_Constructor
 *
 * UPSTREAM BUG (preserved): the C never NULL-checks the HeapAlloc result before
 * writing through lpoah — kept bug-for-bug.
 */
unsafe fn OleAdviseHolderImpl_Constructor() -> *mut IOleAdviseHolder {
    let lpoah: *mut OleAdviseHolderImpl;

    lpoah = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<OleAdviseHolderImpl>() as usize,
    ) as *mut OleAdviseHolderImpl;

    (*lpoah).IOleAdviseHolder_iface.lpVtbl =
        core::ptr::addr_of!(oahvt) as *mut IOleAdviseHolderVtbl;
    (*lpoah).ref_ = 1;
    (*lpoah).max_cons = INITIAL_SINKS;
    (*lpoah).connections = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((*lpoah).max_cons as usize * core::mem::size_of::<STATDATA>()) as usize,
    ) as *mut STATDATA;

    TRACE!(
        b"returning %p\n\0".as_ptr() as *const i8,
        core::ptr::addr_of_mut!((*lpoah).IOleAdviseHolder_iface)
    );
    core::ptr::addr_of_mut!((*lpoah).IOleAdviseHolder_iface)
}

/**************************************************************************
 *  DataAdviseHolder Implementation
 */

unsafe fn impl_from_IDataAdviseHolder(iface: *mut IDataAdviseHolder) -> *mut DataAdviseHolder {
    CONTAINING_RECORD!(iface, DataAdviseHolder, IDataAdviseHolder_iface)
}

unsafe fn DataAdviseHolder_Destructor(ptrToDestroy: *mut DataAdviseHolder) {
    TRACE!(b"%p\n\0".as_ptr() as *const i8, ptrToDestroy);

    let mut index: DWORD = 0;
    while index < (*ptrToDestroy).maxCons {
        if !(*(*ptrToDestroy).connections.add(index as usize))
            .pAdvSink
            .is_null()
        {
            if !(*ptrToDestroy).delegate.is_null()
                && ((*(*ptrToDestroy).connections.add(index as usize)).advf & WINE_ADVF_REMOTE) != 0
            {
                IDataObject_DUnadvise(
                    (*ptrToDestroy).delegate,
                    *(*ptrToDestroy).remote_connections.add(index as usize),
                );
            }

            release_statdata((*ptrToDestroy).connections.add(index as usize));
        }
        index += 1;
    }

    HeapFree(GetProcessHeap(), 0, (*ptrToDestroy).remote_connections as LPVOID);
    HeapFree(GetProcessHeap(), 0, (*ptrToDestroy).connections as LPVOID);
    HeapFree(GetProcessHeap(), 0, ptrToDestroy as LPVOID);
}

stdmethod! {
fn DataAdviseHolder_QueryInterface(
    iface: *mut IDataAdviseHolder,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IDataAdviseHolder(iface);
    TRACE!(
        b"(%p)->(%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if This.is_null() || ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IDataAdviseHolder), riid)
    {
        *ppvObject = iface as *mut c_void;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);
    S_OK
}
}

stdmethod! {
fn DataAdviseHolder_AddRef(iface: *mut IDataAdviseHolder) -> ULONG {
    let This = impl_from_IDataAdviseHolder(iface);
    TRACE!(b"(%p) (ref=%d)\n\0".as_ptr() as *const i8, This, (*This).ref_);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn DataAdviseHolder_Release(iface: *mut IDataAdviseHolder) -> ULONG {
    let This = impl_from_IDataAdviseHolder(iface);
    let ref_count: ULONG;

    TRACE!(b"(%p) (ref=%d)\n\0".as_ptr() as *const i8, This, (*This).ref_);

    ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;
    if ref_count == 0 {
        DataAdviseHolder_Destructor(This);
    }

    ref_count
}
}

stdmethod! {
fn DataAdviseHolder_Advise(
    iface: *mut IDataAdviseHolder,
    pDataObject: *mut IDataObject,
    pFetc: *mut FORMATETC,
    advf: DWORD,
    pAdvise: *mut IAdviseSink,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    let mut index: DWORD;
    let mut new_conn: STATDATA = core::mem::zeroed();
    let This = impl_from_IDataAdviseHolder(iface);

    TRACE!(
        b"(%p)->(%p, %p, %08x, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        pDataObject,
        pFetc,
        advf,
        pAdvise,
        pdwConnection,
    );

    if pdwConnection.is_null() {
        return E_POINTER;
    }

    *pdwConnection = 0;

    index = 0;
    while index < (*This).maxCons {
        if (*(*This).connections.add(index as usize)).pAdvSink.is_null() {
            break;
        }
        index += 1;
    }

    if index == (*This).maxCons {
        (*This).maxCons += INITIAL_SINKS;
        (*This).connections = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).connections as LPVOID,
            ((*This).maxCons as usize * core::mem::size_of::<STATDATA>()) as SIZE_T,
        ) as *mut STATDATA;
        (*This).remote_connections = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).remote_connections as LPVOID,
            ((*This).maxCons as usize * core::mem::size_of::<DWORD>()) as SIZE_T,
        ) as *mut DWORD;
    }

    new_conn.pAdvSink = pAdvise;
    new_conn.advf = advf & !WINE_ADVF_REMOTE;
    new_conn.formatetc = *pFetc;
    /* 0 is not a valid cookie, so increment the index */
    new_conn.dwConnection = index + 1;

    copy_statdata((*This).connections.add(index as usize), &new_conn);

    if !(*(*This).connections.add(index as usize)).pAdvSink.is_null() {
        /* if we are already connected advise the remote object */
        if !(*This).delegate.is_null() {
            let hr: HRESULT;

            hr = IDataObject_DAdvise(
                (*This).delegate,
                core::ptr::addr_of_mut!(new_conn.formatetc),
                new_conn.advf,
                new_conn.pAdvSink,
                (*This).remote_connections.add(index as usize),
            );
            if FAILED(hr) {
                IDataAdviseHolder_Unadvise(iface, new_conn.dwConnection);
                return hr;
            }
            (*(*This).connections.add(index as usize)).advf |= WINE_ADVF_REMOTE;
        } else if (advf & ADVF_PRIMEFIRST) != 0 {
            /* only do this if we have no delegate, since in the above case the
             * delegate will do the priming for us */
            IDataAdviseHolder_SendOnDataChange(iface, pDataObject, 0, advf);
        }
    }

    *pdwConnection = new_conn.dwConnection;

    S_OK
}
}

stdmethod! {
fn DataAdviseHolder_Unadvise(
    iface: *mut IDataAdviseHolder,
    dwConnection: DWORD,
) -> HRESULT {
    let This = impl_from_IDataAdviseHolder(iface);
    let index: DWORD;

    TRACE!(
        b"(%p)->(%u)\n\0".as_ptr() as *const i8,
        This,
        dwConnection,
    );

    /* The connection number is 1 more than the index, see DataAdviseHolder_Advise */
    index = dwConnection.wrapping_sub(1);

    if index >= (*This).maxCons
        || (*(*This).connections.add(index as usize)).pAdvSink.is_null()
    {
        return OLE_E_NOCONNECTION;
    }

    if !(*This).delegate.is_null()
        && ((*(*This).connections.add(index as usize)).advf & WINE_ADVF_REMOTE) != 0
    {
        IDataObject_DUnadvise(
            (*This).delegate,
            *(*This).remote_connections.add(index as usize),
        );
        *(*This).remote_connections.add(index as usize) = 0;
    }

    release_statdata((*This).connections.add(index as usize));

    S_OK
}
}

stdmethod! {
fn DataAdviseHolder_EnumAdvise(
    iface: *mut IDataAdviseHolder,
    enum_advise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    let This = impl_from_IDataAdviseHolder(iface);
    let mut unk: *mut IUnknown = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const i8,
        This,
        enum_advise,
    );

    IDataAdviseHolder_QueryInterface(
        iface,
        core::ptr::addr_of!(IID_IUnknown),
        core::ptr::addr_of_mut!(unk) as *mut *mut c_void,
    );
    hr = EnumSTATDATA_Construct(
        unk,
        0,
        (*This).maxCons,
        (*This).connections,
        TRUE as BOOL,
        enum_advise,
    );
    IUnknown_Release(unk);
    hr
}
}

stdmethod! {
fn DataAdviseHolder_SendOnDataChange(
    iface: *mut IDataAdviseHolder,
    data_obj: *mut IDataObject,
    dwReserved: DWORD,
    advf: DWORD,
) -> HRESULT {
    let mut pEnum: *mut IEnumSTATDATA = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %08x, %08x)\n\0".as_ptr() as *const i8,
        iface,
        data_obj,
        dwReserved,
        advf,
    );

    hr = IDataAdviseHolder_EnumAdvise(iface, core::ptr::addr_of_mut!(pEnum));
    if SUCCEEDED(hr) {
        let mut statdata: STATDATA = core::mem::zeroed();
        while IEnumSTATDATA_Next(
            pEnum,
            1,
            core::ptr::addr_of_mut!(statdata),
            core::ptr::null_mut(),
        ) == S_OK
        {
            let mut stg: STGMEDIUM = core::mem::zeroed();
            stg.tymed = TYMED_NULL;
            stg.__bindgen_anon_1.pstg = core::ptr::null_mut();
            stg.pUnkForRelease = core::ptr::null_mut();

            if (statdata.advf & ADVF_NODATA) == 0 {
                hr = IDataObject_GetData(
                    data_obj,
                    core::ptr::addr_of_mut!(statdata.formatetc),
                    core::ptr::addr_of_mut!(stg),
                );
            }

            IAdviseSink_OnDataChange(
                statdata.pAdvSink,
                core::ptr::addr_of_mut!(statdata.formatetc),
                core::ptr::addr_of_mut!(stg),
            );

            if (statdata.advf & ADVF_ONLYONCE) != 0 {
                IDataAdviseHolder_Unadvise(iface, statdata.dwConnection);
            }

            release_statdata(core::ptr::addr_of_mut!(statdata));
        }
        IEnumSTATDATA_Release(pEnum);
    }

    /* UPSTREAM BUG (preserved): `hr` from GetData/EnumAdvise is computed but the C
     * unconditionally returns S_OK here. */
    let _ = hr;
    S_OK
}
}

#[no_mangle]
pub unsafe extern "C" fn DataAdviseHolder_OnConnect(
    iface: *mut IDataAdviseHolder,
    pDelegate: *mut IDataObject,
) -> HRESULT {
    let This = impl_from_IDataAdviseHolder(iface);
    let mut index: DWORD = 0;
    let mut hr: HRESULT = S_OK;

    while index < (*This).maxCons {
        if !(*(*This).connections.add(index as usize)).pAdvSink.is_null() {
            hr = IDataObject_DAdvise(
                pDelegate,
                core::ptr::addr_of_mut!((*(*This).connections.add(index as usize)).formatetc),
                (*(*This).connections.add(index as usize)).advf,
                (*(*This).connections.add(index as usize)).pAdvSink,
                (*This).remote_connections.add(index as usize),
            );
            if FAILED(hr) {
                break;
            }
            (*(*This).connections.add(index as usize)).advf |= WINE_ADVF_REMOTE;
        }
        index += 1;
    }
    (*This).delegate = pDelegate;
    hr
}

#[no_mangle]
pub unsafe extern "C" fn DataAdviseHolder_OnDisconnect(iface: *mut IDataAdviseHolder) {
    let This = impl_from_IDataAdviseHolder(iface);
    let mut index: DWORD = 0;

    while index < (*This).maxCons {
        if !(*(*This).connections.add(index as usize)).pAdvSink.is_null()
            && ((*(*This).connections.add(index as usize)).advf & WINE_ADVF_REMOTE) != 0
        {
            IDataObject_DUnadvise(
                (*This).delegate,
                *(*This).remote_connections.add(index as usize),
            );
            *(*This).remote_connections.add(index as usize) = 0;
            (*(*This).connections.add(index as usize)).advf &= !WINE_ADVF_REMOTE;
        }
        index += 1;
    }
    (*This).delegate = core::ptr::null_mut();
}

/******************************************************************************
 * DataAdviseHolder_Constructor
 *
 * UPSTREAM BUG (preserved): no NULL check on the HeapAlloc result.
 */
unsafe fn DataAdviseHolder_Constructor() -> *mut IDataAdviseHolder {
    let newHolder: *mut DataAdviseHolder = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<DataAdviseHolder>() as usize,
    ) as *mut DataAdviseHolder;

    (*newHolder).IDataAdviseHolder_iface.lpVtbl =
        core::ptr::addr_of!(DataAdviseHolderImpl_VTable) as *mut IDataAdviseHolderVtbl;
    (*newHolder).ref_ = 1;
    (*newHolder).maxCons = INITIAL_SINKS;
    (*newHolder).connections = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((*newHolder).maxCons as usize * core::mem::size_of::<STATDATA>()) as usize,
    ) as *mut STATDATA;
    (*newHolder).remote_connections = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((*newHolder).maxCons as usize * core::mem::size_of::<DWORD>()) as usize,
    ) as *mut DWORD;
    (*newHolder).delegate = core::ptr::null_mut();

    TRACE!(
        b"returning %p\n\0".as_ptr() as *const i8,
        core::ptr::addr_of_mut!((*newHolder).IDataAdviseHolder_iface)
    );
    core::ptr::addr_of_mut!((*newHolder).IDataAdviseHolder_iface)
}

/***********************************************************************
 * API functions
 */

/***********************************************************************
 * CreateOleAdviseHolder [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CreateOleAdviseHolder(
    ppOAHolder: *mut *mut IOleAdviseHolder,
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, ppOAHolder);

    if ppOAHolder.is_null() {
        return E_POINTER;
    }

    *ppOAHolder = OleAdviseHolderImpl_Constructor();

    if !(*ppOAHolder).is_null() {
        return S_OK;
    }

    E_OUTOFMEMORY
}

/******************************************************************************
 *              CreateDataAdviseHolder        [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDataAdviseHolder(
    ppDAHolder: *mut *mut IDataAdviseHolder,
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, ppDAHolder);

    if ppDAHolder.is_null() {
        return E_POINTER;
    }

    *ppDAHolder = DataAdviseHolder_Constructor();

    if !(*ppDAHolder).is_null() {
        return S_OK;
    }

    E_OUTOFMEMORY
}
