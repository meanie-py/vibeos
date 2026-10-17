use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_DF66F8D67C28: [WCHAR; 13] = [0x004D, 0x004D, 0x0043, 0x004D, 0x0061, 0x0069, 0x006E, 0x0046, 0x0072, 0x0061, 0x006D, 0x0065, 0x0000];

use core::ffi::c_void;







impl CMainWnd {
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
                hIcon: LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE!(IDI_MAINAPP)),
                hCursor: core::ptr::null_mut(),
                hbrBackground: (COLOR_BTNFACE + 1) as usize as HBRUSH,
                lpszMenuName: core::ptr::null_mut(),
                lpszClassName: CPP_COM_WSTR_DF66F8D67C28.as_ptr(),
                hIconSm: LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE!(IDI_MAINAPP)),
            };
            core::ptr::addr_of_mut!(wc).write(core::mem::MaybeUninit::new(CWndClassInfo {
                m_wc: core::mem::transmute_copy(&m_wc),
                m_lpszOrigName: core::ptr::null_mut(),
                pWndProc: None,
                m_lpszCursorID: IDC_ARROW,
                m_bSystemCursor: 1,
                m_atom: 0,
                m_szAutoName: [0; core::mem::size_of::<*const u8>() * 2 + 5],
            }));
            wc_init = true;
        }
        &mut *(core::ptr::addr_of_mut!(wc) as *mut CWndClassInfo)
    }
    pub unsafe fn GetWndClassName() -> LPCTSTR {
        return CMainWnd::GetWndClassInfo().m_wc.lpszClassName as LPCTSTR;
    }
    pub unsafe extern "system" fn XDefFrameProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let pThis: *mut CMainWnd = hWnd as *mut CMainWnd;
        return DefFrameProc((*pThis).__base0.m_hWnd, (*pThis).m_MDIClient.m_hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn InitMain(&mut self) {
        let this_: *mut Self = self;
        CSnapin__delete((*this_).m_RootNode);
        (*this_).m_RootNode = core::ptr::null_mut();

        (*this_).m_DocumentMode = DocumentMode_Author;
        (*this_).m_LogicalReadOnly = (FALSE as i32);
        (*this_).m_PreventViewCustomization = (FALSE as i32);
        (*this_).m_bToolBarVisible = (TRUE as i32);
        (*this_).m_NextViewId = 1;
        (*this_).m_NextNodeId = 1;
    }
    pub unsafe fn GetActiveChildInfo(&mut self) -> *mut CConsoleWnd {
        let this_: *mut Self = self;
        let mut hWndMDIChild: HWND;

        hWndMDIChild = (*this_).m_MDIClient.SendMessageW__0(WM_MDIGETACTIVE, 0, 0) as HWND;
        if hWndMDIChild.is_null() {
            return core::ptr::null_mut();
        }

        return GetWindowLongPtr(hWndMDIChild, 0) as *mut CConsoleWnd;
    }
    pub unsafe fn CreateNewConsoleTitle(&mut self, str: &mut CAtlString) {
        let this_: *mut Self = self;
        let mut args: [DWORD_PTR; 1] = [(*this_).m_ConsoleNumber as DWORD_PTR];
        str.LoadStringW(IDS_CONSOLETITLE);

        let mut lpTarget: LPTSTR = core::ptr::null_mut();
        let Ret: DWORD = FormatMessage(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ARGUMENT_ARRAY,
            str.GetString() as *const core::ffi::c_void,
            0,
            0,
            core::ptr::addr_of_mut!(lpTarget) as LPTSTR,
            0,
            args.as_mut_ptr() as *mut va_list,
        );

        if Ret != 0 {
            *str = CAtlString::from_ptr(lpTarget as *const WCHAR);
            LocalFree(lpTarget as HLOCAL);
        }
    }
}
