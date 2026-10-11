use crate::mmc::console::CreateConsoleWindow;
use crate::mmc::mscfile::HandleRosMscLaunch;
use crate::mmc::console::RegisterMMCWndClasses;
use crate::mmc::console::UnregisterMMCWndClasses;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hAppInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hAppHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hwndMainConsole: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hwndMDIClient: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[export_name = "wWinMain"]
pub unsafe extern "system" fn _tWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPTSTR,
    nCmdShow: i32,
) -> i32 {
    let mut Msg: MSG = core::mem::zeroed();

    hAppInstance = hInstance; // GetModuleHandle(NULL);
    hAppHeap = GetProcessHeap();

    InitCommonControls();

    /* We have extended the MMC_ConsoleFile format with a custom RosLaunch node that can launch external files.
     * This support can be removed after we have converted all our Snap-ins from .exe to real MMC COM DLLs. */
    if HandleRosMscLaunch(lpCmdLine) != 0 {
        return 0;
    }

    if RegisterMMCWndClasses() == 0 {
        /* FIXME - Display error */
        return 1;
    }

    hwndMainConsole = CreateConsoleWindow(core::ptr::null_mut(), nCmdShow);
    if !hwndMainConsole.is_null() {
        while GetMessage(
            core::ptr::addr_of_mut!(Msg),
            core::ptr::null_mut(),
            0,
            0,
        ) != 0
        {
            if TranslateMDISysAccel(hwndMDIClient, core::ptr::addr_of_mut!(Msg)) == 0 {
                TranslateMessage(core::ptr::addr_of_mut!(Msg));
                DispatchMessage(core::ptr::addr_of_mut!(Msg));
            }
        }
    }

    UnregisterMMCWndClasses();

    return 0;
}
