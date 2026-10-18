use crate::prelude::*;
use crate::acppage::CLayerUIPropPage_cpp_h::IsBuiltinLayer;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_4186035AD3F8: [WCHAR; 13] = [0x005C, 0x0073, 0x0079, 0x0073, 0x006D, 0x0061, 0x0069, 0x006E, 0x002E, 0x0073, 0x0064, 0x0062, 0x0000];

use core::ffi::c_void;








impl CLayerStringList {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_root = TAGID_NULL;
        (*this_).m_layer = TAGID_NULL;
        {
            let mut buf = [0u16; (MAX_PATH) as _];
            SdbGetAppPatchDir(core::ptr::null_mut(), buf.as_mut_ptr(), MAX_PATH);
            StringCchCatW(buf.as_mut_ptr(), (_countof!(buf)) as _, CPP_COM_WSTR_4186035AD3F8.as_ptr());
            (*this_).m_db = SdbOpenDatabase(buf.as_mut_ptr(), DOS_PATH);
            (*this_).Reset();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        SdbCloseDatabase((*this_).m_db);
    }
    pub unsafe fn Next(&mut self, celt: ULONG, rgelt: *mut LPOLESTR, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut celt = celt;
        let mut rgelt = rgelt;
        if !pceltFetched.is_null() {
            *pceltFetched = 0;
        }

        while celt != 0 && (*this_).m_layer != 0 {
            let nameid: TAGID = SdbFindFirstTag((*this_).m_db, (*this_).m_layer, (TAG_NAME) as TAG);
            if nameid != 0 {
                let name: LPWSTR = SdbGetStringTagPtr((*this_).m_db, nameid);
                if !name.is_null() && IsBuiltinLayer(name) == 0 {
                    let Size: ULONG = (wcslen(name) as ULONG) + 1;

                    *rgelt = CoTaskMemAlloc(((Size as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPOLESTR;
                    StringCchCopyW(*rgelt, (Size) as _, name);

                    if !pceltFetched.is_null() {
                        *pceltFetched += 1;
                    }

                    celt -= 1;
                    rgelt = rgelt.add(1);
                }
            }
            (*this_).m_layer = SdbFindNextTag((*this_).m_db, (*this_).m_root, (*this_).m_layer);
        }
        return if celt != 0 { S_FALSE } else { S_OK };
    }
    pub unsafe fn Skip(&mut self, mut celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        while (*this_).m_layer != 0 && celt != 0 {
            (*this_).m_layer = SdbFindNextTag((*this_).m_db, (*this_).m_root, (*this_).m_layer);
            celt -= 1;
        }
        if celt != 0 { S_FALSE } else { S_OK }
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_layer = TAGID_NULL;
        (*this_).m_root = (*this_).m_layer;
        if !(*this_).m_db.is_null() {
            (*this_).m_root = SdbFindFirstTag((*this_).m_db, TAGID_ROOT, (TAG_DATABASE) as TAG);
            if (*this_).m_root != TAGID_NULL {
                (*this_).m_layer = SdbFindFirstTag((*this_).m_db, (*this_).m_root, (TAG_LAYER) as TAG);
                return S_OK;
            }
        }
        return E_FAIL;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumString,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}
