//! notepad-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ENCODING_ANSI: ENCODING = ENCODING_ENCODING_ANSI; // island enum
pub const ENCODING_AUTO: ENCODING = ENCODING_ENCODING_AUTO; // island enum
pub const ENCODING_UTF16BE: ENCODING = ENCODING_ENCODING_UTF16BE; // island enum
pub const ENCODING_UTF16LE: ENCODING = ENCODING_ENCODING_UTF16LE; // island enum
pub const ENCODING_UTF8: ENCODING = ENCODING_ENCODING_UTF8; // island enum
pub const ENCODING_UTF8BOM: ENCODING = ENCODING_ENCODING_UTF8BOM; // island enum
pub const EOLN_CR: EOLN = EOLN_EOLN_CR; // island enum
pub const EOLN_CRLF: EOLN = EOLN_EOLN_CRLF; // island enum
pub const EOLN_LF: EOLN = EOLN_EOLN_LF; // island enum
extern "C" {
    pub fn iswctype(c: u16, mask: u16) -> c_int; // ucrt-rust/src/ctype.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
}
extern "system" {
    pub fn AbortDoc(hdc: HDC) -> INT; // aclui-rust/src/types_amd64.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyAcceleratorTable(arg1: HACCEL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DragAcceptFiles(arg1: HWND, arg2: BOOL); // desk-rust/src/types_amd64.rs
    pub fn DragFinish(arg1: HDROP); // desk-rust/src/types_amd64.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDoc(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn EndPage(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FindTextW(arg1: LPFINDREPLACEW) -> HWND; // desk-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetDeviceCaps(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItemInt(arg1: HWND, arg2: ::core::ffi::c_int, arg3: PBOOL, arg4: BOOL) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLocalTime(arg1: LPSYSTEMTIME); // aclui-rust/src/types_amd64.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetUserDefaultLangID() -> LANGID; // desk-rust/src/types_amd64.rs
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_amd64.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GlobalFree(arg1: HGLOBAL) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn InflateRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsClipboardFormatAvailable(arg1: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsTextUnicode( lpv: *const ::core::ffi::c_void, iSize: ::core::ffi::c_int, lpiResult: LPINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalLock(arg1: HLOCAL) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn LocalUnlock(arg1: HLOCAL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_amd64.rs
    pub fn Rectangle( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ReplaceTextW(arg1: LPFINDREPLACEW) -> HWND; // desk-rust/src/types_amd64.rs
    pub fn SHAddToRecentDocs(arg1: UINT, arg2: LPCVOID); // desk-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_amd64.rs
    pub fn SetDlgItemInt(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetEndOfFile(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn SetMapMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowPlacement(hWnd: HWND, arg1: *const WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn StartPage(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::notepad::printing::DIALOG_FilePageSetup;
pub use crate::notepad::printing::DIALOG_FilePrint;
pub use crate::notepad::main::EDIT_WndProc;
pub use crate::notepad::main::Globals;
pub use crate::notepad::main::NOTEPAD_EnableSearchMenu;
pub use crate::notepad::main::NOTEPAD_FindNext;
pub use crate::notepad::settings::NOTEPAD_LoadSettingsFromRegistry;
pub use crate::notepad::settings::NOTEPAD_SaveSettingsToRegistry;
pub use crate::notepad::text::ReadText;
pub use crate::notepad::main::SetFileName;
pub use crate::notepad::text::WriteText;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CDN_FILEOK: u32 = 0xFFFFFDA2; // clang, dialog.c
pub const ENCODING_DEFAULT: i32 = 3; // clang, dialog.c
pub const SBPART_CURPOS: i32 = 0; // clang, dialog.c
pub const SB_SETTEXT: i32 = 0x40B; // clang, dialog.c
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007A_u32 as HRESULT; // clang, printing.c
pub const S_OK: HRESULT = 0; // clang, printing.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsnicmp(
        _Str1: *const wchar_t,
        _Str2: *const wchar_t,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // main.c
    pub fn towupper(_C: wint_t) -> wint_t; // printing.c
    pub fn wcscat(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // dialog.c
    pub fn wcscmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // dialog.c
    pub fn wcscpy(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // dialog.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncmp(
        _Str1: *const wchar_t,
        _Str2: *const wchar_t,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // main.c
    pub fn wcsncpy(_Dest: *mut wchar_t, _Source: *const wchar_t, _Count: size_t) -> *mut wchar_t; // main.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // dialog.c
}
extern "system" {
    pub fn CallWindowProcW(_: WNDPROC, _: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // main.c
    pub fn ChooseFontW(_: LPCHOOSEFONTW) -> BOOL; // dialog.c
    pub fn CreateFontIndirectW(_: *const LOGFONTW) -> HFONT; // dialog.c
    pub fn CreateStatusWindowW(style: LONG, lpszText: LPCWSTR, hwndParent: HWND, wID: UINT) -> HWND; // dialog.c
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        X: ::core::ffi::c_int,
        Y: ::core::ffi::c_int,
        nWidth: ::core::ffi::c_int,
        nHeight: ::core::ffi::c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND; // dialog.c
    pub fn DefWindowProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // main.c
    pub fn DialogBoxParamW(_: HINSTANCE, _: LPCWSTR, _: HWND, _: DLGPROC, _: LPARAM) -> INT_PTR; // dialog.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // main.c
    pub fn DragQueryFileW(hDrop: HDROP, iFile: UINT, lpszFile: LPWSTR, cch: UINT) -> UINT; // main.c
    pub fn DrawTextW(
        hdc: HDC,
        lpchText: LPCWSTR,
        cchText: ::core::ffi::c_int,
        lprc: LPRECT,
        format: UINT,
    ) -> ::core::ffi::c_int; // printing.c
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetDateFormatW(
        _: LCID,
        _: DWORD,
        _: *const SYSTEMTIME,
        _: LPCWSTR,
        _: LPWSTR,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // dialog.c
    pub fn GetDlgItemTextW(
        hDlg: HWND,
        nIDDlgItem: ::core::ffi::c_int,
        lpString: LPWSTR,
        cchMax: ::core::ffi::c_int,
    ) -> UINT; // printing.c
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // dialog.c
    pub fn GetFileTitleW(_: LPCWSTR, _: LPWSTR, _: WORD) -> ::core::ffi::c_short; // main.c
    pub fn GetFullPathNameW(_: LPCWSTR, _: DWORD, _: LPWSTR, _: *mut LPWSTR) -> DWORD; // main.c
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // main.c
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // dialog.c
    pub fn GetOpenFileNameW(_: LPOPENFILENAMEW) -> BOOL; // dialog.c
    pub fn GetSaveFileNameW(_: LPOPENFILENAMEW) -> BOOL; // dialog.c
    pub fn GetTextExtentPoint32W(
        hdc: HDC,
        lpString: LPCWSTR,
        c: ::core::ffi::c_int,
        psizl: LPSIZE,
    ) -> BOOL; // printing.c
    pub fn GetTextMetricsW(_: HDC, _: LPTEXTMETRICW) -> BOOL; // printing.c
    pub fn GetTimeFormatW(
        _: LCID,
        _: DWORD,
        _: *const SYSTEMTIME,
        _: LPCWSTR,
        _: LPWSTR,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // dialog.c
    pub fn GetWindowTextW(
        hWnd: HWND,
        lpString: LPWSTR,
        nMaxCount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // dialog.c
    pub fn IsDialogMessageW(_: HWND, _: LPMSG) -> BOOL; // main.c
    pub fn LoadAcceleratorsW(_: HINSTANCE, _: LPCWSTR) -> HACCEL; // main.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // dialog.c
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // dialog.c
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: LPCWSTR,
        type_0: UINT,
        cx: ::core::ffi::c_int,
        cy: ::core::ffi::c_int,
        fuLoad: UINT,
    ) -> HANDLE; // main.c
    pub fn PageSetupDlgW(_: LPPAGESETUPDLGW) -> BOOL; // printing.c
    pub fn PrintDlgW(_: LPPRINTDLGW) -> BOOL; // printing.c
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
    ) -> LSTATUS; // settings.c
    pub fn RegOpenKeyW(_: HKEY, _: LPCWSTR, _: PHKEY) -> LSTATUS; // settings.c
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // settings.c
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LSTATUS; // settings.c
    pub fn RegisterClassExW(_: *const WNDCLASSEXW) -> ATOM; // main.c
    pub fn RegisterWindowMessageW(_: LPCWSTR) -> UINT; // main.c
    pub fn SetDlgItemTextW(_: HWND, _: ::core::ffi::c_int, _: LPCWSTR) -> BOOL; // printing.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // dialog.c
    pub fn SetWindowTextW(_: HWND, _: LPCWSTR) -> BOOL; // dialog.c
    pub fn ShellAboutW(
        hWnd: HWND,
        szApp: LPCWSTR,
        szOtherStuff: LPCWSTR,
        hIcon: HICON,
    ) -> ::core::ffi::c_int; // dialog.c
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: INT,
    ) -> HINSTANCE; // dialog.c
    pub fn StartDocW(_: HDC, _: *const DOCINFOW) -> ::core::ffi::c_int; // printing.c
    pub fn TranslateAcceleratorW(_: HWND, _: HACCEL, _: LPMSG) -> ::core::ffi::c_int; // main.c
    pub fn WinHelpW(_: HWND, _: LPCWSTR, _: UINT, _: ULONG_PTR) -> BOOL; // dialog.c
    pub fn lstrlenW(_: LPCWSTR) -> ::core::ffi::c_int; // main.c
}
pub use crate::prelude::CallWindowProcW as CallWindowProc; // sdk alias
pub use crate::prelude::ChooseFontW as ChooseFont; // sdk alias
pub use crate::prelude::CreateFileW as CreateFile; // sdk alias
pub use crate::prelude::CreateFontIndirectW as CreateFontIndirect; // sdk alias
pub use crate::prelude::CreateStatusWindowW as CreateStatusWindow; // sdk alias
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DialogBoxParamW as DialogBoxParam; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::DragQueryFileW as DragQueryFile; // sdk alias
pub use crate::prelude::DrawTextW as DrawText; // sdk alias
pub use crate::prelude::FormatMessageW as FormatMessage; // sdk alias
pub use crate::prelude::GetDateFormatW as GetDateFormat; // sdk alias
pub use crate::prelude::GetDlgItemTextW as GetDlgItemText; // sdk alias
pub use crate::prelude::GetFileAttributesW as GetFileAttributes; // sdk alias
pub use crate::prelude::GetFileTitleW as GetFileTitle; // sdk alias
pub use crate::prelude::GetFullPathNameW as GetFullPathName; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetModuleFileNameW as GetModuleFileName; // sdk alias
pub use crate::prelude::GetOpenFileNameW as GetOpenFileName; // sdk alias
pub use crate::prelude::GetSaveFileNameW as GetSaveFileName; // sdk alias
pub use crate::prelude::GetTextExtentPoint32W as GetTextExtentPoint32; // sdk alias
pub use crate::prelude::GetTextMetricsW as GetTextMetrics; // sdk alias
pub use crate::prelude::GetTimeFormatW as GetTimeFormat; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::GetWindowTextLengthW as GetWindowTextLength; // sdk alias
pub use crate::prelude::IsDialogMessageW as IsDialogMessage; // sdk alias
pub use crate::prelude::LoadAcceleratorsW as LoadAccelerators; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::PageSetupDlgW as PageSetupDlg; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::PrintDlgW as PrintDlg; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegOpenKeyW as RegOpenKey; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::RegisterWindowMessageW as RegisterWindowMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use crate::prelude::ShellAboutW as ShellAbout; // sdk alias
pub use crate::prelude::ShellExecuteW as ShellExecute; // sdk alias
pub use crate::prelude::StartDocW as StartDoc; // sdk alias
pub use strsafe_rust::StringCbPrintfW as StringCbPrintf; // sdk alias
pub use strsafe_rust::StringCchCatW as StringCchCat; // sdk alias
pub use strsafe_rust::StringCchCatNW as StringCchCatN; // sdk alias
pub use strsafe_rust::StringCchCopyW as StringCchCopy; // sdk alias
pub use strsafe_rust::StringCchCopyNW as StringCchCopyN; // sdk alias
pub use strsafe_rust::StringCchPrintfW as StringCchPrintf; // sdk alias
pub use crate::prelude::TranslateAcceleratorW as TranslateAccelerator; // sdk alias
pub use crate::prelude::WinHelpW as WinHelp; // sdk alias
pub use crate::prelude::wcscat as _tcscat; // sdk alias
pub use crate::prelude::wcscmp as _tcscmp; // sdk alias
pub use crate::prelude::wcscpy as _tcscpy; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::wcsncmp as _tcsncmp; // sdk alias
pub use crate::prelude::wcsncpy as _tcsncpy; // sdk alias
pub use crate::prelude::_wcsnicmp as _tcsnicmp; // sdk alias
pub use crate::prelude::wcsrchr as _tcsrchr; // sdk alias
pub use crate::prelude::towupper as _totupper; // sdk alias
pub use crate::prelude::lstrlenW as lstrlen; // sdk alias
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
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateWindow { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExW(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! _istalnum { ($p0:expr $(,)?) => { iswctype((($p0) as _), (((((0x0100 | 0x1) | 0x2) | 0x4)) as _)) }; }
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const EDIT_CLASS: *const u16 = (&[0x0045, 0x0044, 0x0049, 0x0054, 0x0000u16]).as_ptr(); // L"EDIT"
pub const FINDMSGSTRINGW: *const u16 = (&[0x0063, 0x006f, 0x006d, 0x006d, 0x0064, 0x006c, 0x0067, 0x005f, 0x0046, 0x0069, 0x006e, 0x0064, 0x0052, 0x0065, 0x0070, 0x006c, 0x0061, 0x0063, 0x0065, 0x0000u16]).as_ptr(); // L"commdlg_FindReplace"
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const IDC_WAIT: LPWSTR = 0x7f02usize as LPWSTR;
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
pub use crate::{CreateWindow, MAKEINTRESOURCE, _istalnum};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/notepad/printing.c:116: #define X_POINTS_TO_PIXELS(hDC, points) MulDiv((points), GetDeviceCaps((hDC), LOGPIXELSX), 72)
#[allow(unused_macros)]
macro_rules! X_POINTS_TO_PIXELS {
    ($hDC:expr, $points:expr $(,)?) => {
        (MulDiv(($points), GetDeviceCaps(($hDC), LOGPIXELSX as _), 72))
    };
}
#[allow(unused_imports)]
pub(crate) use X_POINTS_TO_PIXELS;
// base/applications/notepad/printing.c:117: #define Y_POINTS_TO_PIXELS(hDC, points) MulDiv((points), GetDeviceCaps((hDC), LOGPIXELSY), 72)
#[allow(unused_macros)]
macro_rules! Y_POINTS_TO_PIXELS {
    ($hDC:expr, $points:expr $(,)?) => {
        (MulDiv(($points), GetDeviceCaps(($hDC), LOGPIXELSY as _), 72))
    };
}
#[allow(unused_imports)]
pub(crate) use Y_POINTS_TO_PIXELS;
// ==== end port macros ====
