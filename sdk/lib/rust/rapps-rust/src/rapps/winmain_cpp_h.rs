use crate::prelude::*;
use crate::rapps::settings_cpp_h::LoadSettings;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// NOTE: MAINWINDOWCLASSNAME was not provided; "MainWindowClass" is a placeholder. Verify against the original header.
#[no_mangle]
pub static mut szWindowClass: LPCWSTR = {
    static MAINWINDOWCLASSNAME_W: [u16; 16] = [
        77, 97, 105, 110, 87, 105, 110, 100, 111, 119, 67, 108, 97, 115, 115, 0,
    ];
    MAINWINDOWCLASSNAME_W.as_ptr()
};
#[no_mangle]
pub static mut g_Busy: LONG = 0;
#[no_mangle]
pub static mut hMainWnd: HWND = core::ptr::null_mut();
#[no_mangle]
pub static mut hInst: HINSTANCE = core::ptr::null_mut();
// BEGIN_OBJECT_MAP/END_OBJECT_MAP with no entries yields one zeroed _ATL_OBJMAP_ENTRY.
pub static mut ObjectMap: [_ATL_OBJMAP_ENTRY; 1] = unsafe { core::mem::zeroed() };
// C++ ATL class instances; zero-initialized as an approximation of their constructors.
pub static mut gModule: CComModule = unsafe { core::mem::zeroed() };
pub static mut gWinModule: CAtlWinModule = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, hPrevInstance: HINSTANCE, lpCmdLine: LPWSTR, nShowCmd: INT) -> INT {
    let mut gdiplusStartupInput = GdiplusStartupInput__make_0(None, 0, 0);
    let mut gdiplusToken: ULONG_PTR = 0;

    // CComModule::Init(ObjectMap, hInstance, NULL): ObjectMap holds only the terminator entry
    // (pclsid == NULL), so the pfnObjectMain walk visits nothing (atlbase.h); the module keeps the map.
    gModule.m_pObjMap = core::ptr::addr_of_mut!(ObjectMap) as *mut _ATL_OBJMAP_ENTRY;
    GdiplusStartup(core::ptr::addr_of_mut!(gdiplusToken), core::ptr::addr_of!(gdiplusStartupInput) as *const _, core::ptr::null_mut());

    if GetUserDefaultUILanguage() == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT)
    {
        SetProcessDefaultLayout(LAYOUT_RTL);
    }

    hInst = hInstance;
    let bIsFirstLaunch: BOOL = if LoadSettings(core::ptr::addr_of_mut!(SettingsInfo)) == 0 { 1 } else { 0 };

    InitLogs();
    InitCommonControls();
    SetThreadPriority(GetCurrentThread(), (THREAD_PRIORITY_ABOVE_NORMAL as i32)); // Give UI higher priority than background threads

    // parse cmd-line and perform the corresponding operation
    let bSuccess: BOOL = ParseCmdAndExecute(GetCommandLineW(), bIsFirstLaunch, (SW_SHOWNORMAL as i32));

    GdiplusShutdown(gdiplusToken);
    // CComModule::Term(): the map has no entries (and the auto-map is empty), so nothing to release.

    return if bSuccess != 0 { 0 } else { 1 };
}
