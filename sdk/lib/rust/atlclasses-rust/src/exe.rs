//! ATL's `CAtlExeModuleT<T>` (atlbase.h): the `WinMain` skeleton an EXE's module class derives
//! from -- initialize COM, parse the command line, run, uninitialize.
//!
//! The C++ base reaches the derived class's overrides through `static_cast<T*>(this)`; here
//! that is [`ExeModule`]'s defaults and the overrides `cpp_app` generates for the members the
//! module's class declares.
//!
//! COM class objects: ATL's `RegisterClassObjects` walks the module's object map
//! (`OBJECT_ENTRY_AUTO`). This crate has no object map, so registering is the empty walk,
//! `S_OK` -- correct ONLY for a module that declares none, which `cpp_app` checks before it
//! generates this glue.

use core::ffi::c_void;

const COINIT_APARTMENTTHREADED: u32 = 0x2;

#[repr(C)]
struct Msg {
    hwnd: *mut c_void,
    message: u32,
    w: usize,
    l: isize,
    time: u32,
    pt: [i32; 2],
}

extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
    fn CoUninitialize();
    fn GetCommandLineW() -> *const u16;
    fn GetCurrentThreadId() -> u32;
    fn GetMessageW(msg: *mut Msg, hwnd: *mut c_void, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
}

/// The data `CAtlExeModuleT` adds: the thread that runs the message loop.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct CAtlExeModule {
    pub m_dwMainThreadID: u32,
}

impl CAtlExeModule {
    /// `CAtlExeModuleT()`: `m_dwMainThreadID(::GetCurrentThreadId())`. (Its
    /// `_AtlComModule.ExecuteObjectMain(true)` walks the same empty object map.)
    pub fn new() -> Self {
        Self {
            m_dwMainThreadID: unsafe { GetCurrentThreadId() },
        }
    }
}

impl Default for CAtlExeModule {
    fn default() -> Self {
        Self::new()
    }
}

/// `CAtlExeModuleT<T>`'s overridables, each defaulting to the C++ base's body.
#[allow(unused_variables)]
pub trait ExeModule {
    unsafe fn initialize_com() -> i32
    where
        Self: Sized,
    {
        CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED)
    }
    unsafe fn uninitialize_com()
    where
        Self: Sized,
    {
        CoUninitialize()
    }
    unsafe fn parse_command_line(&mut self, cmd_line: *const u16, ret_code: *mut i32) -> bool {
        true // "unimplemented!" in the C++, which returns true
    }
    unsafe fn run(&mut self, show: i32) -> i32 {
        let hr = self.pre_message_loop(show);
        if hr == 0 {
            self.run_message_loop();
            return self.post_message_loop();
        }
        hr
    }
    unsafe fn pre_message_loop(&mut self, show: i32) -> i32 {
        register_class_objects()
    }
    unsafe fn run_message_loop(&mut self) {
        let mut msg: Msg = core::mem::zeroed();
        while GetMessageW(&mut msg, core::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    unsafe fn post_message_loop(&mut self) -> i32 {
        revoke_class_objects()
    }
}

/// `RegisterClassObjects(dwClsContext, dwFlags)` over the empty object map.
pub fn register_class_objects() -> i32 {
    0
}

/// `RevokeClassObjects()` over the empty object map.
pub fn revoke_class_objects() -> i32 {
    0
}

/// `CAtlExeModuleT::WinMain(nShowCmd)`.
pub unsafe fn win_main<T: ExeModule>(this: *mut T, show: i32) -> i32 {
    let mut hr = T::initialize_com();
    if hr < 0 {
        return hr;
    }
    if (*this).parse_command_line(GetCommandLineW(), &mut hr) {
        hr = (*this).run(show);
    }
    T::uninitialize_com();
    hr
}
