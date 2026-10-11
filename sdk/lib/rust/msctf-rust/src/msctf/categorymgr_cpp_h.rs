use crate::prelude::*;
use crate::msctf::msctf_cpp_h::enumerate_Cookie;
use crate::msctf::msctf_cpp_h::generate_Cookie;
use crate::msctf::msctf_cpp_h::get_Cookie_data;
use crate::msctf::msctf_cpp_h::get_Cookie_magic;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_761FD513C445: [WCHAR; 24] = [0x0043, 0x0061, 0x0074, 0x0065, 0x0067, 0x006F, 0x0072, 0x0079, 0x005C, 0x0043, 0x0061, 0x0074, 0x0065, 0x0067, 0x006F, 0x0072, 0x0079, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_91C2397C2816: [WCHAR; 20] = [0x0043, 0x0061, 0x0074, 0x0065, 0x0067, 0x006F, 0x0072, 0x0079, 0x005C, 0x0049, 0x0074, 0x0065, 0x006D, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_BA2342D760A2: [WCHAR; 6] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_C31B0841CB86: [WCHAR; 23] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0043, 0x0061, 0x0074, 0x0065, 0x0067, 0x006F, 0x0072, 0x0079, 0x005C, 0x0049, 0x0074, 0x0065, 0x006D, 0x005C, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut __wine_dbch_msctf: __wine_debug_channel = __wine_debug_channel {
    flags: 0x07, // ERR|WARN|FIXME enabled by default (WINE_DEFAULT_DEBUG_CHANNEL expansion, assumed)
    name: [b'm' as i8, b's' as i8, b'c' as i8, b't' as i8, b'f' as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};




















#[no_mangle]
pub unsafe extern "C" fn CategoryMgr_Constructor(pUnkOuter: *mut IUnknown, ppOut: *mut *mut IUnknown) -> HRESULT {
    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let This: *mut CCategoryMgr = CCategoryMgr__new_in_0(cpp_alloc_CicNoThrow::<CCategoryMgr>());
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut IUnknown;

    return S_OK;
}

impl CCategoryMgr {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        // TRACE("destroying %p\n", this_);
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if ppvObj.is_null() {
            return E_INVALIDARG;
        }
    
        *ppvObj = core::ptr::null_mut();
    
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) || IsEqualGUID!(&(*riid), &(IID_ITfCategoryMgr)) {
            *ppvObj = this_ as *mut core::ffi::c_void;
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
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CCategoryMgr__delete(this_);
        }
        return ret;
    }
    pub unsafe fn RegisterCategory(&mut self, rclsid: &CLSID, rcatid: &GUID, rguid: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut szFullKey: [WCHAR; 110] = [0; 110];
        let mut szClsid: [WCHAR; 39] = [0; 39];
        let mut szCatid: [WCHAR; 39] = [0; 39];
        let mut szGuid: [WCHAR; 39] = [0; 39];
        let mut hTipKey: HKEY = core::ptr::null_mut();
        let mut hCatKey: HKEY = core::ptr::null_mut();
        let mut hItemKey: HKEY = core::ptr::null_mut();
        let mut error: LSTATUS;
        let mut hr: HRESULT = E_FAIL;
    
        StringFromGUID2(rclsid as *const CLSID, szClsid.as_mut_ptr(), (_countof!(szClsid)) as INT);
        StringFromGUID2(rcatid as *const GUID, szCatid.as_mut_ptr(), (_countof!(szCatid)) as INT);
        StringFromGUID2(rguid as *const GUID, szGuid.as_mut_ptr(), (_countof!(szGuid)) as INT);
    
        StringCchPrintfW!(szFullKey.as_mut_ptr(), _countof!(szFullKey), CPP_COM_WSTR_BA2342D760A2.as_ptr(), szwSystemTIPKey, szClsid.as_mut_ptr());
        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, szFullKey.as_mut_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(hTipKey));
        if (error) as u32 != ERROR_SUCCESS {
            return E_FAIL;
        }
    
        StringCchPrintfW!(szFullKey.as_mut_ptr(), _countof!(szFullKey), CPP_COM_WSTR_761FD513C445.as_ptr(), szCatid.as_mut_ptr(), szGuid.as_mut_ptr());
        error = RegCreateKeyExW(hTipKey, szFullKey.as_mut_ptr(), 0, core::ptr::null_mut(), 0, KEY_READ | KEY_WRITE, core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(hCatKey), core::ptr::null_mut());
        if (error) as u32 == ERROR_SUCCESS {
            RegCloseKey(hCatKey);
    
            StringCchPrintfW!(szFullKey.as_mut_ptr(), _countof!(szFullKey), CPP_COM_WSTR_91C2397C2816.as_ptr(), szGuid.as_mut_ptr(), szCatid.as_mut_ptr());
            error = RegCreateKeyExW(hTipKey, szFullKey.as_mut_ptr(), 0, core::ptr::null_mut(), 0, KEY_READ | KEY_WRITE, core::ptr::null_mut(),
                                    core::ptr::addr_of_mut!(hItemKey), core::ptr::null_mut());
            if (error) as u32 == ERROR_SUCCESS {
                RegCloseKey(hItemKey);
                hr = S_OK;
            }
        }
    
        RegCloseKey(hTipKey);
        return hr;
    }
    pub unsafe fn UnregisterCategory(&mut self, rclsid: &CLSID, rcatid: &GUID, rguid: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut szFullKey = [0u16; 110];
        let mut szClsid = [0u16; 39];
        let mut szCatid = [0u16; 39];
        let mut szGuid = [0u16; 39];
        let mut hTipKey: HKEY = core::ptr::null_mut();
        let mut error: LSTATUS;
    
        StringFromGUID2(rclsid, szClsid.as_mut_ptr(), (_countof!(szClsid)) as INT);
        StringFromGUID2(rcatid, szCatid.as_mut_ptr(), (_countof!(szCatid)) as INT);
        StringFromGUID2(rguid, szGuid.as_mut_ptr(), (_countof!(szGuid)) as INT);
    
        StringCchPrintfW!(szFullKey.as_mut_ptr(), _countof!(szFullKey), CPP_COM_WSTR_BA2342D760A2.as_ptr(), szwSystemTIPKey, szClsid.as_mut_ptr());
        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, szFullKey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(hTipKey));
        if (error) as u32 != ERROR_SUCCESS {
            return E_FAIL;
        }
    
        StringCchPrintfW!(szFullKey.as_mut_ptr(), _countof!(szFullKey), CPP_COM_WSTR_761FD513C445.as_ptr(), szCatid.as_mut_ptr(), szGuid.as_mut_ptr());
        RegDeleteTreeW(hTipKey, szFullKey.as_ptr());
    
        StringCchPrintfW!(szFullKey.as_mut_ptr(), _countof!(szFullKey), CPP_COM_WSTR_91C2397C2816.as_ptr(), szGuid.as_mut_ptr(), szCatid.as_mut_ptr());
        RegDeleteTreeW(hTipKey, szFullKey.as_ptr());
    
        RegCloseKey(hTipKey);
        return S_OK;
    }
    pub unsafe fn EnumCategoriesInItem(&mut self, rguid: &GUID, ppEnum: *mut *mut IEnumGUID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn EnumItemsInCategory(&mut self, rcatid: &GUID, ppEnum: *mut *mut IEnumGUID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn FindClosestCategory(&mut self, rguid: &GUID, pcatid: *mut GUID, ppcatidList: *mut *const GUID, ulCount: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut szFullKey: [u16; 120] = core::mem::zeroed();
        let mut szGuid: [u16; 39] = core::mem::zeroed();
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut hr: HRESULT = S_FALSE;
        let mut dwIndex: DWORD = 0;
        let mut error: LSTATUS = 0;
    
        if pcatid.is_null() || (ulCount != 0 && ppcatidList.is_null()) {
            return E_INVALIDARG;
        }
    
        StringFromGUID2(rguid as *const GUID, szGuid.as_mut_ptr(), (_countof!(szGuid)) as INT);
        StringCchPrintfW!(szFullKey.as_mut_ptr(), _countof!(szFullKey), CPP_COM_WSTR_C31B0841CB86.as_ptr(), szwSystemTIPKey, szGuid.as_ptr(), szGuid.as_ptr());
        *pcatid = GUID_NULL;
    
        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, szFullKey.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey));
        if (error) as u32 != ERROR_SUCCESS {
            return S_FALSE;
        }
    
        // Enumerate subkeys to find the closest category
        while 1 != 0 {
            let mut szCatidName: [u16; 39] = core::mem::zeroed();
            let mut cchCatidName: DWORD = _countof!(szCatidName);
            let mut currentCatid: GUID = core::mem::zeroed();
    
            error = RegEnumKeyExW(hKey, dwIndex, szCatidName.as_mut_ptr(), core::ptr::addr_of_mut!(cchCatidName), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
            if (error) as u32 != ERROR_SUCCESS || (error) as u32 == ERROR_NO_MORE_ITEMS {
                break;
            }
    
            dwIndex += 1;
    
            let hr2: HRESULT = CLSIDFromString(szCatidName.as_ptr(), core::ptr::addr_of_mut!(currentCatid));
            if hr2 < 0 {
                continue; // Skip invalid GUID strings
            }
    
            if ulCount <= 0 {
                *pcatid = currentCatid;
                hr = S_OK; // Found a category
                break;
            }
    
            // If a list of categories is provided, check if the current one is in the list
            let mut bFound: BOOL = (FALSE) as i32;
            let mut j: ULONG = 0;
            while j < ulCount {
                if IsEqualGUID!(&(currentCatid), &(*(*ppcatidList.add(j as usize)))) {
                    bFound = (TRUE) as i32;
                    *pcatid = currentCatid;
                    hr = S_OK; // Found a matching category
                    break;
                }
                j += 1;
            }
            if bFound != 0 {
                break; // Found and matched, so stop searching
            }
        }
    
        RegCloseKey(hKey);
        return hr;
    }
    pub unsafe fn RegisterGUIDDescription(&mut self, rclsid: &CLSID, rguid: &GUID, pchDesc: *const WCHAR, cch: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn UnregisterGUIDDescription(&mut self, rclsid: &CLSID, rguid: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetGUIDDescription(&mut self, rguid: &GUID, pbstrDesc: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn RegisterGUIDDWORD(&mut self, rclsid: &CLSID, rguid: &GUID, dw: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn UnregisterGUIDDWORD(&mut self, rclsid: &CLSID, rguid: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetGUIDDWORD(&mut self, rguid: &GUID, pdw: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, rguid, pdw);
        return E_NOTIMPL;
    }
    pub unsafe fn RegisterGUID(&mut self, rguid: &GUID, pguidatom: *mut TfGuidAtom) -> HRESULT {
        let this_: *mut Self = self;
        if pguidatom.is_null() {
            return E_INVALIDARG;
        }
    
        let mut dwCookieId: DWORD = 0;
        let mut dwEnumIndex: DWORD = 0;
        loop {
            dwCookieId = enumerate_Cookie(COOKIE_MAGIC_GUIDATOM, core::ptr::addr_of_mut!(dwEnumIndex));
            if dwCookieId != 0 && IsEqualGUID!(&(*rguid), &(*(get_Cookie_data(dwCookieId) as *const GUID))) {
                *pguidatom = dwCookieId;
                return S_OK;
            }
            if !(dwCookieId != 0) {
                break;
            }
        }
    
        let pNewGuid: *mut GUID = cicMemAlloc(core::mem::size_of::<GUID>()) as *mut GUID;
        if pNewGuid.is_null() {
            return E_OUTOFMEMORY;
        }
    
        *pNewGuid = *rguid;
    
        dwCookieId = generate_Cookie(COOKIE_MAGIC_GUIDATOM, (pNewGuid).cast());
        if dwCookieId == 0 {
            cicMemFree((pNewGuid).cast());
            return E_FAIL;
        }
    
        *pguidatom = dwCookieId;
        return S_OK;
    }
    pub unsafe fn GetGUID(&mut self, guidatom: TfGuidAtom, pguid: *mut GUID) -> HRESULT {
        let this_: *mut Self = self;
        if pguid.is_null() {
            return E_INVALIDARG;
        }
    
        *pguid = GUID_NULL;
    
        if get_Cookie_magic(guidatom) == COOKIE_MAGIC_GUIDATOM {
            *pguid = *(get_Cookie_data(guidatom) as *const GUID);
        }
    
        return S_OK;
    }
    pub unsafe fn IsEqualTfGuidAtom(&mut self, guidatom: TfGuidAtom, rguid: &GUID, pfEqual: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfEqual.is_null() {
            return E_INVALIDARG;
        }
    
        *pfEqual = (FALSE) as i32;
        if get_Cookie_magic(guidatom) == COOKIE_MAGIC_GUIDATOM {
            if IsEqualGUID!(&(*rguid), &(*(get_Cookie_data(guidatom) as *const GUID))) {
                *pfEqual = (TRUE) as i32;
            }
        }
    
        S_OK
    }
}
