//! wow64-rust prelude — the single import surface the translated modules use.
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
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => { ::win32macros_rust::DBG_TYPECHECK!($($arg)*) }; }
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, strlenW, tolowerW, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= wow64 hand declarations =====================================================
// `VOID` is a #define in the SDK, so the island never carries it.
pub type VOID = c_void;
// Defined by the KEPT-C wow64_thunks.c (the syscall-trace switch); every Rust reader imports it.
extern "C" {
    pub static mut Wow64TraceSyscalls: ULONG;
    // wow64_thunks.c (kept C): the generated syscall table's dispatcher.
    pub fn Wow64DispatchSystemService(ServiceNumber: ULONG, Arguments: PULONG) -> LONG;
}

// wow64.c's WOW64_HOST_PC(Ctx): the host CONTEXT's program counter, per arch (the macros
// stage refused it for having two definitions; these are the C's two arms).
#[cfg(target_arch = "aarch64")]
macro_rules! WOW64_HOST_PC { ($ctx:expr) => { ((*$ctx).Pc as PVOID) }; }
#[cfg(target_arch = "x86_64")]
macro_rules! WOW64_HOST_PC { ($ctx:expr) => { ((*$ctx).Rip as PVOID) }; }
pub(crate) use WOW64_HOST_PC;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
extern "C" {
    pub fn NtUserSetClassLongPtr( hWnd: HWND, Offset: ::core::ffi::c_int, dwNewLong: ULONG_PTR, Ansi: ::core::ffi::c_int, ) -> ULONG_PTR; // win32k-sys/src/bindgen_amd64.rs // gen:7775a17f5e77
    pub fn NtUserSetWindowLongPtr( hWnd: HWND, Index: ::core::ffi::c_ulong, NewValue: LONG_PTR, Ansi: ::core::ffi::c_int, ) -> LONG_PTR; // win32k-sys/src/bindgen_amd64.rs // gen:06a757d05b44
}
extern "system" {
    pub fn NtCancelIoFileEx( arg1: HANDLE, arg2: PIO_STATUS_BLOCK, arg3: PIO_STATUS_BLOCK, ) -> NTSTATUS; // winegdi-rust/src/types_i386.rs // gen:0607745e9062
    pub fn NtCancelSynchronousIoFile( arg1: HANDLE, arg2: PIO_STATUS_BLOCK, arg3: PIO_STATUS_BLOCK, ) -> NTSTATUS; // winegdi-rust/src/types_i386.rs // gen:2833737d1275
    pub fn NtGdiCreateDIBSection( hdc: HDC, hSectionApp: *mut ::core::ffi::c_void, dwOffset: ::core::ffi::c_ulong, pbmi: LPBITMAPINFO, iUsage: ::core::ffi::c_ulong, cjHeader: ::core::ffi::c_uint, fl: ::core::ffi::c_ulong, dwColorSpace: ULONG_PTR, ppvBits: *mut PVOID, ) -> HBITMAP; // win32k-sys/src/bindgen_i386.rs // gen:dce2ebfe86ed
    pub fn NtGdiDeleteObjectApp(hobj: *mut ::core::ffi::c_void) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:ddaa3f312ed1
    pub fn NtGdiExtGetObjectW(h: *mut ::core::ffi::c_void, cj: ::core::ffi::c_int, pvOut: *mut ::core::ffi::c_void) -> INT; // win32k-sys/src/bindgen_i386.rs // gen:7b75093b24e5
    pub fn NtGdiOpenDCW( pustrDevice: PUNICODE_STRING, pdm: *mut DEVMODEW, pustrLogAddr: PUNICODE_STRING, iType: ::core::ffi::c_ulong, bDisplay: ::core::ffi::c_int, hspool: *mut ::core::ffi::c_void, pUMdhpdev: *mut ::core::ffi::c_void, ) -> HDC; // win32k-sys/src/bindgen_i386.rs // gen:4cf11694bf41
    pub fn NtGetNextProcess( ProcessHandle: *mut ::core::ffi::c_void, DesiredAccess: ACCESS_MASK, HandleAttributes: ::core::ffi::c_ulong, Flags: ::core::ffi::c_ulong, NewProcessHandle: *mut *mut ::core::ffi::c_void, ) -> NTSTATUS; // winsrv-sys/src/bindgen_i386.rs // gen:c4f712688c57
    pub fn NtGetNextThread( arg1: HANDLE, arg2: HANDLE, arg3: ACCESS_MASK, arg4: ULONG, arg5: ULONG, arg6: *mut HANDLE, ) -> NTSTATUS; // winegdi-rust/src/types_i386.rs // gen:ecc4d8df2369
    pub fn NtRemoveIoCompletionEx( arg1: HANDLE, arg2: *mut FILE_IO_COMPLETION_INFORMATION, arg3: ULONG, arg4: *mut ULONG, arg5: *mut LARGE_INTEGER, arg6: BOOLEAN, ) -> NTSTATUS; // winegdi-rust/src/types_i386.rs // gen:3f9f301d5166
    pub fn NtUserBeginPaint(hWnd: HWND, lPs: *mut PAINTSTRUCT) -> HDC; // win32k-sys/src/bindgen_i386.rs // gen:64488ecb2e78
    pub fn NtUserBuildHwndList( hDesktop: HDESK, hwndParent: HWND, bChildren: ::core::ffi::c_uchar, dwThreadId: ::core::ffi::c_ulong, cHwnd: ::core::ffi::c_ulong, phwndList: *mut HWND, pcHwndNeeded: *mut ULONG, ) -> NTSTATUS; // win32k-sys/src/bindgen_i386.rs // gen:75f9e956fa37
    pub fn NtUserCallMsgFilter(msg: LPMSG, code: ::core::ffi::c_int) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:307d3463f6b1
    pub fn NtUserChangeDisplaySettings( lpszDeviceName: PUNICODE_STRING, lpDevMode: LPDEVMODEW, dwflags: ::core::ffi::c_ulong, lParam: *mut ::core::ffi::c_void, ) -> LONG; // win32k-sys/src/bindgen_i386.rs // gen:d70c16adb7da
    pub fn NtUserCreateDesktop( poa: POBJECT_ATTRIBUTES, lpszDesktopDevice: PUNICODE_STRING, lpdmw: LPDEVMODEW, dwFlags: ::core::ffi::c_ulong, dwDesiredAccess: ACCESS_MASK, ) -> HDESK; // win32k-sys/src/bindgen_i386.rs // gen:1df250b8bce0
    pub fn NtUserCreateWindowEx( dwExStyle: ::core::ffi::c_ulong, plstrClassName: PLARGE_STRING, plstrClsVersion: PLARGE_STRING, plstrWindowName: PLARGE_STRING, dwStyle: ::core::ffi::c_ulong, x: ::core::ffi::c_int, y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: *mut ::core::ffi::c_void, dwFlags: ::core::ffi::c_ulong, acbiBuffer: *mut ::core::ffi::c_void, ) -> HWND; // win32k-sys/src/bindgen_i386.rs // gen:d16c5628cc12
    pub fn NtUserCreateWindowStation( ObjectAttributes: POBJECT_ATTRIBUTES, dwDesiredAccess: ACCESS_MASK, Unknown2: ::core::ffi::c_ulong, Unknown3: ::core::ffi::c_ulong, Unknown4: ::core::ffi::c_ulong, Unknown5: ::core::ffi::c_ulong, Unknown6: ::core::ffi::c_ulong, ) -> HWINSTA; // win32k-sys/src/bindgen_i386.rs // gen:f8050d388bf8
    pub fn NtUserDefSetText(WindowHandle: HWND, WindowText: PLARGE_STRING) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:79336ff8a35c
    pub fn NtUserDispatchMessage(pMsg: PMSG) -> LRESULT; // win32k-sys/src/bindgen_i386.rs // gen:7a7ba10bedca
    pub fn NtUserEndPaint(hWnd: HWND, lPs: *const PAINTSTRUCT) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:80efa5bc8f0e
    pub fn NtUserEnumDisplayDevices( lpDevice: PUNICODE_STRING, iDevNum: ::core::ffi::c_ulong, lpDisplayDevice: PDISPLAY_DEVICEW, dwFlags: ::core::ffi::c_ulong, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:ea1c45d924f3
    pub fn NtUserEnumDisplayMonitors( hDC: HDC, pRect: LPCRECT, hMonitorList: *mut HMONITOR, monitorRectList: LPRECT, listSize: ::core::ffi::c_ulong, ) -> INT; // winsrv-sys/src/bindgen_i386.rs // gen:2d373f043265
    pub fn NtUserEnumDisplaySettings( lpszDeviceName: PUNICODE_STRING, iModeNum: ::core::ffi::c_ulong, lpDevMode: LPDEVMODEW, dwFlags: ::core::ffi::c_ulong, ) -> NTSTATUS; // win32k-sys/src/bindgen_i386.rs // gen:fdf9ce9424a4
    pub fn NtUserFindExistingCursorIcon( pustrModule: PUNICODE_STRING, pustrRsrc: PUNICODE_STRING, param: *mut FINDEXISTINGCURICONPARAM, ) -> HICON; // win32k-sys/src/bindgen_i386.rs // gen:d4b14765a94e
    pub fn NtUserFindWindowEx( hwndParent: HWND, hwndChildAfter: HWND, ucClassName: PUNICODE_STRING, ucWindowName: PUNICODE_STRING, dwUnknown: ::core::ffi::c_ulong, ) -> HWND; // win32k-sys/src/bindgen_i386.rs // gen:d3f164a3bc07
    pub fn NtUserGetAtomName(atom: ATOM, pustrName: PUNICODE_STRING) -> ULONG; // win32k-sys/src/bindgen_i386.rs // gen:00c96f86543f
    pub fn NtUserGetCPD(hWnd: HWND, Flags: GETCPD, Proc: ULONG_PTR) -> ULONG_PTR; // win32k-sys/src/bindgen_i386.rs // gen:058b74e4bb62
    pub fn NtUserGetClassInfo( hInstance: HINSTANCE, ClassName: PUNICODE_STRING, wcex: LPWNDCLASSEXW, ppszMenuName: *mut LPWSTR, Ansi: ::core::ffi::c_int, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:33446b06eb20
    pub fn NtUserGetClassName(hWnd: HWND, Real: ::core::ffi::c_int, ClassName: PUNICODE_STRING) -> INT; // win32k-sys/src/bindgen_i386.rs // gen:85152a74a185
    pub fn NtUserGetComboBoxInfo(hWnd: HWND, pcbi: PCOMBOBOXINFO) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:ae09a36462f6
    pub fn NtUserGetCursorInfo(pci: PCURSORINFO) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:d95e08573bfd
    pub fn NtUserGetGUIThreadInfo(idThread: ::core::ffi::c_ulong, lpgui: LPGUITHREADINFO) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:c5c7395bfe86
    pub fn NtUserGetIconInfo( hCurIcon: *mut ::core::ffi::c_void, IconInfo: PICONINFO, lpInstName: PUNICODE_STRING, lpResName: PUNICODE_STRING, pbpp: LPDWORD, bInternal: ::core::ffi::c_int, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:b9e204df31b7
    pub fn NtUserGetKeyboardLayoutName(pustrName: PUNICODE_STRING) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:f53e2fafcde0
    pub fn NtUserGetMessage(pMsg: PMSG, hWnd: HWND, MsgFilterMin: ::core::ffi::c_uint, MsgFilterMax: ::core::ffi::c_uint) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:8131a8e08d5e
    pub fn NtUserGetWOWClass(hInstance: HINSTANCE, ClassName: PUNICODE_STRING) -> PCLS; // win32k-sys/src/bindgen_i386.rs // gen:bc96a85bc9af
    pub fn NtUserLoadKeyboardLayoutEx( hFile: *mut ::core::ffi::c_void, offTable: ::core::ffi::c_ulong, pTables: *mut ::core::ffi::c_void, hOldKL: HKL, puszKLID: PUNICODE_STRING, dwNewKL: ::core::ffi::c_ulong, Flags: ::core::ffi::c_uint, ) -> HKL; // win32k-sys/src/bindgen_i386.rs // gen:a8aa36605ec7
    pub fn NtUserMessageCall( hWnd: HWND, Msg: ::core::ffi::c_uint, wParam: WPARAM, lParam: LPARAM, ResultInfo: ULONG_PTR, dwType: ::core::ffi::c_ulong, Ansi: ::core::ffi::c_int, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:65fb07ee3a13
    pub fn NtUserOpenDesktop( ObjectAttributes: POBJECT_ATTRIBUTES, dwFlags: ::core::ffi::c_ulong, dwDesiredAccess: ACCESS_MASK, ) -> HDESK; // win32k-sys/src/bindgen_i386.rs // gen:3439c33e91a7
    pub fn NtUserOpenWindowStation( ObjectAttributes: POBJECT_ATTRIBUTES, dwDesiredAccess: ACCESS_MASK, ) -> HWINSTA; // win32k-sys/src/bindgen_i386.rs // gen:c8cf86eee117
    pub fn NtUserPeekMessage( pMsg: PMSG, hWnd: HWND, MsgFilterMin: ::core::ffi::c_uint, MsgFilterMax: ::core::ffi::c_uint, RemoveMsg: ::core::ffi::c_uint, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:e36dda2c0205
    pub fn NtUserProcessConnect( ProcessHandle: *mut ::core::ffi::c_void, pUserConnect: PUSERCONNECT, Size: ::core::ffi::c_ulong, ) -> NTSTATUS; // win32k-sys/src/bindgen_i386.rs // gen:415b4a443692
    pub fn NtUserRealInternalGetMessage( lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: ::core::ffi::c_uint, wMsgFilterMax: ::core::ffi::c_uint, wRemoveMsg: ::core::ffi::c_uint, bGMSG: ::core::ffi::c_int, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:cf6b4ae39acb
    pub fn NtUserRegisterClassExWOW( lpwcx: *mut WNDCLASSEXW, pustrClassName: PUNICODE_STRING, pustrCVersion: PUNICODE_STRING, pClassMenuName: PCLSMENUNAME, fnID: ::core::ffi::c_ulong, Flags: ::core::ffi::c_ulong, pWow: LPDWORD, ) -> RTL_ATOM; // win32k-sys/src/bindgen_i386.rs // gen:f17bc0063212
    pub fn NtUserRegisterUserApiHook( m_dllname1: PUNICODE_STRING, m_funname1: PUNICODE_STRING, dwUnknown3: ::core::ffi::c_ulong, dwUnknown4: ::core::ffi::c_ulong, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:bb069ce39e15
    pub fn NtUserRegisterWindowMessage(MessageName: PUNICODE_STRING) -> UINT; // win32k-sys/src/bindgen_i386.rs // gen:d78ecaf210fb
    pub fn NtUserResolveDesktop( ProcessHandle: *mut ::core::ffi::c_void, DesktopPath: PUNICODE_STRING, bInherit: ::core::ffi::c_int, phWinSta: *mut HWINSTA, ) -> HDESK; // win32k-sys/src/bindgen_i386.rs // gen:3e955376363b
    pub fn NtUserSendInput(nInputs: ::core::ffi::c_uint, pInput: LPINPUT, cbSize: ::core::ffi::c_int) -> UINT; // win32k-sys/src/bindgen_i386.rs // gen:ab2775b59484
    pub fn NtUserSetClassLong(hWnd: HWND, Offset: ::core::ffi::c_int, dwNewLong: ::core::ffi::c_ulong, Ansi: ::core::ffi::c_int) -> ULONG_PTR; // win32k-sys/src/bindgen_i386.rs // gen:260c0e4a23d3
    pub fn NtUserSetWinEventHook( eventMin: ::core::ffi::c_uint, eventMax: ::core::ffi::c_uint, hmodWinEventProc: HMODULE, puString: PUNICODE_STRING, lpfnWinEventProc: WINEVENTPROC, idProcess: ::core::ffi::c_ulong, idThread: ::core::ffi::c_ulong, dwflags: ::core::ffi::c_uint, ) -> HWINEVENTHOOK; // win32k-sys/src/bindgen_i386.rs // gen:7f8fc5546615
    pub fn NtUserSetWindowLong(hWnd: HWND, Index: ::core::ffi::c_ulong, NewValue: ::core::ffi::c_long, Ansi: ::core::ffi::c_int) -> LONG; // win32k-sys/src/bindgen_i386.rs // gen:51f5ea9408b8
    pub fn NtUserSetWindowsHookEx( Mod: HINSTANCE, ModuleName: PUNICODE_STRING, ThreadId: ::core::ffi::c_ulong, HookId: ::core::ffi::c_int, HookProc: HOOKPROC, Ansi: ::core::ffi::c_int, ) -> HHOOK; // win32k-sys/src/bindgen_i386.rs // gen:dac9fa7ec55b
    pub fn NtUserThunkedMenuInfo(hMenu: HMENU, lpcmi: LPCMENUINFO) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:d767ed1c15c4
    pub fn NtUserThunkedMenuItemInfo( hMenu: HMENU, uItem: ::core::ffi::c_uint, fByPosition: ::core::ffi::c_int, bInsert: ::core::ffi::c_int, lpmii: LPMENUITEMINFOW, lpszCaption: PUNICODE_STRING, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:d2656e9fde70
    pub fn NtUserTrackMouseEvent(lpEventTrack: LPTRACKMOUSEEVENT) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:950bd4d098f6
    pub fn NtUserTranslateAccelerator( Window: HWND, Table: HACCEL, Message: LPMSG, ) -> ::core::ffi::c_int; // win32k-sys/src/bindgen_i386.rs // gen:f091a64af360
    pub fn NtUserTranslateMessage(lpMsg: LPMSG, flags: ::core::ffi::c_uint) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:20bfa915b442
    pub fn NtUserUnregisterClass( ClassNameOrAtom: PUNICODE_STRING, hInstance: HINSTANCE, pClassMenuName: PCLSMENUNAME, ) -> BOOL; // win32k-sys/src/bindgen_i386.rs // gen:c8a1fac73c55
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::wow64::wow64_callback::Wow64ApcRoutine;
pub use crate::wow64::wow64::Wow64CreateForeignGuestThread;
pub use crate::wow64::wow64::Wow64CreateGuestThread;
pub use crate::wow64::wow64::Wow64FreeThreadCpuArea;
pub use crate::wow64::wow64::Wow64IsCurrentProcess;
pub use crate::wow64::wow64::Wow64NotifyAllocAfter;
pub use crate::wow64::wow64::Wow64NotifyAllocBefore;
pub use crate::wow64::wow64::Wow64NotifyFlushInstructionCache;
pub use crate::wow64::wow64::Wow64NotifyFreeAfter;
pub use crate::wow64::wow64::Wow64NotifyFreeBefore;
pub use crate::wow64::wow64::Wow64NotifyMemoryDirty;
pub use crate::wow64::wow64::Wow64NotifyProtectAfter;
pub use crate::wow64::wow64::Wow64NotifyProtectBefore;
pub use crate::wow64::wow64::Wow64NotifyReadFileAfter;
pub use crate::wow64::wow64::Wow64NotifyReadFileBefore;
pub use crate::wow64::wow64::Wow64NotifyUnmapAfter;
pub use crate::wow64::wow64::Wow64NotifyUnmapBefore;
pub use crate::wow64::wow64::Wow64SetExecuteFlags;
pub use crate::wow64::wow64::Wow64pApcRoutineFor;
pub use crate::wow64::wow64::Wow64pBuildGuestExceptionFrame;
pub use crate::wow64::wow64_callback::Wow64pCompleteGuestApc;
pub use crate::wow64::wow64_callback::Wow64pCompleteUserCallback;
pub use crate::wow64::wow64_callback::Wow64pInstallUserCallbackTable;
pub use crate::wow64::wow64::Wow64pKiUserExceptionDispatcher32;
pub use crate::wow64::wow64::Wow64pPatchTransitionSlot;
pub use crate::wow64::wow64::Wow64pProcessHeap;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const WOW64_AUTOMARSHAL_UNIMPL: NTSTATUS = 0xC0BBBBBB_u32 as NTSTATUS; // clang, wow64_manual.c
pub const WOW64_CB_SIZE32__WINDOWPOS: i32 = 0x1C; // clang, wow64_callback.c
pub const WOW64_CB_SIZE32_tagCOPYDATASTRUCT: i32 = 0xC; // clang, wow64_callback.c
pub const WOW64_CB_SIZE32_tagNCCALCSIZE_PARAMS: i32 = 0x34; // clang, wow64_callback.c
pub const WOW64_CB_SIZE64__WINDOWPOS: i32 = 0x28; // clang, wow64_callback.c
pub const WOW64_CB_SIZE64_tagCOPYDATASTRUCT: i32 = 0x18; // clang, wow64_callback.c
pub const WOW64_CB_SIZE64_tagNCCALCSIZE_PARAMS: i32 = 0x38; // clang, wow64_callback.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CMN32: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CMN32 {
    pub pszClientAnsiMenuName: ULONG,
    pub pwszClientUnicodeMenuName: ULONG,
    pub pusMenuName: ULONG,
}
// CMN64: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CMN64 {
    pub pszClientAnsiMenuName: ULONGLONG,
    pub pwszClientUnicodeMenuName: ULONGLONG,
    pub pusMenuName: ULONGLONG,
}
// ITEM32: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ITEM32 {
    pub Atom: USHORT,
    pub Data: ULONG,
}
// ITEM64: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ITEM64 {
    pub Atom: USHORT,
    pub Data: ULONGLONG,
}
// MI32: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MI32 {
    pub cbSize: ULONG,
    pub fMask: ULONG,
    pub dwStyle: ULONG,
    pub cyMax: ULONG,
    pub hbrBack: ULONG,
    pub dwContextHelpID: ULONG,
    pub dwMenuData: ULONG,
}
// MI64: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MI64 {
    pub cbSize: ULONG,
    pub fMask: ULONG,
    pub dwStyle: ULONG,
    pub cyMax: ULONG,
    pub hbrBack: ULONGLONG,
    pub dwContextHelpID: ULONG,
    pub __pad: ULONG,
    pub dwMenuData: ULONGLONG,
}
// MII32: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MII32 {
    pub cbSize: ULONG,
    pub fMask: ULONG,
    pub fType: ULONG,
    pub fState: ULONG,
    pub wID: ULONG,
    pub hSubMenu: ULONG,
    pub hbmpChecked: ULONG,
    pub hbmpUnchecked: ULONG,
    pub dwItemData: ULONG,
    pub dwTypeData: ULONG,
    pub cch: ULONG,
    pub hbmpItem: ULONG,
}
// MII64: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MII64 {
    pub cbSize: ULONG,
    pub fMask: ULONG,
    pub fType: ULONG,
    pub fState: ULONG,
    pub wID: ULONG,
    pub __pad0: ULONG,
    pub hSubMenu: ULONGLONG,
    pub hbmpChecked: ULONGLONG,
    pub hbmpUnchecked: ULONGLONG,
    pub dwItemData: ULONGLONG,
    pub dwTypeData: ULONGLONG,
    pub cch: ULONG,
    pub __pad1: ULONG,
    pub hbmpItem: ULONGLONG,
}
// WC32: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WC32 {
    pub cbSize: ULONG,
    pub style: ULONG,
    pub lpfnWndProc: ULONG,
    pub cbClsExtra: LONG,
    pub cbWndExtra: LONG,
    pub hInstance: ULONG,
    pub hIcon: ULONG,
    pub hCursor: ULONG,
    pub hbrBackground: ULONG,
    pub lpszMenuName: ULONG,
    pub lpszClassName: ULONG,
    pub hIconSm: ULONG,
}
// WC64: c2rust, wow64_manual.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WC64 {
    pub cbSize: ULONG,
    pub style: ULONG,
    pub lpfnWndProc: ULONGLONG,
    pub cbClsExtra: LONG,
    pub cbWndExtra: LONG,
    pub hInstance: ULONGLONG,
    pub hIcon: ULONGLONG,
    pub hCursor: ULONGLONG,
    pub hbrBackground: ULONGLONG,
    pub lpszMenuName: ULONGLONG,
    pub lpszClassName: ULONGLONG,
    pub hIconSm: ULONGLONG,
}
// WOW64INFO_FEX: c2rust, wow64.c
pub type WOW64INFO_FEX = _WOW64INFO_FEX;
// WOW64_CB_MARSHAL: c2rust, wow64_manual.c
pub type WOW64_CB_MARSHAL = _WOW64_CB_MARSHAL;
// _WOW64INFO_FEX: c2rust, wow64.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _WOW64INFO_FEX {
    pub NativeSystemPageSize: ULONG,
    pub CpuFlags: ULONG,
    pub Wow64ExecuteFlags: ULONG,
    pub Unknown: ULONG,
    pub SectionHandle: ULONGLONG,
    pub CrossProcessWorkList: ULONGLONG,
    pub NativeMachineType: USHORT,
    pub EmulatedMachineType: USHORT,
}
// _WOW64_CB_MARSHAL: c2rust, wow64_cb_automarshal.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _WOW64_CB_MARSHAL {
    pub Msg: ::core::ffi::c_uint,
    pub Size32: ::core::ffi::c_uint,
    pub Size64: ::core::ffi::c_uint,
    pub Down:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_uchar, *mut ::core::ffi::c_uchar) -> ()>,
    pub Up:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_uchar, *mut ::core::ffi::c_uchar) -> ()>,
}
// ==== end port local types ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/wow64/wow64_manual.c:5205: #define WOW64_BCP_COPY32(o32, o64) Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_DATA + (o64), Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + (o32))) // gen:d6c9ccfa5e44
#[allow(unused_macros)]
// Hand: the C macro captures the caller's `B32`/`B64` locals, which macro_rules hygiene
// cannot see, so the buffers are explicit parameters: WOW64_BCP_COPY32!(B32, B64, o32, o64).
macro_rules! WOW64_BCP_COPY32 {
    ($B32:expr, $B64:expr, $o32:expr, $o64:expr $(,)?) => {
        (Wow64pCsrPut32(
            $B64,
            WOW64_CSRMSG_BODY64_DATA + ($o64),
            Wow64pCsrGet32($B32, WOW64_CSRMSG_BODY32_DATA + ($o32))
        ))
    };
}
#[allow(unused_imports)]
pub(crate) use WOW64_BCP_COPY32;
// dll/win32/wow64/wow64_manual.c:4872: #define WOW64_BCP_NARROW(o32, o64) Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + (o32), Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + (o64))) // gen:fb60b2d42b02
#[allow(unused_macros)]
// Hand: explicit `R32`/`R64` (see WOW64_BCP_COPY32): WOW64_BCP_NARROW!(R32, R64, o32, o64).
macro_rules! WOW64_BCP_NARROW {
    ($R32:expr, $R64:expr, $o32:expr, $o64:expr $(,)?) => {
        (Wow64pCsrPut32(
            $R32,
            WOW64_CSRMSG_BODY32_DATA + ($o32),
            Wow64pCsrGet32($R64, WOW64_CSRMSG_BODY64_DATA + ($o64))
        ))
    };
}
#[allow(unused_imports)]
pub(crate) use WOW64_BCP_NARROW;
// dll/win32/wow64/wow64_manual.c:5201: #define WOW64_BCP_WIDEN(o32, o64) Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + (o64), (ULONGLONG)Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + (o32))) // gen:ffe9e4b3cd99
#[allow(unused_macros)]
// Hand: explicit `B32`/`B64` (see WOW64_BCP_COPY32): WOW64_BCP_WIDEN!(B32, B64, o32, o64).
macro_rules! WOW64_BCP_WIDEN {
    ($B32:expr, $B64:expr, $o32:expr, $o64:expr $(,)?) => {
        (Wow64pCsrPut64(
            $B64,
            WOW64_CSRMSG_BODY64_DATA + ($o64),
            (Wow64pCsrGet32($B32, WOW64_CSRMSG_BODY32_DATA + ($o32))) as u64
        ))
    };
}
#[allow(unused_imports)]
pub(crate) use WOW64_BCP_WIDEN;
// dll/win32/wow64/wow64_manual.c:6678: #define WOW64_CTX32_WANTS(f, g) (((f) & (WOW64_CTX32_ARCH | (g))) == (WOW64_CTX32_ARCH | (g))) // gen:8e4c76b30fe3
#[allow(unused_macros)]
macro_rules! WOW64_CTX32_WANTS {
    ($f:expr, $g:expr $(,)?) => {
        (($f & (WOW64_CTX32_ARCH | ($g))) == (WOW64_CTX32_ARCH | ($g)))
    };
}
#[allow(unused_imports)]
pub(crate) use WOW64_CTX32_WANTS;
// ==== end port macros ====
