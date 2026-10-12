use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_7BD7EE00AFC9: [WCHAR; 12] = [0x004D, 0x004D, 0x0043, 0x0043, 0x0068, 0x0069, 0x006C, 0x0064, 0x0046, 0x0072, 0x006D, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;



impl CConsoleWnd {
    pub unsafe fn GetWndClassInfo() -> &'static mut CWndClassInfo {
        static mut wc: core::mem::MaybeUninit<CWndClassInfo> = core::mem::MaybeUninit::uninit();
        static mut wc_init: bool = false;
        if !wc_init {
            // The island's `WNDCLASSEX` (inside _ATL_WNDCLASSINFOW) is the A spelling; the
            // class is a UNICODE one, so it is built as WNDCLASSEXW (same layout) and stored.
            let m_wc = WNDCLASSEXW {
                cbSize: core::mem::size_of::<WNDCLASSEXW>() as UINT,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(StartWindowProc),
                cbClsExtra: 0,
                cbWndExtra: core::mem::size_of::<*mut CConsoleWnd>() as i32,
                hInstance: core::ptr::null_mut(),
                hIcon: LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE!(IDI_MAINAPP)),
                hCursor: core::ptr::null_mut(),
                hbrBackground: (COLOR_3DFACE + 1) as usize as HBRUSH,
                lpszMenuName: core::ptr::null_mut(),
                lpszClassName: CPP_COM_WSTR_7BD7EE00AFC9.as_ptr(),
                hIconSm: LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE!(IDI_MAINAPP)),
            };
            core::ptr::addr_of_mut!(wc).write(core::mem::MaybeUninit::new(CWndClassInfo {
                m_wc: core::mem::transmute_copy(&m_wc),
                m_lpszOrigName: core::ptr::null_mut(),
                pWndProc: None,
                m_lpszCursorID: IDC_SIZEWE,
                m_bSystemCursor: 1,
                m_atom: 0,
                m_szAutoName: [0; core::mem::size_of::<*const u8>() * 2 + 5],
            }));
            wc_init = true;
        }
        &mut *(core::ptr::addr_of_mut!(wc) as *mut CWndClassInfo)
    }
    pub unsafe fn GetWndClassName() -> LPCWSTR {
        return CConsoleWnd::GetWndClassInfo().m_wc.lpszClassName as LPCWSTR;
    }
}
