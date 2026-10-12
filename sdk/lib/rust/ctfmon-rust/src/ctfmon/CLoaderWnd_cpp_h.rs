use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut CLoaderWnd__s_bUninitedSystem: i32 = 0;
pub static mut CLoaderWnd__s_bWndClassRegistered: i32 = 0;




impl CLoaderWnd {
    pub unsafe fn Init(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if CLoaderWnd__s_bWndClassRegistered != 0 {
            return (TRUE) as BOOL; // Already registered
        }
    
        // Register a window class
        let mut wc: WNDCLASSEX = core::mem::zeroed();
        // ZeroMemory(&wc, sizeof(wc)): a BYTE count, so the pointer is a byte pointer (a
        // *mut WNDCLASSEX would zero sizeof(WNDCLASSEX) ELEMENTS -- 80x the struct, on the stack).
        core::ptr::write_bytes(core::ptr::addr_of_mut!(wc) as *mut u8, 0, core::mem::size_of::<WNDCLASSEX>());
        wc.cbSize = core::mem::size_of::<WNDCLASSEX>() as u32;
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.hInstance = g_hInst;
        wc.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
        wc.lpfnWndProc = Some(CLoaderWnd::WindowProc);
        // (&[0x0043, 0x0069, 0x0043, 0x0054, 0x0069, 0x0070, 0x0042, 0x0061, 0x0072, 0x0043, 0x006c, 0x0061, 0x0073, 0x0073, 0x0000u16]).as_ptr() /* TEXT "CiCTipBarClass" */ as an explicit NUL-terminated wide array
        let class_name: [u16; 15] = [
            'C' as u16, 'i' as u16, 'C' as u16, 'T' as u16, 'i' as u16,
            'p' as u16, 'B' as u16, 'a' as u16, 'r' as u16, 'C' as u16,
            'l' as u16, 'a' as u16, 's' as u16, 's' as u16, 0,
        ];
        wc.lpszClassName = class_name.as_ptr();
        if RegisterClassEx(core::ptr::addr_of_mut!(wc)) == 0 {
            return (FALSE) as BOOL;
        }
    
        CLoaderWnd__s_bWndClassRegistered = (TRUE) as i32; // Remember
        return (TRUE) as BOOL;
    }
    pub unsafe fn CreateWnd(&mut self) -> HWND {
        let this_: *mut Self = self;
        (*this_).m_hWnd = CreateWindowEx(0, (&[0x0043, 0x0069, 0x0043, 0x0054, 0x0069, 0x0070, 0x0042, 0x0061, 0x0072, 0x0043, 0x006c, 0x0061, 0x0073, 0x0073, 0x0000u16]).as_ptr() /* TEXT "CiCTipBarClass" */, core::ptr::null_mut(), WS_DISABLED,
                                  0, 0, 0, 0, core::ptr::null_mut(), core::ptr::null_mut(), g_hInst, core::ptr::null_mut());
        return (*this_).m_hWnd;
    }
    pub unsafe extern "system" fn WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        match uMsg {
            WM_CREATE => {
            }
    
            WM_DESTROY => {
                PostQuitMessage(0);
            }
    
            WM_QUERYENDSESSION => {
                // NOTE: We don't support Win95/98/Me
                return (TRUE) as LRESULT;
            }
    
            WM_ENDSESSION => {
                if wParam != 0 {
                    // The session is being ended?
                    if CLoaderWnd__s_bUninitedSystem == 0 {
                        // Un-initialize now
                        UninitApp();
                        TF_UninitSystem();
                        CLoaderWnd__s_bUninitedSystem = (TRUE) as i32;
                    }
                } else if CLoaderWnd__s_bUninitedSystem != 0 {
                    // Once un-initialized?
                    // Re-initialize
                    TF_InitSystem();
                    if g_bOnWow64 == 0 {
                        GetPopupTipbar(hwnd, (g_fWinLogon != 0) as BOOL);
                    }
    
                    CLoaderWnd__s_bUninitedSystem = (FALSE) as i32;
                }
            }
    
            WM_SYSCOLORCHANGE | WM_DISPLAYCHANGE => {
                if g_bOnWow64 == 0 {
                    // Is the system x86/x64 native?
                    CRegWatcher::StartSysColorChangeTimer();
                }
            }
    
            _ => {
                return DefWindowProc(hwnd, uMsg, wParam, lParam);
            }
        }
    
        return 0;
    }
}
