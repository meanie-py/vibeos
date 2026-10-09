use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut ODBCProc: APPLET_PROC = None;

#[no_mangle]
pub static mut hLibrary: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCpl: HWND,
    uMsg: UINT,
    lParam1: LPARAM,
    lParam2: LPARAM,
) -> LONG {
    match uMsg {
        CPL_INIT => {
            return (TRUE) as LONG;
        }
        CPL_DBLCLK => {
            if ODBCProc.is_none() {
                let mut szBuffer: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
                let szPath: [u16; 35] = [
                    '%' as u16, 's' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16,
                    'm' as u16, 'r' as u16, 'o' as u16, 'o' as u16, 't' as u16, '%' as u16,
                    '\\' as u16, 's' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16,
                    'm' as u16, '3' as u16, '2' as u16, '\\' as u16, 'o' as u16, 'd' as u16,
                    'b' as u16, 'c' as u16, 'c' as u16, 'p' as u16, '3' as u16, '2' as u16,
                    '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0,
                ];

                if ExpandEnvironmentStrings(
                    (szPath.as_ptr() as LPCTSTR).cast(),
                    (szBuffer.as_mut_ptr() as LPTSTR).cast(),
                    (core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>()
                        / core::mem::size_of::<TCHAR>()) as DWORD,
                ) > 0
                {
                    hLibrary = LoadLibrary((szBuffer.as_ptr()).cast()) as HMODULE;
                    if !hLibrary.is_null() {
                        ODBCProc = core::mem::transmute::<FARPROC, APPLET_PROC>(
                            GetProcAddress(
                                hLibrary as HINSTANCE,
                                b"ODBCCPlApplet\0".as_ptr() as LPCSTR,
                            ),
                        );
                    }
                }
            }

            if ODBCProc.is_some() {
                return (ODBCProc.unwrap_unchecked())(hwndCpl, uMsg, lParam1, lParam2);
            } else {
                if !hLibrary.is_null() {
                    FreeLibrary(hLibrary);
                }

                TerminateProcess(
                    GetCurrentProcess(),
                    -(1i32) as UINT,
                );
                return -(1) as LONG;
            }
        }
        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpReserved: LPVOID,
) -> BOOL {
    let mut InitControls: INITCOMMONCONTROLSEX = core::mem::zeroed();
    UNREFERENCED_PARAMETER!(lpReserved);

    match dwReason {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH => {
            InitControls.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
            InitControls.dwICC = (ICC_LISTVIEW_CLASSES | ICC_UPDOWN_CLASS | ICC_BAR_CLASSES) as DWORD;
            InitCommonControlsEx(core::ptr::addr_of_mut!(InitControls));

            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
