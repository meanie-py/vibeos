use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::message::GlobalAlloc;
use crate::windows::window::HeapAlloc;
#[allow(unused_imports)]
use core::ffi::c_void;

// gdi32/kernel32/CRT glue + CP_ACP live in crate::winbase.
use crate::winbase::{memcpy, WideCharToMultiByte, GetProcessHeap, CP_ACP};

// CRT string helpers (msvcrt).
extern "C" {
    fn strlen(s: LPCSTR) -> usize;
    fn strcpy(dst: LPSTR, src: LPCSTR) -> LPSTR;
}

// user32 exports referenced here (stdcall on i386; resolve within this DLL).
extern "system" {
    fn RegisterWindowMessageA(lpString: LPCSTR) -> UINT;
    fn FindWindowA(lpClassName: LPCSTR, lpWindowName: LPCSTR) -> HWND;
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

// kernel32 exports.
extern "system" {
    fn WinExec(lpCmdLine: LPCSTR, uCmdShow: UINT) -> UINT;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct WINHELP {
    pub size: WORD,
    pub command: WORD,
    pub data: LONG,
    pub reserved: LONG,
    pub ofsFilename: WORD,
    pub ofsData: WORD,
}
pub type LPWINHELP = *mut WINHELP;

#[no_mangle]
pub unsafe extern "system" fn WinHelpA(
    hWnd: HWND,
    lpszHelp: LPCSTR,
    uCommand: UINT,
    dwData: DWORD_PTR,
) -> BOOL {
    static mut WM_WINHELP: WORD = 0;
    let hDest: HWND;
    let lpwh: LPWINHELP;
    let hwh: HGLOBAL;
    let size: i32;
    let dsize: i32;
    let nlen: i32;

    if WM_WINHELP == 0 {
        WM_WINHELP = RegisterWindowMessageA(b"WM_WINHELP\0".as_ptr() as LPCSTR) as WORD;
        if WM_WINHELP == 0 {
            return 0;
        }
    }

    let mut hDest_local = FindWindowA(b"MS_WINHELP\0".as_ptr() as LPCSTR, core::ptr::null_mut());
    if hDest_local.is_null() {
        if uCommand == HELP_QUIT {
            return 1;
        }
        if WinExec(b"winhlp32.exe -x\0".as_ptr() as LPCSTR, SW_SHOWNORMAL) < 32 {
            return 0;
        }
        hDest_local = FindWindowA(b"MS_WINHELP\0".as_ptr() as LPCSTR, core::ptr::null_mut());
        if hDest_local.is_null() {
            return 0;
        }
    }
    hDest = hDest_local;

    match uCommand {
        HELP_CONTEXT | HELP_SETCONTENTS | HELP_CONTENTS | HELP_CONTEXTPOPUP | HELP_FORCEFILE
        | HELP_HELPONHELP | HELP_FINDER | HELP_QUIT => {
            dsize = 0;
        }
        HELP_KEY | HELP_PARTIALKEY | HELP_COMMAND => {
            dsize = if dwData != 0 {
                (strlen(dwData as LPSTR) + 1) as i32
            } else {
                0
            };
        }
        HELP_MULTIKEY => {
            dsize = (*(dwData as LPMULTIKEYHELPA)).mkSize as i32;
        }
        HELP_SETWINPOS => {
            dsize = (*(dwData as LPHELPWININFOA)).wStructSize as i32;
        }
        _ => {
            return 0;
        }
    }

    if !lpszHelp.is_null() {
        nlen = (strlen(lpszHelp) + 1) as i32;
    } else {
        nlen = 0;
    }

    size = core::mem::size_of::<WINHELP>() as i32 + nlen + dsize;
    hwh = GlobalAlloc(0, (size) as SIZE_T);
    if hwh.is_null() {
        return 0;
    }
    lpwh = GlobalLock(hwh) as LPWINHELP;
    (*lpwh).size = size as WORD;
    (*lpwh).command = uCommand as WORD;
    (*lpwh).data = dwData as LONG;
    if nlen != 0 {
        strcpy(
            (lpwh as *mut u8).add(core::mem::size_of::<WINHELP>()) as LPSTR,
            lpszHelp,
        );
        (*lpwh).ofsFilename = core::mem::size_of::<WINHELP>() as WORD;
    } else {
        (*lpwh).ofsFilename = 0;
    }
    if dsize != 0 {
        memcpy(
            (lpwh as *mut u8).add(core::mem::size_of::<WINHELP>() + nlen as usize) as *mut core::ffi::c_void,
            dwData as *const core::ffi::c_void,
            dsize as usize,
        );
        (*lpwh).ofsData = (core::mem::size_of::<WINHELP>() + nlen as usize) as WORD;
    } else {
        (*lpwh).ofsData = 0;
    }
    GlobalUnlock(hwh);
    SendMessageA(
        hDest,
        WM_WINHELP as UINT,
        hWnd as WPARAM,
        hwh as LPARAM,
    ) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn WinHelpW(
    hWnd: HWND,
    lpszHelp: LPCWSTR,
    uCommand: UINT,
    dwData: DWORD_PTR,
) -> BOOL {
    let len: INT;
    let file: LPSTR;
    let mut ret: BOOL = 0;

    if lpszHelp.is_null() {
        return WinHelpA(hWnd, core::ptr::null_mut(), uCommand, dwData);
    }

    len = WideCharToMultiByte(
        CP_ACP,
        0,
        lpszHelp,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    file = HeapAlloc(GetProcessHeap(), 0, (len) as SIZE_T) as LPSTR;
    if !file.is_null() {
        WideCharToMultiByte(
            CP_ACP,
            0,
            lpszHelp,
            -1,
            file,
            len,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        ret = WinHelpA(hWnd, file, uCommand, dwData);
        HeapFree(GetProcessHeap(), 0, file as *mut _);
    }

    ret
}
