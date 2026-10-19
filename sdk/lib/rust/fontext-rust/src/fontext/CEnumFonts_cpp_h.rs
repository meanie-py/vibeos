use crate::prelude::*;
use crate::fontext::CFontCache_cpp_h::g_FontCache;
use core::ffi::c_void;







#[no_mangle]
pub unsafe extern "C" fn _CEnumFonts_CreateInstance(zip: *mut CFontExt, flags: DWORD, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
    ShellObjectCreatorInit!(CEnumFonts, [zip, flags], riid, ppvOut)
}

impl CEnumFonts {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_dwFlags = 0;
        (*this_).m_Index = 0;
        {
        }
    }
    pub unsafe fn Initialize(&mut self, folder: *mut CFontExt, flags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_dwFlags = flags;
        (*this_).m_Index = 0;
        return S_OK;
    }
    pub unsafe fn Next(&mut self, celt: ULONG, rgelt: *mut LPITEMIDLIST, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if rgelt.is_null() || (pceltFetched.is_null() && celt != 1) {
            return E_POINTER;
        }

        let mut hr: HRESULT = S_OK;
        let mut Fetched: ULONG = 0;
        let mut celt = celt;

        while celt != 0 {
            if (*this_).m_Index >= ((*g_FontCache).Size()) as _ {
                hr = S_FALSE;
                break;
            }

            if (((*g_FontCache).IsMarkDeleted(((*this_).m_Index) as _)) == 0) {
                let mut Name = (*g_FontCache).Name(((*this_).m_Index) as _);
                let mut FileName = (*g_FontCache).File(((*this_).m_Index) as _);
                if Name.IsEmpty() || FileName.IsEmpty() {
                } else {
                    // Create a PIDL
                    let item: PITEMID_CHILD = _ILCreate(Name.GetString(), FileName.GetString());
                    if item.is_null() {
                        hr = if Fetched != 0 { S_FALSE } else { E_OUTOFMEMORY };
                        break;
                    }
                    *rgelt.add(Fetched as usize) = item;
                    Fetched += 1;
                    celt -= 1;
                }
            }
            (*this_).m_Index += 1;
        }

        if !pceltFetched.is_null() {
            *pceltFetched = Fetched;
        }
        return hr;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: ULONG = 0;
        while i < celt && (*this_).m_Index < ((*g_FontCache).Size()) as _ {
            while (*this_).m_Index < ((*g_FontCache).Size()) as _ && (((*g_FontCache).IsMarkDeleted(((*this_).m_Index) as _)) != 0) {
                (*this_).m_Index += 1;
            }
            (*this_).m_Index += 1;
            i += 1;
        }
        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_Index = 0;
        return S_OK;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumIDList,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}
