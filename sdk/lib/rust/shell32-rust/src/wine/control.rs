//! Control Panel applet loader — translation of dll/win32/shell32/wine/control.c.
//!
//! Loads `.cpl` modules via LoadLibrary + GetProcAddress("CPlApplet") and drives them
//! through the CPL message protocol (CPL_INIT / CPL_GETCOUNT / CPL_INQUIRE /
//! CPL_NEWINQUIRE / CPL_DBLCLK / CPL_STOP / CPL_EXIT). Translated for the ReactOS config
//! (`__REACTOS__`, `_WIN32_WINNT=0x600`, `UNICODE`/`_UNICODE`), so only the `#ifdef
//! __REACTOS__` branches are reproduced. `Control_DoWindow` is the ReactOS variant that
//! shells out to explorer.exe/filebrowser.exe.
//!
//! Exports (i386 decoration): WINAPI/`extern "system"` → `_Control_RunDLLA@16`,
//! `_Control_RunDLLW@16`, `_Control_FillCache_RunDLLA@16`, `_Control_FillCache_RunDLLW@16`,
//! `_CallCPLEntry16@24`, `_RunDll_CallEntry16@20`; cdecl/`extern "C"` (no `@N`) →
//! `_Control_LoadApplet`, `_Control_UnloadApplet`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::types::*;
// The crate's own functions: the definition is the declaration.
use crate::shell32::shlexec::ShellExecuteW;
// One declaration per external symbol: island.rs.
use crate::island::ActivateActCtx;
use crate::island::BringWindowToTop;
use crate::island::CoCreateInstance;
use crate::island::CreateActCtxW;
use crate::island::DeactivateActCtx;
use crate::island::EnumWindows;
use crate::island::FreeLibrary;
use crate::island::GetLastActivePopup;
use crate::island::GetParent;
use crate::island::GetProcAddress;
use crate::island::GetPropW;
use crate::island::GetShellWindow;
use crate::island::GetSystemMetrics;
use crate::island::HeapAlloc;
use crate::island::HeapReAlloc;
use crate::island::IsWindow;
use crate::island::LoadIconW;
use crate::island::LoadImageW;
use crate::island::LoadLibraryW;
use crate::island::LoadStringW;
use crate::island::RemovePropW;
use crate::island::SendMessageW;
use crate::island::SetPropW;
use crate::island::SetWindowTextW;
use crate::island::ShowWindow;
use core::mem::{offset_of, size_of, zeroed};
use core::ptr::{self, addr_of, addr_of_mut};

// ── types not in types.rs ─────────────────────────────────────────────────────────────
use crate::island::HMODULE;
use crate::island::HICON;
type ATOM = WORD; // u16
type WPARAM = UINT_PTR;
type LPARAM = LONG_PTR;
type LRESULT = LONG_PTR;
type USHORT = u16;
type LANGID = u16;
use ntdll_sys::nt::SIZE_T;

/// CPL applet entry point: `LONG APIENTRY (HWND, UINT, LPARAM, LPARAM)` (cpl.h).
/// APIENTRY == WINAPI == stdcall on i386 → `extern "system"`.
type AppletProc = unsafe extern "system" fn(HWND, UINT, LPARAM, LPARAM) -> LONG;

/// WNDENUMPROC for EnumWindows (winuser.h): `BOOL CALLBACK (HWND, LPARAM)`.
type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;

// ── constants (pinned from headers) ───────────────────────────────────────────────────
// cpl.h
const CPL_DYNAMIC_RES: INT = 0; // cpl.h: CPL_DYNAMIC_RES 0
const CPL_INIT: UINT = 1; // cpl.h: CPL_INIT 1
const CPL_GETCOUNT: UINT = 2; // cpl.h: CPL_GETCOUNT 2
const CPL_INQUIRE: UINT = 3; // cpl.h: CPL_INQUIRE 3
const CPL_DBLCLK: UINT = 5; // cpl.h: CPL_DBLCLK 5
const CPL_STOP: UINT = 6; // cpl.h: CPL_STOP 6
const CPL_EXIT: UINT = 7; // cpl.h: CPL_EXIT 7
const CPL_NEWINQUIRE: UINT = 8; // cpl.h: CPL_NEWINQUIRE 8
const CPL_STARTWPARMSW: UINT = 10; // cpl.h: CPL_STARTWPARMSW 10

// winbase.h / winnt.h
const HEAP_ZERO_MEMORY: DWORD = 0x00000008; // winnt.h
const ACTCTX_FLAG_RESOURCE_NAME_VALID: DWORD = 0x00000008; // winbase.h:534

// winnls.h
const CP_ACP: UINT = 0; // winnls.h: CP_ACP 0

// winuser.h
const SW_SHOWDEFAULT: INT = 10; // winuser.h: SW_SHOWDEFAULT 10
const SW_SHOWMINNOACTIVE: INT = 7; // winuser.h: SW_SHOWMINNOACTIVE 7
const WM_SETICON: UINT = 0x0080; // winuser.h: WM_SETICON 0x0080
const ICON_SMALL: WPARAM = 0; // winuser.h: ICON_SMALL 0
const ICON_BIG: WPARAM = 1; // winuser.h: ICON_BIG 1
const IMAGE_ICON: UINT = 1; // winuser.h: IMAGE_ICON 1
const SM_CXSMICON: INT = 49; // winuser.h: SM_CXSMICON 49
const SM_CYSMICON: INT = 50; // winuser.h: SM_CYSMICON 50

// objbase.h
const CLSCTX_INPROC_SERVER: DWORD = 0x1; // wtypesbase.h/objbase.h: CLSCTX_INPROC_SERVER 0x1

// winbase.h / handleapi.h: INVALID_HANDLE_VALUE = (HANDLE)-1
const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE;

// minwindef.h: MAX_PATH 260
const MAX_PATH: usize = 260;

// ── file-local structs (faithful layouts from headers) ────────────────────────────────

/// GUID (guiddef.h).
pub use crate::island::GUID;

/// CPLINFO (cpl.h, packed(1) via pshpack1.h).
#[repr(C, packed(1))]
#[derive(Copy, Clone)]
pub struct CPLINFO {
    pub idIcon: INT,
    pub idName: INT,
    pub idInfo: INT,
    pub lData: LONG_PTR,
}

/// NEWCPLINFOA (cpl.h, packed(1)).
#[repr(C, packed(1))]
#[derive(Copy, Clone)]
pub struct NEWCPLINFOA {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwHelpContext: DWORD,
    pub lData: LONG_PTR,
    pub hIcon: HICON,
    pub szName: [CHAR; 32],
    pub szInfo: [CHAR; 64],
    pub szHelpFile: [CHAR; 128],
}

/// NEWCPLINFOW (cpl.h, packed(1)).
#[repr(C, packed(1))]
#[derive(Copy, Clone)]
pub struct NEWCPLINFOW {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwHelpContext: DWORD,
    pub lData: LONG_PTR,
    pub hIcon: HICON,
    pub szName: [WCHAR; 32],
    pub szInfo: [WCHAR; 64],
    pub szHelpFile: [WCHAR; 128],
}

/// ACTCTXW (winbase.h:1016).
pub use crate::island::ACTCTXW;

/// `struct applet_info` (wine/cpanel.h, ReactOS layout — includes `idIcon`).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AppletInfo {
    pub data: LONG_PTR,
    pub idIcon: INT, // __REACTOS__
    pub icon: HICON,
    pub name: [WCHAR; 256],
    pub info: [WCHAR; 256],
    pub helpfile: [WCHAR; 128],
}

/// `CPlApplet` (wine/cpanel.h, ReactOS layout — `hActCtx` instead of list `entry`).
/// Trailing `info[1]` is a variable-length array (grown by HeapReAlloc).
#[repr(C)]
pub struct CPlApplet {
    pub hActCtx: HANDLE, // __REACTOS__
    pub hWnd: HWND,
    pub cmd: LPWSTR,
    pub count: UINT,
    pub hModule: HMODULE,
    pub proc_: Option<AppletProc>,
    pub info: [AppletInfo; 1],
}

/// CPanel (wine/cpanel.h, ReactOS: `typedef int CPanel`).
pub type CPanel = INT;

/// AppDlgFindData (control.c, ReactOS-only) — EnumWindows search context.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AppDlgFindData {
    pub szAppFile: LPCWSTR, // PCWSTR
    pub sAppletNo: UINT_PTR,
    pub aCPLName: ATOM,
    pub aCPLFlags: ATOM,
    pub hRunDLL: HWND,
    pub hDlgResult: HWND,
}

// ── imports (resolved from shell32's existing import libs at link) ─────────────────────
extern "system" {
    // kernel32
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    fn GetProcessHeap() -> HANDLE;
    fn GlobalAddAtomW(lpString: LPCWSTR) -> ATOM;
    fn GlobalDeleteAtom(nAtom: ATOM) -> ATOM;
    fn GlobalFindAtomW(lpString: LPCWSTR) -> ATOM;
    fn GlobalGetAtomNameW(nAtom: ATOM, lpBuffer: LPWSTR, nSize: INT) -> UINT;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
    fn lstrlenW(lpString: LPCWSTR) -> INT;
    // user32
    // shlwapi
    fn PathIsFileSpecW(pszPath: LPCWSTR) -> BOOL;
    fn StrChrW(pszStart: LPCWSTR, wMatch: WCHAR) -> LPWSTR;
    fn StrCmpIW(psz1: LPCWSTR, psz2: LPCWSTR) -> INT;
    // kernel32 (SearchPathW under UNICODE)
    fn SearchPathW(
        lpPath: LPCWSTR,
        lpFileName: LPCWSTR,
        lpExtension: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    // shell32 (ShellExecuteW)
    // ole32
    fn CoInitialize(pvReserved: LPVOID) -> HRESULT;
    fn CoUninitialize();
}

extern "C" {
    fn memmove(dest: PVOID, src: LPCVOID, n: SIZE_T) -> PVOID;
    fn memset(dest: PVOID, c: INT, n: SIZE_T) -> PVOID;
    fn wcscmp(s1: LPCWSTR, s2: LPCWSTR) -> INT;
    fn _wtoi(s: LPCWSTR) -> INT;
    // uuid lib data symbols (GUIDs referenced by address)
    static CLSID_TaskbarList: GUID;
    static IID_ITaskbarList: GUID;
}

// ── ITaskbarList COM interface (shobjidl.h) ───────────────────────────────────────────
#[repr(C)]
pub struct ITaskbarListVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut ITaskbarList, *const GUID, *mut LPVOID) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ITaskbarList) -> u32,
    pub Release: unsafe extern "system" fn(*mut ITaskbarList) -> u32,
    pub HrInit: unsafe extern "system" fn(*mut ITaskbarList) -> HRESULT,
    pub AddTab: unsafe extern "system" fn(*mut ITaskbarList, HWND) -> HRESULT,
    pub DeleteTab: unsafe extern "system" fn(*mut ITaskbarList, HWND) -> HRESULT,
    pub ActivateTab: unsafe extern "system" fn(*mut ITaskbarList, HWND) -> HRESULT,
    pub SetActiveAlt: unsafe extern "system" fn(*mut ITaskbarList, HWND) -> HRESULT,
}
#[repr(C)]
pub struct ITaskbarList {
    pub lpVtbl: *const ITaskbarListVtbl,
}

// ── wide string literals (oversized, null-terminated) ─────────────────────────────────
const fn wbuf<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < s.len() && i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}

static EMPTY_W: [WCHAR; 1] = [0];
static CPLAPPLET_A: [u8; 10] = *b"CPlApplet\0";
static CPLNAME_W: [WCHAR; 16] = wbuf(b"CPLName");
static CPLFLAGS_W: [WCHAR; 16] = wbuf(b"CPLFlags");
static OPEN_W: [WCHAR; 8] = wbuf(b"open");
static EXPLORER_W: [WCHAR; 16] = wbuf(b"explorer.exe");
static FILEBROWSER_W: [WCHAR; 16] = wbuf(b"filebrowser.exe");
// "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}\::{21EC2020-3AEA-1069-A2DD-08002B30309D}"
static CPL_NAV_W: [WCHAR; 96] =
    wbuf(b"::{20D04FE0-3AEA-1069-A2D8-08002B30309D}\\::{21EC2020-3AEA-1069-A2DD-08002B30309D}");

// ── small helpers ─────────────────────────────────────────────────────────────────────

/// MAKEINTRESOURCEW(i): truncate to WORD, widen to a pseudo-pointer (winuser.h).
#[inline]
fn make_int_resource(i: INT) -> LPCWSTR {
    (i as u16 as usize) as LPCWSTR
}

/// MAKEINTATOM(a): atom value as a pseudo-pointer string (winbase.h).
#[inline]
fn make_int_atom(a: ATOM) -> LPCWSTR {
    (a as usize) as LPCWSTR
}

/// Faithful StringCchCopyW (strsafe.h): copy up to cchDest-1 chars, null-terminate.
unsafe fn string_cch_copy_w(dest: *mut WCHAR, cch_dest: usize, src: *const WCHAR) {
    if cch_dest == 0 {
        return;
    }
    let mut i = 0usize;
    while i < cch_dest - 1 {
        let c = *src.add(i);
        if c == 0 {
            break;
        }
        *dest.add(i) = c;
        i += 1;
    }
    *dest.add(i) = 0;
}

/// Faithful StringCchCopyNW (strsafe.h): copy at most cchToCopy chars (or until NUL),
/// bounded by cchDest-1, null-terminate.
unsafe fn string_cch_copy_nw(dest: *mut WCHAR, cch_dest: usize, src: *const WCHAR, cch_to_copy: usize) {
    if cch_dest == 0 {
        return;
    }
    let max = if cch_to_copy < cch_dest - 1 { cch_to_copy } else { cch_dest - 1 };
    let mut i = 0usize;
    while i < max {
        let c = *src.add(i);
        if c == 0 {
            break;
        }
        *dest.add(i) = c;
        i += 1;
    }
    *dest.add(i) = 0;
}

#[inline]
unsafe fn info_at(applet: *mut CPlApplet, i: usize) -> *mut AppletInfo {
    (*applet).info.as_mut_ptr().add(i)
}

/// Invoke applet->proc faithfully; proc is non-null after a successful load.
#[inline]
unsafe fn call_proc(applet: *mut CPlApplet, hwnd: HWND, msg: UINT, l1: LPARAM, l2: LPARAM) -> LONG {
    match (*applet).proc_ {
        Some(p) => p(hwnd, msg, l1, l2),
        None => 0,
    }
}

// ── Control_UnloadApplet ──────────────────────────────────────────────────────────────
#[no_mangle]
pub unsafe extern "C" fn Control_UnloadApplet(applet: *mut CPlApplet) {
    let count = (*applet).count;
    let mut i: UINT = 0;
    while i < count {
        let data = (*info_at(applet, i as usize)).data;
        call_proc(applet, (*applet).hWnd, CPL_STOP, i as LPARAM, data);
        i += 1;
    }

    if (*applet).proc_.is_some() {
        call_proc(applet, (*applet).hWnd, CPL_EXIT, 0, 0);
    }
    FreeLibrary(((*applet).hModule).cast());
    HeapFree(GetProcessHeap(), 0, (*applet).cmd as LPVOID);
    HeapFree(GetProcessHeap(), 0, applet as LPVOID);
}

/// `theError:` cleanup path from Control_LoadApplet — frees and returns NULL.
unsafe fn load_applet_error(applet: *mut CPlApplet) -> *mut CPlApplet {
    FreeLibrary(((*applet).hModule).cast());
    HeapFree(GetProcessHeap(), 0, (*applet).cmd as LPVOID);
    HeapFree(GetProcessHeap(), 0, applet as LPVOID);
    ptr::null_mut()
}

// ── Control_LoadApplet ────────────────────────────────────────────────────────────────
#[no_mangle]
pub unsafe extern "C" fn Control_LoadApplet(hWnd: HWND, cmd: LPCWSTR, _panel: *mut CPanel) -> *mut CPlApplet {
    let mut act_ctx: ACTCTXW = zeroed();
    act_ctx.cbSize = size_of::<ACTCTXW>() as DWORD;
    act_ctx.dwFlags = ACTCTX_FLAG_RESOURCE_NAME_VALID;
    let mut cookie: ULONG_PTR = 0;
    let bActivated: BOOL;
    let mut file_buffer: [WCHAR; MAX_PATH] = [0; MAX_PATH];

    let mut applet: *mut CPlApplet =
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (size_of::<CPlApplet>()) as _) as *mut CPlApplet;
    if applet.is_null() {
        return applet;
    }

    let len: DWORD = ExpandEnvironmentStringsW(cmd, ptr::null_mut(), 0);
    if len > 0 {
        (*applet).cmd =
            HeapAlloc(GetProcessHeap(), 0, (((len + 1) as usize) * size_of::<WCHAR>()) as _) as LPWSTR;
        if (*applet).cmd.is_null() {
            return load_applet_error(applet);
        }
        ExpandEnvironmentStringsW(cmd, (*applet).cmd, len + 1);
    } else {
        return load_applet_error(applet);
    }

    (*applet).hWnd = hWnd;

    string_cch_copy_w(file_buffer.as_mut_ptr(), MAX_PATH, (*applet).cmd);
    if PathIsFileSpecW(file_buffer.as_ptr()) != FALSE {
        SearchPathW(
            ptr::null(),
            file_buffer.as_ptr(),
            ptr::null(),
            MAX_PATH as DWORD,
            file_buffer.as_mut_ptr(),
            ptr::null_mut(),
        );
    }
    act_ctx.lpSource = file_buffer.as_ptr();
    act_ctx.lpResourceName = 123usize as LPCWSTR; // (LPCWSTR)123
    (*applet).hActCtx = CreateActCtxW(&mut act_ctx);
    bActivated = if (*applet).hActCtx != INVALID_HANDLE_VALUE {
        ActivateActCtx((*applet).hActCtx, &mut cookie)
    } else {
        FALSE
    };

    (*applet).hModule = (LoadLibraryW((*applet).cmd)).cast();
    if (*applet).hModule.is_null() {
        return load_applet_error(applet);
    }
    let Some(proc_fn) = GetProcAddress(((*applet).hModule).cast(), CPLAPPLET_A.as_ptr() as LPCSTR) else {
        return load_applet_error(applet);
    };
    (*applet).proc_ = Some(core::mem::transmute::<_, AppletProc>(proc_fn));

    if call_proc(applet, hWnd, CPL_INIT, 0, 0) == 0 {
        return load_applet_error(applet);
    }
    (*applet).count = call_proc(applet, hWnd, CPL_GETCOUNT, 0, 0) as UINT;
    if (*applet).count == 0 {
        call_proc(applet, (*applet).hWnd, CPL_EXIT, 0, 0);
        return load_applet_error(applet);
    }

    // FIELD_OFFSET(CPlApplet, info[applet->count])
    let new_size = offset_of!(CPlApplet, info) + (*applet).count as usize * size_of::<AppletInfo>();
    applet = HeapReAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, applet as LPVOID, (new_size) as SIZE_T) as *mut CPlApplet;

    let count = (*applet).count;
    let mut i: UINT = 0;
    while i < count {
        let info_i = info_at(applet, i as usize);
        let mut newinfo: NEWCPLINFOW = zeroed();
        memset(addr_of_mut!(newinfo) as PVOID, 0, (size_of::<NEWCPLINFOW>()) as _);
        newinfo.dwSize = size_of::<NEWCPLINFOA>() as DWORD;
        (*info_i).helpfile[0] = 0;

        let mut info: CPLINFO = zeroed();
        call_proc(applet, hWnd, CPL_INQUIRE, i as LPARAM, addr_of_mut!(info) as LPARAM);
        (*info_i).data = info.lData;
        (*info_i).idIcon = info.idIcon; // __REACTOS__

        if info.idIcon != CPL_DYNAMIC_RES {
            (*info_i).icon = (LoadIconW(((*applet).hModule).cast(), make_int_resource(info.idIcon))).cast();
        }
        if info.idName != CPL_DYNAMIC_RES {
            LoadStringW(((*applet).hModule).cast(), info.idName as UINT, (*info_i).name.as_mut_ptr(), 256);
        }
        if info.idInfo != CPL_DYNAMIC_RES {
            LoadStringW(((*applet).hModule).cast(), info.idInfo as UINT, (*info_i).info.as_mut_ptr(), 256);
        }

        // introspection fallbacks (broken applets)
        if (*info_i).name[0] == 0 {
            info.idName = CPL_DYNAMIC_RES;
        }
        if (*info_i).info[0] == 0 {
            info.idInfo = CPL_DYNAMIC_RES;
        }
        if (*info_i).icon.is_null() {
            info.idIcon = CPL_DYNAMIC_RES;
        }

        if (info.idIcon == CPL_DYNAMIC_RES) || (info.idName == CPL_DYNAMIC_RES) || (info.idInfo == CPL_DYNAMIC_RES)
        {
            call_proc(applet, hWnd, CPL_NEWINQUIRE, i as LPARAM, addr_of_mut!(newinfo) as LPARAM);

            (*info_i).data = newinfo.lData;
            if info.idIcon == CPL_DYNAMIC_RES {
                (*info_i).icon = newinfo.hIcon;
            }
            if newinfo.dwSize == size_of::<NEWCPLINFOW>() as DWORD {
                if info.idName == CPL_DYNAMIC_RES {
                    // memcpy(name, newinfo.szName, sizeof(szName)) — 32 WCHAR
                    ptr::copy_nonoverlapping(addr_of!(newinfo.szName) as *const WCHAR, (*info_i).name.as_mut_ptr(), 32);
                }
                if info.idInfo == CPL_DYNAMIC_RES {
                    ptr::copy_nonoverlapping(addr_of!(newinfo.szInfo) as *const WCHAR, (*info_i).info.as_mut_ptr(), 64);
                }
                ptr::copy_nonoverlapping(
                    addr_of!(newinfo.szHelpFile) as *const WCHAR,
                    (*info_i).helpfile.as_mut_ptr(),
                    128,
                );
            } else {
                // reinterpret newinfo as NEWCPLINFOA
                let pa = addr_of!(newinfo) as *const NEWCPLINFOA;
                if info.idName == CPL_DYNAMIC_RES {
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        addr_of!((*pa).szName) as LPCSTR,
                        32, // sizeof(szName)/sizeof(CHAR)
                        (*info_i).name.as_mut_ptr(),
                        256,
                    );
                }
                if info.idInfo == CPL_DYNAMIC_RES {
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        addr_of!((*pa).szInfo) as LPCSTR,
                        64, // sizeof(szInfo)/sizeof(CHAR)
                        (*info_i).info.as_mut_ptr(),
                        256,
                    );
                }
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    addr_of!((*pa).szHelpFile) as LPCSTR,
                    128, // sizeof(szHelpFile)/sizeof(CHAR)
                    (*info_i).helpfile.as_mut_ptr(),
                    128,
                );
            }
        }
        i += 1;
    }

    if bActivated != FALSE {
        DeactivateActCtx(0, cookie);
    }

    applet
}

// ── Control_DoWindow (ReactOS variant) ────────────────────────────────────────────────
unsafe fn Control_DoWindow(_panel: *mut CPanel, _hWnd: HWND, _hInst: HINSTANCE) {
    // NOTE: If Explorer shell is not available, use ReactOS's alternative file browser.
    let file: LPCWSTR = if !GetShellWindow().is_null() {
        EXPLORER_W.as_ptr()
    } else {
        FILEBROWSER_W.as_ptr()
    };
    ShellExecuteW(
        ptr::null_mut(),
        OPEN_W.as_ptr(),
        file,
        CPL_NAV_W.as_ptr(),
        ptr::null(),
        SW_SHOWDEFAULT,
    );
}

// ── Control_EnumWinProc (ReactOS-only) ────────────────────────────────────────────────
unsafe extern "system" fn Control_EnumWinProc(hwnd: HWND, lParam: LPARAM) -> BOOL {
    let pData = lParam as *mut AppDlgFindData;
    let mut szAppFile: [WCHAR; MAX_PATH] = [0; MAX_PATH];

    if (*pData).hRunDLL == hwnd {
        return TRUE; // Skip self instance
    }

    let sAppletNo = GetPropW(hwnd, make_int_atom((*pData).aCPLFlags)) as UINT_PTR;
    if sAppletNo != (*pData).sAppletNo {
        return TRUE; // Continue enumeration
    }

    let hRes = GetPropW(hwnd, make_int_atom((*pData).aCPLName));
    GlobalGetAtomNameW(
        (hRes as usize as u32) as ATOM, // (ATOM)HandleToUlong(hRes)
        szAppFile.as_mut_ptr(),
        MAX_PATH as INT,
    );
    if wcscmp(szAppFile.as_ptr(), (*pData).szAppFile) == 0 {
        let hDialog = GetLastActivePopup(hwnd);
        if IsWindow(hDialog) != FALSE {
            (*pData).hDlgResult = hDialog;
            return FALSE; // Stop enumeration
        }
    }

    TRUE // Continue enumeration
}

// ── Control_ShowAppletInTaskbar (ReactOS-only) ────────────────────────────────────────
unsafe fn Control_ShowAppletInTaskbar(applet: *mut CPlApplet, index: UINT) {
    let mut pTaskbar: *mut ITaskbarList = ptr::null_mut();
    let info_i = info_at(applet, index as usize);

    // Only add a taskbar button if the applet's parent window is the desktop.
    if !GetParent((*applet).hWnd).is_null() {
        return;
    }

    SetWindowTextW((*applet).hWnd, (*info_i).name.as_ptr());

    // Large icon for the taskbar button.
    if !(*info_i).icon.is_null() {
        SendMessageW((*applet).hWnd, WM_SETICON, ICON_BIG, (*info_i).icon as LPARAM);
    }

    // Try loading the small icon for the taskbar button.
    let hSmallIcon: HICON = LoadImageW(
        ((*applet).hModule).cast(),
        make_int_resource((*info_i).idIcon),
        IMAGE_ICON,
        GetSystemMetrics(SM_CXSMICON),
        GetSystemMetrics(SM_CYSMICON),
        0,
    ) as HICON;
    if !hSmallIcon.is_null() {
        SendMessageW((*applet).hWnd, WM_SETICON, ICON_SMALL, hSmallIcon as LPARAM);
    } else if !(*info_i).icon.is_null() {
        SendMessageW((*applet).hWnd, WM_SETICON, ICON_SMALL, (*info_i).icon as LPARAM);
    }

    // Add button to the taskbar.
    ShowWindow((*applet).hWnd, SW_SHOWMINNOACTIVE);

    // Activate the corresponding button in the taskbar.
    CoInitialize(ptr::null_mut());
    if CoCreateInstance(
        &CLSID_TaskbarList,
        ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_ITaskbarList,
        addr_of_mut!(pTaskbar) as *mut LPVOID,
    ) == S_OK
    {
        let vtbl = (*pTaskbar).lpVtbl;
        if ((*vtbl).HrInit)(pTaskbar) == S_OK {
            ((*vtbl).ActivateTab)(pTaskbar, (*applet).hWnd);
        }
        ((*vtbl).Release)(pTaskbar);
    }
    CoUninitialize();
}

// ── Control_DoLaunch (ReactOS variant) ────────────────────────────────────────────────
unsafe fn Control_DoLaunch(panel: *mut CPanel, hWnd: HWND, wszCmd: LPCWSTR) {
    let mut sp: INT = -1;
    let mut extraPmts: LPCWSTR = EMPTY_W.as_ptr();
    let mut quoted: BOOL = FALSE;
    let mut pchFirstComma: LPCWSTR = ptr::null();
    let mut pchSecondComma: LPCWSTR = ptr::null();
    let mut pchLastUnquotedSpace: LPCWSTR = ptr::null();
    let nLen: SIZE_T = lstrlenW(wszCmd) as SIZE_T;

    let buffer: LPWSTR =
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (size_of::<WCHAR>() * ((nLen + 1)) as usize) as _) as LPWSTR;
    let wszDialogBoxName: LPWSTR =
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (size_of::<WCHAR>() * ((nLen + 1)) as usize) as _) as LPWSTR;
    if wszDialogBoxName.is_null() || buffer.is_null() {
        if !buffer.is_null() {
            HeapFree(GetProcessHeap(), 0, buffer as LPVOID);
        }
        if !wszDialogBoxName.is_null() {
            HeapFree(GetProcessHeap(), 0, wszDialogBoxName as LPVOID);
        }
        return;
    }

    // Search for unquoted commas and spaces.
    let mut i: SIZE_T = 0;
    while i < nLen {
        let ch = *wszCmd.add((i) as _);
        if quoted != FALSE && ch != b'"' as WCHAR {
            i += 1;
            continue;
        }
        if ch == b'"' as WCHAR {
            quoted = (quoted == FALSE) as BOOL;
        } else if ch == b',' as WCHAR {
            if pchFirstComma.is_null() {
                pchFirstComma = wszCmd.add((i) as _);
            } else if pchSecondComma.is_null() {
                pchSecondComma = wszCmd.add((i) as _);
            }
        } else if ch == b' ' as WCHAR {
            if pchFirstComma.is_null() {
                pchFirstComma = wszCmd.add((i) as _);
            } else {
                pchLastUnquotedSpace = wszCmd.add((i) as _);
            }
        }
        i += 1;
    }

    if pchFirstComma.is_null() {
        // No unquoted comma: either space-separated params, or the whole string is the path.
        if !pchLastUnquotedSpace.is_null() {
            let mut nSpaces: usize = 0;
            while *pchLastUnquotedSpace.add(nSpaces) == b' ' as WCHAR {
                nSpaces += 1;
            }
            string_cch_copy_nw(buffer, (nLen + 1) as _, wszCmd, pchLastUnquotedSpace.offset_from(wszCmd) as usize);
            string_cch_copy_w(wszDialogBoxName, (nLen + 1) as _, pchLastUnquotedSpace.add(nSpaces));
        } else {
            string_cch_copy_w(buffer, (nLen + 1) as _, wszCmd);
        }
    } else {
        if pchSecondComma.is_null() {
            pchSecondComma = wszCmd.add((nLen) as _);
        }
        string_cch_copy_nw(buffer, (nLen + 1) as _, wszCmd, pchFirstComma.offset_from(wszCmd) as usize);
        string_cch_copy_nw(
            wszDialogBoxName,
            (nLen + 1) as _,
            pchFirstComma.add(1),
            (pchSecondComma.offset_from(pchFirstComma) - 1) as usize,
        );
        if pchSecondComma != wszCmd.add((nLen) as _) {
            extraPmts = pchSecondComma.add(1);
        }
    }

    // Remove the quotes from both buffers.
    loop {
        let ptr = StrChrW(buffer, b'"' as WCHAR);
        if ptr.is_null() {
            break;
        }
        memmove(ptr as PVOID, ptr.add(1) as LPCVOID, (lstrlenW(ptr) as usize * size_of::<WCHAR>()) as SIZE_T);
    }
    loop {
        let ptr = StrChrW(wszDialogBoxName, b'"' as WCHAR);
        if ptr.is_null() {
            break;
        }
        memmove(ptr as PVOID, ptr.add(1) as LPCVOID, (lstrlenW(ptr) as usize * size_of::<WCHAR>()) as SIZE_T);
    }

    if *wszDialogBoxName == b'@' as WCHAR {
        sp = _wtoi(wszDialogBoxName.add(1));
    }

    let applet = Control_LoadApplet(hWnd, buffer, panel);
    if !applet.is_null() {
        let mut cookie: ULONG_PTR = 0;
        let count = (*applet).count;

        // we've been given a textual parameter (or none at all)
        if sp == -1 {
            loop {
                sp += 1;
                if sp as UINT == count {
                    break;
                }
                if StrCmpIW(wszDialogBoxName, (*info_at(applet, sp as usize)).name.as_ptr()) == 0 {
                    break;
                }
            }
        }

        let first = *wszDialogBoxName;
        if count == 1 || (sp as UINT >= count && (first == 0 || first == b'@' as WCHAR)) {
            sp = 0;
        }

        let bActivated: BOOL = if (*applet).hActCtx != INVALID_HANDLE_VALUE {
            ActivateActCtx((*applet).hActCtx, &mut cookie)
        } else {
            FALSE
        };

        if (sp as UINT) < count {
            let mut aCPLPath = GlobalFindAtomW((*applet).cmd);
            if aCPLPath == 0 {
                aCPLPath = GlobalAddAtomW((*applet).cmd);
            }

            let mut aCPLName = GlobalFindAtomW(CPLNAME_W.as_ptr());
            if aCPLName == 0 {
                aCPLName = GlobalAddAtomW(CPLNAME_W.as_ptr());
            }

            let mut aCPLFlags = GlobalFindAtomW(CPLFLAGS_W.as_ptr());
            if aCPLFlags == 0 {
                aCPLFlags = GlobalAddAtomW(CPLFLAGS_W.as_ptr());
            }

            let mut findData: AppDlgFindData = zeroed();
            findData.szAppFile = (*applet).cmd;
            findData.sAppletNo = (sp + 1) as UINT_PTR;
            findData.aCPLName = aCPLName;
            findData.aCPLFlags = aCPLFlags;
            findData.hRunDLL = (*applet).hWnd;
            findData.hDlgResult = ptr::null_mut();
            // Find the dialog of this applet in the first instance.
            EnumWindows(Some(Control_EnumWinProc), addr_of_mut!(findData) as LPARAM);
            if !findData.hDlgResult.is_null() {
                BringWindowToTop(findData.hDlgResult);
            } else {
                SetPropW(
                    (*applet).hWnd,
                    make_int_atom(aCPLName),
                    make_int_atom(aCPLPath) as HANDLE,
                );
                SetPropW(
                    (*applet).hWnd,
                    make_int_atom(aCPLFlags),
                    ((sp + 1) as u32 as usize) as HANDLE, // UlongToHandle(sp + 1)
                );
                Control_ShowAppletInTaskbar(applet, sp as UINT);

                if *extraPmts == 0
                    || call_proc(applet, (*applet).hWnd, CPL_STARTWPARMSW, sp as LPARAM, extraPmts as LPARAM) == 0
                {
                    let data = (*info_at(applet, sp as usize)).data;
                    call_proc(applet, (*applet).hWnd, CPL_DBLCLK, sp as LPARAM, data);
                }

                RemovePropW((*applet).hWnd, (*applet).cmd);
                GlobalDeleteAtom(aCPLPath);
            }
        }

        Control_UnloadApplet(applet);

        if bActivated != FALSE {
            DeactivateActCtx(0, cookie);
        }
    }

    HeapFree(GetProcessHeap(), 0, buffer as LPVOID);
    HeapFree(GetProcessHeap(), 0, wszDialogBoxName as LPVOID);
}

// ── Control_RunDLLW ───────────────────────────────────────────────────────────────────
#[no_mangle]
pub unsafe extern "system" fn Control_RunDLLW(hWnd: HWND, hInst: HINSTANCE, cmd: LPCWSTR, _nCmdShow: DWORD) {
    let mut panel: CPanel = 0; // ReactOS: CPanel is `int`, left uninitialized (unused)

    if cmd.is_null() || *cmd == 0 {
        Control_DoWindow(&mut panel, hWnd, hInst);
    } else {
        Control_DoLaunch(&mut panel, hWnd, cmd);
    }
}

// ── Control_RunDLLA ───────────────────────────────────────────────────────────────────
#[no_mangle]
pub unsafe extern "system" fn Control_RunDLLA(hWnd: HWND, hInst: HINSTANCE, cmd: LPCSTR, nCmdShow: DWORD) {
    let len: DWORD = MultiByteToWideChar(CP_ACP, 0, cmd, -1, ptr::null_mut(), 0) as DWORD;
    let wszCmd: LPWSTR = HeapAlloc(GetProcessHeap(), 0, (len as usize * size_of::<WCHAR>()) as _) as LPWSTR;
    if !wszCmd.is_null() && MultiByteToWideChar(CP_ACP, 0, cmd, -1, wszCmd, len as INT) != 0 {
        Control_RunDLLW(hWnd, hInst, wszCmd, nCmdShow);
    }
    HeapFree(GetProcessHeap(), 0, wszCmd as LPVOID);
}

// ── Control_FillCache_RunDLLW / A (stubs) ─────────────────────────────────────────────
#[no_mangle]
pub unsafe extern "system" fn Control_FillCache_RunDLLW(
    _hWnd: HWND,
    _hModule: HANDLE,
    _w: DWORD,
    _x: DWORD,
) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn Control_FillCache_RunDLLA(
    hWnd: HWND,
    hModule: HANDLE,
    w: DWORD,
    x: DWORD,
) -> HRESULT {
    Control_FillCache_RunDLLW(hWnd, hModule, w, x)
}

// ── RunDll_CallEntry16 (ReactOS, [SHELL32.122]) — 16-bit thunk stub ───────────────────
// C body is a FIXME-only stub returning void; preserved as a no-op.
#[no_mangle]
pub unsafe extern "system" fn RunDll_CallEntry16(
    _proc: DWORD,
    _hwnd: HWND,
    _inst: HINSTANCE,
    _cmdline: LPCSTR,
    _cmdshow: INT,
) {
}

// ── CallCPLEntry16 (ReactOS, [SHELL32.166]) — 16-bit thunk stub ───────────────────────
// C body is a FIXME-only stub returning 0x0deadbee (ReactOS: DECLARE_HANDLE(FARPROC16),
// LRESULT WINAPI(HINSTANCE, FARPROC16, HWND, UINT, LPARAM, LPARAM)).
#[no_mangle]
pub unsafe extern "system" fn CallCPLEntry16(
    _hMod: HINSTANCE,
    _pFunc: HANDLE, // FARPROC16 = DECLARE_HANDLE
    _dw3: HWND,
    _dw4: UINT,
    _dw5: LPARAM,
    _dw6: LPARAM,
) -> LRESULT {
    0x0deadbee // verbatim ReactOS stub return value
}

// ADDED-TO-TYPES?: CPLINFO, NEWCPLINFOA, NEWCPLINFOW, GUID, ACTCTXW (common SDK structs —
//   candidates for hoisting into types.rs). CPlApplet / AppletInfo / AppDlgFindData /
//   ITaskbarList(+Vtbl) / CPanel are shell32-internal and stay file-local. The `*16`
//   thunk entries (CallCPLEntry16 / RunDll_CallEntry16) are faithful FIXME stubs in the
//   C source (return 0x0deadbee / void) — NOT routed through any real 16-bit thunk; no
//   thunk machinery added.
