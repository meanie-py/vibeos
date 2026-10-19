use crate::prelude::*;
use crate::netshell::connectmanager_cpp_h::CNetConnectionManager_CreateInstance;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ENUMLIST {
    pub pNext: *mut ENUMLIST,
    pub pidl: PITEMID_CHILD,
}

pub type LPENUMLIST = *mut ENUMLIST;











#[no_mangle]
pub unsafe extern "C" fn ILGetConnData(pidl: PCITEMID_CHILD) -> PNETCONIDSTRUCT {
    if !pidl.is_null() && ((*pidl).mkid.cb as usize) >= (2 as usize) + core::mem::size_of::<NETCONIDSTRUCT>() {
        let pData: PNETCONIDSTRUCT = core::ptr::addr_of!((*pidl).mkid.abID) as PNETCONIDSTRUCT;
        if (*pData).Signature == (NETCONIDSTRUCT_SIG as u16) {
            return pData;
        }
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn ILGetConnName(pidl: PCITEMID_CHILD) -> PWCHAR {
    let pdata: PNETCONIDSTRUCT = ILGetConnData(pidl);
    if pdata.is_null() {
        return core::ptr::null_mut();
    }
    let off: usize = (*pdata).uNameOffset as usize;
    core::ptr::addr_of!((*pidl).mkid.abID)
        .cast::<u8>()
        .add(off) as PWCHAR
}
#[no_mangle]
pub unsafe extern "C" fn ILGetDeviceName(pidl: PCITEMID_CHILD) -> PWCHAR {
    let pdata: PNETCONIDSTRUCT = ILGetConnData(pidl);
    if pdata.is_null() {
        return core::ptr::null_mut();
    }
    return core::ptr::addr_of!((*core::ptr::addr_of!((*pidl).mkid.abID[0]).add((*pdata).uDeviceNameOffset as usize))) as PWCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn ILCreateNetConnectItem(pItem: *mut INetConnection) -> PITEMID_CHILD {
    let mut pidl: PITEMID_CHILD = core::ptr::null_mut();
    let mut size: ULONG_PTR;
    let mut pProperties: *mut NETCON_PROPERTIES = core::ptr::null_mut();
    let mut pnetid: PNETCONIDSTRUCT;
    let mut pwchName: PWCHAR;

    if (*pItem).GetProperties(core::ptr::addr_of_mut!(pProperties)) != S_OK {
        return core::ptr::null_mut();
    }

    size = core::mem::size_of::<WORD>() as ULONG_PTR; /* nr of bytes in this item */
    size += core::mem::size_of::<NETCONIDSTRUCT>() as ULONG_PTR;
    size += (wcslen((*pProperties).pszwName) as ULONG_PTR + 1 as ULONG_PTR) * core::mem::size_of::<WCHAR>() as ULONG_PTR;
    size += (wcslen((*pProperties).pszwDeviceName) as ULONG_PTR + 1 as ULONG_PTR) * core::mem::size_of::<WCHAR>() as ULONG_PTR;

    'end: {
        /* Allocate enough memory for the trailing id which will indicate that this is a simple id */
        pidl = SHAlloc(size + (core::mem::size_of::<SHITEMID>()) as ULONG_PTR) as LPITEMIDLIST;
        if pidl.is_null() {
            break 'end;
        }
        (*pidl).mkid.cb = size as WORD;
        (*((*pidl).mkid.abID.as_mut_ptr() as PNETCONIDSTRUCT)).Signature = (NETCONIDSTRUCT_SIG as u16);

        /* Copy the connection properties */
        pnetid = ILGetConnData(pidl);
        memset(
            core::ptr::addr_of_mut!((*pnetid).Unknown) as *mut core::ffi::c_void,
            0,
            core::mem::size_of_val(&*core::ptr::addr_of!((*pnetid).Unknown)),
        );
        (*pnetid).clsidThisObject = (*pProperties).clsidThisObject;
        (*pnetid).guidId = (*pProperties).guidId;
        (*pnetid).Status = (*pProperties).Status;
        (*pnetid).MediaType = (*pProperties).MediaType;
        (*pnetid).dwCharacter = (*pProperties).dwCharacter;
        (*pnetid).uNameOffset = core::mem::size_of::<NETCONIDSTRUCT>() as _;
        (*pnetid).uDeviceNameOffset = ((*pnetid).uNameOffset as usize
            + (wcslen((*pProperties).pszwName) as usize + 1 as usize)
                * core::mem::size_of::<WCHAR>()) as ULONG;

        pwchName = ILGetConnName(pidl);
        wcscpy(pwchName, (*pProperties).pszwName);

        pwchName = ILGetDeviceName(pidl);
        wcscpy(pwchName, (*pProperties).pszwDeviceName);

        /* Set the trailing id to null */
        memset(
            ((pidl as ULONG_PTR) + size) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<SHITEMID>(),
        );
    }
    // end:
    NcFreeNetconProperties(pProperties);

    pidl
}
#[no_mangle]
pub unsafe extern "C" fn ILGetConnection(pidl: PCITEMID_CHILD, pItem: *mut *mut INetConnection) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut pNetConMan: CComPtr<INetConnectionManager> = CComPtr::new();
    let mut pEnumCon: CComPtr<IEnumNetConnection> = CComPtr::new();
    let mut INetCon: CComPtr<INetConnection> = CComPtr::new();
    let mut Count: ULONG = 0;
    let mut pProperties: *mut NETCON_PROPERTIES = core::ptr::null_mut();

    let pdata: PNETCONIDSTRUCT = ILGetConnData(pidl);
    if pdata.is_null() {
        return E_FAIL;
    }

    /* get an instance to of IConnectionManager */
    hr = CNetConnectionManager_CreateInstance(&IID_INetConnectionManager, pNetConMan.GetAddressOf() as *mut LPVOID);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = (*pNetConMan.Get()).EnumConnections(NCME_DEFAULT, pEnumCon.GetAddressOf());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    while 1 != 0 {
        hr = (*pEnumCon.Get()).Next(1, INetCon.GetAddressOf(), core::ptr::addr_of_mut!(Count));
        if hr != S_OK {
            return E_FAIL;
        }

        hr = (*INetCon.Get()).GetProperties(core::ptr::addr_of_mut!(pProperties));
        if FAILED_UNEXPECTEDLY!(hr) {
            continue;
        }

        let bSame: i32 = if memcmp(
            core::ptr::addr_of!((*pProperties).guidId) as *const core::ffi::c_void,
            core::ptr::addr_of!((*pdata).guidId) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        ) == 0 { 1 } else { 0 };

        NcFreeNetconProperties(pProperties);

        if bSame != 0 {
            *pItem = INetCon.Detach();
            return S_OK;
        }
    }

    return E_FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn CEnumIDList_CreateInstance(hwndOwner: HWND, dwFlags: DWORD, riid: &IID, ppv: *mut LPVOID) -> HRESULT {
    return ShellObjectCreatorInit!(CEnumIDList, [], riid, ppv);
}

impl tagENUMLIST {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CEnumIDList {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_pFirst = core::ptr::null_mut();
        (*this_).m_pLast = core::ptr::null_mut();
        (*this_).m_pCurrent = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut pDelete: LPENUMLIST;

        while !(*this_).m_pFirst.is_null() {
            pDelete = (*this_).m_pFirst;
            (*this_).m_pFirst = (*pDelete).pNext;
            SHFree(((*pDelete).pidl).cast());
            SHFree((pDelete).cast());
        }
    }
    pub unsafe fn Initialize(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;
        let mut pNetConMan: CComPtr<INetConnectionManager> = CComPtr::new();
        let mut pEnumCon: CComPtr<IEnumNetConnection> = CComPtr::new();
        let mut Count: ULONG = 0;
        let mut pidl: PITEMID_CHILD = core::ptr::null_mut();

        /* get an instance to of IConnectionManager */
        hr = CNetConnectionManager_CreateInstance(
            core::ptr::addr_of!(IID_INetConnectionManager),
            pNetConMan.GetAddressOf() as *mut *mut core::ffi::c_void,
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return S_OK;
        }

        hr = (*pNetConMan.Get()).EnumConnections(NCME_DEFAULT, pEnumCon.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) {
            return S_OK;
        }

        while S_OK as i32 != 0 || true {
            let mut INetCon: CComPtr<INetConnection> = CComPtr::new();

            hr = (*pEnumCon.Get()).Next(
                1,
                INetCon.GetAddressOf(),
                core::ptr::addr_of_mut!(Count),
            );
            if hr != S_OK {
                break;
            }

            pidl = ILCreateNetConnectItem(INetCon.Get());
            if !pidl.is_null() {
                (*this_).AddToEnumList(pidl);
            }
        }

        return S_OK;
    }
    pub unsafe fn Next(&mut self, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: ULONG;
        let mut hr: HRESULT = S_OK;
        let mut temp: PITEMID_CHILD;

        if !pceltFetched.is_null() {
            *pceltFetched = 0;
        }

        if celt > 1 && pceltFetched.is_null() {
            return E_INVALIDARG;
        }

        if celt > 0 && (*this_).m_pCurrent.is_null() {
            return S_FALSE;
        }

        i = 0;
        while i < celt {
            if (*this_).m_pCurrent.is_null() {
                hr = S_FALSE;
                break;
            }

            temp = ILClone((*(*this_).m_pCurrent).pidl);
            if temp.is_null() {
                hr = if i != 0 { S_FALSE } else { E_OUTOFMEMORY };
                break;
            }
            *rgelt.add(i as usize) = temp;
            (*this_).m_pCurrent = (*(*this_).m_pCurrent).pNext;
            i += 1;
        }

        if !pceltFetched.is_null() {
            *pceltFetched = i;
        }

        return hr;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut dwIndex: u32;
        let mut hr: HRESULT = S_OK;

        dwIndex = 0;
        while dwIndex < celt {
            if (*this_).m_pCurrent.is_null() {
                hr = S_FALSE;
                break;
            }
            (*this_).m_pCurrent = (*(*this_).m_pCurrent).pNext;
            dwIndex += 1;
        }

        return hr;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pCurrent = (*this_).m_pFirst;
        return S_OK;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumIDList,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn AddToEnumList(&mut self, pidl: PITEMID_CHILD) -> BOOL {
        let this_: *mut Self = self;
        let mut pNew: LPENUMLIST;

        if pidl.is_null() {
            return (FALSE) as BOOL;
        }

        pNew = SHAlloc((core::mem::size_of::<ENUMLIST>()) as SIZE_T) as LPENUMLIST;
        if !pNew.is_null() {
            (*pNew).pNext = core::ptr::null_mut();
            (*pNew).pidl = pidl;

            if (*this_).m_pFirst.is_null() {
                (*this_).m_pFirst = pNew;
                (*this_).m_pCurrent = pNew;
            }

            if !(*this_).m_pLast.is_null() {
                /*add the new item to the end of the list */
                (*(*this_).m_pLast).pNext = pNew;
            }

            /*update the last item pointer */
            (*this_).m_pLast = pNew;
            return (TRUE) as BOOL;
        }
        return (FALSE) as BOOL;
    }
}
