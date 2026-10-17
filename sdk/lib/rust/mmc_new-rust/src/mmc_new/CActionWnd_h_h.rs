use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_6C8C0FCAF11A: [WCHAR; 16] = [0x004D, 0x004D, 0x0043, 0x0041, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;



impl CActionWnd {
    pub unsafe fn GetWndClassInfo() -> &'static mut CWndClassInfo {
        static mut wc: core::mem::MaybeUninit<CWndClassInfo> = core::mem::MaybeUninit::uninit();
        static mut wc_init: bool = false;
        if !wc_init {
            // The island's `WNDCLASSEX` (inside _ATL_WNDCLASSINFOW) is the A spelling; the
            // class is a UNICODE one, so it is built as WNDCLASSEXW (same layout) and stored.
            let m_wc = WNDCLASSEXW {
                cbSize: core::mem::size_of::<WNDCLASSEXW>() as UINT,
                style: 0,
                lpfnWndProc: Some(StartWindowProc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: core::ptr::null_mut(),
                hIcon: core::ptr::null_mut(), // LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE(IDI_MAINAPP)),
                hCursor: core::ptr::null_mut(),
                hbrBackground: (COLOR_BTNFACE + 1) as usize as HBRUSH,
                lpszMenuName: core::ptr::null_mut(),
                lpszClassName: CPP_COM_WSTR_6C8C0FCAF11A.as_ptr(),
                hIconSm: core::ptr::null_mut(), // LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE(IDI_MAINAPP))
            };
            core::ptr::addr_of_mut!(wc).write(core::mem::MaybeUninit::new(CWndClassInfo {
                m_wc: core::mem::transmute_copy(&m_wc),
                m_lpszOrigName: core::ptr::null_mut(),
                pWndProc: None,
                m_lpszCursorID: IDC_ARROW,
                m_bSystemCursor: TRUE as _,
                m_atom: 0,
                // L"": all zero.
                m_szAutoName: [0; core::mem::size_of::<*const u8>() * 2 + 5],
            }));
            wc_init = true;
        }
        &mut *(core::ptr::addr_of_mut!(wc) as *mut CWndClassInfo)
    }
    pub unsafe fn GetWndClassName() -> LPCTSTR {
        return (CActionWnd::GetWndClassInfo().m_wc.lpszClassName).cast();
    }
}
