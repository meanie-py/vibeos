//! calc-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ---- hand ports (outside the generated blocks, so a replay keeps them) ----
// utl_ieee.c:39 -- a TU-local `#define`, `#undef`'d at the end of the file, so the clang
// facts probe (which reads the macro table at end of TU) never sees it.
pub const MAX_LD_WIDTH: core::ffi::c_int = 16;
// winmain.c:23 `HTMLHELP_PATH(_pt)`: `_T("...chm::") _T(_pt)` is string-literal
// concatenation in a UNICODE build -> one NUL-terminated UTF-16 literal, built at compile time.
#[allow(unused_macros)]
macro_rules! HTMLHELP_PATH {
    ($pt:literal $(,)?) => {{
        const B: &[u8] = concat!("%systemroot%\\Help\\calc.chm::", $pt).as_bytes();
        const W: [u16; B.len() + 1] = {
            let mut a = [0u16; B.len() + 1];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        W.as_ptr()
    }};
}
#[allow(unused_imports)]
pub(crate) use HTMLHELP_PATH;
// calc.h:61 `calc_HtmlHelp` -> `calc_HtmlHelpW`, a function-pointer VARIABLE (htmlhelp.c
// starts it at dummy_HtmlHelpW and only ever replaces it with a non-NULL GetProcAddress), so
// the C call is a call through the pointer.
#[allow(unused_macros)]
// Exceeding the C: calc.h types the 4th parameter `DWORD`, but the pointer is HtmlHelpW's,
// whose `dwData` is a DWORD_PTR -- and the context-help call passes `(DWORD_PTR)&popup`, which
// a DWORD truncates on amd64/arm64. Called through HtmlHelpW's real signature (the dummy
// fallback reads only the low half, as its DWORD parameter does on every target).
macro_rules! calc_HtmlHelp {
    ($h:expr, $file:expr, $cmd:expr, $data:expr $(,)?) => {
        core::mem::transmute::<_, unsafe extern "system" fn(HWND, LPCWSTR, UINT, DWORD_PTR) -> HWND>(
            calc_HtmlHelpW.unwrap_unchecked(),
        )($h as _, $file as _, $cmd as _, $data as DWORD_PTR)
    };
}
#[allow(unused_imports)]
pub(crate) use calc_HtmlHelp;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BP_PUSHBUTTON: BUTTONPARTS = BUTTONPARTS_BP_PUSHBUTTON; // island enum
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // access-rust/src/prelude.rs
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400; // comctl32-rust/src/prelude.rs
pub const PBS_DISABLED: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_DISABLED; // island enum
pub const PBS_HOT: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_HOT; // island enum
pub const PBS_PRESSED: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_PRESSED; // island enum
pub const TRANSPARENT: u32 = 1; // access-rust/src/types_amd64.rs
pub const UCHAR_MAX: u32 = 255; // msvcrt-sys/src/bindgen_amd64.rs
pub const ULONG_MAX: u32 = 4294967295; // msv1_0-rust/src/types_amd64.rs
pub const USHRT_MAX: u32 = 65535; // win32k-sys/src/bindgen_amd64.rs
pub const WHITE_BRUSH: u32 = 0; // access-rust/src/types_amd64.rs
extern "C" {
    pub fn _finite(x: f64) -> i32; // cnt-rust/src/cabi/math.rs
    pub fn _isnan(x: f64) -> i32; // cnt-rust/src/cabi/math.rs
    pub fn _strdup(str_: *const c_char) -> *mut c_char; // crt-shared-rust/src/prelude.rs
    pub fn _strupr(_String: *mut u8) -> *mut u8; // cnt-rust/src/cabi/generated.rs
    pub fn acos(x: f64) -> f64; // msvcrt-crt-rust/src/msvcrt/math.rs
    pub fn asin(x: f64) -> f64; // msvcrt-crt-rust/src/msvcrt/math.rs
    pub fn atan(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn cbrt(x: f64) -> f64; // ucrt-rust/src/c99math.rs
    pub fn ceil(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn cos(x: f64) -> f64; // crt-startup-rust/src/math.rs
    pub fn cosh(x: f64) -> f64; // msvcrt-crt-rust/src/msvcrt/math.rs
    pub fn exp(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn fabs(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn fmod(x: f64, y: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn log(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn log10(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn modf(x: f64, iptr: *mut f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn pow(x: f64, y: f64) -> f64; // msvcrt-crt-rust/src/msvcrt/math.rs
    pub fn sin(x: f64) -> f64; // crt-startup-rust/src/math.rs
    pub fn sinh(x: f64) -> f64; // msvcrt-crt-rust/src/msvcrt/math.rs
    pub fn sqrt(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn strlen(s: *const c_char) -> usize; // browser-rust/src/prelude.rs
    pub fn tan(x: f64) -> f64; // msvcrt-crt-rust/src/msvcrt/math.rs
    pub fn tanh(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
}
extern "system" {
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn CheckRadioButton( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreatePopupMenu() -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DrawFrameControl(arg1: HDC, arg2: LPRECT, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EmptyClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnumChildWindows(arg1: HWND, arg2: WNDENUMPROC, arg3: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetAsyncKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetClipboardData(arg1: UINT) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetFocus() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetKeyboardLayout(arg1: DWORD) -> HKL; // aclui-rust/src/types_i386.rs
    pub fn GetKeyboardState(lpKeyState: PBYTE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_i386.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsClipboardFormatAvailable(arg1: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowEnabled(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenClipboard(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SetBkMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SetClipboardData(arg1: UINT, arg2: HANDLE) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetMenu(arg1: HWND, arg2: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ToAsciiEx( arg1: UINT, arg2: UINT, arg3: *const BYTE, arg4: LPWORD, arg5: UINT, arg6: HKL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn TrackMouseEvent(arg1: LPTRACKMOUSEEVENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnhookWindowsHookEx(arg1: HHOOK) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const _UI64_MAX: u64 = 0xFFFFFFFFFFFFFFFF; // clang, fun_ieee.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsicmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // winmain.c
    pub fn swscanf(_Src: *const wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // utl_ieee.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // utl_ieee.c
    pub fn wcscmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // convert.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn AppendMenuW(_: HMENU, _: UINT, _: UINT_PTR, _: LPCWSTR) -> BOOL; // winmain.c
    pub fn CallWindowProcW(_: WNDPROC, _: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // winmain.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // winmain.c
    pub fn DrawStateW(
        _: HDC,
        _: HBRUSH,
        _: DRAWSTATEPROC,
        _: LPARAM,
        _: WPARAM,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: UINT,
    ) -> BOOL; // winmain.c
    pub fn GetClassNameW(
        hWnd: HWND,
        lpClassName: LPWSTR,
        nMaxCount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // winmain.c
    pub fn GetDlgItemTextW(
        hDlg: HWND,
        nIDDlgItem: ::core::ffi::c_int,
        lpString: LPWSTR,
        cchMax: ::core::ffi::c_int,
    ) -> UINT; // winmain.c
    pub fn GetLocaleInfoW(
        Locale: LCID,
        LCType: LCTYPE,
        lpLCData: LPWSTR,
        cchData: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // winmain.c
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // winmain.c
    pub fn GetProfileIntW(_: LPCWSTR, _: LPCWSTR, _: INT) -> UINT; // winmain.c
    pub fn GetTextExtentPoint32W(
        hdc: HDC,
        lpString: LPCWSTR,
        c: ::core::ffi::c_int,
        psizl: LPSIZE,
    ) -> BOOL; // winmain.c
    pub fn GetVersionExW(_: LPOSVERSIONINFOW) -> BOOL; // winmain.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // winmain.c
    pub fn GetWindowTextW(
        hWnd: HWND,
        lpString: LPWSTR,
        nMaxCount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // winmain.c
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: LPCWSTR,
        type_0: UINT,
        cx: ::core::ffi::c_int,
        cy: ::core::ffi::c_int,
        fuLoad: UINT,
    ) -> HANDLE; // winmain.c
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // htmlhelp.c
    pub fn LoadMenuW(_: HINSTANCE, _: LPCWSTR) -> HMENU; // winmain.c
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // convert.c
    pub fn MapVirtualKeyExW(_: UINT, _: UINT, _: HKL) -> UINT; // winmain.c
    pub fn PostMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // convert.c
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        Reserved: DWORD,
        lpClass: LPWSTR,
        dwOptions: DWORD,
        samDesired: REGSAM,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        phkResult: PHKEY,
        lpdwDisposition: PDWORD,
    ) -> LSTATUS; // winmain.c
    pub fn RegOpenKeyExW(_: HKEY, _: LPCWSTR, _: DWORD, _: REGSAM, _: PHKEY) -> LSTATUS; // winmain.c
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // winmain.c
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LSTATUS; // winmain.c
    pub fn SendDlgItemMessageW(
        _: HWND,
        _: ::core::ffi::c_int,
        _: UINT,
        _: WPARAM,
        _: LPARAM,
    ) -> LRESULT; // convert.c
    pub fn SendMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // convert.c
    pub fn SetDlgItemTextW(_: HWND, _: ::core::ffi::c_int, _: LPCWSTR) -> BOOL; // winmain.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // winmain.c
    pub fn SetWindowsHookExW( arg1: ::core::ffi::c_int, arg2: HOOKPROC, arg3: HINSTANCE, arg4: DWORD, ) -> HHOOK; // aclui-rust/src/types_i386.rs
    pub fn ShellAboutW(
        hWnd: HWND,
        szApp: LPCWSTR,
        szOtherStuff: LPCWSTR,
        hIcon: HICON,
    ) -> ::core::ffi::c_int; // winmain.c
    pub fn WriteProfileStringW(_: LPCWSTR, _: LPCWSTR, _: LPCWSTR) -> BOOL; // winmain.c
}
pub use crate::prelude::AppendMenuW as AppendMenu; // sdk alias
pub use crate::prelude::CallWindowProcW as CallWindowProc; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::DrawStateW as DrawState; // sdk alias
pub use crate::prelude::GetClassNameW as GetClassName; // sdk alias
pub use crate::prelude::GetDlgItemTextW as GetDlgItemText; // sdk alias
pub use crate::prelude::GetLocaleInfoW as GetLocaleInfo; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetProfileIntW as GetProfileInt; // sdk alias
pub use crate::prelude::GetTextExtentPoint32W as GetTextExtentPoint32; // sdk alias
pub use crate::prelude::GetVersionExW as GetVersionEx; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::LoadMenuW as LoadMenu; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MapVirtualKeyExW as MapVirtualKeyEx; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetWindowsHookExW as SetWindowsHookEx; // sdk alias
pub use crate::prelude::ShellAboutW as ShellAbout; // sdk alias
pub use strsafe_rust::StringCbCatW as StringCbCat; // sdk alias
pub use strsafe_rust::StringCbCopyW as StringCbCopy; // sdk alias
pub use strsafe_rust::StringCbCopyExW as StringCbCopyEx; // sdk alias
pub use strsafe_rust::StringCbPrintfW as StringCbPrintf; // sdk alias
pub use strsafe_rust::StringCbPrintfExW as StringCbPrintfEx; // sdk alias
pub use strsafe_rust::StringCchPrintfW as StringCchPrintf; // sdk alias
pub use strsafe_rust::StringCchPrintfExW as StringCchPrintfEx; // sdk alias
pub use crate::prelude::WriteProfileStringW as WriteProfileString; // sdk alias
pub use crate::prelude::swscanf as _stscanf; // sdk alias
pub use crate::prelude::wcschr as _tcschr; // sdk alias
pub use crate::prelude::wcscmp as _tcscmp; // sdk alias
pub use crate::prelude::_wcsicmp as _tcsicmp; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
// ==== end port externs ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
pub type OSVERSIONINFO = OSVERSIONINFOW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateDialog { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { CreateDialogParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as WPARAM) }; }
pub const WC_BUTTON: *const u16 = (&[0x0042, 0x0075, 0x0074, 0x0074, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"Button"
pub use crate::{CreateDialog, MAKEINTRESOURCE, MAKELPARAM, MAKEWPARAM};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/calc/fun_ieee.c:336: #define NEXT(n, i) (((n) + (i)/(n)) >> 1)
#[allow(unused_macros)]
macro_rules! NEXT {
    ($n:expr, $i:expr $(,)?) => {
        ((($n) + ($i) / ($n)) >> 1)
    };
}
#[allow(unused_imports)]
pub(crate) use NEXT;
// base/applications/calc/calc.h:97: #define SIZEOF(_ar) (sizeof(_ar)/sizeof(_ar[1]))
#[allow(unused_macros)]
macro_rules! SIZEOF {
    ($ar:expr $(,)?) => {
        (core::mem::size_of_val(&$ar) / core::mem::size_of_val(&$ar[0]))
    };
}
#[allow(unused_imports)]
pub(crate) use SIZEOF;
// ==== end port macros ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::calc::winmain::calc;
pub use crate::calc::theme::calc_CloseThemeData;
pub use crate::calc::theme::calc_DrawThemeBackground;
pub use crate::calc::theme::calc_DrawThemeParentBackground;
pub use crate::calc::theme::calc_IsAppThemed;
pub use crate::calc::theme::calc_IsThemeActive;
pub use crate::calc::theme::calc_IsThemeBackgroundPartiallyTransparent;
pub use crate::calc::theme::calc_OpenThemeData;
// ==== end cross-TU definitions ====
