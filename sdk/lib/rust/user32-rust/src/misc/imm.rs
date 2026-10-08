use win32k_sys::win32k::*;
use core::ffi::c_void;

// ── No-op debug macros (the C ERR/WARN/TRACE/FIXME compile to nothing under the
//   Wine debug-channel guard). Consume args so side effects still type-check.
//   Mirror windows/menu.rs 23-26. Must precede first use (textual macro scope).

// ── ntdll Rtl* string helpers + LOWORD/HIWORD/MAKELONG + ValidateHwnd live in
//   crate::internal; the gpsi per-user accessor + NtCurrentTeb too.
use crate::internal::{LOWORD, NtCurrentTeb, ValidateHwnd};
// ── gdi32/kernel32/CRT glue (MultiByteToWideChar/WideCharToMultiByte/RtlZeroMemory
//   /memcpy/wcslen/GetProcessHeap/CP_ACP) live in crate::winbase.
use crate::winbase::{wcslen, RtlZeroMemory, WideCharToMultiByte, CP_ACP};
// ── gSharedInfo is owned by misc/dllmain.rs (pub static mut SHAREDINFO).
use crate::misc::dllmain::gSharedInfo;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// win32k-sys defines FALSE/TRUE as `u32`, but BOOL is `c_int` (i32). Shadow them
// with BOOL-typed constants (same pattern as windows/menu.rs / defwnd.rs).
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── ValidateHandle is `PVOID FASTCALL` (user32p.h) → @Name@N on i386.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHandle(hObject: HANDLE, ObjType: UINT) -> *mut core::ffi::c_void;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHandle(hObject: HANDLE, ObjType: UINT) -> *mut core::ffi::c_void;
}

// ── user32 public exports / sibling-TU exports referenced here. WINAPI = stdcall
//   on i386 → `extern "system"`. kernel32/gdi32 imports resolve through the DLL.
extern "system" {
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn IsWindow(hwnd: HWND) -> BOOL;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int) -> LONG_PTR;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetParent(hWnd: HWND) -> HWND;
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;
    fn GetClassInfoW(hInstance: HINSTANCE, lpClassName: LPCWSTR, lpWndClass: LPWNDCLASSW) -> BOOL;
    fn CreateWindowExW(dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, x: core::ffi::c_int, y: core::ffi::c_int, nWidth: core::ffi::c_int, nHeight: core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn CreateWindowExA(dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, x: core::ffi::c_int, y: core::ffi::c_int, nWidth: core::ffi::c_int, nHeight: core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    // kernel32 imports.
    fn GetModuleHandleW(lpModuleName: PCWSTR) -> HMODULE;
    fn LoadLibraryW(lpLibFileName: PCWSTR) -> HMODULE;
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    fn GetLastError() -> DWORD;
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn WinHelpW(hWndMain: HWND, lpszHelp: LPCWSTR, uCommand: UINT, dwData: ULONG_PTR) -> BOOL;
}

// Interlocked{Increment,Decrement}: kernel32 imports on i386 but compiler intrinsics
// (no import symbol) on amd64/arm64 — implement via Rust atomics for uniform cohab link.
#[inline]
unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}
#[inline]
unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}

// ── UserHMGetHandle macro (user_x.h): read the object's handle, the `.h` field of
//   the embedded _HEAD at offset 0 of `head` (bindgen flattens _HEAD's leading
//   member into head). Mirror windows/class.rs's private copy.
#[inline]
unsafe fn UserHMGetHandle(spwnd: PWND) -> HWND {
    *(core::ptr::addr_of!((*spwnd).head) as *const HWND)
}

// CreateWindowW/A are the variadic-less wrappers → map onto the *Ex forms with the
// extended style of 0 (exactly what windows.h's #define does).
#[inline]
unsafe fn CreateWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, x: core::ffi::c_int, y: core::ffi::c_int, w: core::ffi::c_int, h: core::ffi::c_int, parent: HWND, menu: HMENU, inst: HINSTANCE, param: LPVOID) -> HWND {
    CreateWindowExW(0, lpClassName, lpWindowName, dwStyle, x, y, w, h, parent, menu, inst, param)
}
#[inline]
unsafe fn CreateWindowA(lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, x: core::ffi::c_int, y: core::ffi::c_int, w: core::ffi::c_int, h: core::ffi::c_int, parent: HWND, menu: HMENU, inst: HINSTANCE, param: LPVOID) -> HWND {
    CreateWindowExA(0, lpClassName, lpWindowName, dwStyle, x, y, w, h, parent, menu, inst, param)
}

// _wcsicmp (msvcrt) — declared cdecl.
extern "C" {
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> core::ffi::c_int;
}

// ── debugstr_w: a no-op (TRACE args are never evaluated). Mirror dialog.rs 243.
#[inline]
unsafe fn debugstr_w(_s: PCWSTR) -> *const i8 {
    core::ptr::null()
}

// ── GetWin32ClientInfo() — ntuser.h macro:
//   #define GetWin32ClientInfo() ((PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo))
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO
}

// ── IS_CICERO_MODE() (ntuser.h):
//   (gpsi && (gpsi->dwSRVIFlags & SRVINFO_CTFIME_ENABLED))
#[inline]
unsafe fn IS_CICERO_MODE() -> BOOL {
    (!gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_CTFIME_ENABLED) != 0) as BOOL
}

// ── IS_IME_HKL (imm32_undoc.h):
//   ((((ULONG_PTR)(hKL)) & 0xF0000000) == IME_MASK)
#[inline]
unsafe fn IS_IME_HKL(hKL: HKL) -> BOOL {
    (((hKL as ULONG_PTR) & 0xF0000000) == IME_MASK as ULONG_PTR) as BOOL
}

// ── GET_X_LPARAM/GET_Y_LPARAM (windowsx.h): (int)(short)LO/HIWORD(lp)
#[inline]
fn GET_X_LPARAM(lp: LPARAM) -> core::ffi::c_int {
    (LOWORD(lp as DWORD) as i16) as core::ffi::c_int
}
#[inline]
fn GET_Y_LPARAM(lp: LPARAM) -> core::ffi::c_int {
    (((lp as DWORD) >> 16) as u16 as i16) as core::ffi::c_int
}

// ── HRESULT_FROM_WIN32 (winerror.h).
#[inline]
fn HRESULT_FROM_WIN32(x: DWORD) -> HRESULT {
    if x as i32 <= 0 {
        x as HRESULT
    } else {
        (((x & 0x0000FFFF) | (7u32 << 16) | 0x80000000) as i32) as HRESULT
    }
}

// ── MAKEINTRESOURCEA (winuser.h): ((LPSTR)(ULONG_PTR)(WORD)(i))
#[inline]
fn MAKEINTRESOURCEA(i: u32) -> LPCSTR {
    ((i as u16) as usize) as LPCSTR
}

// ── strsafe.h inline helpers (no linkable symbol — reproduced faithfully as
//   bounded NUL-terminated concat/copy on UTF-16 buffers).
unsafe fn StringCchCopyW(dst: PWSTR, cch: SIZE_T, src: PCWSTR) -> HRESULT {
    let cch = cch as usize;
    if cch == 0 {
        return HRESULT_FROM_WIN32(87 /* ERROR_INVALID_PARAMETER */);
    }
    let mut i: usize = 0;
    while i + 1 < cch && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
    if *src.add(i) != 0 {
        0x8007007Au32 as i32 as HRESULT /* STRSAFE_E_INSUFFICIENT_BUFFER */
    } else {
        0 /* S_OK */
    }
}
unsafe fn StringCchCatW(dst: PWSTR, cch: SIZE_T, src: PCWSTR) -> HRESULT {
    let cch = cch as usize;
    let mut len: usize = 0;
    while len < cch && *dst.add(len) != 0 {
        len += 1;
    }
    if len >= cch {
        return 0x8007007Au32 as i32 as HRESULT;
    }
    StringCchCopyW(dst.add(len), (cch - len) as SIZE_T, src)
}

// ── INPUTCONTEXTDX flattens its C++ base INPUTCONTEXT into __bindgen_padding_0;
//   reach the base fields (hWnd/fOpen/fdwConversion/cfCompForm/cfCandForm/lfFont)
//   by re-casting the same pointer to LPINPUTCONTEXT (they overlay byte-for-byte).
#[inline]
unsafe fn icx_base(p: LPINPUTCONTEXTDX) -> LPINPUTCONTEXT {
    p as LPINPUTCONTEXT
}

// ── THRDESKHEAD begins with THROBJHEAD { HANDLE h; DWORD cLockObj; PTHREADINFO pti; }.
// Read through the typed head, never an index into bindgen's __bindgen_padding_0: its
// ELEMENT type is the generator's choice. The 2026-09-27 regeneration turned [u64; 3]
// into [u8; 24], `[2]` became one byte of the handle, and every posted message dispatched
// through IntCallMessageProc was dropped as ERROR_MESSAGE_SYNC_ONLY (no app closed on
// WM_CLOSE) with every build still green.
#[inline]
unsafe fn head_pti(pWnd: PWND) -> usize {
    (*(pWnd as *const _THROBJHEAD)).pti as usize
}

// ───────────────────────────────────────────────────────────────────────────
//   IMM_FN dispatch table (immtable.h → Imm32ApiTable gImmApiEntries).
//   Each entry is a WINAPI (`extern "system"`) function pointer, initialized to a
//   stub. IntInitializeImmEntryTable() rebinds them via GetProcAddress(imm32, ...).
//   IMM_FN!(name) expands to `gImmApiEntries.pname`.  Faithful to user32p.h's
//   `#define IMM_FN(name) gImmApiEntries.p##name`.
// ───────────────────────────────────────────────────────────────────────────

type Fn_ImmWINNLSEnableIME = unsafe extern "system" fn(HWND, BOOL) -> BOOL;
type Fn_ImmWINNLSGetEnableStatus = unsafe extern "system" fn(HWND) -> BOOL;
type Fn_ImmSendIMEMessageExW = unsafe extern "system" fn(HWND, LPARAM) -> LRESULT;
type Fn_ImmSendIMEMessageExA = unsafe extern "system" fn(HWND, LPARAM) -> LRESULT;
type Fn_ImmIMPGetIMEW = unsafe extern "system" fn(HWND, LPIMEPROW) -> BOOL;
type Fn_ImmIMPGetIMEA = unsafe extern "system" fn(HWND, LPIMEPROA) -> BOOL;
type Fn_ImmIMPQueryIMEW = unsafe extern "system" fn(LPIMEPROW) -> BOOL;
type Fn_ImmIMPQueryIMEA = unsafe extern "system" fn(LPIMEPROA) -> BOOL;
type Fn_ImmIMPSetIMEW = unsafe extern "system" fn(HWND, LPIMEPROW) -> BOOL;
type Fn_ImmIMPSetIMEA = unsafe extern "system" fn(HWND, LPIMEPROA) -> BOOL;
type Fn_ImmAssociateContext = unsafe extern "system" fn(HWND, HIMC) -> HIMC;
type Fn_ImmEscapeA = unsafe extern "system" fn(HKL, HIMC, UINT, LPVOID) -> LRESULT;
type Fn_ImmEscapeW = unsafe extern "system" fn(HKL, HIMC, UINT, LPVOID) -> LRESULT;
type Fn_ImmGetCompositionStringA = unsafe extern "system" fn(HIMC, DWORD, LPVOID, DWORD) -> LONG;
type Fn_ImmGetCompositionStringW = unsafe extern "system" fn(HIMC, DWORD, LPVOID, DWORD) -> LONG;
type Fn_ImmGetCompositionWindow = unsafe extern "system" fn(HIMC, LPCOMPOSITIONFORM) -> BOOL;
type Fn_ImmGetContext = unsafe extern "system" fn(HWND) -> HIMC;
type Fn_ImmGetDefaultIMEWnd = unsafe extern "system" fn(HWND) -> HWND;
type Fn_ImmIsIME = unsafe extern "system" fn(HKL) -> BOOL;
type Fn_ImmReleaseContext = unsafe extern "system" fn(HWND, HIMC) -> BOOL;
type Fn_ImmRegisterClient = unsafe extern "system" fn(LPVOID, HINSTANCE) -> BOOL;
type Fn_ImmGetCompositionFontW = unsafe extern "system" fn(HIMC, LPLOGFONTW) -> BOOL;
type Fn_ImmGetCompositionFontA = unsafe extern "system" fn(HIMC, LPLOGFONTA) -> BOOL;
type Fn_ImmSetCompositionFontW = unsafe extern "system" fn(HIMC, LPLOGFONTW) -> BOOL;
type Fn_ImmSetCompositionFontA = unsafe extern "system" fn(HIMC, LPLOGFONTA) -> BOOL;
type Fn_ImmSetCompositionWindow = unsafe extern "system" fn(HIMC, LPCOMPOSITIONFORM) -> BOOL;
type Fn_ImmNotifyIME = unsafe extern "system" fn(HIMC, DWORD, DWORD, DWORD_PTR) -> BOOL;
type Fn_ImmLockIMC = unsafe extern "system" fn(HIMC) -> LPVOID;
type Fn_ImmUnlockIMC = unsafe extern "system" fn(HIMC) -> BOOL;
type Fn_ImmLoadIME = unsafe extern "system" fn(HKL) -> BOOL;
type Fn_ImmSetOpenStatus = unsafe extern "system" fn(HIMC, BOOL) -> BOOL;
type Fn_ImmFreeLayout = unsafe extern "system" fn(HKL) -> BOOL;
type Fn_ImmActivateLayout = unsafe extern "system" fn(HKL) -> BOOL;
type Fn_ImmGetCandidateWindow = unsafe extern "system" fn(HIMC, DWORD, LPCANDIDATEFORM) -> BOOL;
type Fn_ImmSetCandidateWindow = unsafe extern "system" fn(HIMC, LPCANDIDATEFORM) -> BOOL;
type Fn_ImmConfigureIMEW = unsafe extern "system" fn(HKL, HWND, DWORD, LPVOID) -> BOOL;
type Fn_ImmGetConversionStatus = unsafe extern "system" fn(HIMC, LPDWORD, LPDWORD) -> BOOL;
type Fn_ImmSetConversionStatus = unsafe extern "system" fn(HIMC, DWORD, DWORD) -> BOOL;
type Fn_ImmSetStatusWindowPos = unsafe extern "system" fn(HIMC, LPPOINT) -> BOOL;
type Fn_ImmGetImeInfoEx = unsafe extern "system" fn(PIMEINFOEX, IMEINFOEXCLASS, LPVOID) -> BOOL;
type Fn_ImmLockImeDpi = unsafe extern "system" fn(HKL) -> LPVOID;
type Fn_ImmUnlockImeDpi = unsafe extern "system" fn(LPVOID);
type Fn_ImmGetOpenStatus = unsafe extern "system" fn(HIMC) -> BOOL;
type Fn_ImmSetActiveContext = unsafe extern "system" fn(HWND, HIMC, BOOL) -> BOOL;
type Fn_ImmTranslateMessage = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> BOOL;
type Fn_ImmLoadLayout = unsafe extern "system" fn(HKL, PIMEINFOEX) -> BOOL;
type Fn_ImmProcessKey = unsafe extern "system" fn(HWND, HKL, UINT, LPARAM, DWORD) -> DWORD;
type Fn_ImmPutImeMenuItemsIntoMappedFile = unsafe extern "system" fn(HIMC) -> LRESULT;
type Fn_ImmGetProperty = unsafe extern "system" fn(HKL, DWORD) -> DWORD;
type Fn_ImmSetCompositionStringA = unsafe extern "system" fn(HIMC, DWORD, LPCVOID, DWORD, LPCVOID, DWORD) -> BOOL;
type Fn_ImmSetCompositionStringW = unsafe extern "system" fn(HIMC, DWORD, LPCVOID, DWORD, LPCVOID, DWORD) -> BOOL;
type Fn_ImmEnumInputContext = unsafe extern "system" fn(DWORD, IMCENUMPROC, LPARAM) -> BOOL;
type Fn_ImmSystemHandler = unsafe extern "system" fn(HIMC, WPARAM, LPARAM) -> LRESULT;
type Fn_CtfImmTIMActivate = unsafe extern "system" fn(HKL) -> HRESULT;
type Fn_CtfImmRestoreToolbarWnd = unsafe extern "system" fn(DWORD);
type Fn_CtfImmHideToolbarWnd = unsafe extern "system" fn() -> DWORD;
type Fn_CtfImmDispatchDefImeMessage = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

// stub functions (IMMSTUB_##name) — return the table's default (0 / FALSE / void).
unsafe extern "system" fn IMMSTUB_ImmWINNLSEnableIME(_: HWND, _: BOOL) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmWINNLSGetEnableStatus(_: HWND) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSendIMEMessageExW(_: HWND, _: LPARAM) -> LRESULT { 0 }
unsafe extern "system" fn IMMSTUB_ImmSendIMEMessageExA(_: HWND, _: LPARAM) -> LRESULT { 0 }
unsafe extern "system" fn IMMSTUB_ImmIMPGetIMEW(_: HWND, _: LPIMEPROW) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmIMPGetIMEA(_: HWND, _: LPIMEPROA) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmIMPQueryIMEW(_: LPIMEPROW) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmIMPQueryIMEA(_: LPIMEPROA) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmIMPSetIMEW(_: HWND, _: LPIMEPROW) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmIMPSetIMEA(_: HWND, _: LPIMEPROA) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmAssociateContext(_: HWND, _: HIMC) -> HIMC { core::ptr::null_mut() }
unsafe extern "system" fn IMMSTUB_ImmEscapeA(_: HKL, _: HIMC, _: UINT, _: LPVOID) -> LRESULT { 0 }
unsafe extern "system" fn IMMSTUB_ImmEscapeW(_: HKL, _: HIMC, _: UINT, _: LPVOID) -> LRESULT { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetCompositionStringA(_: HIMC, _: DWORD, _: LPVOID, _: DWORD) -> LONG { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetCompositionStringW(_: HIMC, _: DWORD, _: LPVOID, _: DWORD) -> LONG { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetCompositionWindow(_: HIMC, _: LPCOMPOSITIONFORM) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetContext(_: HWND) -> HIMC { core::ptr::null_mut() }
unsafe extern "system" fn IMMSTUB_ImmGetDefaultIMEWnd(_: HWND) -> HWND { core::ptr::null_mut() }
unsafe extern "system" fn IMMSTUB_ImmIsIME(_: HKL) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmReleaseContext(_: HWND, _: HIMC) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmRegisterClient(_: LPVOID, _: HINSTANCE) -> BOOL { 1 /* TRUE */ }
unsafe extern "system" fn IMMSTUB_ImmGetCompositionFontW(_: HIMC, _: LPLOGFONTW) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetCompositionFontA(_: HIMC, _: LPLOGFONTA) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetCompositionFontW(_: HIMC, _: LPLOGFONTW) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetCompositionFontA(_: HIMC, _: LPLOGFONTA) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetCompositionWindow(_: HIMC, _: LPCOMPOSITIONFORM) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmNotifyIME(_: HIMC, _: DWORD, _: DWORD, _: DWORD_PTR) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmLockIMC(_: HIMC) -> LPVOID { core::ptr::null_mut() }
unsafe extern "system" fn IMMSTUB_ImmUnlockIMC(_: HIMC) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmLoadIME(_: HKL) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetOpenStatus(_: HIMC, _: BOOL) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmFreeLayout(_: HKL) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmActivateLayout(_: HKL) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetCandidateWindow(_: HIMC, _: DWORD, _: LPCANDIDATEFORM) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetCandidateWindow(_: HIMC, _: LPCANDIDATEFORM) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmConfigureIMEW(_: HKL, _: HWND, _: DWORD, _: LPVOID) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetConversionStatus(_: HIMC, _: LPDWORD, _: LPDWORD) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetConversionStatus(_: HIMC, _: DWORD, _: DWORD) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetStatusWindowPos(_: HIMC, _: LPPOINT) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetImeInfoEx(_: PIMEINFOEX, _: IMEINFOEXCLASS, _: LPVOID) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmLockImeDpi(_: HKL) -> LPVOID { core::ptr::null_mut() }
unsafe extern "system" fn IMMSTUB_ImmUnlockImeDpi(_: LPVOID) {}
unsafe extern "system" fn IMMSTUB_ImmGetOpenStatus(_: HIMC) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetActiveContext(_: HWND, _: HIMC, _: BOOL) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmTranslateMessage(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmLoadLayout(_: HKL, _: PIMEINFOEX) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmProcessKey(_: HWND, _: HKL, _: UINT, _: LPARAM, _: DWORD) -> DWORD { 0 }
unsafe extern "system" fn IMMSTUB_ImmPutImeMenuItemsIntoMappedFile(_: HIMC) -> LRESULT { 0 }
unsafe extern "system" fn IMMSTUB_ImmGetProperty(_: HKL, _: DWORD) -> DWORD { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetCompositionStringA(_: HIMC, _: DWORD, _: LPCVOID, _: DWORD, _: LPCVOID, _: DWORD) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSetCompositionStringW(_: HIMC, _: DWORD, _: LPCVOID, _: DWORD, _: LPCVOID, _: DWORD) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmEnumInputContext(_: DWORD, _: IMCENUMPROC, _: LPARAM) -> BOOL { 0 }
unsafe extern "system" fn IMMSTUB_ImmSystemHandler(_: HIMC, _: WPARAM, _: LPARAM) -> LRESULT { 0 }
unsafe extern "system" fn IMMSTUB_CtfImmTIMActivate(_: HKL) -> HRESULT { 0 }
unsafe extern "system" fn IMMSTUB_CtfImmRestoreToolbarWnd(_: DWORD) {}
unsafe extern "system" fn IMMSTUB_CtfImmHideToolbarWnd() -> DWORD { 0 }
unsafe extern "system" fn IMMSTUB_CtfImmDispatchDefImeMessage(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT { 0 }

#[repr(C)]
pub struct Imm32ApiTable {
    pub ImmWINNLSEnableIME: Fn_ImmWINNLSEnableIME,
    pub ImmWINNLSGetEnableStatus: Fn_ImmWINNLSGetEnableStatus,
    pub ImmSendIMEMessageExW: Fn_ImmSendIMEMessageExW,
    pub ImmSendIMEMessageExA: Fn_ImmSendIMEMessageExA,
    pub ImmIMPGetIMEW: Fn_ImmIMPGetIMEW,
    pub ImmIMPGetIMEA: Fn_ImmIMPGetIMEA,
    pub ImmIMPQueryIMEW: Fn_ImmIMPQueryIMEW,
    pub ImmIMPQueryIMEA: Fn_ImmIMPQueryIMEA,
    pub ImmIMPSetIMEW: Fn_ImmIMPSetIMEW,
    pub ImmIMPSetIMEA: Fn_ImmIMPSetIMEA,
    pub ImmAssociateContext: Fn_ImmAssociateContext,
    pub ImmEscapeA: Fn_ImmEscapeA,
    pub ImmEscapeW: Fn_ImmEscapeW,
    pub ImmGetCompositionStringA: Fn_ImmGetCompositionStringA,
    pub ImmGetCompositionStringW: Fn_ImmGetCompositionStringW,
    pub ImmGetCompositionWindow: Fn_ImmGetCompositionWindow,
    pub ImmGetContext: Fn_ImmGetContext,
    pub ImmGetDefaultIMEWnd: Fn_ImmGetDefaultIMEWnd,
    pub ImmIsIME: Fn_ImmIsIME,
    pub ImmReleaseContext: Fn_ImmReleaseContext,
    pub ImmRegisterClient: Fn_ImmRegisterClient,
    pub ImmGetCompositionFontW: Fn_ImmGetCompositionFontW,
    pub ImmGetCompositionFontA: Fn_ImmGetCompositionFontA,
    pub ImmSetCompositionFontW: Fn_ImmSetCompositionFontW,
    pub ImmSetCompositionFontA: Fn_ImmSetCompositionFontA,
    pub ImmSetCompositionWindow: Fn_ImmSetCompositionWindow,
    pub ImmNotifyIME: Fn_ImmNotifyIME,
    pub ImmLockIMC: Fn_ImmLockIMC,
    pub ImmUnlockIMC: Fn_ImmUnlockIMC,
    pub ImmLoadIME: Fn_ImmLoadIME,
    pub ImmSetOpenStatus: Fn_ImmSetOpenStatus,
    pub ImmFreeLayout: Fn_ImmFreeLayout,
    pub ImmActivateLayout: Fn_ImmActivateLayout,
    pub ImmGetCandidateWindow: Fn_ImmGetCandidateWindow,
    pub ImmSetCandidateWindow: Fn_ImmSetCandidateWindow,
    pub ImmConfigureIMEW: Fn_ImmConfigureIMEW,
    pub ImmGetConversionStatus: Fn_ImmGetConversionStatus,
    pub ImmSetConversionStatus: Fn_ImmSetConversionStatus,
    pub ImmSetStatusWindowPos: Fn_ImmSetStatusWindowPos,
    pub ImmGetImeInfoEx: Fn_ImmGetImeInfoEx,
    pub ImmLockImeDpi: Fn_ImmLockImeDpi,
    pub ImmUnlockImeDpi: Fn_ImmUnlockImeDpi,
    pub ImmGetOpenStatus: Fn_ImmGetOpenStatus,
    pub ImmSetActiveContext: Fn_ImmSetActiveContext,
    pub ImmTranslateMessage: Fn_ImmTranslateMessage,
    pub ImmLoadLayout: Fn_ImmLoadLayout,
    pub ImmProcessKey: Fn_ImmProcessKey,
    pub ImmPutImeMenuItemsIntoMappedFile: Fn_ImmPutImeMenuItemsIntoMappedFile,
    pub ImmGetProperty: Fn_ImmGetProperty,
    pub ImmSetCompositionStringA: Fn_ImmSetCompositionStringA,
    pub ImmSetCompositionStringW: Fn_ImmSetCompositionStringW,
    pub ImmEnumInputContext: Fn_ImmEnumInputContext,
    pub ImmSystemHandler: Fn_ImmSystemHandler,
    pub CtfImmTIMActivate: Fn_CtfImmTIMActivate,
    pub CtfImmRestoreToolbarWnd: Fn_CtfImmRestoreToolbarWnd,
    pub CtfImmHideToolbarWnd: Fn_CtfImmHideToolbarWnd,
    pub CtfImmDispatchDefImeMessage: Fn_CtfImmDispatchDefImeMessage,
}

#[no_mangle]
pub static mut gImmApiEntries: Imm32ApiTable = Imm32ApiTable {
    ImmWINNLSEnableIME: IMMSTUB_ImmWINNLSEnableIME,
    ImmWINNLSGetEnableStatus: IMMSTUB_ImmWINNLSGetEnableStatus,
    ImmSendIMEMessageExW: IMMSTUB_ImmSendIMEMessageExW,
    ImmSendIMEMessageExA: IMMSTUB_ImmSendIMEMessageExA,
    ImmIMPGetIMEW: IMMSTUB_ImmIMPGetIMEW,
    ImmIMPGetIMEA: IMMSTUB_ImmIMPGetIMEA,
    ImmIMPQueryIMEW: IMMSTUB_ImmIMPQueryIMEW,
    ImmIMPQueryIMEA: IMMSTUB_ImmIMPQueryIMEA,
    ImmIMPSetIMEW: IMMSTUB_ImmIMPSetIMEW,
    ImmIMPSetIMEA: IMMSTUB_ImmIMPSetIMEA,
    ImmAssociateContext: IMMSTUB_ImmAssociateContext,
    ImmEscapeA: IMMSTUB_ImmEscapeA,
    ImmEscapeW: IMMSTUB_ImmEscapeW,
    ImmGetCompositionStringA: IMMSTUB_ImmGetCompositionStringA,
    ImmGetCompositionStringW: IMMSTUB_ImmGetCompositionStringW,
    ImmGetCompositionWindow: IMMSTUB_ImmGetCompositionWindow,
    ImmGetContext: IMMSTUB_ImmGetContext,
    ImmGetDefaultIMEWnd: IMMSTUB_ImmGetDefaultIMEWnd,
    ImmIsIME: IMMSTUB_ImmIsIME,
    ImmReleaseContext: IMMSTUB_ImmReleaseContext,
    ImmRegisterClient: IMMSTUB_ImmRegisterClient,
    ImmGetCompositionFontW: IMMSTUB_ImmGetCompositionFontW,
    ImmGetCompositionFontA: IMMSTUB_ImmGetCompositionFontA,
    ImmSetCompositionFontW: IMMSTUB_ImmSetCompositionFontW,
    ImmSetCompositionFontA: IMMSTUB_ImmSetCompositionFontA,
    ImmSetCompositionWindow: IMMSTUB_ImmSetCompositionWindow,
    ImmNotifyIME: IMMSTUB_ImmNotifyIME,
    ImmLockIMC: IMMSTUB_ImmLockIMC,
    ImmUnlockIMC: IMMSTUB_ImmUnlockIMC,
    ImmLoadIME: IMMSTUB_ImmLoadIME,
    ImmSetOpenStatus: IMMSTUB_ImmSetOpenStatus,
    ImmFreeLayout: IMMSTUB_ImmFreeLayout,
    ImmActivateLayout: IMMSTUB_ImmActivateLayout,
    ImmGetCandidateWindow: IMMSTUB_ImmGetCandidateWindow,
    ImmSetCandidateWindow: IMMSTUB_ImmSetCandidateWindow,
    ImmConfigureIMEW: IMMSTUB_ImmConfigureIMEW,
    ImmGetConversionStatus: IMMSTUB_ImmGetConversionStatus,
    ImmSetConversionStatus: IMMSTUB_ImmSetConversionStatus,
    ImmSetStatusWindowPos: IMMSTUB_ImmSetStatusWindowPos,
    ImmGetImeInfoEx: IMMSTUB_ImmGetImeInfoEx,
    ImmLockImeDpi: IMMSTUB_ImmLockImeDpi,
    ImmUnlockImeDpi: IMMSTUB_ImmUnlockImeDpi,
    ImmGetOpenStatus: IMMSTUB_ImmGetOpenStatus,
    ImmSetActiveContext: IMMSTUB_ImmSetActiveContext,
    ImmTranslateMessage: IMMSTUB_ImmTranslateMessage,
    ImmLoadLayout: IMMSTUB_ImmLoadLayout,
    ImmProcessKey: IMMSTUB_ImmProcessKey,
    ImmPutImeMenuItemsIntoMappedFile: IMMSTUB_ImmPutImeMenuItemsIntoMappedFile,
    ImmGetProperty: IMMSTUB_ImmGetProperty,
    ImmSetCompositionStringA: IMMSTUB_ImmSetCompositionStringA,
    ImmSetCompositionStringW: IMMSTUB_ImmSetCompositionStringW,
    ImmEnumInputContext: IMMSTUB_ImmEnumInputContext,
    ImmSystemHandler: IMMSTUB_ImmSystemHandler,
    CtfImmTIMActivate: IMMSTUB_CtfImmTIMActivate,
    CtfImmRestoreToolbarWnd: IMMSTUB_CtfImmRestoreToolbarWnd,
    CtfImmHideToolbarWnd: IMMSTUB_CtfImmHideToolbarWnd,
    CtfImmDispatchDefImeMessage: IMMSTUB_CtfImmDispatchDefImeMessage,
};

// user32p.h: #define IMM_FN(name) gImmApiEntries.p##name
// (Rust macros can't `##`-concat an ident into a field name on stable, so the table
//  fields are stored without the `p` prefix and this expands to `gImmApiEntries.name`.)
macro_rules! IMM_FN {
    ($name:ident) => {
        gImmApiEntries.$name
    };
}

// ---- file-scope preamble (translated) ----
pub const IMM_INIT_MAGIC: u32 = 0x19650412;

pub const MAX_CANDIDATEFORM: u32 = 4;

pub static mut ghImm32: HINSTANCE = core::ptr::null_mut();
pub static mut gbImmInitializing: BOOL = 0;
pub static mut gfConIme: INT = -1;

pub type FN_GetPenMenuData = core::option::Option<unsafe extern "system" fn(pnID: PUINT, pdwMenuData: PDWORD_PTR)>;
pub static mut s_pGetPenMenuData: FN_GetPenMenuData = None;

pub const IFN_GetPenMenuData: u32 = 12;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetTopLevelWindow(mut hWnd: HWND) -> HWND {
    let mut style: DWORD;

    while !hWnd.is_null() {
        style = GetWindowLongPtrW(hWnd, GWL_STYLE as _) as DWORD;
        if (style & WS_CHILD) == 0 {
            break;
        }
        hWnd = GetParent(hWnd);
    }

    hWnd
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetTopLevelWindow(mut hWnd: HWND) -> HWND {
    let mut style: DWORD;

    while !hWnd.is_null() {
        style = GetWindowLongPtrW(hWnd, GWL_STYLE as _) as DWORD;
        if (style & WS_CHILD) == 0 {
            break;
        }
        hWnd = GetParent(hWnd);
    }

    hWnd
}

unsafe fn User32GetSystemFilePath(
    lpBuffer: PWSTR,
    cchBuffer: SIZE_T,
    pszFileName: PCWSTR,
) -> HRESULT {
    let length: UINT = GetSystemDirectoryW(lpBuffer, cchBuffer as UINT);
    if length != 0 && (length as SIZE_T) < cchBuffer {
        StringCchCatW(lpBuffer, cchBuffer, w!("\\").as_ptr() as PCWSTR);
        return StringCchCatW(lpBuffer, cchBuffer, pszFileName);
    }
    let dwError: DWORD = GetLastError();
    StringCchCopyW(lpBuffer, cchBuffer, pszFileName);
    HRESULT_FROM_WIN32(dwError)
}

unsafe fn IntInitializeImmEntryTable() -> BOOL {
    let mut ImmFile: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut imm32: HMODULE = ghImm32;

    /* Check whether the IMM table has already been initialized */
    if IMM_FN!(ImmWINNLSEnableIME) as usize != IMMSTUB_ImmWINNLSEnableIME as usize {
        return TRUE;
    }

    User32GetSystemFilePath(
        ImmFile.as_mut_ptr(),
        ImmFile.len() as SIZE_T,
        w!("imm32.dll").as_ptr() as PCWSTR,
    );
    TRACE!(b"File %s\n\0".as_ptr(), debugstr_w(ImmFile.as_ptr()));

    /* If IMM32 is already loaded, use it without increasing reference count. */
    if imm32.is_null() {
        imm32 = GetModuleHandleW(ImmFile.as_ptr());
    }

    /*
     * Loading imm32.dll will call imm32!DllMain function.
     * imm32!DllMain calls User32InitializeImmEntryTable.
     * Thus, if imm32.dll was loaded, the table has been loaded.
     */
    if imm32.is_null() {
        ghImm32 = LoadLibraryW(ImmFile.as_ptr());
        imm32 = ghImm32;
        if imm32.is_null() {
            ERR!("Did not load imm32.dll!\n");
            return FALSE;
        }
        return TRUE;
    }

    /* load imm procedures (immtable.h DEFINE_IMM_ENTRY load-loop) */
    macro_rules! LOAD_IMM_PROC {
        ($field:ident, $ty:ty, $name:literal) => {{
            let proc = GetProcAddress(imm32, $name.as_ptr() as LPCSTR);
            if proc.is_none() {
                ERR!("Could not load proc\n");
                return FALSE;
            }
            gImmApiEntries.$field = core::mem::transmute::<_, $ty>(proc);
        }};
    }

    LOAD_IMM_PROC!(ImmWINNLSEnableIME, Fn_ImmWINNLSEnableIME, b"ImmWINNLSEnableIME\0");
    LOAD_IMM_PROC!(ImmWINNLSGetEnableStatus, Fn_ImmWINNLSGetEnableStatus, b"ImmWINNLSGetEnableStatus\0");
    LOAD_IMM_PROC!(ImmSendIMEMessageExW, Fn_ImmSendIMEMessageExW, b"ImmSendIMEMessageExW\0");
    LOAD_IMM_PROC!(ImmSendIMEMessageExA, Fn_ImmSendIMEMessageExA, b"ImmSendIMEMessageExA\0");
    LOAD_IMM_PROC!(ImmIMPGetIMEW, Fn_ImmIMPGetIMEW, b"ImmIMPGetIMEW\0");
    LOAD_IMM_PROC!(ImmIMPGetIMEA, Fn_ImmIMPGetIMEA, b"ImmIMPGetIMEA\0");
    LOAD_IMM_PROC!(ImmIMPQueryIMEW, Fn_ImmIMPQueryIMEW, b"ImmIMPQueryIMEW\0");
    LOAD_IMM_PROC!(ImmIMPQueryIMEA, Fn_ImmIMPQueryIMEA, b"ImmIMPQueryIMEA\0");
    LOAD_IMM_PROC!(ImmIMPSetIMEW, Fn_ImmIMPSetIMEW, b"ImmIMPSetIMEW\0");
    LOAD_IMM_PROC!(ImmIMPSetIMEA, Fn_ImmIMPSetIMEA, b"ImmIMPSetIMEA\0");
    LOAD_IMM_PROC!(ImmAssociateContext, Fn_ImmAssociateContext, b"ImmAssociateContext\0");
    LOAD_IMM_PROC!(ImmEscapeA, Fn_ImmEscapeA, b"ImmEscapeA\0");
    LOAD_IMM_PROC!(ImmEscapeW, Fn_ImmEscapeW, b"ImmEscapeW\0");
    LOAD_IMM_PROC!(ImmGetCompositionStringA, Fn_ImmGetCompositionStringA, b"ImmGetCompositionStringA\0");
    LOAD_IMM_PROC!(ImmGetCompositionStringW, Fn_ImmGetCompositionStringW, b"ImmGetCompositionStringW\0");
    LOAD_IMM_PROC!(ImmGetCompositionWindow, Fn_ImmGetCompositionWindow, b"ImmGetCompositionWindow\0");
    LOAD_IMM_PROC!(ImmGetContext, Fn_ImmGetContext, b"ImmGetContext\0");
    LOAD_IMM_PROC!(ImmGetDefaultIMEWnd, Fn_ImmGetDefaultIMEWnd, b"ImmGetDefaultIMEWnd\0");
    LOAD_IMM_PROC!(ImmIsIME, Fn_ImmIsIME, b"ImmIsIME\0");
    LOAD_IMM_PROC!(ImmReleaseContext, Fn_ImmReleaseContext, b"ImmReleaseContext\0");
    LOAD_IMM_PROC!(ImmRegisterClient, Fn_ImmRegisterClient, b"ImmRegisterClient\0");
    LOAD_IMM_PROC!(ImmGetCompositionFontW, Fn_ImmGetCompositionFontW, b"ImmGetCompositionFontW\0");
    LOAD_IMM_PROC!(ImmGetCompositionFontA, Fn_ImmGetCompositionFontA, b"ImmGetCompositionFontA\0");
    LOAD_IMM_PROC!(ImmSetCompositionFontW, Fn_ImmSetCompositionFontW, b"ImmSetCompositionFontW\0");
    LOAD_IMM_PROC!(ImmSetCompositionFontA, Fn_ImmSetCompositionFontA, b"ImmSetCompositionFontA\0");
    LOAD_IMM_PROC!(ImmSetCompositionWindow, Fn_ImmSetCompositionWindow, b"ImmSetCompositionWindow\0");
    LOAD_IMM_PROC!(ImmNotifyIME, Fn_ImmNotifyIME, b"ImmNotifyIME\0");
    LOAD_IMM_PROC!(ImmLockIMC, Fn_ImmLockIMC, b"ImmLockIMC\0");
    LOAD_IMM_PROC!(ImmUnlockIMC, Fn_ImmUnlockIMC, b"ImmUnlockIMC\0");
    LOAD_IMM_PROC!(ImmLoadIME, Fn_ImmLoadIME, b"ImmLoadIME\0");
    LOAD_IMM_PROC!(ImmSetOpenStatus, Fn_ImmSetOpenStatus, b"ImmSetOpenStatus\0");
    LOAD_IMM_PROC!(ImmFreeLayout, Fn_ImmFreeLayout, b"ImmFreeLayout\0");
    LOAD_IMM_PROC!(ImmActivateLayout, Fn_ImmActivateLayout, b"ImmActivateLayout\0");
    LOAD_IMM_PROC!(ImmGetCandidateWindow, Fn_ImmGetCandidateWindow, b"ImmGetCandidateWindow\0");
    LOAD_IMM_PROC!(ImmSetCandidateWindow, Fn_ImmSetCandidateWindow, b"ImmSetCandidateWindow\0");
    LOAD_IMM_PROC!(ImmConfigureIMEW, Fn_ImmConfigureIMEW, b"ImmConfigureIMEW\0");
    LOAD_IMM_PROC!(ImmGetConversionStatus, Fn_ImmGetConversionStatus, b"ImmGetConversionStatus\0");
    LOAD_IMM_PROC!(ImmSetConversionStatus, Fn_ImmSetConversionStatus, b"ImmSetConversionStatus\0");
    LOAD_IMM_PROC!(ImmSetStatusWindowPos, Fn_ImmSetStatusWindowPos, b"ImmSetStatusWindowPos\0");
    LOAD_IMM_PROC!(ImmGetImeInfoEx, Fn_ImmGetImeInfoEx, b"ImmGetImeInfoEx\0");
    LOAD_IMM_PROC!(ImmLockImeDpi, Fn_ImmLockImeDpi, b"ImmLockImeDpi\0");
    LOAD_IMM_PROC!(ImmUnlockImeDpi, Fn_ImmUnlockImeDpi, b"ImmUnlockImeDpi\0");
    LOAD_IMM_PROC!(ImmGetOpenStatus, Fn_ImmGetOpenStatus, b"ImmGetOpenStatus\0");
    LOAD_IMM_PROC!(ImmSetActiveContext, Fn_ImmSetActiveContext, b"ImmSetActiveContext\0");
    LOAD_IMM_PROC!(ImmTranslateMessage, Fn_ImmTranslateMessage, b"ImmTranslateMessage\0");
    LOAD_IMM_PROC!(ImmLoadLayout, Fn_ImmLoadLayout, b"ImmLoadLayout\0");
    LOAD_IMM_PROC!(ImmProcessKey, Fn_ImmProcessKey, b"ImmProcessKey\0");
    LOAD_IMM_PROC!(ImmPutImeMenuItemsIntoMappedFile, Fn_ImmPutImeMenuItemsIntoMappedFile, b"ImmPutImeMenuItemsIntoMappedFile\0");
    LOAD_IMM_PROC!(ImmGetProperty, Fn_ImmGetProperty, b"ImmGetProperty\0");
    LOAD_IMM_PROC!(ImmSetCompositionStringA, Fn_ImmSetCompositionStringA, b"ImmSetCompositionStringA\0");
    LOAD_IMM_PROC!(ImmSetCompositionStringW, Fn_ImmSetCompositionStringW, b"ImmSetCompositionStringW\0");
    LOAD_IMM_PROC!(ImmEnumInputContext, Fn_ImmEnumInputContext, b"ImmEnumInputContext\0");
    LOAD_IMM_PROC!(ImmSystemHandler, Fn_ImmSystemHandler, b"ImmSystemHandler\0");
    LOAD_IMM_PROC!(CtfImmTIMActivate, Fn_CtfImmTIMActivate, b"CtfImmTIMActivate\0");
    LOAD_IMM_PROC!(CtfImmRestoreToolbarWnd, Fn_CtfImmRestoreToolbarWnd, b"CtfImmRestoreToolbarWnd\0");
    LOAD_IMM_PROC!(CtfImmHideToolbarWnd, Fn_CtfImmHideToolbarWnd, b"CtfImmHideToolbarWnd\0");
    LOAD_IMM_PROC!(CtfImmDispatchDefImeMessage, Fn_CtfImmDispatchDefImeMessage, b"CtfImmDispatchDefImeMessage\0");

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn InitializeImmEntryTable() -> BOOL {
    gbImmInitializing = 1;
    IntInitializeImmEntryTable()
}
#[no_mangle]
pub unsafe extern "system" fn User32InitializeImmEntryTable(magic: DWORD) -> BOOL {
    if magic != IMM_INIT_MAGIC {
        return 0;
    }

    if IMM_FN!(ImmWINNLSEnableIME) as usize != IMMSTUB_ImmWINNLSEnableIME as usize {
        return 1;
    }

    IntInitializeImmEntryTable();

    if ghImm32.is_null() && gbImmInitializing == 0 {
        let mut ImmFile: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        User32GetSystemFilePath(
            ImmFile.as_mut_ptr(),
            ImmFile.len() as SIZE_T,
            w!("imm32.dll").as_ptr() as PCWSTR,
        );
        ghImm32 = LoadLibraryW(ImmFile.as_ptr());
        if ghImm32.is_null() {
            return 0;
        }
    }

    IMM_FN!(ImmRegisterClient)(core::ptr::addr_of_mut!(gSharedInfo) as LPVOID, ghImm32)
}

unsafe fn User32CanSetImeWindowToImc(hIMC: HIMC, hImeWnd: HWND) -> BOOL {
    let pIMC: PIMC = ValidateHandle(hIMC as HANDLE, TYPE_INPUTCONTEXT as _) as PIMC;
    (!pIMC.is_null()
        && ((*pIMC).hImeWnd.is_null()
            || (*pIMC).hImeWnd == hImeWnd
            || ValidateHwnd((*pIMC).hImeWnd).is_null())) as BOOL
}

unsafe fn User32GetImeShowStatus() -> BOOL {
    NtUserCallNoParam(NOPARAM_ROUTINE_GETIMESHOWSTATUS) as BOOL
}

unsafe fn User32SendImeUIMessage(
    pimeui: PIMEUI,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let mut ret: LRESULT = 0;
    let hwndUI = (*pimeui).hwndUI;
    let pwnd: PWND;
    let pwndUI: PWND;

    pwnd = (*pimeui).spwnd;
    pwndUI = ValidateHwnd(hwndUI);
    if pwnd.is_null()
        || ((*pwnd).state & WNDS_DESTROYED) != 0
        || ((*pwnd).state2 & WNDS2_INDESTROY) != 0
        || pwndUI.is_null()
        || ((*pwndUI).state & WNDS_DESTROYED) != 0
        || ((*pwndUI).state2 & WNDS2_INDESTROY) != 0
    {
        return 0;
    }

    InterlockedIncrement(core::ptr::addr_of_mut!((*pimeui).nCntInIMEProc));

    if unicode != 0 {
        ret = SendMessageW(hwndUI, uMsg, wParam, lParam);
    } else {
        ret = SendMessageA(hwndUI, uMsg, wParam, lParam);
    }

    InterlockedDecrement(core::ptr::addr_of_mut!((*pimeui).nCntInIMEProc));

    ret
}

unsafe fn User32NotifyOpenStatus(pimeui: PIMEUI, hwndIMC: HWND, bOpen: BOOL) {
    let wParam: WPARAM = if bOpen != 0 {
        IMN_OPENSTATUSWINDOW as WPARAM
    } else {
        IMN_CLOSESTATUSWINDOW as WPARAM
    };

    (*pimeui).__bindgen_anon_1.set_fShowStatus((bOpen) as UINT);

    if LOWORD((*GetWin32ClientInfo()).dwExpWinVer) >= 0x400 {
        SendMessageW(hwndIMC, WM_IME_NOTIFY, wParam, 0);
    } else {
        User32SendImeUIMessage(pimeui, WM_IME_NOTIFY, wParam, 0, TRUE as BOOL);
    }
}

unsafe fn User32SetImeWindowOfImc(hIMC: HIMC, hImeWnd: HWND) {
    let pIMC: PIMC = ValidateHandle(hIMC as HANDLE, TYPE_INPUTCONTEXT as _) as PIMC;
    if pIMC.is_null() || (*pIMC).hImeWnd == hImeWnd {
        return;
    }

    NtUserUpdateInputContext(hIMC, UIC_IMEWINDOW, hImeWnd as ULONG_PTR);
}

unsafe fn User32UpdateImcOfImeUI(pimeui: PIMEUI, hNewIMC: HIMC) {
    let hImeWnd: HWND;
    let hOldIMC: HIMC = (*pimeui).hIMC;

    hImeWnd = UserHMGetHandle((*pimeui).spwnd);

    if hNewIMC == hOldIMC {
        return;
    }

    if !hOldIMC.is_null() {
        User32SetImeWindowOfImc(hOldIMC, core::ptr::null_mut());
    }

    (*pimeui).hIMC = hNewIMC;

    if !hNewIMC.is_null() {
        User32SetImeWindowOfImc(hNewIMC, hImeWnd);
    }
}

unsafe fn ImeWnd_OnImeNotify(pimeui: PIMEUI, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut ret: LRESULT = 0;
    let hIMC: HIMC;
    let pIC: LPINPUTCONTEXT;
    let hwndUI: HWND;
    let hwndIMC: HWND;
    let hImeWnd: HWND;
    let hwndOwner: HWND;

    match wParam as u32 {
        IMN_SETCONVERSIONMODE | IMN_SETOPENSTATUS => {
            hIMC = (*pimeui).hIMC;
            pIC = IMM_FN!(ImmLockIMC)(hIMC) as LPINPUTCONTEXT;
            if !pIC.is_null() {
                hwndIMC = (*pimeui).hwndIMC;
                if IsWindow(hwndIMC) != 0 {
                    NtUserNotifyIMEStatus(hwndIMC, (*pIC).fOpen, (*pIC).fdwConversion);
                } else if gfConIme == 1 /* TRUE */ && !(*pimeui).spwnd.is_null() {
                    hImeWnd = UserHMGetHandle((*pimeui).spwnd);
                    hwndOwner = GetWindow(hImeWnd, GW_OWNER as UINT);
                    if !hwndOwner.is_null() {
                        NtUserNotifyIMEStatus(hwndOwner, (*pIC).fOpen, (*pIC).fdwConversion);
                    }
                }

                IMM_FN!(ImmUnlockIMC)(hIMC);
            }
            ret = User32SendImeUIMessage(pimeui, WM_IME_NOTIFY, wParam, lParam, 1);
        }
        IMN_PRIVATE => {
            hwndUI = (*pimeui).hwndUI;
            if IsWindow(hwndUI) != 0 {
                ret = SendMessageW(hwndUI, WM_IME_NOTIFY, wParam, lParam);
            }
        }
        _ => {
            ret = User32SendImeUIMessage(pimeui, WM_IME_NOTIFY, wParam, lParam, 1);
        }
    }

    ret
}

unsafe fn User32CreateImeUIWindow(pimeui: PIMEUI, mut hKL: HKL) -> HWND {
    let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
    let pImeDpi: PIMEDPI;
    let mut wc: WNDCLASSW = core::mem::zeroed();
    let mut hwndUI: HWND = core::ptr::null_mut();
    let mut szUIClass: [CHAR; 32] = [0; 32];
    let pwnd: PWND = (*pimeui).spwnd;

    if pwnd.is_null()
        || IMM_FN!(ImmGetImeInfoEx)(
            &mut ImeInfoEx,
            ImeInfoExKeyboardLayout,
            &mut hKL as *mut HKL as LPVOID,
        ) == 0
    {
        return core::ptr::null_mut();
    }

    pImeDpi = IMM_FN!(ImmLockImeDpi)(hKL) as PIMEDPI;
    if pImeDpi.is_null() {
        return core::ptr::null_mut();
    }

    'quit: {
        if GetClassInfoW(
            (*pImeDpi).hInst,
            ImeInfoEx.wszUIClass.as_ptr(),
            &mut wc,
        ) == 0
        {
            break 'quit;
        }

        if ImeInfoEx.ImeInfo.fdwProperty & IME_PROP_UNICODE != 0 {
            hwndUI = CreateWindowW(
                ImeInfoEx.wszUIClass.as_ptr(),
                ImeInfoEx.wszUIClass.as_ptr(),
                (WS_POPUP | WS_DISABLED) as _,
                0,
                0,
                0,
                0,
                UserHMGetHandle(pwnd),
                0 as _,
                wc.hInstance,
                core::ptr::null_mut(),
            );
        } else {
            WideCharToMultiByte(
                CP_ACP,
                0,
                ImeInfoEx.wszUIClass.as_ptr(),
                -1,
                szUIClass.as_mut_ptr(),
                szUIClass.len() as _,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            szUIClass[szUIClass.len() - 1] = 0;

            hwndUI = CreateWindowA(
                szUIClass.as_ptr(),
                szUIClass.as_ptr(),
                (WS_POPUP | WS_DISABLED) as _,
                0,
                0,
                0,
                0,
                UserHMGetHandle(pwnd),
                0 as _,
                wc.hInstance,
                core::ptr::null_mut(),
            );
        }

        // C: NtUserSetWindowLongPtr, which ntuser.h aliases to NtUserSetWindowLong only on
        // x86. On 64-bit the 4-byte store truncated the HIMC and left the slot's upper
        // half as whatever it held before.
        if !hwndUI.is_null() {
            #[cfg(target_arch = "x86")]
            NtUserSetWindowLong(
                hwndUI,
                IMMGWLP_IMC,
                (*pimeui).hIMC as LONG,
                FALSE,
            );
            #[cfg(not(target_arch = "x86"))]
            NtUserSetWindowLongPtr(
                hwndUI,
                IMMGWLP_IMC,
                (*pimeui).hIMC as LONG_PTR,
                FALSE,
            );
        }
    }

    IMM_FN!(ImmUnlockImeDpi)(pImeDpi as LPVOID);
    hwndUI
}

unsafe fn ImeWnd_OnCreate(pimeui: PIMEUI, lpCS: LPCREATESTRUCT) -> INT {
    let pWnd: PWND = (*pimeui).spwnd;
    let pParentWnd: PWND;
    let mut hIMC: HIMC = core::ptr::null_mut();

    if pWnd.is_null()
        || ((*pWnd).style & (WS_DISABLED | WS_POPUP)) != (WS_DISABLED | WS_POPUP)
    {
        return -1;
    }

    pParentWnd = ValidateHwnd((*lpCS).hwndParent);
    if !pParentWnd.is_null() {
        hIMC = (*pParentWnd).hImc;
        if !hIMC.is_null() && User32CanSetImeWindowToImc(hIMC, UserHMGetHandle(pWnd)) == 0 {
            hIMC = core::ptr::null_mut();
        }
    }

    User32UpdateImcOfImeUI(pimeui, hIMC);

    (*pimeui).__bindgen_anon_1.set_fShowStatus(FALSE as UINT);
    (*pimeui).nCntInIMEProc = 0;
    (*pimeui).__bindgen_anon_1.set_fActivate(FALSE as UINT);
    (*pimeui).__bindgen_anon_1.set_fDestroy(FALSE as UINT);
    (*pimeui).hwndIMC = core::ptr::null_mut();
    (*pimeui).hKL = (*GetWin32ClientInfo()).hKL;
    (*pimeui).__bindgen_anon_1.set_fCtrlShowStatus(TRUE as UINT);
    (*pimeui).dwLastStatus = 0;

    0
}

unsafe fn User32DestroyImeUIWindow(pimeui: PIMEUI) {
    let hwndUI: HWND = (*pimeui).hwndUI;

    if IsWindow(hwndUI) != 0 {
        (*pimeui).__bindgen_anon_1.set_fDestroy(1);
        NtUserDestroyWindow(hwndUI);
    }

    (*pimeui).__bindgen_anon_1.set_fDestroy(0);
    (*pimeui).__bindgen_anon_1.set_fShowStatus(0);
    (*pimeui).hwndUI = core::ptr::null_mut();
}

unsafe fn ImeWnd_OnImeSelect(pimeui: PIMEUI, wParam: WPARAM, lParam: LPARAM) {
    let hKL: HKL;
    let hwndUI: HWND;
    let hwndIMC: HWND = (*pimeui).hwndIMC;

    if wParam != 0 {
        hKL = lParam as HKL;
        (*pimeui).hKL = hKL;

        if (*pimeui).__bindgen_anon_1.fActivate() == 0 {
            return;
        }

        hwndUI = User32CreateImeUIWindow(pimeui, hKL);
        (*pimeui).hwndUI = hwndUI;
        if !hwndUI.is_null() {
            User32SendImeUIMessage(pimeui, WM_IME_SELECT, wParam, lParam, 1);
        }

        if User32GetImeShowStatus() != 0 && (*pimeui).__bindgen_anon_1.fCtrlShowStatus() != 0 {
            if (*pimeui).__bindgen_anon_1.fShowStatus() == 0
                && (*pimeui).__bindgen_anon_1.fActivate() != 0
                && IsWindow(hwndIMC) != 0
            {
                User32NotifyOpenStatus(pimeui, hwndIMC, 1);
            }
        }
    } else {
        if (*pimeui).__bindgen_anon_1.fShowStatus() != 0
            && (*pimeui).__bindgen_anon_1.fActivate() != 0
            && IsWindow(hwndIMC) != 0
        {
            User32NotifyOpenStatus(pimeui, hwndIMC, 0);
        }

        User32SendImeUIMessage(pimeui, WM_IME_SELECT, wParam, lParam, 1);
        User32DestroyImeUIWindow(pimeui);
        (*pimeui).hKL = core::ptr::null_mut();
    }
}

unsafe fn ImeWnd_OnImeControl(
    pimeui: PIMEUI,
    wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let hIMC = (*pimeui).hIMC;
    let mut dwConversion: DWORD = 0;
    let mut dwSentence: DWORD = 0;
    let mut pt: POINT = core::mem::zeroed();

    if IS_CICERO_MODE() != 0 {
        if wParam == IMC_OPENSTATUSWINDOW as WPARAM {
            IMM_FN!(CtfImmRestoreToolbarWnd)((*pimeui).dwLastStatus);
            (*pimeui).dwLastStatus = 0;
        } else if wParam == IMC_CLOSESTATUSWINDOW as WPARAM {
            (*pimeui).dwLastStatus = IMM_FN!(CtfImmHideToolbarWnd)();
        }
    }

    if hIMC.is_null() {
        return 0;
    }

    match wParam as u32 {
        IMC_GETCONVERSIONMODE => {
            if IMM_FN!(ImmGetConversionStatus)(hIMC, &mut dwConversion, &mut dwSentence) == 0 {
                return 1;
            }
            return dwConversion as LRESULT;
        }

        IMC_GETSENTENCEMODE => {
            if IMM_FN!(ImmGetConversionStatus)(hIMC, &mut dwConversion, &mut dwSentence) == 0 {
                return 1;
            }
            return dwSentence as LRESULT;
        }

        IMC_GETOPENSTATUS => {
            return IMM_FN!(ImmGetOpenStatus)(hIMC) as LRESULT;
        }

        IMC_SETCONVERSIONMODE => {
            if IMM_FN!(ImmGetConversionStatus)(hIMC, &mut dwConversion, &mut dwSentence) == 0
                || IMM_FN!(ImmSetConversionStatus)(hIMC, lParam as DWORD, dwSentence) == 0
            {
                return 1;
            }
        }

        IMC_SETSENTENCEMODE => {
            if IMM_FN!(ImmGetConversionStatus)(hIMC, &mut dwConversion, &mut dwSentence) == 0
                || IMM_FN!(ImmSetConversionStatus)(hIMC, dwConversion, lParam as DWORD) == 0
            {
                return 1;
            }
        }

        IMC_SETOPENSTATUS => {
            if IMM_FN!(ImmSetOpenStatus)(hIMC, lParam as BOOL) == 0 {
                return 1;
            }
        }

        IMC_GETCANDIDATEPOS
        | IMC_GETCOMPOSITIONWINDOW
        | IMC_GETSOFTKBDPOS
        | IMC_SETSOFTKBDPOS
        | IMC_GETSTATUSWINDOWPOS => {
            return User32SendImeUIMessage(pimeui, WM_IME_CONTROL, wParam, lParam, unicode);
        }

        IMC_SETCANDIDATEPOS => {
            if IMM_FN!(ImmSetCandidateWindow)(hIMC, lParam as LPCANDIDATEFORM) == 0 {
                return 1;
            }
        }

        IMC_GETCOMPOSITIONFONT => {
            if unicode != 0 {
                if IMM_FN!(ImmGetCompositionFontW)(hIMC, lParam as LPLOGFONTW) == 0 {
                    return 1;
                }
            } else {
                if IMM_FN!(ImmGetCompositionFontA)(hIMC, lParam as LPLOGFONTA) == 0 {
                    return 1;
                }
            }
        }

        IMC_SETCOMPOSITIONFONT => {
            if unicode != 0 {
                if IMM_FN!(ImmSetCompositionFontW)(hIMC, lParam as LPLOGFONTW) == 0 {
                    return 1;
                }
            } else {
                if IMM_FN!(ImmSetCompositionFontA)(hIMC, lParam as LPLOGFONTA) == 0 {
                    return 1;
                }
            }
        }

        IMC_SETCOMPOSITIONWINDOW => {
            if IMM_FN!(ImmSetCompositionWindow)(hIMC, lParam as LPCOMPOSITIONFORM) == 0 {
                return 1;
            }
        }

        IMC_SETSTATUSWINDOWPOS => {
            pt.x = GET_X_LPARAM(lParam);
            pt.y = GET_Y_LPARAM(lParam);
            if IMM_FN!(ImmSetStatusWindowPos)(hIMC, &mut pt) == 0 {
                return 1;
            }
        }

        IMC_CLOSESTATUSWINDOW => {
            if (*pimeui).__bindgen_anon_1.fShowStatus() != 0 && User32GetImeShowStatus() != 0 {
                (*pimeui).__bindgen_anon_1.set_fShowStatus(0);
                User32SendImeUIMessage(
                    pimeui,
                    WM_IME_NOTIFY,
                    IMN_CLOSESTATUSWINDOW as WPARAM,
                    0,
                    1,
                );
            }
            (*pimeui).__bindgen_anon_1.set_fCtrlShowStatus(0);
        }

        IMC_OPENSTATUSWINDOW => {
            if (*pimeui).__bindgen_anon_1.fShowStatus() == 0 && User32GetImeShowStatus() != 0 {
                (*pimeui).__bindgen_anon_1.set_fShowStatus(1);
                User32SendImeUIMessage(
                    pimeui,
                    WM_IME_NOTIFY,
                    IMN_OPENSTATUSWINDOW as WPARAM,
                    0,
                    1,
                );
            }
            (*pimeui).__bindgen_anon_1.set_fCtrlShowStatus(1);
        }

        _ => {}
    }

    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn User32SetImeActivenessOfWindow(hWnd: HWND, bActive: i32) {
    User32SetImeActivenessOfWindow_impl(hWnd, bActive)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn User32SetImeActivenessOfWindow(hWnd: HWND, bActive: i32) {
    User32SetImeActivenessOfWindow_impl(hWnd, bActive)
}

unsafe fn User32SetImeActivenessOfWindow_impl(hWnd: HWND, bActive: i32) {
    let hIMC: HIMC;

    if hWnd.is_null() || IsWindow(hWnd) == 0 {
        IMM_FN!(ImmSetActiveContext)(core::ptr::null_mut(), core::ptr::null_mut(), bActive);
        return;
    }

    hIMC = IMM_FN!(ImmGetContext)(hWnd);
    IMM_FN!(ImmSetActiveContext)(hWnd, hIMC, bActive);
    IMM_FN!(ImmReleaseContext)(hWnd, hIMC);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CtfLoadThreadLayout(pimeui: PIMEUI) {
    CtfLoadThreadLayout_impl(pimeui);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CtfLoadThreadLayout(pimeui: PIMEUI) {
    CtfLoadThreadLayout_impl(pimeui);
}

unsafe fn CtfLoadThreadLayout_impl(pimeui: PIMEUI) {
    IMM_FN!(CtfImmTIMActivate)((*pimeui).hKL);
    (*pimeui).hKL = (*GetWin32ClientInfo()).hKL;
    IMM_FN!(ImmLoadIME)((*pimeui).hKL);
    (*pimeui).hwndUI = core::ptr::null_mut();
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn User32DoImeHelp(pimeui: PIMEUI, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    User32DoImeHelp_impl(pimeui, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn User32DoImeHelp(pimeui: PIMEUI, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    User32DoImeHelp_impl(pimeui, wParam, lParam)
}

unsafe fn User32DoImeHelp_impl(pimeui: PIMEUI, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut szHelpFile: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut ret: DWORD;
    let mut dwEsc: DWORD = IME_ESC_GETHELPFILENAME as DWORD;
    let cch: usize;

    ret = IMM_FN!(ImmEscapeW)(
        (*pimeui).hKL,
        (*pimeui).hIMC,
        IME_ESC_QUERY_SUPPORT,
        core::ptr::addr_of_mut!(dwEsc) as *mut core::ffi::c_void,
    ) as DWORD;
    if ret == 0 || lParam == 0 {
        return ret as LRESULT;
    }

    if IMM_FN!(ImmEscapeW)(
        (*pimeui).hKL,
        (*pimeui).hIMC,
        IME_ESC_GETHELPFILENAME,
        szHelpFile.as_mut_ptr() as *mut core::ffi::c_void,
    ) != 0
    {
        cch = wcslen(szHelpFile.as_ptr());
        if cch > 4
            && _wcsicmp(
                szHelpFile.as_ptr().add(cch - 4),
                widestr_hlp().as_ptr(),
            ) == 0
        {
            TRACE!(b"szHelpFile: %s\n\0".as_ptr(), debugstr_w(szHelpFile.as_ptr()));
            WinHelpW(core::ptr::null_mut(), szHelpFile.as_ptr(), HELP_FINDER, 0);
        } else {
            FIXME!(
                "(%p, %p, %p): %s\n",
                pimeui,
                wParam,
                lParam,
                debugstr_w(szHelpFile.as_ptr()),
            );
            ret = 0;
        }
    }

    ret as LRESULT
}

unsafe fn widestr_hlp() -> [u16; 5] {
    [0x002E, 0x0048, 0x004C, 0x0050, 0x0000]
}

unsafe extern "system" fn ImeWnd_SwitchSoftKbdProc(hIMC: HIMC, lParam: LPARAM) -> BOOL {
    ImeWnd_SwitchSoftKbdProc_impl(hIMC, lParam)
}

unsafe fn ImeWnd_SwitchSoftKbdProc_impl(hIMC: HIMC, lParam: LPARAM) -> BOOL {
    let mut dwConversion: u32 = 0;
    let mut dwSentence: u32 = 0;
    let dwNewConversion: u32;

    IMM_FN!(ImmGetConversionStatus)(hIMC, &mut dwConversion, &mut dwSentence);

    if lParam != 0 {
        dwNewConversion = dwConversion | IME_CMODE_SOFTKBD;
    } else {
        dwNewConversion = dwConversion & !IME_CMODE_SOFTKBD;
    }

    if dwNewConversion != dwConversion {
        IMM_FN!(ImmSetConversionStatus)(hIMC, dwNewConversion, dwSentence);
    }

    TRUE
}

unsafe extern "system" fn User32GetPenMenuData(
    pnID: PUINT,
    pdwMenuData: PDWORD_PTR,
) -> BOOL {
    if (*(&raw const s_pGetPenMenuData)).is_none() {
        let mut szPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut hIndicDll = GetModuleHandleW(
            w!("indicdll.dll").as_ptr() as PCWSTR,
        );
        if hIndicDll.is_null() {
            User32GetSystemFilePath(
                szPath.as_mut_ptr(),
                szPath.len() as SIZE_T,
                w!("indicdll.dll").as_ptr() as PCWSTR,
            );
            hIndicDll = LoadLibraryW(szPath.as_ptr());
        }
        if hIndicDll.is_null() {
            return 0;
        }

        s_pGetPenMenuData = core::mem::transmute(GetProcAddress(
            hIndicDll,
            MAKEINTRESOURCEA(IFN_GetPenMenuData),
        ));
    }

    match s_pGetPenMenuData {
        None => 0,
        Some(func) => {
            func(pnID, pdwMenuData);
            1
        }
    }
}

unsafe extern "system" fn User32ImeMenuItemSelected(hwndTarget: HWND) {
    if IsWindow(hwndTarget) == 0 {
        return;
    }

    let hIMC: HIMC = IMM_FN!(ImmGetContext)(hwndTarget);
    if hIMC.is_null() {
        return;
    }

    let mut nID: UINT = 0;
    let mut dwMenuData: DWORD_PTR = 0;
    if User32GetPenMenuData(&mut nID, &mut dwMenuData) != 0 {
        IMM_FN!(ImmNotifyIME)(hIMC, NI_IMEMENUSELECTED, nID, dwMenuData);
    }

    IMM_FN!(ImmReleaseContext)(hwndTarget, hIMC);
}

fastcall_fn! {
unsafe fn ImeWnd_OnImeSystem(pimeui: PIMEUI, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut ret: LRESULT = 0;
    let pIC: LPINPUTCONTEXTDX;
    let hIMC: HIMC = (*pimeui).hIMC;
    let pCandForm: LPCANDIDATEFORM;
    let pCompForm: LPCOMPOSITIONFORM;
    let mut dwConversion: DWORD = 0;
    let mut dwSentence: DWORD = 0;
    let hImeWnd: HWND;
    let mut bCompForm: BOOL;
    let mut CandForm: CANDIDATEFORM = core::mem::zeroed();
    let mut CompForm: COMPOSITIONFORM = core::mem::zeroed();
    let mut iCandForm: UINT;

    match wParam as u32 {
        IMS_NOTIFYIMESHOW => {
            if User32GetImeShowStatus() == ((lParam == 0) as BOOL) {
                hImeWnd = UserHMGetHandle((*pimeui).spwnd);
                NtUserCallHwndParamLock(hImeWnd, lParam as DWORD_PTR, TWOPARAM_ROUTINE_IMESHOWSTATUSCHANGE);
            }
        }

        IMS_UPDATEIMEUI => 'block: {
            if hIMC.is_null() {
                break 'block;
            }

            bCompForm = 1;
            let pIC = IMM_FN!(ImmLockIMC)(hIMC) as LPINPUTCONTEXTDX;
            if !pIC.is_null() {
                bCompForm = (((*pIC).dwUIFlags & 0x2) == 0) as BOOL;
                IMM_FN!(ImmUnlockIMC)(hIMC);
            }

            if IsWindow((*pimeui).hwndIMC) == 0 {
                break 'block;
            }

            if bCompForm != 0 && IMM_FN!(ImmGetCompositionWindow)(hIMC, &mut CompForm) != 0 {
                if CompForm.dwStyle != 0 {
                    IMM_FN!(ImmSetCompositionWindow)(hIMC, &mut CompForm);
                }
            }

            iCandForm = 0;
            while iCandForm < MAX_CANDIDATEFORM {
                if IMM_FN!(ImmGetCandidateWindow)(hIMC, iCandForm, &mut CandForm) != 0 {
                    if CandForm.dwStyle != 0 {
                        IMM_FN!(ImmSetCandidateWindow)(hIMC, &mut CandForm);
                    }
                }
                iCandForm += 1;
            }
        }

        IMS_SETCANDFORM => 'block: {
            pIC = IMM_FN!(ImmLockIMC)(hIMC) as LPINPUTCONTEXTDX;
            if pIC.is_null() {
                break 'block;
            }

            pCandForm = core::ptr::addr_of_mut!((*icx_base(pIC)).cfCandForm[lParam as usize]);
            IMM_FN!(ImmSetCandidateWindow)(hIMC, pCandForm);
            IMM_FN!(ImmUnlockIMC)(hIMC);
        }

        IMS_SETCOMPFONT => 'block: {
            pIC = IMM_FN!(ImmLockIMC)(hIMC) as LPINPUTCONTEXTDX;
            if pIC.is_null() {
                break 'block;
            }

            IMM_FN!(ImmSetCompositionFontW)(hIMC, core::ptr::addr_of_mut!((*icx_base(pIC)).lfFont.W));
            IMM_FN!(ImmUnlockIMC)(hIMC);
        }

        IMS_SETCOMPFORM => 'block: {
            pIC = IMM_FN!(ImmLockIMC)(hIMC) as LPINPUTCONTEXTDX;
            if pIC.is_null() {
                break 'block;
            }

            pCompForm = core::ptr::addr_of_mut!((*icx_base(pIC)).cfCompForm);
            (*pIC).dwUIFlags |= 0x8;
            IMM_FN!(ImmSetCompositionWindow)(hIMC, pCompForm);
            IMM_FN!(ImmUnlockIMC)(hIMC);
        }

        IMS_CONFIGURE => {
            IMM_FN!(ImmConfigureIMEW)(lParam as HKL, (*pimeui).hwndIMC, IME_CONFIG_GENERAL, core::ptr::null_mut());
        }

        IMS_SETOPENSTATUS => {
            if !hIMC.is_null() {
                IMM_FN!(ImmSetOpenStatus)(hIMC, lParam as BOOL);
            }
        }

        IMS_FREELAYOUT => {
            ret = IMM_FN!(ImmFreeLayout)(lParam as HKL) as LRESULT;
        }

        IMS_SOFTKBDONOFF => {
            IMM_FN!(ImmEnumInputContext)(0, Some(ImeWnd_SwitchSoftKbdProc), lParam);
        }

        IMS_GETCONVSTATUS => {
            IMM_FN!(ImmGetConversionStatus)(hIMC, &mut dwConversion, &mut dwSentence);
            ret = dwConversion as LRESULT;
        }

        IMS_IMEHELP => {
            return User32DoImeHelp(pimeui, wParam, lParam);
        }

        IMS_IMEACTIVATE => {
            User32SetImeActivenessOfWindow(lParam as HWND, 1);
        }

        IMS_IMEDEACTIVATE => {
            User32SetImeActivenessOfWindow(lParam as HWND, 0);
        }

        IMS_ACTIVATELAYOUT => {
            ret = IMM_FN!(ImmActivateLayout)(lParam as HKL) as LRESULT;
        }

        IMS_GETIMEMENU => {
            ret = IMM_FN!(ImmPutImeMenuItemsIntoMappedFile)(lParam as HIMC) as LRESULT;
        }

        IMS_IMEMENUITEMSELECTED => {
            User32ImeMenuItemSelected(lParam as HWND);
        }

        IMS_GETCONTEXT => {
            ret = IMM_FN!(ImmGetContext)(lParam as HWND) as ULONG_PTR as LRESULT;
        }

        IMS_SENDNOTIFICATION | IMS_COMPLETECOMPSTR | IMS_SETLANGBAND | IMS_UNSETLANGBAND => {
            ret = IMM_FN!(ImmSystemHandler)(hIMC, wParam, lParam);
        }

        IMS_LOADTHREADLAYOUT => {
            CtfLoadThreadLayout(pimeui);
        }

        _ => {}
    }

    ret
}
}

unsafe fn ImeWnd_OnImeSetContext(pimeui: PIMEUI, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let ret: LRESULT;
    let mut hIMC: HIMC;
    let pIC: LPINPUTCONTEXTDX;
    let mut hwndFocus: HWND;
    let hwndOldImc: HWND;
    let hwndNewImc: HWND;
    let mut hImeWnd: HWND;
    let hwndActive: HWND;
    let hwndOwner: HWND;
    let pwndFocus: PWND;
    let mut pImeWnd: PWND;
    let pwndOwner: PWND;
    let mut CompForm: COMPOSITIONFORM = core::mem::zeroed();

    (*pimeui).__bindgen_anon_1.set_fActivate(if wParam != 0 { 1 } else { 0 });
    hwndOldImc = (*pimeui).hwndIMC;

    if wParam != 0 {
        if (*pimeui).hwndUI.is_null() {
            (*pimeui).hwndUI = User32CreateImeUIWindow(pimeui, (*pimeui).hKL);
        }

        if gfConIme == -1 {
            gfConIme = NtUserGetThreadState(THREADSTATE_CHECKCONIME as DWORD) as INT;
            if gfConIme != 0 {
                (*pimeui).__bindgen_anon_1.set_fCtrlShowStatus(0);
            }
        }

        hImeWnd = UserHMGetHandle((*pimeui).spwnd);

        if gfConIme != 0 {
            hwndOwner = GetWindow(hImeWnd, GW_OWNER as UINT);
            pwndOwner = ValidateHwnd(hwndOwner);
            if !pwndOwner.is_null() {
                User32UpdateImcOfImeUI(pimeui, (*pwndOwner).hImc);

                if !(*pimeui).hwndUI.is_null() {
                    SetWindowLongPtrW((*pimeui).hwndUI, IMMGWLP_IMC as core::ffi::c_int, (*pwndOwner).hImc as LONG_PTR);
                }
            }

            return User32SendImeUIMessage(pimeui, WM_IME_SETCONTEXT, wParam, lParam, 1);
        }

        hwndFocus = NtUserQueryWindow(hImeWnd, QUERY_WINDOW_FOCUS) as HWND;

        hIMC = IMM_FN!(ImmGetContext)(hwndFocus);

        if !hIMC.is_null() && User32CanSetImeWindowToImc(hIMC, hImeWnd) == 0 {
            User32UpdateImcOfImeUI(pimeui, core::ptr::null_mut());
            return 0;
        }

        User32UpdateImcOfImeUI(pimeui, hIMC);

        if !(*pimeui).hwndUI.is_null() {
            SetWindowLongPtrW((*pimeui).hwndUI, IMMGWLP_IMC as core::ffi::c_int, hIMC as LONG_PTR);
        }

        if !hIMC.is_null() {
            pIC = IMM_FN!(ImmLockIMC)(hIMC) as LPINPUTCONTEXTDX;
            if pIC.is_null() {
                return 0;
            }

            if hwndFocus != (*icx_base(pIC)).hWnd {
                IMM_FN!(ImmUnlockIMC)(hIMC);
                return 0;
            }

            if ((*pIC).dwUIFlags & 0x40000) != 0 && hwndOldImc != hwndFocus {
                RtlZeroMemory(
                    core::ptr::addr_of_mut!(CompForm) as *mut _,
                    core::mem::size_of::<COMPOSITIONFORM>(),
                );
                IMM_FN!(ImmSetCompositionWindow)(hIMC, core::ptr::addr_of_mut!(CompForm));

                (*pIC).dwUIFlags &= !0x40000;
            }

            IMM_FN!(ImmUnlockIMC)(hIMC);

            hImeWnd = UserHMGetHandle((*pimeui).spwnd);
            if NtUserSetImeOwnerWindow(hImeWnd, hwndFocus) != 0 {
                (*pimeui).hwndIMC = hwndFocus;
            }
        } else {
            (*pimeui).hwndIMC = hwndFocus;

            hImeWnd = UserHMGetHandle((*pimeui).spwnd);
            NtUserSetImeOwnerWindow(hImeWnd, core::ptr::null_mut());
        }
    }

    ret = User32SendImeUIMessage(pimeui, WM_IME_SETCONTEXT, wParam, lParam, 1);

    if (*pimeui).spwnd.is_null() {
        return 0;
    }

    if (*pimeui).__bindgen_anon_1.fCtrlShowStatus() == 0 || User32GetImeShowStatus() == 0 {
        return ret;
    }

    hImeWnd = UserHMGetHandle((*pimeui).spwnd);
    hwndFocus = NtUserQueryWindow(hImeWnd, QUERY_WINDOW_FOCUS) as HWND;
    pwndFocus = ValidateHwnd(hwndFocus);

    if wParam != 0 {
        pImeWnd = ValidateHwnd(hImeWnd);
        if !pwndFocus.is_null()
            && !pImeWnd.is_null()
            && head_pti(pImeWnd) == head_pti(pwndFocus)
        {
            hwndNewImc = (*pimeui).hwndIMC;
            if (*pimeui).__bindgen_anon_1.fShowStatus() != 0 {
                if !hwndOldImc.is_null()
                    && !hwndNewImc.is_null()
                    && hwndOldImc != hwndNewImc
                    && IntGetTopLevelWindow(hwndOldImc) != IntGetTopLevelWindow(hwndNewImc)
                {
                    User32NotifyOpenStatus(pimeui, hwndOldImc, 0);
                    User32NotifyOpenStatus(pimeui, hwndNewImc, 1);
                }
            } else {
                if !ValidateHwnd(hwndNewImc).is_null() {
                    User32NotifyOpenStatus(pimeui, hwndNewImc, 1);
                }
            }
        }

        pImeWnd = (*pimeui).spwnd;
        hImeWnd = if !pImeWnd.is_null() {
            UserHMGetHandle(pImeWnd)
        } else {
            core::ptr::null_mut()
        };
        if !hImeWnd.is_null() {
            NtUserCallHwndLock(hImeWnd, HWNDLOCK_ROUTINE_CHECKIMESHOWSTATUSINTHRD);
        }
    } else {
        pImeWnd = (*pimeui).spwnd;
        hImeWnd = UserHMGetHandle(pImeWnd);
        hwndActive = NtUserQueryWindow(hImeWnd, QUERY_WINDOW_ACTIVE) as HWND;
        if pwndFocus.is_null()
            || hwndActive.is_null()
            || head_pti(pImeWnd) != head_pti(pwndFocus)
        {
            if IsWindow(hwndOldImc) != 0 {
                User32NotifyOpenStatus(pimeui, hwndOldImc, 0);
            } else {
                (*pimeui).__bindgen_anon_1.set_fShowStatus(0);
                User32SendImeUIMessage(pimeui, WM_IME_NOTIFY, IMN_CLOSESTATUSWINDOW as WPARAM, 0, 1);
            }
        }
    }

    ret
}

unsafe extern "system" fn ImeWndProc_common(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let pWnd: PWND;
    let pimeui: PIMEUI;
    let ret: LRESULT;

    pWnd = ValidateHwnd(hwnd);
    if pWnd.is_null() {
        ERR!("hwnd was %p\n", hwnd);
        return 0;
    }

    if (*pWnd).fnid == 0 {
        NtUserSetWindowFNID(hwnd, FNID_IME as u16);
    } else if (*pWnd).fnid != FNID_IME as DWORD {
        ERR!("fnid was 0x%x\n", (*pWnd).fnid as u32);
        return 0;
    }

    if msg == WM_NCCREATE {
        pimeui = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<IMEUI>() as SIZE_T,
        ) as PIMEUI;
        if pimeui.is_null() {
            ERR!("HeapAlloc failed\n");
            NtUserSetWindowFNID(hwnd, FNID_DESTROY as u16);
            return 0;
        }
        (*pimeui).spwnd = pWnd;
        SetWindowLongPtrW(hwnd, GWLP_IMEWND_PIMEUI as core::ffi::c_int, pimeui as LONG_PTR);
    } else {
        pimeui = GetWindowLongPtrW(hwnd, GWLP_IMEWND_PIMEUI as core::ffi::c_int) as PIMEUI;
        if pimeui.is_null() {
            ERR!("Invalid IMEWND\n");
            NtUserSetWindowFNID(hwnd, FNID_DESTROY as u16);
            return 0;
        }
    }

    if IS_CICERO_MODE() != 0 {
        ret = IMM_FN!(CtfImmDispatchDefImeMessage)(hwnd, msg, wParam, lParam);
        if ret != 0 {
            return ret;
        }
    }

    'finish: {
        if (*pimeui).nCntInIMEProc > 0 {
            match msg {
                WM_IME_CHAR
                | WM_IME_COMPOSITIONFULL
                | WM_IME_CONTROL
                | WM_IME_REQUEST
                | WM_IME_SELECT
                | WM_IME_SETCONTEXT
                | WM_IME_STARTCOMPOSITION
                | WM_IME_COMPOSITION
                | WM_IME_ENDCOMPOSITION => {
                    return 0;
                }
                WM_IME_NOTIFY => {
                    if wParam < IMN_PRIVATE as WPARAM
                        || IS_IME_HKL((*pimeui).hKL) != 0
                        || IS_CICERO_MODE() == 0
                    {
                        return 0;
                    }
                }
                WM_IME_SYSTEM => match wParam {
                    0x03 | 0x10 | 0x13 => {}
                    _ => {
                        return 0;
                    }
                },
                _ => {
                    break 'finish;
                }
            }
        }

        if ((*pWnd).state2 & WNDS2_INDESTROY) != 0 || ((*pWnd).state & WNDS_DESTROYED) != 0 {
            match msg {
                WM_DESTROY | WM_NCDESTROY | WM_FINALDESTROY => {}
                _ => {
                    return 0;
                }
            }
        }

        match msg {
            WM_CREATE => {
                return ImeWnd_OnCreate(pimeui, lParam as LPCREATESTRUCT) as LRESULT;
            }
            WM_DESTROY => {
                User32DestroyImeUIWindow(pimeui);
                return 0;
            }
            WM_NCDESTROY | WM_FINALDESTROY => {
                (*pimeui).spwnd = core::ptr::null_mut();
                HeapFree(GetProcessHeap(), 0, pimeui as *mut _);
                NtUserSetWindowFNID(hwnd, FNID_DESTROY as u16);
            }
            WM_ERASEBKGND => {
                return 1;
            }
            WM_PAINT => {
                return 0;
            }
            WM_IME_STARTCOMPOSITION | WM_IME_COMPOSITION | WM_IME_ENDCOMPOSITION => {
                return User32SendImeUIMessage(pimeui, msg, wParam, lParam, unicode);
            }
            WM_IME_CONTROL => {
                return ImeWnd_OnImeControl(pimeui, wParam, lParam, unicode);
            }
            WM_IME_NOTIFY => {
                return ImeWnd_OnImeNotify(pimeui, wParam, lParam);
            }
            WM_IME_REQUEST => {
                return 0;
            }
            WM_IME_SELECT => {
                ImeWnd_OnImeSelect(pimeui, wParam, lParam);
                return pimeui as LRESULT;
            }
            WM_IME_SETCONTEXT => {
                return ImeWnd_OnImeSetContext(pimeui, wParam, lParam);
            }
            WM_IME_SYSTEM => {
                return ImeWnd_OnImeSystem(pimeui, wParam, lParam);
            }
            _ => {}
        }
    }

    if unicode != 0 {
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    }
    DefWindowProcA(hwnd, msg, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn ImeWndProcA(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    ImeWndProc_common(hwnd, msg, wParam, lParam, 0)
}

#[no_mangle]
pub unsafe extern "system" fn ImeWndProcW(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    ImeWndProc_common(hwnd, msg, wParam, lParam, 1)
}

#[no_mangle]
pub unsafe extern "system" fn UpdatePerUserImmEnabling() -> BOOL {

    let imm32: HMODULE;
    let mut ret: BOOL;

    ret = NtUserCallNoParam(NOPARAM_ROUTINE_UPDATEPERUSERIMMENABLING as _) as BOOL;
    if ret == 0 || gpsi.is_null() || ((*gpsi).dwSRVIFlags & SRVINFO_IMM32) == 0 {
        return 0;
    }

    let imm32_existing = GetModuleHandleW(w!("imm32.dll").as_ptr() as PCWSTR);
    if !imm32_existing.is_null() {
        return 1;
    }

    imm32 = LoadLibraryW(w!("imm32.dll").as_ptr() as PCWSTR);
    if imm32.is_null() {
        ERR!("Imm32 not installed!\n");
        ret = 0;
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn IMPSetIMEW(hwnd: HWND, ime: LPIMEPROW) -> i32 {
    IMM_FN!(ImmIMPSetIMEW)(hwnd, ime)
}

#[no_mangle]
pub unsafe extern "system" fn IMPQueryIMEW(ime: LPIMEPROW) -> BOOL {
    IMM_FN!(ImmIMPQueryIMEW)(ime)
}

#[no_mangle]
pub unsafe extern "system" fn IMPGetIMEW(hwnd: HWND, ime: LPIMEPROW) -> BOOL {
    IMM_FN!(ImmIMPGetIMEW)(hwnd, ime)
}

#[no_mangle]
pub unsafe extern "system" fn IMPSetIMEA(hwnd: HWND, ime: LPIMEPROA) -> BOOL {
    IMM_FN!(ImmIMPSetIMEA)(hwnd, ime)
}

#[no_mangle]
pub unsafe extern "system" fn IMPQueryIMEA(ime: LPIMEPROA) -> BOOL {
    IMM_FN!(ImmIMPQueryIMEA)(ime)
}

#[no_mangle]
pub unsafe extern "system" fn IMPGetIMEA(hwnd: HWND, ime: LPIMEPROA) -> BOOL {
    IMM_FN!(ImmIMPGetIMEA)(hwnd, ime)
}

#[no_mangle]
pub unsafe extern "system" fn SendIMEMessageExW(hwnd: HWND, lParam: LPARAM) -> LRESULT {
    IMM_FN!(ImmSendIMEMessageExW)(hwnd, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn SendIMEMessageExA(hwnd: HWND, lParam: LPARAM) -> LRESULT {
    IMM_FN!(ImmSendIMEMessageExA)(hwnd, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn WINNLSEnableIME(hwnd: HWND, enable: BOOL) -> BOOL {
    IMM_FN!(ImmWINNLSEnableIME)(hwnd, enable)
}

#[no_mangle]
pub unsafe extern "system" fn WINNLSGetEnableStatus(hwnd: HWND) -> BOOL {
    IMM_FN!(ImmWINNLSGetEnableStatus)(hwnd)
}

#[no_mangle]
pub unsafe extern "system" fn WINNLSGetIMEHotkey(hwnd: HWND) -> UINT {
    0
}
