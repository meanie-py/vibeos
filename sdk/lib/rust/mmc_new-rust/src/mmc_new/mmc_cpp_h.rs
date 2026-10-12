use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn wWinMain(_hInstance: HINSTANCE, _hPrevInstance: HINSTANCE, _lpCmdLine: LPWSTR, nCmdShow: i32) -> i32 {
    __cxx_global_init();
    let mut icc: INITCOMMONCONTROLSEX = INITCOMMONCONTROLSEX {
        dwSize: core::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_BAR_CLASSES | ICC_COOL_CLASSES,
    };

    if InitCommonControlsEx(core::ptr::addr_of_mut!(icc)) == 0 {
        DPRINT1!("ERROR: InitCommonControlsEx\n");
    }

    return g_AtlModule.WinMain(nCmdShow);
}
#[no_mangle]
pub unsafe extern "C" fn __cxx_global_init() {
    // `static CMMCModule g_AtlModule;` -- its constructor (CMainWnd's included).
    (*core::ptr::addr_of_mut!(g_AtlModule)).__ctor_0();
}

impl CMMCModule {
    pub unsafe fn PreMessageLoop(&mut self, nShowCmd: i32) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = (*this_).RegisterClassObjects(CLSCTX_LOCAL_SERVER, REGCLS_MULTIPLEUSE);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
    
        (*this_).m_hAccel = LoadAcceleratorsW(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCEW!(IDA_MMC));
        if (*this_).m_hAccel.is_null() {
            DPRINT1!("ERROR: Failed to load accelerators\n");
            return E_FAIL;
        }
    
        let mut ret: WNDPROC = core::mem::zeroed();
        if CConsoleWnd::GetWndClassInfo().Register(core::ptr::addr_of_mut!(ret)) == 0 {
            DPRINT1!("ERROR: Failed to register CConsoleWnd class\n");
            return E_FAIL;
        }
    
        let mut Filename: LPCWSTR = core::ptr::null();
    
        if (*this_).m_MainWnd.Create(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            CPP_COM_WSTR_96A296D224F2.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
            WS_EX_WINDOWEDGE,
            0,
            Filename as LPVOID,
        ) == core::ptr::null_mut()
        {
            DPRINT1!("ERROR: Failed to create m_MainWnd\n");
            return E_FAIL;
        }
    
        (*this_).m_MainWnd.__base0.ShowWindow(nShowCmd);
        return S_OK;
    }
    pub unsafe fn RunMessageLoop(&mut self) {
        let this_: *mut Self = self;
        let mut msg: MSG = core::mem::zeroed();
        let mut bRet: BOOL;
        loop {
            bRet = GetMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0);
            if bRet == 0 {
                break;
            }
            if bRet == -1 {
                DPRINT1!("ERROR: GetMessage\n");
                break;
            }
            if !(TranslateMDISysAccel((*this_).m_MainWnd.m_MDIClient.op_to_HWND(), core::ptr::addr_of_mut!(msg)) != 0)
                && !(TranslateAcceleratorW((*this_).m_MainWnd.op_to_HWND(), (*this_).m_hAccel, core::ptr::addr_of_mut!(msg)) != 0)
            {
                TranslateMessage(core::ptr::addr_of_mut!(msg));
                DispatchMessageW(core::ptr::addr_of_mut!(msg));
            }
        }
    }
    pub unsafe fn InitializeCom() -> HRESULT {
        return CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        // `CMainWnd m_MainWnd;` is a member with a user-provided constructor, run by the implicit
        // CMMCModule constructor (the cppapp-generated body only sets the vtables).
        (*this_).m_MainWnd.__ctor_0();
    }
}
