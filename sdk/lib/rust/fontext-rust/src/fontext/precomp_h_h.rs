use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_13C2F0CCF85C: [WCHAR; 5] = [0x002E, 0x006F, 0x0074, 0x0066, 0x0000];
pub static CPP_COM_WSTR_3860EAB6D2FA: [WCHAR; 5] = [0x002E, 0x0066, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_47F50243F401: [WCHAR; 5] = [0x002E, 0x0074, 0x0074, 0x0066, 0x0000];
pub static CPP_COM_WSTR_5A737710FEF3: [WCHAR; 5] = [0x002E, 0x006F, 0x0074, 0x0063, 0x0000];
pub static CPP_COM_WSTR_5B20018FE43E: [WCHAR; 5] = [0x002E, 0x0074, 0x0074, 0x0063, 0x0000];
pub static CPP_COM_WSTR_F1E8C5348F26: [WCHAR; 5] = [0x002E, 0x0066, 0x006F, 0x006E, 0x0000];

use core::ffi::c_void;








#[no_mangle]
pub unsafe extern "C" fn IsFontDotExt(pchDotExt: LPCWSTR) -> BOOL {
    let array: [LPCWSTR; 7] = [
        CPP_COM_WSTR_47F50243F401.as_ptr(), CPP_COM_WSTR_5B20018FE43E.as_ptr(), CPP_COM_WSTR_13C2F0CCF85C.as_ptr(), CPP_COM_WSTR_5A737710FEF3.as_ptr(), CPP_COM_WSTR_F1E8C5348F26.as_ptr(), CPP_COM_WSTR_3860EAB6D2FA.as_ptr(), core::ptr::null()
    ];
    let mut pp: *const LPCWSTR = array.as_ptr();
    while !(*pp).is_null() {
        if StrCmpIW(*pp, pchDotExt) == 0 {
            return (TRUE) as BOOL;
        }
        pp = pp.add(1);
    }
    return (FALSE) as BOOL;
}

impl tagINSTALL_FONT_DATA {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).pDataObj = core::ptr::null_mut();
        (*this_).hrResult = S_OK;
        (*this_).hwnd = core::ptr::null_mut();
        (*this_).iStep = 0;
        (*this_).cSteps = 0;
        (*this_).bCanceled = (FALSE as i32);
        (*this_).pidlParent = core::ptr::null_mut();
        (*this_).apidl = core::ptr::null_mut();
    }
}

impl CRegKeyHandleArray {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).cKeys = 0;
        {}
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        CloseRegKeyArray(core::ptr::addr_of_mut!((*this_).hKeys) as *mut HKEY, (*this_).cKeys);
    }
    pub unsafe fn op_to_HKEY(&mut self) -> *mut HKEY {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).hKeys) as *mut HKEY;
    }
    pub unsafe fn op_to_unsigned_int(&mut self) -> *mut UINT {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).cKeys);
    }
    pub unsafe fn op_to_unsigned_int_2(&mut self) -> UINT {
        let this_: *mut Self = self;
        return (*this_).cKeys;
    }
    pub unsafe fn op_index(&mut self, i: SIZE_T) -> &mut HKEY {
        let this_: *mut Self = self;
        return &mut (*this_).hKeys[i as usize];
    }
}
