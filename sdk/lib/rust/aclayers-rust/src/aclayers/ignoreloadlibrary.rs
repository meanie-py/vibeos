use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type LOADLIBRARYAPROC = unsafe extern "system" fn(lpLibFileName: LPCSTR) -> HMODULE;

pub type LOADLIBRARYEXAPROC =
    unsafe extern "system" fn(lpLibFileName: LPCSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;

pub type LOADLIBRARYWPROC = unsafe extern "system" fn(lpLibFileName: LPCWSTR) -> HMODULE;

pub type LOADLIBRARYEXWPROC =
    unsafe extern "system" fn(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;

// SHIM_NS IgnoreLoadLibrary: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(IgnoreLoadLibrary);

// winbase.h / winnt.h macros the island does not carry.
const SEM_FAILCRITICALERRORS: UINT = 0x0001;
const SEM_NOGPFAULTERRORBOX: UINT = 0x0002;
const SEM_NOOPENFILEERRORBOX: UINT = 0x8000;

extern "system" {
    fn SetErrorMode(uMode: UINT) -> UINT;
}

pub unsafe extern "system" fn APIHook_LoadLibraryA(lpLibFileName: LPCSTR) -> HMODULE {
    let Module: HMODULE;
    let dwOldErrorMode: DWORD;

    dwOldErrorMode = SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    Module = CALL_SHIM!(0, LOADLIBRARYAPROC)(lpLibFileName);
    SetErrorMode(dwOldErrorMode);

    return Module;
}

pub unsafe extern "system" fn APIHook_LoadLibraryExA(lpLibFileName: LPCSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE {
    let Module: HMODULE;
    let dwOldErrorMode: DWORD;

    dwOldErrorMode = SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    Module = CALL_SHIM!(1, LOADLIBRARYEXAPROC)(lpLibFileName, hFile, dwFlags);
    SetErrorMode(dwOldErrorMode);

    return Module;
}

pub unsafe extern "system" fn APIHook_LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE {
    let Module: HMODULE;
    let dwOldErrorMode: DWORD;

    dwOldErrorMode = SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    Module = CALL_SHIM!(2, LOADLIBRARYWPROC)(lpLibFileName);
    SetErrorMode(dwOldErrorMode);

    return Module;
}

pub unsafe extern "system" fn APIHook_LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE {
    let Module: HMODULE;
    let dwOldErrorMode: DWORD;

    dwOldErrorMode = SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    Module = CALL_SHIM!(3, LOADLIBRARYEXWPROC)(lpLibFileName, hFile, dwFlags);
    SetErrorMode(dwOldErrorMode);

    return Module;
}

// SHIM_NUM_HOOKS 4, no SHIM_NOTIFY_FN
implement_shim!(IgnoreLoadLibrary, 4usize, None, [
    (0, "KERNEL32.DLL", "LoadLibraryA", APIHook_LoadLibraryA),
    (1, "KERNEL32.DLL", "LoadLibraryExA", APIHook_LoadLibraryExA),
    (2, "KERNEL32.DLL", "LoadLibraryW", APIHook_LoadLibraryW),
    (3, "KERNEL32.DLL", "LoadLibraryExW", APIHook_LoadLibraryExW),
]);
