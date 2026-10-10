use crate::servman::misc::AllocAndLoadString;
use crate::servman::mainwnd::CreateMainWindow;
use crate::servman::mainwnd::InitMainWindowImpl;
use crate::servman::mainwnd::UninitMainWindowImpl;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut ProcessHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut g_hProgDlg: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;
#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hThisInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lpAppName: LPWSTR = core::ptr::null_mut();
    let mut hMainWnd: HWND = core::ptr::null_mut();
    let mut hAccelTable: HACCEL = core::ptr::null_mut();
    let mut Msg: MSG = core::mem::zeroed::<MSG>();
    let mut Ret: ::core::ffi::c_int = 1;
    let mut icex: INITCOMMONCONTROLSEX = core::mem::zeroed::<INITCOMMONCONTROLSEX>();

    let uiLang: ::core::ffi::c_int = GetUserDefaultUILanguage() as ::core::ffi::c_int;
    if uiLang == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) as ::core::ffi::c_int {
        SetProcessDefaultLayout(LAYOUT_RTL as DWORD);
    }

    hInstance = hThisInstance;
    ProcessHeap = GetProcessHeap();

    icex.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    icex.dwICC = (ICC_WIN95_CLASSES | ICC_COOL_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(icex));

    if AllocAndLoadString(
        core::ptr::addr_of_mut!(lpAppName),
        hInstance,
        IDS_APPNAME as UINT,
    ) == 0
    {
        return 1;
    }

    hAccelTable = LoadAcceleratorsW(
        hInstance,
        MAKEINTRESOURCEW!(IDA_SERVMAN) as LPCWSTR,
    );

    if InitMainWindowImpl() != 0 {
        hMainWnd = CreateMainWindow(lpAppName, nCmdShow);
        if !hMainWnd.is_null() {
            /* pump the message queue */
            while GetMessageW(
                core::ptr::addr_of_mut!(Msg),
                core::ptr::null_mut(),
                0 as UINT,
                0 as UINT,
            ) != 0
            {
                //if ( !hProgDlg || !IsWindow(hProgDlg) || !IsDialogMessage(hProgDlg, &Msg) )
                //if (!IsDialogMessage(g_hProgDlg, &Msg))
                if TranslateAcceleratorW(
                    hMainWnd,
                    hAccelTable,
                    core::ptr::addr_of_mut!(Msg),
                ) == 0
                {
                    TranslateMessage(core::ptr::addr_of_mut!(Msg));
                    DispatchMessageW(core::ptr::addr_of_mut!(Msg));
                }
            }

            Ret = 0;
        }

        UninitMainWindowImpl();
    }

    LocalFree(lpAppName as HLOCAL);

    return Ret;
}
