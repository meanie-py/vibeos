//! windows/window.c → Rust. user32 window-management client.
//!
//! Faithful port of win32ss/user/user32/windows/window.c. Cohab-linked into
//! user32.dll via libuser32_rust.a. Substrate:
//!   - `win32k_sys::win32k::*`  — all NtUser* syscalls, Win32 types, win32k structs.
//!   - `crate::ntwrapper::*`    — NtUserx* inline wrappers.
//!   - `crate::internal::*`     — shared ntdll Rtl*/Nt* + cohab ValidateHwnd family.
//!
//! The remaining user32-internal C helpers referenced by window.c (and which are
//! NOT in internal.rs) are declared as local externs with the ABI from
//! user32/include/{user32p.h,regcontrol.h} or reproduced inline when they are
//! macros / static-inlines with no exported symbol.

#![allow(non_snake_case)]

use crate::internal::*;
// The crate's own functions: the definition is the declaration.
use crate::misc::rtlstr::RtlInitLargeAnsiString;
use crate::misc::rtlstr::RtlInitLargeUnicodeString;
use crate::ntwrapper::*;
use crate::windows::class::GetWindowLongPtrW;
use win32k_sys::win32k::*;

// ───────────────────────────────────────────────────────────────────────────
// Local consts/macros not present in win32k-sys.
// ───────────────────────────────────────────────────────────────────────────

// wingdi.h: #define ERROR 0  (region-error sentinel; used as a value, not a type)
const ERROR: core::ffi::c_int = 0;
// winuser.h: #define CW_USEDEFAULT ((int)0x80000000)
const CW_USEDEFAULT: core::ffi::c_int = 0x80000000u32 as core::ffi::c_int;
// undocuser.h: #define CW_USEDEFAULT16 ((short)0x8000)
const CW_USEDEFAULT16: core::ffi::c_int = 0xffff8000u32 as core::ffi::c_int; // (short)0x8000 sign-extended
// winnls.h
const CP_ACP: UINT = 0;
// ntstatus.h
const STATUS_SUCCESS: i32 = 0;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004u32 as i32;

// IS_ATOM(x): (((ULONG_PTR)(x) > 0x0) && ((ULONG_PTR)(x) < 0x10000))  (user32p.h)
// NB: the upper bound is 0x10000 (high 48 bits clear), NOT MAXINTATOM (0xC000).
// Registered-class atoms start at MAXINTATOM (0xC000), so a 0xC000 bound here
// misclassifies every real class atom as a string pointer -> AV in RtlInitUnicodeString.
#[inline]
unsafe fn IS_ATOM<T>(x: *const T) -> bool {
    let v = x as usize;
    v > 0 && v < 0x10000
}

// IS_INTRESOURCE(_r): ((((ULONG_PTR)(_r)) >> 16) == 0)
#[inline]
unsafe fn IS_INTRESOURCE<T>(r: *const T) -> bool {
    ((r as usize) >> 16) == 0
}

// UlongToHandle(ul): ((HANDLE)(ULONG_PTR)(ul))
#[inline]
fn UlongToHandle(ul: ULONG) -> HANDLE {
    ul as usize as HANDLE
}

// NT_SUCCESS(Status): ((NTSTATUS)(Status) >= 0)
#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// RtlZeroMemory(dst, len)
#[inline]
unsafe fn RtlZeroMemory(dst: *mut core::ffi::c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// WINE_DEFAULT_DEBUG_CHANNEL trace macros — no-ops in this build (DUMMY build,
// the C versions resolve to DbgPrint; the bodies are not load-bearing).

// ───────────────────────────────────────────────────────────────────────────
// gdi32 / kernel32 / user32 public exports (stdcall → extern "system" on i386).
// ───────────────────────────────────────────────────────────────────────────
extern "system" {
    // kernel32.dll
    fn SetLastError(dwErrCode: DWORD);
    fn GetLastError() -> DWORD;
    fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    fn GetModuleFileNameA(hModule: HMODULE, lpFilename: LPSTR, nSize: DWORD) -> DWORD;
    fn GetModuleFileNameW(hModule: HMODULE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD;
    fn GetCurrentThreadId() -> DWORD;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: core::ffi::c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: core::ffi::c_int,
    ) -> core::ffi::c_int;

    // user32.dll (other modules / public API forwarders)
    // SetActiveWindow / ShowWindow are EXTINLINE ntwrapper.h wrappers (no exported
    // symbol — they inline to the NtUser* syscall); use the crate::ntwrapper::*
    // inline versions (imported above) rather than an extern decl.
    // GetWindowLongPtrW is the i386 winuser.h macro for GetWindowLongW; provided as
    // an inline in windows::class (imported below).
    fn GetActiveWindow() -> HWND;
    fn GetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int;
    fn OffsetRect(lprc: LPRECT, dx: core::ffi::c_int, dy: core::ffi::c_int) -> BOOL;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetClassInfoExA(hInstance: HINSTANCE, lpszClass: LPCSTR, lpwcx: LPWNDCLASSEXA) -> BOOL;
    fn GetClassInfoExW(hInstance: HINSTANCE, lpszClass: LPCWSTR, lpwcx: LPWNDCLASSEXW) -> BOOL;
    fn LoadMenuA(hInstance: HINSTANCE, lpMenuName: LPCSTR) -> HMENU;
    fn LoadMenuW(hInstance: HINSTANCE, lpMenuName: LPCWSTR) -> HMENU;

    // ntdll.dll — NTAPI (stdcall on i386). RtlInitLarge* live in ntdll.
}

// ───────────────────────────────────────────────────────────────────────────
// user32-internal C symbols still in the cohab C objects of this DLL.
//   FASTCALL → extern "fastcall" on i386 (`@Name@N`); cdecl/WINAPI → extern "C"
//   / "system" per the header.  (ValidateHwnd family comes from crate::internal.)
// ───────────────────────────────────────────────────────────────────────────
#[cfg(target_arch = "x86")]
extern "fastcall" {
    // user32p.h
    fn GetThreadDesktopWnd() -> PWND;                 // PWND FASTCALL GetThreadDesktopWnd(VOID)
    fn TestWindowProcess(pwnd: PWND) -> BOOL;         // BOOL FASTCALL TestWindowProcess(PWND)
    // window.c (file-local extern): LPCWSTR FASTCALL ClassNameToVersion(...)
    fn ClassNameToVersion(
        lpszClass: *const core::ffi::c_void,
        lpszMenuName: LPCWSTR,
        plpLibFileName: *mut LPCWSTR,
        pContext: *mut HANDLE,
        bAnsi: BOOL,
    ) -> LPCWSTR;
    // regcontrol.h: BOOL FASTCALL VersionRegisterClass(PCWSTR,LPCWSTR,HANDLE,HMODULE *)
    fn VersionRegisterClass(
        pszClass: PCWSTR,
        lpLibFileName: LPCWSTR,
        pContext: HANDLE,
        phLibModule: *mut HMODULE,
    ) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    // user32p.h
    fn GetThreadDesktopWnd() -> PWND;                 // PWND FASTCALL GetThreadDesktopWnd(VOID)
    fn TestWindowProcess(pwnd: PWND) -> BOOL;         // BOOL FASTCALL TestWindowProcess(PWND)
    // window.c (file-local extern): LPCWSTR FASTCALL ClassNameToVersion(...)
    fn ClassNameToVersion(
        lpszClass: *const core::ffi::c_void,
        lpszMenuName: LPCWSTR,
        plpLibFileName: *mut LPCWSTR,
        pContext: *mut HANDLE,
        bAnsi: BOOL,
    ) -> LPCWSTR;
    // regcontrol.h: BOOL FASTCALL VersionRegisterClass(PCWSTR,LPCWSTR,HANDLE,HMODULE *)
    fn VersionRegisterClass(
        pszClass: PCWSTR,
        lpLibFileName: LPCWSTR,
        pContext: HANDLE,
        phLibModule: *mut HMODULE,
    ) -> BOOL;
}

// nonclient.rs defines UserGetWindowBorders as plain cdecl (`extern "C"`, no
// @N) — the decl MUST match or LLVM UB-folds the calling branch on i386 (the
// silent MessageInit/U32AccelCacheLock class; field manual §9.30).
extern "C" {
    fn UserGetWindowBorders(Style: DWORD, ExStyle: DWORD, Size: *mut SIZE, WithClient: BOOL);
}

extern "system" {
    // user32p.h: VOID WINAPI UserSetLastError(IN DWORD dwErrCode)
    fn UserSetLastError(dwErrCode: DWORD);
    // regcontrol.h: BOOL WINAPI RegisterSystemControls(VOID)
    fn RegisterSystemControls() -> BOOL;
    // regcontrol.h: extern DWORD RegisterDefaultClasses;
    static RegisterDefaultClasses: DWORD;
}

// These two are plain __cdecl in mdi.c (no WINAPI/FASTCALL macro) → extern "C"
// (`_Name`, no `@N`).
extern "C" {
    // user32p.h: HWND* WIN_ListChildren(HWND hWndparent)
    fn WIN_ListChildren(hwnd: HWND) -> *mut HWND;
    // windows/mdi.c: void MDI_CalcDefaultChildPos(HWND, INT, LPPOINT, INT, UINT*)
    fn MDI_CalcDefaultChildPos(
        hwndClient: HWND,
        total: INT,
        lpPos: LPPOINT,
        delta: INT,
        id: *mut UINT,
    );
}

extern "C" {
    // misc/misc.c: PTHREADINFO GetW32ThreadInfo(VOID) — plain cdecl
    fn GetW32ThreadInfo() -> PTHREADINFO;
    // win32ss/user/rtl/image.c: ULONG RtlGetExpWinVer(PVOID BaseAddress) — plain cdecl
    fn RtlGetExpWinVer(BaseAddress: PVOID) -> ULONG;
}

// ───────────────────────────────────────────────────────────────────────────
// Inlined macros / static-inlines from user32 headers (no exported symbol).
// ───────────────────────────────────────────────────────────────────────────

// ntuser.h: #define UserHMGetHandle(obj) ((obj)->head.h)
// `head` is a bindgen-flattened THRDESKHEAD: padding[0] == HEAD.h (offset 0).
#[inline]
unsafe fn UserHMGetHandle(wnd: PWND) -> HWND {
    (*(wnd as *const _HEAD)).h as HWND
}

// ntuser.h: #define GetWin32ClientInfo() ((PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo))
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO
}

// user_x.h: static __inline PVOID DesktopPtrToUser(PVOID Ptr)
#[inline]
unsafe fn DesktopPtrToUser(ptr: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(ptr)
}

// ntwrapper.h: EXTINLINE PVOID NtUserxGetDesktopMapping(PVOID ptr)
#[inline]
unsafe fn NtUserxGetDesktopMapping(ptr: PVOID) -> PVOID {
    NtUserCallOneParam(ptr as DWORD_PTR, ONEPARAM_ROUTINE_GETDESKTOPMAPPING as DWORD) as PVOID
}

/// `head.pti` reader (THROBJHEAD.pti at offset 8 of the flattened THRDESKHEAD).
#[inline]
unsafe fn WndHeadPti(wnd: PWND) -> PTHREADINFO {
    (*(wnd as *const _THROBJHEAD)).pti
}

// window.c FORCEINLINE: RtlInitLargeString
#[inline]
unsafe fn RtlInitLargeString(plstr: PLARGE_STRING, psz: LPCVOID, bUnicode: BOOL) {
    if bUnicode != 0 {
        RtlInitLargeUnicodeString(plstr as PLARGE_UNICODE_STRING, psz as PWSTR, 0);
    } else {
        RtlInitLargeAnsiString(plstr as PLARGE_ANSI_STRING, psz as PSTR, 0);
    }
}

// window.c: VOID NTAPI RtlFreeLargeString(PLARGE_STRING LargeString)
unsafe extern "system" fn RtlFreeLargeString(LargeString: PLARGE_STRING) {
    if !(*LargeString).Buffer.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, (*LargeString).Buffer);
        RtlZeroMemory(
            LargeString as *mut core::ffi::c_void,
            core::mem::size_of::<LARGE_STRING>(),
        );
    }
}

/* FUNCTIONS *****************************************************************/

// NTSTATUS WINAPI User32CallSendAsyncProcForKernel(PVOID, ULONG)
// On i386 WINAPI == stdcall; elsewhere cdecl. Kept as cfg pair (cross-TU,
// callback dispatch from kernel).
#[no_mangle]
pub unsafe extern "system" fn User32CallSendAsyncProcForKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let CallbackArgs: PSENDASYNCPROC_CALLBACK_ARGUMENTS =
        Arguments as PSENDASYNCPROC_CALLBACK_ARGUMENTS;
    TRACE!("User32CallSendAsyncProcKernel()\n");
    if ArgumentLength as usize != core::mem::size_of::<SENDASYNCPROC_CALLBACK_ARGUMENTS>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    if let Some(Callback) = (*CallbackArgs).Callback {
        Callback(
            (*CallbackArgs).Wnd,
            (*CallbackArgs).Msg,
            (*CallbackArgs).Context,
            (*CallbackArgs).Result,
        );
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn AllowSetForegroundWindow(dwProcessId: DWORD) -> BOOL {
    NtUserxAllowSetForegroundWindow(dwProcessId)
}

#[no_mangle]
pub unsafe extern "system" fn BeginDeferWindowPos(nNumWindows: core::ffi::c_int) -> HDWP {
    NtUserxBeginDeferWindowPos(nNumWindows)
}

#[no_mangle]
pub unsafe extern "system" fn BringWindowToTop(hWnd: HWND) -> BOOL {
    // HWND_TOP == (HWND)0
    NtUserSetWindowPos(
        hWnd,
        0 as HWND,
        0,
        0,
        0,
        0,
        (SWP_NOSIZE | SWP_NOMOVE) as UINT,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SwitchToThisWindow(hwnd: HWND, fAltTab: BOOL) {
    NtUserxSwitchToThisWindow(hwnd, fAltTab);
}

#[no_mangle]
pub unsafe extern "system" fn ChildWindowFromPoint(hWndParent: HWND, Point: POINT) -> HWND {
    NtUserChildWindowFromPointEx(hWndParent, Point.x, Point.y, 0) as HWND
}

#[no_mangle]
pub unsafe extern "system" fn ChildWindowFromPointEx(
    hwndParent: HWND,
    pt: POINT,
    uFlags: UINT,
) -> HWND {
    NtUserChildWindowFromPointEx(hwndParent, pt.x, pt.y, uFlags) as HWND
}

#[no_mangle]
pub unsafe extern "system" fn CloseWindow(hWnd: HWND) -> BOOL {
    // NOTE: CloseWindow minimizes, it doesn't close.
    SetActiveWindow(hWnd);
    ShowWindow(hWnd, SW_SHOWMINIMIZED as core::ffi::c_int)
}

#[no_mangle]
pub unsafe extern "system" fn User32CreateWindowEx(
    mut dwExStyle: DWORD,
    lpClassName: LPCSTR,
    lpWindowName: LPCSTR,
    dwStyle: DWORD,
    x: core::ffi::c_int,
    y: core::ffi::c_int,
    nWidth: core::ffi::c_int,
    nHeight: core::ffi::c_int,
    hWndParent: HWND,
    mut hMenu: HMENU,
    hInstance: HINSTANCE,
    lpParam: LPVOID,
    dwFlags: DWORD,
) -> HWND {
    let mut WindowName: LARGE_STRING = core::mem::zeroed();
    let mut lstrClassName: LARGE_STRING = core::mem::zeroed();
    let plstrClassName: *mut LARGE_STRING;
    let mut lstrClassVersion: LARGE_STRING = core::mem::zeroed();
    let plstrClassVersion: *mut LARGE_STRING;
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut ClassVersion: _UNICODE_STRING = core::mem::zeroed();
    let mut wceA: WNDCLASSEXA = core::mem::zeroed();
    let mut wceW: WNDCLASSEXW = core::mem::zeroed();
    let mut hLibModule: HMODULE = core::ptr::null_mut();
    let mut dwLastError: DWORD;
    let Unicode: BOOL;
    let mut ClassFound: BOOL = 0;
    let mut Handle: HWND = core::ptr::null_mut();
    let lpszClsVersion: LPCWSTR;
    let mut lpLibFileName: LPCWSTR = core::ptr::null_mut();
    let mut pCtx: HANDLE = core::ptr::null_mut();
    let dwFlagsVer: DWORD;

    dwFlagsVer = RtlGetExpWinVer(if !hInstance.is_null() {
        hInstance as PVOID
    } else {
        GetModuleHandleW(core::ptr::null_mut()) as PVOID
    });
    TRACE!("Module Version %x\n", dwFlagsVer);

    if RegisterDefaultClasses == 0 {
        TRACE!("RegisterSystemControls\n");
        RegisterSystemControls();
    }

    Unicode = (dwFlags & NUCWE_ANSI == 0) as BOOL;

    if IS_ATOM(lpClassName) {
        plstrClassName = lpClassName as *mut core::ffi::c_void as *mut LARGE_STRING;
    } else {
        if Unicode != 0 {
            RtlInitUnicodeString(&mut ClassName, lpClassName as PCWSTR);
        } else if RtlCreateUnicodeStringFromAsciiz(&mut ClassName, lpClassName as PCSZ) == 0 {
            SetLastError(ERROR_OUTOFMEMORY);
            return core::ptr::null_mut();
        }

        // Copy it to a LARGE_STRING.
        lstrClassName.Buffer = ClassName.Buffer as PVOID;
        lstrClassName.Length = ClassName.Length as ULONG;
        lstrClassName.set_MaximumLength(ClassName.MaximumLength as ULONG);
        plstrClassName = &mut lstrClassName;
    }

    // Initialize a LARGE_STRING.
    RtlInitLargeString(&mut WindowName, lpWindowName as LPCVOID, Unicode);

    'cleanup: {
        // HACK: the current implementation expects the Window name to be UNICODE.
        if Unicode == 0 {
            let AnsiBuffer = WindowName.Buffer as PSTR;
            let AnsiLength = WindowName.Length;

            WindowName.Length = 0;
            WindowName.set_MaximumLength(AnsiLength * core::mem::size_of::<WCHAR>() as ULONG);
            WindowName.Buffer = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                WindowName.MaximumLength() as SIZE_T,
            );
            if WindowName.Buffer.is_null() {
                SetLastError(ERROR_OUTOFMEMORY);
                break 'cleanup;
            }

            let Status = RtlMultiByteToUnicodeN(
                WindowName.Buffer as PWCH,
                WindowName.MaximumLength(),
                &mut WindowName.Length,
                AnsiBuffer as *const CHAR,
                AnsiLength,
            );
            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }
        }

        if hMenu.is_null() && (dwStyle & (WS_OVERLAPPEDWINDOW | WS_POPUP)) != 0 {
            if Unicode != 0 {
                wceW.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
                if GetClassInfoExW(hInstance, lpClassName as LPCWSTR, &mut wceW) != 0
                    && !wceW.lpszMenuName.is_null()
                {
                    hMenu = LoadMenuW(hInstance, wceW.lpszMenuName);
                }
            } else {
                wceA.cbSize = core::mem::size_of::<WNDCLASSEXA>() as UINT;
                if GetClassInfoExA(hInstance, lpClassName, &mut wceA) != 0
                    && !wceA.lpszMenuName.is_null()
                {
                    hMenu = LoadMenuA(hInstance, wceA.lpszMenuName);
                }
            }
        }

        if Unicode == 0 {
            dwExStyle |= WS_EX_SETANSICREATOR;
        }

        lpszClsVersion = ClassNameToVersion(
            lpClassName as *const core::ffi::c_void,
            core::ptr::null_mut(),
            &mut lpLibFileName,
            &mut pCtx,
            (Unicode == 0) as BOOL,
        );
        if lpszClsVersion.is_null() {
            plstrClassVersion = plstrClassName;
        } else {
            RtlInitUnicodeString(&mut ClassVersion, lpszClsVersion);
            lstrClassVersion.Buffer = ClassVersion.Buffer as PVOID;
            lstrClassVersion.Length = ClassVersion.Length as ULONG;
            lstrClassVersion.set_MaximumLength(ClassVersion.MaximumLength as ULONG);
            plstrClassVersion = &mut lstrClassVersion;
        }

        loop {
            Handle = NtUserCreateWindowEx(
                dwExStyle,
                plstrClassName,
                plstrClassVersion,
                &mut WindowName,
                dwStyle,
                x,
                y,
                nWidth,
                nHeight,
                hWndParent,
                hMenu,
                hInstance,
                lpParam,
                dwFlagsVer,
                pCtx,
            );
            if !Handle.is_null() {
                break;
            }
            if lpLibFileName.is_null() {
                break;
            }
            if ClassFound == 0 {
                dwLastError = GetLastError();
                if dwLastError == ERROR_CANNOT_FIND_WND_CLASS {
                    ClassFound = VersionRegisterClass(
                        ClassName.Buffer,
                        lpLibFileName,
                        pCtx,
                        &mut hLibModule,
                    );
                    if ClassFound != 0 {
                        continue;
                    }
                }
            }
            if !hLibModule.is_null() {
                dwLastError = GetLastError();
                FreeLibrary(hLibModule);
                SetLastError(dwLastError);
                hLibModule = core::ptr::null_mut();
            }
            break;
        }
    } // 'cleanup

    if Unicode == 0 {
        if !IS_ATOM(lpClassName) {
            RtlFreeUnicodeString(&mut ClassName);
        }
        RtlFreeLargeString(&mut WindowName);
    }

    Handle
}

#[no_mangle]
pub unsafe extern "system" fn CreateWindowExA(
    dwExStyle: DWORD,
    lpClassName: LPCSTR,
    lpWindowName: LPCSTR,
    mut dwStyle: DWORD,
    mut x: core::ffi::c_int,
    mut y: core::ffi::c_int,
    mut nWidth: core::ffi::c_int,
    mut nHeight: core::ffi::c_int,
    hWndParent: HWND,
    mut hMenu: HMENU,
    hInstance: HINSTANCE,
    mut lpParam: LPVOID,
) -> HWND {
    let mut mdi: MDICREATESTRUCTA = core::mem::zeroed();
    let hwnd: HWND;

    if RegisterDefaultClasses == 0 {
        TRACE!("CreateWindowExA RegisterSystemControls\n");
        RegisterSystemControls();
    }

    'skip_mdi: {
        if (dwExStyle & WS_EX_MDICHILD) != 0 {
            let mut mPos: [POINT; 2] = core::mem::zeroed();
            let mut id: UINT = 0;
            let top_child: HWND;
            let pWndParent: PWND;

            pWndParent = ValidateHwnd(hWndParent);

            if pWndParent.is_null() {
                return core::ptr::null_mut();
            }

            if (*pWndParent).fnid != FNID_MDICLIENT {
                WARN!("WS_EX_MDICHILD, but parent %p is not MDIClient\n", hWndParent);
                break 'skip_mdi;
            }

            // lpParams of WM_[NC]CREATE is different for MDI children.
            mdi.szClass = lpClassName;
            mdi.szTitle = lpWindowName;
            mdi.hOwner = hInstance as HANDLE;
            mdi.x = x;
            mdi.y = y;
            mdi.cx = nWidth;
            mdi.cy = nHeight;
            mdi.style = dwStyle;
            mdi.lParam = lpParam as LPARAM;

            lpParam = core::ptr::addr_of_mut!(mdi) as LPVOID;

            if ((*pWndParent).style & MDIS_ALLCHILDSTYLES) != 0 {
                if (dwStyle & WS_POPUP) != 0 {
                    WARN!("WS_POPUP with MDIS_ALLCHILDSTYLES is not allowed\n");
                    return core::ptr::null_mut();
                }
                dwStyle |= WS_CHILD | WS_CLIPSIBLINGS;
            } else {
                dwStyle &= !WS_POPUP;
                dwStyle |= WS_CHILD
                    | WS_VISIBLE
                    | WS_CLIPSIBLINGS
                    | WS_CAPTION
                    | WS_SYSMENU
                    | WS_THICKFRAME
                    | WS_MINIMIZEBOX
                    | WS_MAXIMIZEBOX;
            }

            top_child = GetWindow(hWndParent, GW_CHILD);

            if !top_child.is_null() {
                // Restore current maximized child.
                if (dwStyle & WS_VISIBLE) != 0 && IsZoomed(top_child) != 0 {
                    TRACE!("Restoring current maximized child %p\n", top_child);
                    SendMessageW(top_child, WM_SETREDRAW, 0 as WPARAM, 0 as LPARAM);
                    ShowWindow(top_child, SW_RESTORE as core::ffi::c_int);
                    SendMessageW(top_child, WM_SETREDRAW, 1 as WPARAM, 0 as LPARAM);
                }
            }

            MDI_CalcDefaultChildPos(
                hWndParent,
                -1,
                mPos.as_mut_ptr(),
                0,
                core::ptr::addr_of_mut!(id),
            );

            if (dwStyle & WS_POPUP) == 0 {
                hMenu = UlongToHandle(id) as HMENU;
            }

            if (dwStyle & (WS_CHILD | WS_POPUP)) != 0 {
                if x == CW_USEDEFAULT || x == CW_USEDEFAULT16 {
                    x = mPos[0].x;
                    y = mPos[0].y;
                }
                if nWidth == CW_USEDEFAULT || nWidth == CW_USEDEFAULT16 || nWidth == 0 {
                    nWidth = mPos[1].x;
                }
                if nHeight == CW_USEDEFAULT || nHeight == CW_USEDEFAULT16 || nHeight == 0 {
                    nHeight = mPos[1].y;
                }
            }
        }
    } // 'skip_mdi

    hwnd = User32CreateWindowEx(
        dwExStyle,
        lpClassName,
        lpWindowName,
        dwStyle,
        x,
        y,
        nWidth,
        nHeight,
        hWndParent,
        hMenu,
        hInstance,
        lpParam,
        NUCWE_ANSI,
    );
    hwnd
}

#[no_mangle]
pub unsafe extern "system" fn CreateWindowExW(
    dwExStyle: DWORD,
    lpClassName: LPCWSTR,
    lpWindowName: LPCWSTR,
    mut dwStyle: DWORD,
    mut x: core::ffi::c_int,
    mut y: core::ffi::c_int,
    mut nWidth: core::ffi::c_int,
    mut nHeight: core::ffi::c_int,
    hWndParent: HWND,
    mut hMenu: HMENU,
    hInstance: HINSTANCE,
    mut lpParam: LPVOID,
) -> HWND {
    let mut mdi: MDICREATESTRUCTW = core::mem::zeroed();
    let hwnd: HWND;

    if RegisterDefaultClasses == 0 {
        ERR!("CreateWindowExW RegisterSystemControls\n");
        RegisterSystemControls();
    }

    'skip_mdi: {
        if (dwExStyle & WS_EX_MDICHILD) != 0 {
            let mut mPos: [POINT; 2] = core::mem::zeroed();
            let mut id: UINT = 0;
            let top_child: HWND;
            let pWndParent: PWND;

            pWndParent = ValidateHwnd(hWndParent);

            if pWndParent.is_null() {
                return core::ptr::null_mut();
            }

            if (*pWndParent).fnid != FNID_MDICLIENT {
                WARN!("WS_EX_MDICHILD, but parent %p is not MDIClient\n", hWndParent);
                break 'skip_mdi;
            }

            mdi.szClass = lpClassName;
            mdi.szTitle = lpWindowName;
            mdi.hOwner = hInstance as HANDLE;
            mdi.x = x;
            mdi.y = y;
            mdi.cx = nWidth;
            mdi.cy = nHeight;
            mdi.style = dwStyle;
            mdi.lParam = lpParam as LPARAM;

            lpParam = core::ptr::addr_of_mut!(mdi) as LPVOID;

            if ((*pWndParent).style & MDIS_ALLCHILDSTYLES) != 0 {
                if (dwStyle & WS_POPUP) != 0 {
                    WARN!("WS_POPUP with MDIS_ALLCHILDSTYLES is not allowed\n");
                    return core::ptr::null_mut();
                }
                dwStyle |= WS_CHILD | WS_CLIPSIBLINGS;
            } else {
                dwStyle &= !WS_POPUP;
                dwStyle |= WS_CHILD
                    | WS_VISIBLE
                    | WS_CLIPSIBLINGS
                    | WS_CAPTION
                    | WS_SYSMENU
                    | WS_THICKFRAME
                    | WS_MINIMIZEBOX
                    | WS_MAXIMIZEBOX;
            }

            top_child = GetWindow(hWndParent, GW_CHILD);

            if !top_child.is_null() {
                if (dwStyle & WS_VISIBLE) != 0 && IsZoomed(top_child) != 0 {
                    TRACE!("Restoring current maximized child %p\n", top_child);
                    SendMessageW(top_child, WM_SETREDRAW, 0 as WPARAM, 0 as LPARAM);
                    ShowWindow(top_child, SW_RESTORE as core::ffi::c_int);
                    SendMessageW(top_child, WM_SETREDRAW, 1 as WPARAM, 0 as LPARAM);
                }
            }

            MDI_CalcDefaultChildPos(hWndParent, -1, mPos.as_mut_ptr(), 0, &mut id);

            if (dwStyle & WS_POPUP) == 0 {
                hMenu = UlongToHandle(id) as HMENU;
            }

            if (dwStyle & (WS_CHILD | WS_POPUP)) != 0 {
                if x == CW_USEDEFAULT || x == CW_USEDEFAULT16 {
                    x = mPos[0].x;
                    y = mPos[0].y;
                }
                if nWidth == CW_USEDEFAULT || nWidth == CW_USEDEFAULT16 || nWidth == 0 {
                    nWidth = mPos[1].x;
                }
                if nHeight == CW_USEDEFAULT || nHeight == CW_USEDEFAULT16 || nHeight == 0 {
                    nHeight = mPos[1].y;
                }
            }
        }
    } // 'skip_mdi

    hwnd = User32CreateWindowEx(
        dwExStyle,
        lpClassName as LPCSTR,
        lpWindowName as LPCSTR,
        dwStyle,
        x,
        y,
        nWidth,
        nHeight,
        hWndParent,
        hMenu,
        hInstance,
        lpParam,
        0,
    );
    hwnd
}

#[no_mangle]
pub unsafe extern "system" fn DeferWindowPos(
    hWinPosInfo: HDWP,
    hWnd: HWND,
    hWndInsertAfter: HWND,
    x: core::ffi::c_int,
    y: core::ffi::c_int,
    cx: core::ffi::c_int,
    cy: core::ffi::c_int,
    uFlags: UINT,
) -> HDWP {
    NtUserDeferWindowPos(hWinPosInfo, hWnd, hWndInsertAfter, x, y, cx, cy, uFlags)
}

#[no_mangle]
pub unsafe extern "system" fn EndDeferWindowPos(hWinPosInfo: HDWP) -> BOOL {
    NtUserEndDeferWindowPosEx(hWinPosInfo, 0)
}

#[no_mangle]
pub unsafe extern "system" fn GetDesktopWindow() -> HWND {
    let Wnd: PWND;
    let mut Ret: HWND = core::ptr::null_mut();

    // _SEH2_TRY
    {
        Wnd = GetThreadDesktopWnd();
        if !Wnd.is_null() {
            Ret = UserHMGetHandle(Wnd);
        }
    }
    // _SEH2_EXCEPT: DUMMY_PSEH → handler is dead code.

    Ret
}

unsafe fn User32EnumWindows(
    hDesktop: HDESK,
    hWndparent: HWND,
    lpfn: WNDENUMPROC,
    lParam: LPARAM,
    dwThreadId: DWORD,
    bChildren: BOOL,
) -> BOOL {
    let mut dwCount: DWORD = 0;
    let pHwnd: *mut HWND;
    let hHeap: HANDLE;
    let mut Status: NTSTATUS;

    let lpfn = match lpfn {
        Some(f) => f,
        None => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    };

    // First get how many window entries there are.
    Status = NtUserBuildHwndList(
        hDesktop,
        hWndparent,
        bChildren as BOOLEAN,
        dwThreadId,
        dwCount,
        core::ptr::null_mut(),
        &mut dwCount,
    );
    if !NT_SUCCESS(Status) {
        return 0;
    }

    if dwCount == 0 {
        if dwThreadId == 0 {
            return 0;
        } else {
            return 1;
        }
    }

    // Allocate buffer to receive HWND handles.
    hHeap = GetProcessHeap();
    pHwnd = HeapAlloc(
        hHeap,
        0,
        (core::mem::size_of::<HWND>() as DWORD * (dwCount + 1)) as SIZE_T,
    ) as *mut HWND;
    if pHwnd.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    // Now call kernel again to fill the buffer this time.
    Status = NtUserBuildHwndList(
        hDesktop,
        hWndparent,
        bChildren as BOOLEAN,
        dwThreadId,
        dwCount,
        pHwnd,
        &mut dwCount,
    );
    if !NT_SUCCESS(Status) {
        if !pHwnd.is_null() {
            HeapFree(hHeap, 0, pHwnd as LPVOID);
        }
        return 0;
    }

    let mut i: DWORD = 0;
    while i < dwCount {
        if (*pHwnd.add(i as usize)).is_null() {
            i += 1;
            continue;
        }
        if lpfn(*pHwnd.add(i as usize), lParam) == 0 {
            HeapFree(hHeap, 0, pHwnd as LPVOID);
            return 0;
        }
        i += 1;
    }
    if !pHwnd.is_null() {
        HeapFree(hHeap, 0, pHwnd as LPVOID);
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn EnumChildWindows(
    hWndParent: HWND,
    lpEnumFunc: WNDENUMPROC,
    lParam: LPARAM,
) -> BOOL {
    if hWndParent.is_null() {
        return EnumWindows(lpEnumFunc, lParam);
    }
    User32EnumWindows(core::ptr::null_mut(), hWndParent, lpEnumFunc, lParam, 0, 1)
}

#[no_mangle]
pub unsafe extern "system" fn EnumThreadWindows(
    mut dwThreadId: DWORD,
    lpfn: WNDENUMPROC,
    lParam: LPARAM,
) -> BOOL {
    if dwThreadId == 0 {
        dwThreadId = GetCurrentThreadId();
    }
    User32EnumWindows(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpfn,
        lParam,
        dwThreadId,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL {
    User32EnumWindows(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpEnumFunc,
        lParam,
        0,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn EnumDesktopWindows(
    hDesktop: HDESK,
    lpfn: WNDENUMPROC,
    lParam: LPARAM,
) -> BOOL {
    User32EnumWindows(hDesktop, core::ptr::null_mut(), lpfn, lParam, 0, 0)
}

#[no_mangle]
pub unsafe extern "system" fn FindWindowExA(
    hwndParent: HWND,
    hwndChildAfter: HWND,
    lpszClass: LPCSTR,
    lpszWindow: LPCSTR,
) -> HWND {
    let mut titleW: LPWSTR = core::ptr::null_mut();
    let mut hwnd: HWND = core::ptr::null_mut();

    if !lpszWindow.is_null() {
        let len: DWORD =
            MultiByteToWideChar(CP_ACP, 0, lpszWindow, -1, core::ptr::null_mut(), 0) as DWORD;
        titleW = HeapAlloc(
            GetProcessHeap(),
            0,
            ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        if titleW.is_null() {
            return core::ptr::null_mut();
        }
        MultiByteToWideChar(CP_ACP, 0, lpszWindow, -1, titleW, len as core::ffi::c_int);
    }

    if !IS_INTRESOURCE(lpszClass) {
        let mut classW: [WCHAR; 256] = [0; 256];
        if MultiByteToWideChar(
            CP_ACP,
            0,
            lpszClass,
            -1,
            classW.as_mut_ptr(),
            (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>())
                as core::ffi::c_int,
        ) != 0
        {
            hwnd = FindWindowExW(hwndParent, hwndChildAfter, classW.as_ptr(), titleW);
        }
    } else {
        hwnd = FindWindowExW(hwndParent, hwndChildAfter, lpszClass as LPCWSTR, titleW);
    }

    HeapFree(GetProcessHeap(), 0, titleW as LPVOID);
    hwnd
}

#[no_mangle]
pub unsafe extern "system" fn FindWindowExW(
    hwndParent: HWND,
    hwndChildAfter: HWND,
    lpszClass: LPCWSTR,
    lpszWindow: LPCWSTR,
) -> HWND {
    let mut ucClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut pucClassName: *mut _UNICODE_STRING = core::ptr::null_mut();
    let mut ucWindowName: _UNICODE_STRING = core::mem::zeroed();
    let mut pucWindowName: *mut _UNICODE_STRING = core::ptr::null_mut();

    if IS_ATOM(lpszClass) {
        ucClassName.Length = 0;
        ucClassName.Buffer = lpszClass as PWCH;
        pucClassName = core::ptr::addr_of_mut!(ucClassName);
    } else if !lpszClass.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ucClassName), lpszClass);
        pucClassName = core::ptr::addr_of_mut!(ucClassName);
    }

    if !lpszWindow.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ucWindowName), lpszWindow);
        pucWindowName = core::ptr::addr_of_mut!(ucWindowName);
    }

    NtUserFindWindowEx(hwndParent, hwndChildAfter, pucClassName, pucWindowName, 0)
}

#[no_mangle]
pub unsafe extern "system" fn FindWindowA(lpClassName: LPCSTR, lpWindowName: LPCSTR) -> HWND {
    FindWindowExA(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpClassName,
        lpWindowName,
    )
}

#[no_mangle]
pub unsafe extern "system" fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND {
    FindWindowExW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpClassName,
        lpWindowName,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GetAltTabInfoA(
    hwnd: HWND,
    iItem: core::ffi::c_int,
    pati: PALTTABINFO,
    pszItemText: LPSTR,
    cchItemText: UINT,
) -> BOOL {
    NtUserGetAltTabInfo(hwnd, iItem, pati, pszItemText as LPWSTR, cchItemText, 1 as BOOL)
}

#[no_mangle]
pub unsafe extern "system" fn GetAltTabInfoW(
    hwnd: HWND,
    iItem: core::ffi::c_int,
    pati: PALTTABINFO,
    pszItemText: LPWSTR,
    cchItemText: UINT,
) -> BOOL {
    NtUserGetAltTabInfo(hwnd, iItem, pati, pszItemText, cchItemText, 0)
}

#[no_mangle]
pub unsafe extern "system" fn GetAncestor(hwnd: HWND, uType: UINT) -> HWND {
    let pWnd: PWND = ValidateHwnd(hwnd);
    if pWnd.is_null() || pWnd == GetThreadDesktopWnd() {
        return core::ptr::null_mut();
    }

    // Special handling optimized for speed.
    if uType == GA_PARENT {
        let mut hwndAncestor: HWND = core::ptr::null_mut();

        // _SEH2_TRY
        {
            if !(*pWnd).spwndParent.is_null() && (*pWnd).fnid != FNID_MESSAGEWND {
                let pwndAncestor: PWND = crate::wow64::desktop_wnd_to_user((*pWnd).spwndParent as PVOID);
                if !pwndAncestor.is_null() {
                    hwndAncestor = UserHMGetHandle(pwndAncestor);
                }
            }
        }
        // _SEH2_EXCEPT: DUMMY_PSEH → dead code.

        return hwndAncestor;
    }

    NtUserGetAncestor(hwnd, uType)
}

#[no_mangle]
pub unsafe extern "system" fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL {
    let Wnd: PWND = ValidateHwnd(hWnd);

    if Wnd.is_null() {
        return 0;
    }
    if (*Wnd).style & WS_MINIMIZED != 0 {
        (*lpRect).top = 0;
        (*lpRect).left = 0;
        (*lpRect).right = GetSystemMetrics(SM_CXMINIMIZED as core::ffi::c_int);
        (*lpRect).bottom = GetSystemMetrics(SM_CYMINIMIZED as core::ffi::c_int);
        return 1;
    }
    if hWnd != GetDesktopWindow() {
        // RECTL → RECT: layout-identical; copy through pointer cast.
        *lpRect = *(core::ptr::addr_of!((*Wnd).rcClient) as *const RECT);
        OffsetRect(lpRect, -(*Wnd).rcClient.left, -(*Wnd).rcClient.top);
    } else {
        (*lpRect).top = 0;
        (*lpRect).left = 0;
        (*lpRect).right = (*Wnd).rcClient.right;
        (*lpRect).bottom = (*Wnd).rcClient.bottom;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn GetLastActivePopup(hWnd: HWND) -> HWND {
    let Wnd: PWND;
    let mut Ret: HWND = hWnd;

    Wnd = ValidateHwnd(hWnd);
    if !Wnd.is_null() {
        // _SEH2_TRY
        {
            if !(*Wnd).spwndLastActive.is_null() {
                let LastActive: PWND = crate::wow64::desktop_wnd_to_user((*Wnd).spwndLastActive as PVOID);
                Ret = UserHMGetHandle(LastActive);
            }
        }
        // _SEH2_EXCEPT: DUMMY_PSEH → dead code.
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetParent(hWnd: HWND) -> HWND {
    let mut WndParent: PWND;
    let mut Ret: HWND = core::ptr::null_mut();

    let Wnd: PWND = ValidateHwnd(hWnd);
    if !Wnd.is_null() {
        // _SEH2_TRY
        {
            WndParent = core::ptr::null_mut();
            if (*Wnd).style & WS_POPUP != 0 {
                if !(*Wnd).spwndOwner.is_null() {
                    WndParent = crate::wow64::desktop_wnd_to_user((*Wnd).spwndOwner as PVOID);
                }
            } else if (*Wnd).style & WS_CHILD != 0 {
                if !(*Wnd).spwndParent.is_null() {
                    WndParent = crate::wow64::desktop_wnd_to_user((*Wnd).spwndParent as PVOID);
                }
            }

            if !WndParent.is_null() {
                Ret = UserHMGetHandle(WndParent);
            }
        }
        // _SEH2_EXCEPT: DUMMY_PSEH → dead code.
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessDefaultLayout(pdwDefaultLayout: *mut DWORD) -> BOOL {
    NtUserCallOneParam(
        pdwDefaultLayout as DWORD_PTR,
        ONEPARAM_ROUTINE_GETPROCDEFLAYOUT as DWORD,
    ) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND {
    let Wnd: PWND;
    let mut FoundWnd: PWND;
    let mut Ret: HWND = core::ptr::null_mut();

    Wnd = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return core::ptr::null_mut();
    }

    // _SEH2_TRY
    {
        FoundWnd = core::ptr::null_mut();
        match uCmd {
            GW_OWNER => {
                if !(*Wnd).spwndOwner.is_null() {
                    FoundWnd = crate::wow64::desktop_wnd_to_user((*Wnd).spwndOwner as PVOID);
                }
            }

            GW_HWNDFIRST => {
                if !(*Wnd).spwndParent.is_null() {
                    FoundWnd = crate::wow64::desktop_wnd_to_user((*Wnd).spwndParent as PVOID);
                    if !(*FoundWnd).spwndChild.is_null() {
                        FoundWnd = crate::wow64::desktop_wnd_to_user((*FoundWnd).spwndChild as PVOID);
                    }
                }
            }

            GW_HWNDNEXT => {
                if !(*Wnd).spwndNext.is_null() {
                    FoundWnd = crate::wow64::desktop_wnd_to_user((*Wnd).spwndNext as PVOID);
                }
            }

            GW_HWNDPREV => {
                if !(*Wnd).spwndPrev.is_null() {
                    FoundWnd = crate::wow64::desktop_wnd_to_user((*Wnd).spwndPrev as PVOID);
                }
            }

            GW_CHILD => {
                if !(*Wnd).spwndChild.is_null() {
                    FoundWnd = crate::wow64::desktop_wnd_to_user((*Wnd).spwndChild as PVOID);
                }
            }

            GW_HWNDLAST => {
                FoundWnd = Wnd;
                while !(*FoundWnd).spwndNext.is_null() {
                    FoundWnd = crate::wow64::desktop_wnd_to_user((*FoundWnd).spwndNext as PVOID);
                }
            }

            GW_ENABLEDPOPUP => {
                let pwndPopup: PWND =
                    NtUserCallHwnd(hWnd, HWND_ROUTINE_DWP_GETENABLEDPOPUP as DWORD) as PWND;
                if !pwndPopup.is_null() {
                    FoundWnd = DesktopPtrToUser(pwndPopup as PVOID) as PWND;
                }
            }

            _ => {
                UserSetLastError(ERROR_INVALID_GW_COMMAND);
            }
        }

        if !FoundWnd.is_null() {
            Ret = UserHMGetHandle(FoundWnd);
        }
    }
    // _SEH2_EXCEPT: DUMMY_PSEH → dead code.

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetTopWindow(mut hWnd: HWND) -> HWND {
    if hWnd.is_null() {
        hWnd = GetDesktopWindow();
    }
    GetWindow(hWnd, GW_CHILD)
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowInfo(hWnd: HWND, pwi: PWINDOWINFO) -> BOOL {
    let pWnd: PWND;
    let mut pCls: PCLS;
    let mut Size: SIZE = SIZE { cx: 0, cy: 0 };
    let mut Ret: BOOL = 0;

    if pwi.is_null() || (*pwi).cbSize as usize != core::mem::size_of::<WINDOWINFO>() {
        SetLastError(ERROR_INVALID_PARAMETER); // Just set the error and go!
    }

    pWnd = ValidateHwnd(hWnd);
    if pWnd.is_null() {
        return Ret;
    }

    UserGetWindowBorders((*pWnd).style, (*pWnd).ExStyle, &mut Size, 0);

    // _SEH2_TRY
    {
        pCls = crate::wow64::desktop_cls_to_user((*pWnd).pcls as PVOID);
        (*pwi).rcWindow = (*pWnd).rcWindow;
        (*pwi).rcClient = (*pWnd).rcClient;
        (*pwi).dwStyle = (*pWnd).style;
        (*pwi).dwExStyle = (*pWnd).ExStyle;
        (*pwi).cxWindowBorders = Size.cx as UINT;
        (*pwi).cyWindowBorders = Size.cy as UINT;
        (*pwi).dwWindowStatus = 0;
        if ((*pWnd).state & WNDS_ACTIVEFRAME) != 0 || (GetActiveWindow() == hWnd) {
            (*pwi).dwWindowStatus = WS_ACTIVECAPTION;
        }
        (*pwi).atomWindowType = if !pCls.is_null() { (*pCls).atomClassName } else { 0 };

        if ((*pWnd).state2 & WNDS2_WIN50COMPAT) != 0 {
            (*pwi).wCreatorVersion = 0x500;
        } else if ((*pWnd).state2 & WNDS2_WIN40COMPAT) != 0 {
            (*pwi).wCreatorVersion = 0x400;
        } else if ((*pWnd).state2 & WNDS2_WIN31COMPAT) != 0 {
            (*pwi).wCreatorVersion = 0x30A;
        } else {
            (*pwi).wCreatorVersion = 0x300;
        }

        Ret = 1;
    }
    // _SEH2_EXCEPT: DUMMY_PSEH → dead code.

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowModuleFileNameA(
    hwnd: HWND,
    lpszFileName: LPSTR,
    cchFileNameMax: UINT,
) -> UINT {
    let Wnd: PWND = ValidateHwnd(hwnd);

    if Wnd.is_null() {
        return 0;
    }

    GetModuleFileNameA((*Wnd).hModule, lpszFileName, cchFileNameMax)
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowModuleFileNameW(
    hwnd: HWND,
    lpszFileName: LPWSTR,
    cchFileNameMax: UINT,
) -> UINT {
    let Wnd: PWND = ValidateHwnd(hwnd);

    if Wnd.is_null() {
        return 0;
    }

    GetModuleFileNameW((*Wnd).hModule, lpszFileName, cchFileNameMax)
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowRect(hWnd: HWND, lpRect: LPRECT) -> BOOL {
    let Wnd: PWND = ValidateHwnd(hWnd);

    if Wnd.is_null() {
        return 0;
    }
    if hWnd != GetDesktopWindow() {
        // RECTL → RECT: layout-identical; copy through pointer cast.
        *lpRect = *(core::ptr::addr_of!((*Wnd).rcWindow) as *const RECT);
    } else {
        (*lpRect).left = 0;
        (*lpRect).top = 0;
        (*lpRect).right = (*Wnd).rcWindow.right;
        (*lpRect).bottom = (*Wnd).rcWindow.bottom;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowTextA(
    hWnd: HWND,
    lpString: LPSTR,
    nMaxCount: core::ffi::c_int,
) -> core::ffi::c_int {
    let Wnd: PWND;
    let Length: INT;

    if lpString.is_null() || nMaxCount == 0 {
        return 0;
    }

    Wnd = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    *lpString = 0;

    if TestWindowProcess(Wnd) == 0 {
        // _SEH2_TRY
        Length = DefWindowProcA(hWnd, WM_GETTEXT, nMaxCount as WPARAM, lpString as LPARAM) as INT;
        // _SEH2_EXCEPT → Length = 0 (dead code under DUMMY_PSEH).
    } else {
        Length = SendMessageA(hWnd, WM_GETTEXT, nMaxCount as WPARAM, lpString as LPARAM) as INT;
    }

    Length
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowTextLengthA(hWnd: HWND) -> core::ffi::c_int {
    let Wnd: PWND = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    if TestWindowProcess(Wnd) == 0 {
        DefWindowProcA(hWnd, WM_GETTEXTLENGTH, 0, 0) as core::ffi::c_int
    } else {
        SendMessageA(hWnd, WM_GETTEXTLENGTH, 0, 0) as core::ffi::c_int
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowTextLengthW(hWnd: HWND) -> core::ffi::c_int {
    let Wnd: PWND = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    if TestWindowProcess(Wnd) == 0 {
        DefWindowProcW(hWnd, WM_GETTEXTLENGTH, 0, 0) as core::ffi::c_int
    } else {
        SendMessageW(hWnd, WM_GETTEXTLENGTH, 0, 0) as core::ffi::c_int
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowTextW(
    hWnd: HWND,
    lpString: LPWSTR,
    nMaxCount: core::ffi::c_int,
) -> core::ffi::c_int {
    let Wnd: PWND;
    let Length: INT;

    if lpString.is_null() || nMaxCount == 0 {
        return 0;
    }

    Wnd = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    *lpString.offset(0) = 0u16;

    if TestWindowProcess(Wnd) == 0 {
        // _SEH2_TRY
        Length = DefWindowProcW(hWnd, WM_GETTEXT, nMaxCount as WPARAM, lpString as LPARAM) as INT;
        // _SEH2_EXCEPT → Length = 0 (dead code under DUMMY_PSEH).
    } else {
        Length = SendMessageW(hWnd, WM_GETTEXT, nMaxCount as WPARAM, lpString as LPARAM) as INT;
    }

    Length
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowThreadProcessId(
    hWnd: HWND,
    lpdwProcessId: LPDWORD,
) -> DWORD {
    let mut Ret: DWORD = 0;
    let pWnd: PWND = ValidateHwnd(hWnd);

    if pWnd.is_null() {
        return Ret;
    }

    let ti: PTHREADINFO = WndHeadPti(pWnd);

    if !ti.is_null() {
        if ti == GetW32ThreadInfo() {
            // We are current.
            if !lpdwProcessId.is_null() {
                *lpdwProcessId =
                    (*NtCurrentTeb()).ClientId.UniqueProcess as DWORD_PTR as DWORD;
            }
            Ret = (*NtCurrentTeb()).ClientId.UniqueThread as DWORD_PTR as DWORD;
        } else {
            // Ask kernel for info.
            if !lpdwProcessId.is_null() {
                *lpdwProcessId =
                    NtUserQueryWindow(hWnd, QUERY_WINDOW_UNIQUE_PROCESS_ID as DWORD) as DWORD;
            }
            Ret = NtUserQueryWindow(hWnd, QUERY_WINDOW_UNIQUE_THREAD_ID as DWORD) as DWORD;
        }
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn IsChild(hWndParent: HWND, hWnd: HWND) -> BOOL {
    let WndParent: PWND;
    let DesktopWnd: PWND;
    let mut Wnd: PWND;
    let mut Ret: BOOL = 0;

    WndParent = ValidateHwnd(hWndParent);
    if WndParent.is_null() {
        return 0;
    }
    Wnd = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    DesktopWnd = GetThreadDesktopWnd();
    if DesktopWnd.is_null() {
        return 0;
    }

    // _SEH2_TRY
    {
        while !Wnd.is_null() && ((*Wnd).style & (WS_POPUP | WS_CHILD)) == WS_CHILD {
            if !(*Wnd).spwndParent.is_null() {
                Wnd = crate::wow64::desktop_wnd_to_user((*Wnd).spwndParent as PVOID);

                if Wnd == WndParent {
                    Ret = 1;
                    break;
                }
            } else {
                break;
            }
        }
    }
    // _SEH2_EXCEPT: DUMMY_PSEH → dead code.

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn IsIconic(hWnd: HWND) -> BOOL {
    let Wnd: PWND = ValidateHwnd(hWnd);

    if !Wnd.is_null() {
        return (((*Wnd).style & WS_MINIMIZE) != 0) as BOOL;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn IsWindow(hWnd: HWND) -> BOOL {
    let Wnd: PWND = ValidateHwndNoErr(hWnd);
    if !Wnd.is_null() {
        if (*Wnd).state & WNDS_DESTROYED != 0 {
            return 0;
        }
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn IsWindowUnicode(hWnd: HWND) -> BOOL {
    let Wnd: PWND = ValidateHwnd(hWnd);

    if !Wnd.is_null() {
        return (*Wnd).Unicode() as BOOL;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn IsWindowVisible(hWnd: HWND) -> BOOL {
    let mut Ret: BOOL = 0;
    let mut Wnd: PWND = ValidateHwnd(hWnd);

    if !Wnd.is_null() {
        // _SEH2_TRY
        {
            Ret = 1;

            loop {
                if ((*Wnd).style & WS_VISIBLE) == 0 {
                    Ret = 0;
                    break;
                }

                if !(*Wnd).spwndParent.is_null() {
                    Wnd = crate::wow64::desktop_wnd_to_user((*Wnd).spwndParent as PVOID);
                } else {
                    break;
                }

                if Wnd.is_null() {
                    break;
                }
            }
        }
        // _SEH2_EXCEPT → Ret = FALSE (dead code under DUMMY_PSEH).
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn IsWindowEnabled(hWnd: HWND) -> BOOL {
    (GetWindowLongPtrW(hWnd, GWL_STYLE as core::ffi::c_int) as isize & WS_DISABLED as LONG_PTR as isize == 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn IsZoomed(hWnd: HWND) -> BOOL {
    ((GetWindowLongPtrW(hWnd, GWL_STYLE as core::ffi::c_int) as isize & WS_MAXIMIZE as LONG_PTR as isize) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn LockSetForegroundWindow(uLockCode: UINT) -> BOOL {
    NtUserxLockSetForegroundWindow(uLockCode)
}

#[no_mangle]
pub unsafe extern "system" fn AnimateWindow(hwnd: HWND, _dwTime: DWORD, dwFlags: DWORD) -> BOOL {
    // FIXME Add animation code.
    let visible: BOOL = IsWindowVisible(hwnd);
    if IsWindow(hwnd) == 0
        || (visible != 0 && (dwFlags & AW_HIDE) == 0)
        || (visible == 0 && (dwFlags & AW_HIDE) != 0)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let nCmdShow = if (dwFlags & AW_HIDE) != 0 {
        SW_HIDE
    } else if (dwFlags & AW_ACTIVATE) != 0 {
        SW_SHOW
    } else {
        SW_SHOWNA
    };

    ShowWindow(hwnd, nCmdShow as core::ffi::c_int);

    1
}

#[no_mangle]
pub unsafe extern "system" fn OpenIcon(hWnd: HWND) -> BOOL {
    if (GetWindowLongPtrW(hWnd, GWL_STYLE as core::ffi::c_int) as isize & WS_MINIMIZE as LONG_PTR as isize) == 0 {
        return 0;
    }

    ShowWindow(hWnd, SW_RESTORE as core::ffi::c_int);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RealChildWindowFromPoint(
    hwndParent: HWND,
    ptParentClientCoords: POINT,
) -> HWND {
    NtUserRealChildWindowFromPoint(hwndParent, ptParentClientCoords.x, ptParentClientCoords.y)
}

#[no_mangle]
pub unsafe extern "system" fn SetForegroundWindow(hWnd: HWND) -> BOOL {
    NtUserxSetForegroundWindow(hWnd)
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessDefaultLayout(dwDefaultLayout: DWORD) -> BOOL {
    NtUserCallOneParam(
        dwDefaultLayout as DWORD_PTR,
        ONEPARAM_ROUTINE_SETPROCDEFLAYOUT as DWORD,
    ) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowTextA(hWnd: HWND, lpString: LPCSTR) -> BOOL {
    let pwnd: PWND = ValidateHwnd(hWnd);
    if !pwnd.is_null() {
        if TestWindowProcess(pwnd) == 0 {
            // do not send WM_GETTEXT messages to other processes
            return (DefWindowProcA(hWnd, WM_SETTEXT, 0, lpString as LPARAM) >= 0) as BOOL;
        }
        return (SendMessageA(hWnd, WM_SETTEXT, 0, lpString as LPARAM) >= 0) as BOOL;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL {
    let pwnd: PWND = ValidateHwnd(hWnd);
    if !pwnd.is_null() {
        if TestWindowProcess(pwnd) == 0 {
            return (DefWindowProcW(hWnd, WM_SETTEXT, 0, lpString as LPARAM) >= 0) as BOOL;
        }
        return (SendMessageW(hWnd, WM_SETTEXT, 0, lpString as LPARAM) >= 0) as BOOL;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn ShowOwnedPopups(hWnd: HWND, fShow: BOOL) -> BOOL {
    NtUserxShowOwnedPopups(hWnd, fShow)
}

#[no_mangle]
pub unsafe extern "system" fn UpdateLayeredWindow(
    hwnd: HWND,
    hdcDst: HDC,
    pptDst: *mut POINT,
    psize: *mut SIZE,
    hdcSrc: HDC,
    pptSrc: *mut POINT,
    crKey: COLORREF,
    pbl: *mut BLENDFUNCTION,
    dwFlags: DWORD,
) -> BOOL {
    // only valid for UpdateLayeredWindowIndirect
    if (dwFlags & ULW_EX_NORESIZE) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    NtUserUpdateLayeredWindow(
        hwnd,
        hdcDst,
        pptDst as *mut POINTL,
        psize,
        hdcSrc,
        pptSrc as *mut POINTL,
        crKey,
        pbl,
        dwFlags,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn UpdateLayeredWindowIndirect(
    hwnd: HWND,
    info: *const UPDATELAYEREDWINDOWINFO,
) -> BOOL {
    if !info.is_null()
        && (*info).cbSize as usize == core::mem::size_of::<UPDATELAYEREDWINDOWINFO>()
    {
        return NtUserUpdateLayeredWindow(
            hwnd,
            (*info).hdcDst,
            (*info).pptDst as *mut POINTL,
            (*info).psize as *mut SIZE,
            (*info).hdcSrc,
            (*info).pptSrc as *mut POINTL,
            (*info).crKey,
            (*info).pblend as *mut BLENDFUNCTION,
            (*info).dwFlags,
            (*info).prcDirty as *mut RECTL,
        );
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    0
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowContextHelpId(hwnd: HWND, dwContextHelpId: DWORD) -> BOOL {
    NtUserxSetWindowContextHelpId(hwnd, dwContextHelpId as DWORD_PTR)
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowContextHelpId(hwnd: HWND) -> DWORD {
    NtUserxGetWindowContextHelpId(hwnd)
}

#[no_mangle]
pub unsafe extern "system" fn InternalGetWindowText(
    hWnd: HWND,
    lpString: LPWSTR,
    nMaxCount: core::ffi::c_int,
) -> core::ffi::c_int {
    let Ret = NtUserInternalGetWindowText(hWnd, lpString, nMaxCount);
    if Ret == 0 && !lpString.is_null() {
        *lpString = 0u16;
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn IsHungAppWindow(hwnd: HWND) -> BOOL {
    (NtUserQueryWindow(hwnd, QUERY_WINDOW_ISHUNG as DWORD) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetLastErrorEx(dwErrCode: DWORD, _dwType: DWORD) {
    SetLastError(dwErrCode);
}

#[no_mangle]
pub unsafe extern "system" fn GetFocus() -> HWND {
    NtUserGetThreadState(THREADSTATE_FOCUSWINDOW as DWORD) as HWND
}

#[no_mangle]
pub unsafe extern "system" fn GetRealWindowOwner(hwnd: HWND) -> DWORD {
    NtUserQueryWindow(hwnd, QUERY_WINDOW_REAL_ID as DWORD) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn SetTaskmanWindow(hWnd: HWND) -> HWND {
    NtUserxSetTaskmanWindow(hWnd)
}

#[no_mangle]
pub unsafe extern "system" fn SetProgmanWindow(hWnd: HWND) -> HWND {
    NtUserxSetProgmanWindow(hWnd)
}

#[no_mangle]
pub unsafe extern "system" fn GetProgmanWindow() -> HWND {
    NtUserGetThreadState(THREADSTATE_PROGMANWINDOW as DWORD) as HWND
}

#[no_mangle]
pub unsafe extern "system" fn GetTaskmanWindow() -> HWND {
    NtUserGetThreadState(THREADSTATE_TASKMANWINDOW as DWORD) as HWND
}

#[no_mangle]
pub unsafe extern "system" fn ScrollWindow(
    hWnd: HWND,
    dx: core::ffi::c_int,
    dy: core::ffi::c_int,
    lpRect: *const RECT,
    prcClip: *const RECT,
) -> BOOL {
    (NtUserScrollWindowEx(
        hWnd,
        dx,
        dy,
        lpRect as *const RECTL,
        prcClip as *const RECTL,
        0 as HRGN,
        core::ptr::null_mut(),
        ((if !lpRect.is_null() { 0 } else { SW_SCROLLCHILDREN as core::ffi::c_int })
            | (SW_ERASE | SW_INVALIDATE | SW_SCROLLWNDDCE) as core::ffi::c_int)
            as UINT,
    ) != ERROR as DWORD) as BOOL
}

/* ScrollWindow uses the window DC, ScrollWindowEx doesn't */

#[no_mangle]
pub unsafe extern "system" fn ScrollWindowEx(
    hWnd: HWND,
    dx: core::ffi::c_int,
    dy: core::ffi::c_int,
    prcScroll: *const RECT,
    prcClip: *const RECT,
    hrgnUpdate: HRGN,
    prcUpdate: LPRECT,
    flags: UINT,
) -> INT {
    if (flags & SW_SMOOTHSCROLL) != 0 {
        // FIXME("SW_SMOOTHSCROLL not supported.\n"); Fall through....
    }
    NtUserScrollWindowEx(
        hWnd,
        dx,
        dy,
        prcScroll as *const RECTL,
        prcClip as *const RECTL,
        hrgnUpdate,
        prcUpdate as LPRECTL,
        flags,
    ) as INT
}

#[no_mangle]
pub unsafe extern "system" fn AnyPopup() -> BOOL {
    let mut i: core::ffi::c_int;
    let retvalue: BOOL;
    let list: *mut HWND = WIN_ListChildren(GetDesktopWindow());

    if list.is_null() {
        return 0;
    }
    i = 0;
    while !(*list.offset(i as isize)).is_null() {
        if IsWindowVisible(*list.offset(i as isize)) != 0
            && !GetWindow(*list.offset(i as isize), GW_OWNER).is_null()
        {
            break;
        }
        i += 1;
    }
    retvalue = (!(*list.offset(i as isize)).is_null()) as BOOL;
    HeapFree(GetProcessHeap(), 0, list as LPVOID);
    retvalue
}

#[no_mangle]
pub unsafe extern "system" fn IsWindowInDestroy(hWnd: HWND) -> BOOL {
    let pwnd: PWND = ValidateHwnd(hWnd);
    if pwnd.is_null() {
        return 0;
    }
    (((*pwnd).state2 & WNDS2_INDESTROY) == WNDS2_INDESTROY) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn DisableProcessWindowsGhosting() {
    NtUserxEnableProcessWindowGhosting(0);
}

/* EOF */
