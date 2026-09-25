//! shell32's DllMain, the `shell32_hInstance` global, and the three stdcall trampolines that used
//! to be C++ in shell32.cpp — the last of its runtime half (Phase 3 of retiring that file).
#![allow(non_upper_case_globals, non_snake_case)]

use crate::island::{
    DisableThreadLibraryCalls, GetModuleFileNameW, InitCommonControlsEx, BOOL, DWORD, HINSTANCE, HRESULT,
    INITCOMMONCONTROLSEX, INT, LPCITEMIDLIST, LPSTR, LPVOID, SIZE_T, UINT};
use core::ffi::c_void;

const DLL_PROCESS_DETACH: DWORD = 0;
const DLL_PROCESS_ATTACH: DWORD = 1;
const MAX_PATH: usize = 260;
// commctrl.h
const ICC_WIN95_CLASSES: DWORD = 0x0000_00FF;
const ICC_DATE_CLASSES: DWORD = 0x0000_0100;
const ICC_USEREX_CLASSES: DWORD = 0x0000_0200;

/// Defined by shell32.cpp until now; every Rust module that reads it declares it
/// `extern "C" { static shell32_hInstance: HINSTANCE; }`, which resolves here.
#[no_mangle]
pub static mut shell32_hInstance: HINSTANCE = core::ptr::null_mut();

/// shell32.cpp's DllMain, in its order. `gModule.Init(ObjectMap, ...)` and `gModule.Term()` have
/// no counterpart: Init recorded the LIBID and ran each entry's ObjectMain — the empty
/// CComObjectRootBase one for all 36 (checked: no class overrides it) — and the class-object
/// table is static in com_server.rs; Term released ATL's cached class factories, which here are
/// statics with nothing to free.
#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, _reserved: LPVOID) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH {
        shell32_hInstance = hInstance;
        DisableThreadLibraryCalls(hInstance);

        // full path to this DLL, for IExtractIconW_fnGetIconLocation()
        let name = core::ptr::addr_of_mut!(crate::folders::swShell32Name) as *mut u16;
        GetModuleFileNameW(hInstance, name, MAX_PATH as DWORD);
        *name.add(MAX_PATH - 1) = 0;

        let init = INITCOMMONCONTROLSEX {
            dwSize: core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD,
            dwICC: ICC_WIN95_CLASSES | ICC_DATE_CLASSES | ICC_USEREX_CLASSES,
        };
        InitCommonControlsEx(&init as *const _ as *mut INITCOMMONCONTROLSEX);

        // (the C++ comment: "Bad idea, initialization in DllMain!")
        crate::shell32::changenotify::InitChangeNotifications();
    } else if dwReason == DLL_PROCESS_DETACH {
        shell32_hInstance = core::ptr::null_mut();
        crate::shell32::iconcache::SIC_Destroy();
        crate::shell32::changenotify::FreeChangeNotifications();
    }
    1
}

// The generated modules declare these three helpers in `extern "system"` blocks (stdcall) under
// these link names, while the Rust implementations are `extern "C"` (cdecl) — so these are real
// CONVERSIONS, not aliases: calling a cdecl function through a stdcall declaration unbalances
// the stack on i386. They were C++ wrappers in shell32.cpp; same job, same symbols.

#[no_mangle]
pub unsafe extern "system" fn Shell32RustCoCreateInitSF(
    pidlRoot: LPCITEMIDLIST, clsid: *const crate::island::GUID, csidl: INT,
    riid: *const crate::island::IID, ppvOut: *mut LPVOID,
) -> HRESULT {
    crate::shell32::shlfolder::SHELL32_CoCreateInitSFWithCsidl(
        pidlRoot.cast(), clsid.cast(), csidl, riid.cast(), ppvOut.cast(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn Shell32RustMapContextMenuVerbToCmdId(pInfo: *mut c_void, pMap: *const c_void) -> INT {
    crate::shell32::utils::SHELL_MapContextMenuVerbToCmdId(pInfo.cast(), pMap.cast())
}

#[no_mangle]
pub unsafe extern "system" fn Shell32RustGetCommandStringImpl(
    CmdId: SIZE_T, uFlags: UINT, Buf: LPSTR, cchBuf: UINT, pMap: *const c_void,
) -> HRESULT {
    crate::shell32::utils::SHELL_GetCommandStringImpl(CmdId as usize, uFlags, Buf.cast(), cchBuf, pMap.cast())
}
