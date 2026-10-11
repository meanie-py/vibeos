//! regedit-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ---- hand ports (outside the generated blocks, so a replay keeps them) ----
// security.c:63 `impl_to_interface(impl, iface)` is `(struct iface *)(&(impl)->lp##iface##Vtbl)`:
// the address of the interface's vtable slot inside the CRegKeySecurity object. `##` pastes an
// identifier, which macro_rules! cannot, so each interface the object implements is an arm
// (ISecurityInformation2 is compiled out: REGEDIT_IMPLEMENT_ISECURITYINFORMATION2 is 0).
#[allow(unused_macros)]
macro_rules! impl_to_interface {
    ($impl:expr, ISecurityInformation) => {
        core::ptr::addr_of_mut!((*$impl).lpISecurityInformationVtbl) as *mut ISecurityInformation
    };
    ($impl:expr, IEffectivePermission) => {
        core::ptr::addr_of_mut!((*$impl).lpIEffectivePermissionVtbl) as *mut IEffectivePermission
    };
    ($impl:expr, ISecurityObjectTypeInfo) => {
        core::ptr::addr_of_mut!((*$impl).lpISecurityObjectTypeInfoVtbl)
            as *mut ISecurityObjectTypeInfo
    };
}
#[allow(unused_imports)]
pub(crate) use impl_to_interface;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CBus: _INTERFACE_TYPE = _INTERFACE_TYPE_CBus; // island enum
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // control-rust/src/prelude.rs
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const CmResourceShareDeviceExclusive: _CM_SHARE_DISPOSITION = _CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive; // island enum
pub const CmResourceShareDriverExclusive: _CM_SHARE_DISPOSITION = _CM_SHARE_DISPOSITION_CmResourceShareDriverExclusive; // island enum
pub const CmResourceShareShared: _CM_SHARE_DISPOSITION = _CM_SHARE_DISPOSITION_CmResourceShareShared; // island enum
pub const CmResourceShareUndetermined: _CM_SHARE_DISPOSITION = _CM_SHARE_DISPOSITION_CmResourceShareUndetermined; // island enum
pub const DATE_SHORTDATE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const DVASPECT_CONTENT: DWORD = 1; // ole32-rust/src/compobj/com.rs
pub const Eisa: _INTERFACE_TYPE = _INTERFACE_TYPE_Eisa; // island enum
pub const INT_MAX: u32 = 2147483647; // comctl32-rust/src/prelude.rs
pub const InterfaceTypeUndefined: _INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined; // island enum
pub const Internal: _INTERFACE_TYPE = _INTERFACE_TYPE_Internal; // island enum
pub const InternalPowerBus: _INTERFACE_TYPE = _INTERFACE_TYPE_InternalPowerBus; // island enum
pub const Isa: _INTERFACE_TYPE = _INTERFACE_TYPE_Isa; // island enum
pub const MPIBus: _INTERFACE_TYPE = _INTERFACE_TYPE_MPIBus; // island enum
pub const MPSABus: _INTERFACE_TYPE = _INTERFACE_TYPE_MPSABus; // island enum
pub const MicroChannel: _INTERFACE_TYPE = _INTERFACE_TYPE_MicroChannel; // island enum
pub const NM_CLICK: u32 = 0xFFFFFFFE; // comctl32-rust/src/prelude.rs
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // comctl32-rust/src/prelude.rs
pub const NM_RETURN: u32 = 0xFFFFFFFC; // comctl32-rust/src/prelude.rs
pub const NORM_IGNORECASE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const NuBus: _INTERFACE_TYPE = _INTERFACE_TYPE_NuBus; // island enum
pub const PCIBus: _INTERFACE_TYPE = _INTERFACE_TYPE_PCIBus; // island enum
pub const PCMCIABus: _INTERFACE_TYPE = _INTERFACE_TYPE_PCMCIABus; // island enum
pub const PNPBus: _INTERFACE_TYPE = _INTERFACE_TYPE_PNPBus; // island enum
pub const PNPISABus: _INTERFACE_TYPE = _INTERFACE_TYPE_PNPISABus; // island enum
pub const ProcessorInternal: _INTERFACE_TYPE = _INTERFACE_TYPE_ProcessorInternal; // island enum
pub const SE_REGISTRY_KEY: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_REGISTRY_KEY; // island enum
pub const TIME_NOSECONDS: u32 = 2; // appwiz-rust/src/types_amd64.rs
pub const TVHT_ONITEM: i32 = 0x46; // comctl32-rust/src/prelude.rs
pub const TYMED_HGLOBAL: DWORD = 1; // shell32-rust/src/dialogs/drvdefext.rs
pub const TurboChannel: _INTERFACE_TYPE = _INTERFACE_TYPE_TurboChannel; // island enum
pub const VMEBus: _INTERFACE_TYPE = _INTERFACE_TYPE_VMEBus; // island enum
pub const _MAX_PATH: u32 = 260; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_BINARY: u32 = 32768; // msvcrt-sys/src/bindgen_amd64.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
extern "C" {
    pub fn __iob_func() -> *mut FILE; // crt-startup-rust/src/misc.rs
    pub fn _errno() -> *mut c_int; // crt-shared-rust/src/prelude.rs
    pub fn _fileno(stream: *mut c_void) -> c_int; // ucrt-rust/src/output.rs
    pub fn _setmode(fd: i32, mode: i32) -> i32; // conutils-rust/src/conutils/stream.rs
    pub fn _snwprintf(buf: *mut u16, count: usize, format: *const u16, ...) -> c_int; // browser-rust/src/prelude.rs
    pub fn _vscwprintf(format: *const u16, args: *mut u8) -> i32; // msvcrt-crt-rust/src/printf_shells.rs
    pub fn _vsnwprintf(buffer: *mut u16, count: usize, format: *const u16, argptr: va_list) -> i32; // conutils-rust/src/conutils/outstream.rs
    pub fn _wcsdup(arg1: *const wchar_t) -> *mut wchar_t; // msvcrt-sys/src/bindgen_i386.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // control-rust/src/prelude.rs
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn _wfopen(file: *const u16, mode: *const u16) -> *mut c_void; // ucrt-rust/src/stdio/files/open.rs
    pub fn _wperror(str: *const u16); // msvcrt-crt-rust/src/msvcrt/errno.rs
    pub fn abs(j: i32) -> i32; // cnt-rust/src/cabi/math.rs
    pub fn exit(exitcode: i32); // msvcrt-crt-rust/src/msvcrt/exit.rs
    pub fn fclose(stream: *mut FILE) -> c_int; // dnsrslvr-rust/src/prelude.rs
    pub fn fputs(s: *const i8, file: *mut FILE) -> i32; // msvcrt-crt-rust/src/msvcrt/file.rs
    pub fn fread(ptr: *mut core::ffi::c_void, size: usize, nmemb: usize, file: *mut FILE,) -> usize; // msvcrt-crt-rust/src/msvcrt/file.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn fwrite(buffer: *const c_void, size: usize, count: usize, stream: *mut c_void) -> usize; // ucrt-rust/src/stdio/files/readwrite.rs
    pub fn iscntrl(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isdigit(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isprint(c: c_int) -> c_int; // dhcpcsvc-rust/src/prelude.rs
    pub fn isxdigit(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn qsort( base: *mut c_void, num: usize, size: usize, cmp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>, ); // iphlpapi-rust/src/prelude.rs
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn strlen(s: *const c_char) -> usize; // browser-rust/src/prelude.rs
    pub fn strpbrk(s: *const u8, set: *const u8) -> *mut u8; // ucrt-rust/src/argv/wildcards.rs
    pub fn towlower(c: u16) -> u16; // cnt-ctype/src/lib.rs
    pub fn towupper(c: c_int) -> c_int; // dnsapi-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // kernel32-rust/src/prelude.rs
    pub fn wcsnicmp(a: *const wchar_t, b: *const wchar_t, n: size_t) -> c_int; // crt-shared-rust/src/prelude.rs
    pub fn wcspbrk(s: *const u16, set: *const u16) -> *const u16; // ucrt-rust/src/argv/wildcards.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG; // kernel32-rust/src/prelude.rs
}
extern "system" {
    pub fn AdjustTokenPrivileges( arg1: HANDLE, arg2: BOOL, arg3: PTOKEN_PRIVILEGES, arg4: DWORD, arg5: PTOKEN_PRIVILEGES, arg6: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn BeginDeferWindowPos(arg1: ::core::ffi::c_int) -> HDWP; // aclui-rust/src/types_i386.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CallWindowProcW( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn CheckRadioButton( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CommDlgExtendedError() -> DWORD; // desk-rust/src/types_i386.rs
    pub fn CommandLineToArgvW(arg1: LPCWSTR, arg2: *mut ::core::ffi::c_int) -> *mut LPWSTR; // desk-rust/src/types_i386.rs
    pub fn CompareStringW( Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCWSTR, cchCount1: ::core::ffi::c_int, lpString2: LPCWSTR, cchCount2: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn CreateBitmap( cx: INT, cy: INT, cPlanes: UINT, cBitsPerPel: UINT, pvBits: *const ::core::ffi::c_void, ) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn CreateCaret( arg1: HWND, arg2: HBITMAP, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreatePatternBrush(arg1: HBITMAP) -> HBRUSH; // aclui-rust/src/types_i386.rs
    pub fn CreateStatusWindowW(style: LONG, lpszText: LPCWSTR, hwndParent: HWND, wID: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DeferWindowPos( arg1: HDWP, arg2: HWND, arg3: HWND, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: UINT, ) -> HDWP; // aclui-rust/src/types_i386.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyCaret() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn EmptyClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDeferWindowPos(arg1: HDWP) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ExtTextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, options: UINT, lprect: *const RECT, lpString: LPCWSTR, c: UINT, lpDx: *const INT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeInheritedFromArray(pInheritArray: PINHERITED_FROMW, AceCnt: USHORT, pfnArray: PFN_OBJECT_MGR_FUNCTS,) -> DWORD; // advapi32-rust/src/sec/ac.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetAsyncKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetCapture() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetClipboardData(arg1: UINT) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCommandLineW() -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDCEx(arg1: HWND, arg2: HRGN, arg3: DWORD) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDateFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItemTextW( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPWSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetFocus() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetInheritanceSourceW(pObjectName: LPWSTR, ObjectType: SE_OBJECT_TYPE, SecurityInfo: SECURITY_INFORMATION, Container: BOOL, pObjectClassGuids: *mut *mut GUID, GuidCount: DWORD, pAcl: PACL, pfnArray: PFN_OBJECT_MGR_FUNCTS, pGenericMapping: PGENERIC_MAPPING, pInheritArray: PINHERITED_FROMW,) -> DWORD; // advapi32-rust/src/sec/ac.rs
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetMenuItemCount(arg1: HMENU) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetMenuState(arg1: HMENU, arg2: UINT, arg3: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetNamedSecurityInfoW(pObjectName: LPWSTR, ObjectType: SE_OBJECT_TYPE, SecurityInfo: SECURITY_INFORMATION, ppsidOwner: *mut PSID, ppsidGroup: *mut PSID, ppDacl: *mut PACL, ppSacl: *mut PACL, ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR) -> DWORD; // advapi32-rust/src/externs.rs
    pub fn GetNextDlgTabItem(arg1: HWND, arg2: HWND, arg3: BOOL) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetScrollInfo(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPSCROLLINFO) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetSecurityDescriptorDacl( arg1: PSECURITY_DESCRIPTOR, arg2: LPBOOL, arg3: *mut PACL, arg4: LPBOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetTimeFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetUpdateRect(arg1: HWND, arg2: LPRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_i386.rs
    pub fn GetVersion() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalFree(arg1: HGLOBAL) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapSize(arg1: HANDLE, arg2: DWORD, arg3: LPCVOID) -> SIZE_T; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_GetImageCount(himl: HIMAGELIST) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ImageList_ReplaceIcon( himl: HIMAGELIST, i: ::core::ffi::c_int, hicon: HICON, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn InflateRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsCharAlphaNumericW(arg1: WCHAR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDialogMessageW(arg1: HWND, arg2: LPMSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadAcceleratorsW(arg1: HINSTANCE, arg2: LPCWSTR) -> HACCEL; // aclui-rust/src/types_i386.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadMenuW(arg1: HINSTANCE, arg2: LPCWSTR) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalLock(arg1: HLOCAL) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn LocalReAlloc(arg1: HLOCAL, arg2: SIZE_T, arg3: UINT) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalSize(arg1: HLOCAL) -> SIZE_T; // aclui-rust/src/types_i386.rs
    pub fn LocalUnlock(arg1: HLOCAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LookupPrivilegeValueW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: PLUID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MapGenericMask(arg1: PDWORD, arg2: PGENERIC_MAPPING); // aclui-rust/src/types_i386.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MessageBeep(uType: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn OleInitialize(pvReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn OpenClipboard(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PatBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PathFindNextComponentW(arg1: LPCWSTR) -> LPWSTR; // desk-rust/src/types_i386.rs
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn RedrawWindow(arg1: HWND, arg2: LPCRECT, arg3: HRGN, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegDeleteTreeW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // shell32-rust/src/island_i386.rs
    pub fn RegDeleteValueW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegFlushKey(arg1: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegLoadKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryInfoKeyW( hKey: HKEY, lpClass: LPWSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSaveKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: LPSECURITY_ATTRIBUTES) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegUnLoadKeyW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn RegisterClipboardFormatW(arg1: LPCWSTR) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn ReleaseCapture() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ReleaseStgMedium(arg1: LPSTGMEDIUM); // aclui-rust/src/types_i386.rs
    pub fn RemoveMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ScrollWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPCRECT, arg5: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SearchPathW( lpPath: LPCWSTR, lpFileName: LPCWSTR, lpExtension: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetBkMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SetCapture(hWnd: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetCaretPos(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetClipboardData(arg1: UINT, arg2: HANDLE) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn SetCursorPos(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemInt(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SetMenuDefaultItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetScrollInfo( arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCSCROLLINFO, arg4: BOOL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn SetWindowPlacement(hWnd: HWND, arg1: *const WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShellAboutW( hWnd: HWND, szApp: LPCWSTR, szOtherStuff: LPCWSTR, hIcon: HICON, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ShowCaret(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowScrollBar(arg1: HWND, arg2: ::core::ffi::c_int, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn StrCmpLogicalW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn StrCmpNW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: INT) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, lpString: LPCWSTR, c: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateAcceleratorW(arg1: HWND, arg2: HACCEL, arg3: LPMSG) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnregisterClassW(arg1: LPCWSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WinHelpW(arg1: HWND, arg2: LPCWSTR, arg3: UINT, arg4: ULONG_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const LVN_BEGINLABELEDIT: u32 = 0xFFFFFF51; // clang, listview.c
pub const LVN_ENDLABELEDIT: u32 = 0xFFFFFF50; // clang, listview.c
pub const LVN_GETDISPINFO: u32 = 0xFFFFFF4F; // clang, listview.c
pub const NM_SETFOCUS: u32 = 0xFFFFFFF9; // clang, listview.c
pub const TVN_BEGINLABELEDIT: u32 = 0xFFFFFE35; // clang, treeview.c
pub const TVN_ENDLABELEDIT: u32 = 0xFFFFFE34; // clang, treeview.c
pub const TVN_ITEMEXPANDING: u32 = 0xFFFFFE3A; // clang, treeview.c
pub const TVN_SELCHANGED: u32 = 0xFFFFFE3D; // clang, treeview.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, listview.c
pub type HIMAGELIST = *mut _IMAGELIST;
// HTREEITEM: c2rust, childwnd.c
pub type HTREEITEM = *mut _TREEITEM;
// SI_PAGE_TYPE: c2rust, security.c
pub type SI_PAGE_TYPE = _SI_PAGE_TYPE;
// parser_state: c2rust, regproc.c
pub type parser_state = ::core::ffi::c_uint;
// reg_versions: c2rust, regproc.c
pub type reg_versions = ::core::ffi::c_uint;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wsprintfW(_: LPWSTR, _: LPCWSTR, ...) -> ::core::ffi::c_int; // edit.c
}
extern "system" {
    pub fn AppendMenuW(_: HMENU, _: UINT, _: UINT_PTR, _: LPCWSTR) -> BOOL; // childwnd.c
    pub fn BuildTrusteeWithSidW(_: PTRUSTEE_W, _: PSID); // security.c
    pub fn CreateFontIndirectW(_: *const LOGFONTW) -> HFONT; // hexedit.c
    pub fn GetEffectiveRightsFromAclW(_: PACL, _: PTRUSTEE_W, _: PACCESS_MASK) -> DWORD; // security.c
    pub fn GetMenuItemInfoW(_: HMENU, _: UINT, _: BOOL, _: LPMENUITEMINFOW) -> BOOL; // childwnd.c
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // hexedit.c
    pub fn GetOpenFileNameW(_: LPOPENFILENAMEW) -> BOOL; // framewnd.c
    pub fn GetSaveFileNameW(_: LPOPENFILENAMEW) -> BOOL; // framewnd.c
    pub fn GetTextMetricsW(_: HDC, _: LPTEXTMETRICW) -> BOOL; // hexedit.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // childwnd.c
    pub fn InsertMenuItemW(_: HMENU, _: UINT, _: BOOL, _: LPCMENUITEMINFOW) -> BOOL; // childwnd.c
    pub fn PrintDlgW(_: LPPRINTDLGW) -> BOOL; // framewnd.c
    pub fn RegRestoreKeyW(_: HKEY, _: LPCWSTR, _: DWORD) -> LSTATUS; // framewnd.c
    pub fn SHDeleteKeyW(_: HKEY, _: LPCWSTR) -> DWORD; // edit.c
    pub fn SetMenuItemInfoW(_: HMENU, _: UINT, _: BOOL, _: LPCMENUITEMINFOW) -> BOOL; // childwnd.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // childwnd.c
}
pub use crate::prelude::AppendMenuW as AppendMenu; // sdk alias
pub use crate::prelude::BuildTrusteeWithSidW as BuildTrusteeWithSid; // sdk alias
pub use crate::prelude::CreateFontIndirectW as CreateFontIndirect; // sdk alias
pub use crate::prelude::GetEffectiveRightsFromAclW as GetEffectiveRightsFromAcl; // sdk alias
pub use crate::prelude::GetMenuItemInfoW as GetMenuItemInfo; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::GetObjectW as GetObject; // sdk alias
pub use crate::prelude::GetOpenFileNameW as GetOpenFileName; // sdk alias
pub use crate::prelude::GetSaveFileNameW as GetSaveFileName; // sdk alias
pub use crate::prelude::GetTextMetricsW as GetTextMetrics; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::GetWindowTextLengthW as GetWindowTextLength; // sdk alias
pub use crate::prelude::InsertMenuItemW as InsertMenuItem; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::PrintDlgW as PrintDlg; // sdk alias
pub use crate::prelude::RegDeleteValueW as RegDeleteValue; // sdk alias
pub use crate::prelude::RegRestoreKeyW as RegRestoreKey; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SHDeleteKeyW as SHDeleteKey; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetMenuItemInfoW as SetMenuItemInfo; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::wsprintfW as wsprintf; // sdk alias
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
pub type CREATESTRUCT = CREATESTRUCTW;
// ==== end UNICODE TCHAR family ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static CLSID_AutoComplete: GUID = GUID { Data1: 0x00bb2763, Data2: 0x6a77, Data3: 0x11d0, Data4: [0xa5, 0x35, 0x00, 0xc0, 0x4f, 0xd7, 0xd0, 0x62] };
#[no_mangle]
pub static CLSID_DsObjectPicker: GUID = GUID { Data1: 0x17d6ccd8, Data2: 0x3b7b, Data3: 0x11d2, Data4: [0x00b9, 0xe0, 0x00, 0xc0, 0x4f, 0xd8, 0xdb, 0xf7] };
#[no_mangle]
pub static IID_IDsObjectPicker: GUID = GUID { Data1: 0x0c87e64e, Data2: 0x3b7a, Data3: 0x11d2, Data4: [0x00b9, 0xe0, 0x00, 0xc0, 0x4f, 0xd8, 0xdb, 0xf7] };
// ==== end INITGUID storage ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateDialogW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { CreateDialogParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! DialogBoxW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! Header_GetItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1200 + 11)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! Header_SetItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1200 + 12)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ImageList_AddIcon { ($p0:expr, $p1:expr $(,)?) => { ImageList_ReplaceIcon((($p0) as _), (((-(1 as c_int))) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! ListView_DeleteAllItems { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 9)) as _), ((0) as _), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_EditLabel { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 118)) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as HWND) }; }
#[macro_export] macro_rules! ListView_EnsureVisible { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 19)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_FindItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 83)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetColumnWidth { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 29)) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetHeader { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 31)) as _), ((0) as _), ((0) as _))) as HWND) }; }
#[macro_export] macro_rules! ListView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 75)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetNextItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 12)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetSelectedCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 50)) as _), ((0) as _), ((0) as _))) as UINT) }; }
#[macro_export] macro_rules! ListView_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 97)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 77)) as _), ((0) as _), (($p1) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_SetColumnWidth { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 30)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_SetExtendedListViewStyle { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 54)) as _), ((0) as _), (($p1) as _))) as DWORD) }; }
#[macro_export] macro_rules! ListView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 3)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! ListView_SetItemState { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = (($p3) as _); }; { _ms_lvi.state = (($p2) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! ListView_SetItemText { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.iSubItem = (($p2) as _); }; { _ms_lvi.pszText = (($p3) as _); }; SendMessageW((($p0) as _), (((0x1000 + 116)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! ListView_SortItems { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 48)) as _), (((($p2) as LPARAM)) as WPARAM), ((core::mem::transmute::<PFNLVCOMPARE, usize>(({ let __cb: PFNLVCOMPARE = $p1; __cb }))) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as WPARAM) }; }
#[macro_export] macro_rules! TreeView_DeleteItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 1)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_EditLabel { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 65)) as _), ((0) as _), (($p1) as LPARAM))) as HWND) }; }
#[macro_export] macro_rules! TreeView_EnsureVisible { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 20)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_Expand { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 2)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetChild { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x4) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetEditControl { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 15)) as _), ((0) as _), ((0) as _))) as HWND) }; }
#[macro_export] macro_rules! TreeView_GetImageList { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 8)) as _), (($p1) as _), ((0) as _))) as HIMAGELIST) }; }
#[macro_export] macro_rules! TreeView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 62)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetItemRect { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { { (*(($p2) as *mut HTREEITEM)) = (($p1) as _); }; ((SendMessageW((($p0) as _), (((0x1100 + 4)) as _), (($p3) as WPARAM), (($p2) as LPARAM))) as BOOL) } }; }
#[macro_export] macro_rules! TreeView_GetItemState { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 39)) as _), (($p1) as WPARAM), (($p2) as LPARAM))) as UINT) }; }
#[macro_export] macro_rules! TreeView_GetNextSibling { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x1) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetParent { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x3) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetRoot { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x0) as WPARAM), ((core::ptr::null_mut::<c_void>()) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetSelection { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x9) as WPARAM), ((core::ptr::null_mut::<c_void>()) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_HitTest { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 17)) as _), ((0) as _), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 50)) as _), ((0) as _), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_Select { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 11)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_SelectItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 11)) as _), ((0x9) as WPARAM), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 9)) as _), (($p2) as _), (($p1) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! TreeView_SetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 63)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const CFSTR_DSOP_DS_SELECTION_LIST: *const u16 = (&[0x0043, 0x0046, 0x0053, 0x0054, 0x0052, 0x005f, 0x0044, 0x0053, 0x004f, 0x0050, 0x005f, 0x0044, 0x0053, 0x005f, 0x0053, 0x0045, 0x004c, 0x0045, 0x0043, 0x0054, 0x0049, 0x004f, 0x004e, 0x005f, 0x004c, 0x0049, 0x0053, 0x0054, 0x0000u16]).as_ptr(); // L"CFSTR_DSOP_DS_SELECTION_LIST"
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub type HDITEM = HDITEMW;
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_CURRENT_CONFIG: HKEY = ((((((0x80000005u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_DYN_DATA: HKEY = ((((((0x80000006u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_USERS: HKEY = ((((((0x80000003u32) as LONG)) as LONG_PTR)) as HKEY);
#[inline]
pub unsafe fn IAutoComplete_Init(This: *mut IAutoComplete, hwndEdit: HWND, punkACL: *mut IUnknown, pwszRegKeyPath: LPCOLESTR, pwszQuickComplete: LPCOLESTR) -> HRESULT { ((*(*This).lpVtbl).Init.unwrap_unchecked())(This, hwndEdit, punkACL, pwszRegKeyPath, pwszQuickComplete) } // widl C wrapper
#[inline]
pub unsafe fn IAutoComplete_Release(This: *mut IAutoComplete) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const IDC_IBEAM: LPWSTR = 0x7f01usize as LPWSTR;
pub const IDC_SIZEWE: LPWSTR = 0x7f84usize as LPWSTR;
pub const IDC_WAIT: LPWSTR = 0x7f02usize as LPWSTR;
pub const LOCALE_SYSTEM_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x02) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
pub const LPSTR_TEXTCALLBACK: LPWSTR = (((-(1 as c_int))) as LPWSTR);
pub type LVCOLUMN = LVCOLUMNW;
pub type NMLVDISPINFO = NMLVDISPINFOW;
pub type NMTREEVIEW = NMTREEVIEWW;
pub type REFIID = *const IID;
pub const SE_BACKUP_NAME: *const u16 = (&[0x0053, 0x0065, 0x0042, 0x0061, 0x0063, 0x006b, 0x0075, 0x0070, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006c, 0x0065, 0x0067, 0x0065, 0x0000u16]).as_ptr(); // L"SeBackupPrivilege"
pub const SE_RESTORE_NAME: *const u16 = (&[0x0053, 0x0065, 0x0052, 0x0065, 0x0073, 0x0074, 0x006f, 0x0072, 0x0065, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006c, 0x0065, 0x0067, 0x0065, 0x0000u16]).as_ptr(); // L"SeRestorePrivilege"
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub type TVITEM = TVITEMW;
pub const TVI_FIRST: HTREEITEM = (((((-(0xffff as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_LAST: HTREEITEM = (((((-(0xfffe as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_ROOT: HTREEITEM = (((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const WC_LISTVIEW: *const u16 = (&[0x0053, 0x0079, 0x0073, 0x004c, 0x0069, 0x0073, 0x0074, 0x0056, 0x0069, 0x0065, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"SysListView32"
pub const WC_TREEVIEW: *const u16 = (&[0x0053, 0x0079, 0x0073, 0x0054, 0x0072, 0x0065, 0x0065, 0x0056, 0x0069, 0x0065, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"SysTreeView32"
pub use crate::{CreateDialogW, DialogBoxW, Header_GetItem, Header_SetItem, ImageList_AddIcon, IsEqualIID, ListView_DeleteAllItems, ListView_EditLabel, ListView_EnsureVisible, ListView_FindItem, ListView_GetColumnWidth, ListView_GetHeader, ListView_GetItem, ListView_GetItemCount, ListView_GetNextItem, ListView_GetSelectedCount, ListView_InsertColumn, ListView_InsertItem, ListView_SetColumnWidth, ListView_SetExtendedListViewStyle, ListView_SetImageList, ListView_SetItemState, ListView_SetItemText, ListView_SortItems, MAKELPARAM, MAKEWPARAM, TreeView_DeleteItem, TreeView_EditLabel, TreeView_EnsureVisible, TreeView_Expand, TreeView_GetChild, TreeView_GetEditControl, TreeView_GetImageList, TreeView_GetItem, TreeView_GetItemRect, TreeView_GetItemState, TreeView_GetNextSibling, TreeView_GetParent, TreeView_GetRoot, TreeView_GetSelection, TreeView_HitTest, TreeView_InsertItem, TreeView_Select, TreeView_SelectItem, TreeView_SetImageList, TreeView_SetItem, max, min, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/regedit/main.h:20: #define ARRAY_SIZE(A) (sizeof(A)/sizeof(*A))
#[allow(unused_macros)]
macro_rules! ARRAY_SIZE {
    ($A:expr $(,)?) => {
        (core::mem::size_of_val(&$A) / core::mem::size_of_val(&$A[0]))
    };
}
#[allow(unused_imports)]
pub(crate) use ARRAY_SIZE;
// base/applications/regedit/hexedit.h:30: #define HexEdit_CopyBuffer(hWnd, Buffer, nMax) SendMessageW((hWnd), HEM_COPYBUFFER, (WPARAM)(Buffer), (LPARAM)(nMax))
#[allow(unused_macros)]
macro_rules! HexEdit_CopyBuffer {
    ($hWnd:expr, $Buffer:expr, $nMax:expr $(,)?) => {
        (SendMessageW($hWnd, HEM_COPYBUFFER, ($Buffer) as WPARAM, ($nMax) as LPARAM))
    };
}
#[allow(unused_imports)]
pub(crate) use HexEdit_CopyBuffer;
// base/applications/regedit/hexedit.h:33: #define HexEdit_GetBufferSize(hWnd) SendMessageW((hWnd), HEM_COPYBUFFER, 0, 0)
#[allow(unused_macros)]
macro_rules! HexEdit_GetBufferSize {
    ($hWnd:expr $(,)?) => {
        (SendMessageW($hWnd, HEM_COPYBUFFER, 0, 0))
    };
}
#[allow(unused_imports)]
pub(crate) use HexEdit_GetBufferSize;
// base/applications/regedit/hexedit.h:24: #define HexEdit_LoadBuffer(hWnd, Buffer, Size) SendMessageW((hWnd), HEM_LOADBUFFER, (WPARAM)(Buffer), (LPARAM)(Size))
#[allow(unused_macros)]
macro_rules! HexEdit_LoadBuffer {
    ($hWnd:expr, $Buffer:expr, $Size:expr $(,)?) => {
        (SendMessageW($hWnd, HEM_LOADBUFFER, ($Buffer) as WPARAM, ($Size) as LPARAM))
    };
}
#[allow(unused_imports)]
pub(crate) use HexEdit_LoadBuffer;
// base/applications/regedit/hexedit.h:36: #define HexEdit_SetMaxBufferSize(hWnd, Size) SendMessageW((hWnd), HEM_SETMAXBUFFERSIZE, 0, (LPARAM)(Size))
#[allow(unused_macros)]
macro_rules! HexEdit_SetMaxBufferSize {
    ($hWnd:expr, $Size:expr $(,)?) => {
        (SendMessageW($hWnd, HEM_SETMAXBUFFERSIZE, 0, ($Size) as LPARAM))
    };
}
#[allow(unused_imports)]
pub(crate) use HexEdit_SetMaxBufferSize;
// base/applications/regedit/childwnd.c:199: #define MIN(a, b) ((a < b) ? (a) : (b))
#[allow(unused_macros)]
macro_rules! MIN {
    ($a:expr, $b:expr $(,)?) => {
        (if $a < $b { $a } else { $b })
    };
}
#[allow(unused_imports)]
pub(crate) use MIN;
// ==== end port macros ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::regedit::childwnd::g_pChildWnd;
pub use crate::regedit::settings::g_szGeneralRegKey;
pub use crate::regedit::main::hFrameWnd;
pub use crate::regedit::main::hInst;
pub use crate::regedit::main::hMenuFrame;
pub use crate::regedit::main::hPopupMenus;
pub use crate::regedit::main::hStatusBar;
pub use crate::regedit::main::reg_class_namesW;
pub use crate::regedit::main::szChildClass;
pub use crate::regedit::main::szTitle;
// ==== end cross-TU definitions ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_regedit: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x72\x65\x67\x65\x64\x69\x74\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_regedit };
// ==== end wine debug channels ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn ErrorMessageBox(hWnd: HWND, lpTitle: LPCWSTR, dwErrorCode: DWORD, ...) -> ::core::ffi::c_int;
    pub fn InfoMessageBox(hWnd: HWND, uType: UINT, lpTitle: LPCWSTR, lpMessage: LPCWSTR, ...) -> core::ffi::c_int;
    pub fn error(hwnd: HWND, resId: INT, ...);
    pub fn error_exit(id: ::core::ffi::c_uint, ...);
    pub fn output_message(id: ::core::ffi::c_uint, ...);
    pub fn txt_fprintf(fp: *mut FILE, format: LPCWSTR, ...);
    pub fn warning(hwnd: HWND, resId: INT, ...);
}
// ==== end port variadic ====
