//! xinput1_3-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME): wine's own flag test on the default
// channel, then ros_dbg_log -- win32macros-rust's WINE_*! macros, which expand HERE (the
// channel storage and the ros_dbg_log extern are this crate's; `port resolve` writes them).
// MACROS, never out-of-line fns: an out-of-line wrapper is an optimization barrier that
// uncloaks latent ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => { win32macros_rust::WINE_TRACE!($($arg)*) }; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => { win32macros_rust::WINE_ERR!($($arg)*) }; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => { win32macros_rust::WINE_WARN!($($arg)*) }; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { win32macros_rust::WINE_FIXME!($($arg)*) }; }
pub use crate::{ERR, FIXME, TRACE, WARN};

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! HidP_GetButtonCaps { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { HidP_GetSpecificButtonCaps((($p0) as _), ((0) as _), ((0) as _), ((0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; } // gen:5000983d8350
#[macro_export] macro_rules! HidP_GetValueCaps { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { HidP_GetSpecificValueCaps((($p0) as _), ((0) as _), ((0) as _), ((0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; } // gen:77cd91b41dda
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
pub const HIDP_STATUS_SUCCESS: NTSTATUS = 0x110000usize as NTSTATUS; // gen:5fd48cd569c2
pub const HID_USAGE_GENERIC_GAMEPAD: USAGE = 0x5usize as USAGE; // gen:9422f042456d
pub const HID_USAGE_GENERIC_HATSWITCH: USAGE = 0x39usize as USAGE; // gen:db7beae9ceea
pub const HID_USAGE_GENERIC_JOYSTICK: USAGE = 0x4usize as USAGE; // gen:ea3412225915
pub const HID_USAGE_GENERIC_MULTI_AXIS_CONTROLLER: USAGE = 0x8usize as USAGE; // gen:d9a2c4114a1d
pub const HID_USAGE_GENERIC_RX: USAGE = 0x33usize as USAGE; // gen:6e8a75d16d20
pub const HID_USAGE_GENERIC_RY: USAGE = 0x34usize as USAGE; // gen:c9ac810bc02a
pub const HID_USAGE_GENERIC_RZ: USAGE = 0x35usize as USAGE; // gen:271f9dce1013
pub const HID_USAGE_GENERIC_X: USAGE = 0x30usize as USAGE; // gen:c80b46550d4e
pub const HID_USAGE_GENERIC_Y: USAGE = 0x31usize as USAGE; // gen:3be83e8493c9
pub const HID_USAGE_GENERIC_Z: USAGE = 0x32usize as USAGE; // gen:119925d8b101
pub const HID_USAGE_HAPTICS_INTENSITY: USAGE = 0x23usize as USAGE; // gen:b9bd31a275be
pub const HID_USAGE_HAPTICS_SIMPLE_CONTROLLER: USAGE = 0x1usize as USAGE; // gen:0f08f69882dd
pub const HID_USAGE_HAPTICS_WAVEFORM_BUZZ: USAGE = 0x1004usize as USAGE; // gen:9dd9236fb9e1
pub const HID_USAGE_HAPTICS_WAVEFORM_LIST: USAGE = 0x10usize as USAGE; // gen:033b0d85380a
pub const HID_USAGE_HAPTICS_WAVEFORM_RUMBLE: USAGE = 0x1005usize as USAGE; // gen:db2306284c3a
pub const HID_USAGE_PAGE_BUTTON: USAGE = 0x9usize as USAGE; // gen:6c3dfb02f58f
pub const HID_USAGE_PAGE_GENERIC: USAGE = 0x1usize as USAGE; // gen:295302758082
pub const HID_USAGE_PAGE_HAPTICS: USAGE = 0xeusize as USAGE; // gen:44664e5882c3
pub const HID_USAGE_PAGE_ORDINAL: USAGE = 0xausize as USAGE; // gen:be822146eff1
pub const HID_USAGE_PAGE_VENDOR_DEFINED_BEGIN: USAGE = 0xff00usize as USAGE; // gen:0a704d7a204f
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY); // gen:f003d9cba1a1
pub const HWND_MESSAGE: HWND = (((-(3 as c_int))) as HWND); // gen:c12d02e5242d
pub use crate::{HidP_GetButtonCaps, HidP_GetValueCaps, max};
// ==== end SDK header macros ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const HidP_Feature: u32 = 2; // hid-sys/src/lib.rs
pub const HidP_Input: u32 = 0; // hid-sys/src/lib.rs
pub const HidP_Output: u32 = 1; // hid-sys/src/lib.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/dll.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs
}
extern "system" {
    pub fn AcquireSRWLockExclusive(Lock: PRTL_SRWLOCK); // kernel32-vista-rust/src/sync.rs
    pub fn AcquireSRWLockShared(Lock: PRTL_SRWLOCK); // kernel32-vista-rust/src/sync.rs
    pub fn CancelIoEx(hFile: *mut c_void, lpOverlapped: LPOVERLAPPED) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateEventA( lpEventAttributes: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn FreeLibraryAndExitThread(arg1: HMODULE, arg2: DWORD) -> !; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThread() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleExW(arg1: DWORD, arg2: LPCWSTR, arg3: *mut HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetOverlappedResult( hFile: *mut ::core::ffi::c_void, lpOverlapped: LPOVERLAPPED, lpNumberOfBytesTransferred: LPDWORD, bWait: ::core::ffi::c_int, ) -> BOOL; // winlogon-sys/src/bindgen_i386.rs
    pub fn HidD_FreePreparsedData(PreparsedData: PHIDP_PREPARSED_DATA,) -> BOOLEAN; // hid-rust/src/hid/hid.rs
    pub fn HidD_GetAttributes(HidDeviceObject: HANDLE, Attributes: PHIDD_ATTRIBUTES,) -> BOOLEAN; // hid-rust/src/hid/hid.rs
    pub fn HidD_GetFeature(HidDeviceObject: HANDLE, ReportBuffer: PVOID, ReportBufferLength: ULONG,) -> BOOLEAN; // hid-rust/src/hid/hid.rs
    pub fn HidD_GetPreparsedData(HidDeviceObject: HANDLE, PreparsedData: *mut PHIDP_PREPARSED_DATA,) -> BOOLEAN; // hid-rust/src/hid/hid.rs
    pub fn HidD_GetProductString(HidDeviceObject: HANDLE, Buffer: PVOID, BufferLength: ULONG,) -> BOOLEAN; // hid-rust/src/hid/hid.rs
    pub fn HidD_SetOutputReport(HidDeviceObject: HANDLE, ReportBuffer: PVOID, ReportBufferLength: ULONG,) -> BOOLEAN; // hid-rust/src/hid/hid.rs
    pub fn InitOnceExecuteOnce( InitOnce: PINIT_ONCE, InitFn: PINIT_ONCE_FN, Parameter: PVOID, Context: *mut LPVOID, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn MsgWaitForMultipleObjectsEx( nCount: DWORD, pHandles: *const HANDLE, dwMilliseconds: DWORD, dwWakeMask: DWORD, dwFlags: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn RegisterDeviceNotificationW(arg1: HANDLE, arg2: LPVOID, arg3: DWORD) -> HDEVNOTIFY; // aclui-rust/src/types_i386.rs
    pub fn ReleaseSRWLockExclusive(Lock: PRTL_SRWLOCK); // kernel32-vista-rust/src/sync.rs
    pub fn ReleaseSRWLockShared(Lock: PRTL_SRWLOCK); // kernel32-vista-rust/src/sync.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetThreadDescription(hThread: *mut ::core::ffi::c_void, lpThreadDescription: PCWSTR) -> HRESULT; // winlogon-sys/src/bindgen_i386.rs
    pub fn SetupDiDestroyDeviceInfoList(arg1: HDEVINFO) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupDiEnumDeviceInterfaces( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: *const GUID, arg4: DWORD, arg5: PSP_DEVICE_INTERFACE_DATA, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupDiGetClassDevsW( arg1: *const GUID, arg2: PCWSTR, arg3: HWND, arg4: DWORD, ) -> HDEVINFO; // desk-rust/src/types_i386.rs
    pub fn SetupDiGetDeviceInterfaceDetailW( DeviceInfoSet: HDEVINFO, DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA, DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W, DeviceInterfaceDetailDataSize: DWORD, RequiredSize: PDWORD, DeviceInfoData: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn UnregisterClassW(arg1: LPCWSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnregisterDeviceNotification(arg1: HDEVNOTIFY) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// PHIDP_PREPARSED_DATA: c2rust, main.c
pub type PHIDP_PREPARSED_DATA = *mut _HIDP_PREPARSED_DATA;
// SRWLOCK: c2rust, main.c
pub type SRWLOCK = RTL_SRWLOCK;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // acppage-rust/src/prelude.rs
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // main.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // main.c
}
extern "system" {
    pub fn HidP_GetCaps(PreparsedData: PHIDP_PREPARSED_DATA, Capabilities: PHIDP_CAPS) -> NTSTATUS; // main.c
    pub fn HidP_GetLinkCollectionNodes(
        LinkCollectionNodes: PHIDP_LINK_COLLECTION_NODE,
        LinkCollectionNodesLength: PULONG,
        PreparsedData: PHIDP_PREPARSED_DATA,
    ) -> NTSTATUS; // main.c
    pub fn HidP_GetSpecificButtonCaps(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: USHORT,
        Usage: USAGE,
        ButtonCaps: PHIDP_BUTTON_CAPS,
        ButtonCapsLength: PUSHORT,
        PreparsedData: PHIDP_PREPARSED_DATA,
    ) -> NTSTATUS; // main.c
    pub fn HidP_GetSpecificValueCaps(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: USHORT,
        Usage: USAGE,
        ValueCaps: PHIDP_VALUE_CAPS,
        ValueCapsLength: PUSHORT,
        PreparsedData: PHIDP_PREPARSED_DATA,
    ) -> NTSTATUS; // main.c
    pub fn HidP_GetUsageValue(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: USHORT,
        Usage: USAGE,
        UsageValue: PULONG,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: PCHAR,
        ReportLength: ULONG,
    ) -> NTSTATUS; // main.c
    pub fn HidP_GetUsages(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: USHORT,
        UsageList: *mut USAGE,
        UsageLength: *mut ULONG,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: PCHAR,
        ReportLength: ULONG,
    ) -> NTSTATUS; // main.c
    pub fn HidP_InitializeReportForID(
        ReportType: HIDP_REPORT_TYPE,
        ReportID: UCHAR,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: PCHAR,
        ReportLength: ULONG,
    ) -> NTSTATUS; // main.c
    pub fn HidP_SetUsageValue(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: USHORT,
        Usage: USAGE,
        UsageValue: ULONG,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: PCHAR,
        ReportLength: ULONG,
    ) -> NTSTATUS; // main.c
}
pub use crate::prelude::_wcsicmp as wcsicmp; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_xinput: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x78\x69\x6e\x70\x75\x74\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_xinput };
// ==== end wine debug channels ====
