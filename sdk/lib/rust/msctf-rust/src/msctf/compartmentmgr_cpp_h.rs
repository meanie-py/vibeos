use crate::prelude::*;
use crate::msctf::msctf_cpp_h::advise_sink;
use crate::msctf::msctf_cpp_h::free_sinks;
use crate::msctf::msctf_cpp_h::get_Cookie_magic;
use crate::msctf::msctf_cpp_h::list;
use crate::msctf::msctf_cpp_h::unadvise_sink;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `typedef struct tagCompartmentValue {..} CompartmentValue;` -- cppapp lays the struct out.
pub type CompartmentValue = tagCompartmentValue;

pub type PCompartmentValue = *mut CompartmentValue;

































#[no_mangle]
pub unsafe extern "C" fn CompartmentMgr_Constructor(pUnkOuter: *mut IUnknown, riid: *const IID, ppOut: *mut *mut IUnknown) -> HRESULT {
    return CCompartmentMgr::CreateInstance(pUnkOuter, &*riid, ppOut);
}
#[no_mangle]
pub unsafe extern "C" fn CompartmentMgr_Destructor(iface: *mut ITfCompartmentMgr) -> HRESULT {
    (((*(*iface).lpVtbl).Release).unwrap_unchecked())(iface);
    return S_OK;
}

impl tagCompartmentValue {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CCompartmentMgr {
    pub unsafe fn __ctor_0(&mut self, pUnkOuter: *mut IUnknown) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_pUnkOuter = pUnkOuter;
        {
            list_init(core::ptr::addr_of_mut!((*this_).m_values));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut cursor: *mut list;
        let mut cursor2: *mut list;
    
        let list_head_ptr: *mut list = core::ptr::addr_of_mut!((*this_).m_values);
        cursor = (*list_head_ptr).next;
        cursor2 = (*cursor).next;
        while cursor != list_head_ptr {
            let value: *mut CompartmentValue =
                (cursor as *mut u8).sub(core::mem::offset_of!(CompartmentValue, entry)) as *mut CompartmentValue;
            list_remove((cursor).cast());
            (*(*value).compartment).Release();
            cicMemFree((value).cast());
            cursor = cursor2;
            cursor2 = (*cursor).next;
        }
    }
    pub unsafe fn CreateInstance(pUnkOuter: *mut IUnknown, riid: &IID, ppOut: *mut *mut IUnknown) -> HRESULT {
        if ppOut.is_null() {
            return E_POINTER;
        }
    
        if !pUnkOuter.is_null() && !IsEqualGUID!(&(*riid), &(IID_IUnknown)) {
            return CLASS_E_NOAGGREGATION;
        }
    
        let This: *mut CCompartmentMgr = CCompartmentMgr__new_in_0(cpp_alloc_CicNoThrow::<CCompartmentMgr>(), pUnkOuter);
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        if !pUnkOuter.is_null() {
            *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut IUnknown;
    
            return S_OK;
        } else {
            let mut hr: HRESULT;
            hr = (*This).QueryInterface(riid, ppOut as *mut LPVOID);
            if hr < 0 {
                CCompartmentMgr__delete(This);
            }
            return hr;
        }
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pUnkOuter.is_null() {
            return (*(*this_).m_pUnkOuter).QueryInterface(iid, ppvOut);
        }
    
        *ppvOut = core::ptr::null_mut();
    
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_ITfCompartmentMgr)) {
            *ppvOut = core::ptr::addr_of_mut!((*this_).__base0) as *mut ITfCompartmentMgr as LPVOID;
        }
    
        if !(*ppvOut).is_null() {
            (*this_).AddRef();
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if !(*this_).m_pUnkOuter.is_null() {
            return (*(*this_).m_pUnkOuter).AddRef();
        }
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if !(*this_).m_pUnkOuter.is_null() {
            return (*(*this_).m_pUnkOuter).Release();
        }
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CCompartmentMgr__delete(this_);
        }
        return ret;
    }
    pub unsafe fn GetCompartment(&mut self, rguid: &GUID, ppcomp: *mut *mut ITfCompartment) -> HRESULT {
        let this_: *mut Self = self;
        if ppcomp.is_null() {
            return E_POINTER;
        }
        let mut value: *mut CompartmentValue = core::ptr::null_mut();
        let mut cursor: *mut list;
        let mut hr: HRESULT;
    
        let list_head: *mut list = core::ptr::addr_of_mut!((*this_).m_values);
        cursor = (*list_head).next;
        while cursor != list_head {
            value = (cursor as *mut u8).sub(core::mem::offset_of!(CompartmentValue, entry)) as *mut CompartmentValue;
            if IsEqualGUID!(&(*rguid), &((*value).guid)) {
                (*(*value).compartment).AddRef();
                *ppcomp = (*value).compartment;
                return S_OK;
            }
            cursor = (*cursor).next;
        }
    
        value = cicMemAlloc(core::mem::size_of::<CompartmentValue>()) as *mut CompartmentValue;
        if value.is_null() {
            return E_OUTOFMEMORY;
        }
    
        (*value).guid = *rguid;
        (*value).owner = 0;
        hr = CCompartment::CreateInstance(value, core::ptr::addr_of_mut!((*value).compartment));
        if hr >= 0 {
            list_add_head(core::ptr::addr_of_mut!((*this_).m_values), (core::ptr::addr_of_mut!((*value).entry)).cast());
            (*(*value).compartment).AddRef();
            *ppcomp = (*value).compartment;
        } else {
            cicMemFree((value).cast());
            *ppcomp = core::ptr::null_mut();
        }
    
        return hr;
    }
    pub unsafe fn ClearCompartment(&mut self, tid: TfClientId, rguid: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut cursor: *mut list;
    
        let list_head: *mut list = core::ptr::addr_of_mut!((*this_).m_values);
        cursor = (*list_head).next;
        while cursor != list_head {
            let value: *mut CompartmentValue = LIST_ENTRY!(cursor, CompartmentValue, entry);
            if IsEqualGUID!(rguid, &(*value).guid) {
                if (*value).owner != 0 && tid != (*value).owner {
                    return E_UNEXPECTED;
                }
                list_remove((cursor).cast());
                (*(*value).compartment).Release();
                cicMemFree((value).cast());
                return S_OK;
            }
            cursor = (*cursor).next;
        }
    
        return CONNECT_E_NOCONNECTION;
    }
    pub unsafe fn EnumCompartments(&mut self, ppEnum: *mut *mut IEnumGUID) -> HRESULT {
        let this_: *mut Self = self;
        if ppEnum.is_null() {
            return E_INVALIDARG;
        }
    
        return CCompartmentEnumGuid::CreateInstance__0((core::ptr::addr_of_mut!((*this_).m_values)).cast(), ppEnum);
    }
}

impl CCompartmentEnumGuid {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_values = core::ptr::null_mut();
        (*this_).m_cursor = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        // TRACE("destroying %p\n", this_);
    }
    pub unsafe fn CreateInstance__0(values: *mut list, ppOut: *mut *mut IEnumGUID) -> HRESULT {
        let This: *mut CCompartmentEnumGuid = CCompartmentEnumGuid__new_in_0(cpp_alloc_CicNoThrow::<CCompartmentEnumGuid>());
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        (*This).m_values = (values).cast();
        (*This).m_cursor = list_head(values);
    
        *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut IEnumGUID;
    
        return S_OK;
    }
    pub unsafe fn CreateInstance__1(values: *mut list, ppOut: *mut *mut IEnumGUID, cursor: *mut list) -> HRESULT {
        let This: *mut CCompartmentEnumGuid = CCompartmentEnumGuid__new_in_0(cpp_alloc_CicNoThrow::<CCompartmentEnumGuid>());
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        (*This).m_values = (values).cast();
        (*This).m_cursor = (cursor).cast();
    
        *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut IEnumGUID;
    
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let guid_eq = |a: &GUID, b: &GUID| -> bool {
            a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
        };
    
        *ppvObj = core::ptr::null_mut();
    
        if guid_eq(iid, &IID_IUnknown) || guid_eq(iid, &IID_IEnumGUID) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as LPVOID;
        }
    
        if !(*ppvObj).is_null() {
            (*this_).AddRef();
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let mut ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CCompartmentEnumGuid__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Next(&mut self, celt: ULONG, rgelt: *mut GUID, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut fetched: ULONG = 0;
        let mut rgelt = rgelt;
    
        if rgelt.is_null() {
            return E_POINTER;
        }
    
        while fetched < celt && !(*this_).m_cursor.is_null() {
            let value: *mut CompartmentValue = LIST_ENTRY!((*this_).m_cursor, CompartmentValue, entry);
            if value.is_null() {
                break;
            }
    
            (*this_).m_cursor = list_next((*this_).m_values, (*this_).m_cursor);
            *rgelt = (*value).guid;
    
            fetched += 1;
            rgelt = rgelt.add(1);
        }
    
        if !pceltFetched.is_null() {
            *pceltFetched = fetched;
        }
        if fetched == celt { S_OK } else { S_FALSE }
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_cursor = list_next((*this_).m_values, (*this_).m_cursor);
        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_cursor = list_head((*this_).m_values);
        return S_OK;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumGUID) -> HRESULT {
        let this_: *mut Self = self;
        if ppenum.is_null() {
            return E_POINTER;
        }
    
        return CCompartmentEnumGuid::CreateInstance__1(((*this_).m_values).cast(), ppenum, ((*this_).m_cursor).cast());
    }
}

impl CCompartment {
    pub unsafe fn __ctor_0(&mut self, valueData: *mut CompartmentValue) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_valueData = valueData;
        {
            VariantInit(core::ptr::addr_of_mut!((*this_).m_variant));
            list_init(core::ptr::addr_of_mut!((*this_).m_CompartmentEventSink));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        VariantClear(core::ptr::addr_of_mut!((*this_).m_variant));
        free_sinks((core::ptr::addr_of_mut!((*this_).m_CompartmentEventSink)).cast());
    }
    pub unsafe fn CreateInstance(valueData: *mut CompartmentValue, ppOut: *mut *mut ITfCompartment) -> HRESULT {
        let This: *mut CCompartment = CCompartment__new_in_0(cpp_alloc_CicNoThrow::<CCompartment>(), valueData);
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut ITfCompartment;
    
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObj = core::ptr::null_mut();
    
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_ITfCompartment)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as LPVOID;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfSource)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base1) as LPVOID;
        }
    
        if !(*ppvObj).is_null() {
            (*this_).AddRef();
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let mut ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CCompartment__delete(this_);
        }
        return ret;
    }
    pub unsafe fn SetValue(&mut self, tid: TfClientId, pvarValue: *const VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        let mut sink: *mut ITfCompartmentEventSink;
        let mut cursor: *mut list;
    
        if pvarValue.is_null() {
            return E_INVALIDARG;
        }
    
        if !(V_VT!(pvarValue) == (VT_BSTR as u16) || V_VT!(pvarValue) == (VT_I4 as u16) || V_VT!(pvarValue) == VT_UNKNOWN) {
            return E_INVALIDARG;
        }
    
        if (*(*this_).m_valueData).owner == 0 {
            (*(*this_).m_valueData).owner = tid;
        }
    
        VariantClear(core::ptr::addr_of_mut!((*this_).m_variant));
    
        // Shallow copy of value and type
        (*this_).m_variant = *pvarValue;
    
        if V_VT!(pvarValue) == (VT_BSTR as u16) {
            V_BSTR!(core::ptr::addr_of_mut!((*this_).m_variant)) = SysAllocStringByteLen(
                V_BSTR!(pvarValue) as *mut i8,
                SysStringByteLen(V_BSTR!(pvarValue)),
            );
        } else if V_VT!(pvarValue) == VT_UNKNOWN {
            (*V_UNKNOWN!(core::ptr::addr_of_mut!((*this_).m_variant))).AddRef();
        }
    
        SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_CompartmentEventSink), ITfCompartmentEventSink, sink, {
            (*sink).OnChange(&((*(*this_).m_valueData).guid));
        });
    
        return S_OK;
    }
    pub unsafe fn GetValue(&mut self, pvarValue: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if pvarValue.is_null() {
            return E_INVALIDARG;
        }
    
        VariantInit(pvarValue);
        if V_VT!(core::ptr::addr_of_mut!((*this_).m_variant)) == VT_EMPTY {
            return S_FALSE;
        }
        return VariantCopy(pvarValue, core::ptr::addr_of_mut!((*this_).m_variant));
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if cicIsNullPtr(riid as *const IID) || punk.is_null() || pdwCookie.is_null() {
            return E_INVALIDARG;
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfCompartmentEventSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_CompartmentEventSink)).cast(), &IID_ITfCompartmentEventSink, COOKIE_MAGIC_COMPARTMENTSINK, punk, pdwCookie);
        }
    
        return E_NOTIMPL;
    }
    pub unsafe fn UnadviseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if get_Cookie_magic(dwCookie) != COOKIE_MAGIC_COMPARTMENTSINK {
            return E_INVALIDARG;
        }
    
        return unadvise_sink(dwCookie);
    }
}
