use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_B0EE5A657FD2: [WCHAR; 13] = [0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x005C, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn GetInterfaceName(InterfaceString: *const WCHAR, buf: *mut WCHAR, size: usize) -> bool {
    let mut LocalBuf: [WCHAR; 100] = [0 as WCHAR; 100];
    let mut dwType: DWORD = 0;
    let mut dwDataSize: DWORD = (size * core::mem::size_of::<WCHAR>()) as DWORD;

    if !((StringCchPrintfW!(LocalBuf.as_mut_ptr(), _countof!(LocalBuf), CPP_COM_WSTR_B0EE5A657FD2.as_ptr(), InterfaceString)) >= 0) {
        return false;
    }

    return SHRegGetValueW(HKEY_CLASSES_ROOT, LocalBuf.as_mut_ptr(), core::ptr::null_mut(), (RRF_RT_REG_SZ) as SRRF, core::ptr::addr_of_mut!(dwType), (buf).cast(), core::ptr::addr_of_mut!(dwDataSize)) == (ERROR_SUCCESS as i32);
}
#[no_mangle]
pub unsafe extern "C" fn guid2string(iid: REFCLSID) -> *mut WCHAR {
    #[allow(non_upper_case_globals)]
    static mut buf: [[WCHAR; 300]; 2] = [[0; 300]; 2];
    #[allow(non_upper_case_globals)]
    static mut idx: i32 = 0;
    let bufp = core::ptr::addr_of_mut!(buf);

    idx ^= 1;

    let row: *mut WCHAR = (*bufp)[idx as usize].as_mut_ptr();

    let mut tmp: LPOLESTR = core::ptr::null_mut();
    let hr: HRESULT = ProgIDFromCLSID(iid, core::ptr::addr_of_mut!(tmp));
    if hr >= 0
    {
        wcscpy(row, tmp);
        CoTaskMemFree(tmp as *mut core::ffi::c_void);
        return row;
    }
    StringFromGUID2(iid, row, (_countof!((*bufp)[idx as usize])) as INT);
    if GetInterfaceName(row, row, (_countof!((*bufp)[idx as usize])) as _)
    {
        return row;
    }
    StringFromGUID2(iid, row, (_countof!((*bufp)[idx as usize])) as INT);

    row
}
