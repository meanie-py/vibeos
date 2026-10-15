//! evr-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const AM_FILTER_MISC_FLAGS_IS_RENDERER: _AM_FILTER_MISC_FLAGS = _AM_FILTER_MISC_FLAGS_AM_FILTER_MISC_FLAGS_IS_RENDERER; // island enum
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const D3DBACKBUFFER_TYPE_MONO: _D3DBACKBUFFER_TYPE = _D3DBACKBUFFER_TYPE_D3DBACKBUFFER_TYPE_MONO; // island enum
pub const D3DDEVTYPE_HAL: _D3DDEVTYPE = _D3DDEVTYPE_D3DDEVTYPE_HAL; // island enum
pub const D3DFMT_UYVY: _D3DFORMAT = _D3DFORMAT_D3DFMT_UYVY; // island enum
pub const D3DFMT_X8R8G8B8: _D3DFORMAT = _D3DFORMAT_D3DFMT_X8R8G8B8; // island enum
pub const D3DFMT_YUY2: _D3DFORMAT = _D3DFORMAT_D3DFMT_YUY2; // island enum
pub const D3DLOCK_DISCARD: u32 = 8192; // dxdiag-rust/src/types_amd64.rs
pub const D3DLOCK_READONLY: u32 = 16; // dxdiag-rust/src/types_amd64.rs
pub const D3DPOOL_DEFAULT: _D3DPOOL = _D3DPOOL_D3DPOOL_DEFAULT; // island enum
pub const D3DPOOL_SYSTEMMEM: _D3DPOOL = _D3DPOOL_D3DPOOL_SYSTEMMEM; // island enum
pub const D3DPRESENTFLAG_VIDEO: u32 = 16; // dxdiag-rust/src/types_amd64.rs
pub const D3DSWAPEFFECT_COPY: _D3DSWAPEFFECT = _D3DSWAPEFFECT_D3DSWAPEFFECT_COPY; // island enum
pub const D3DTEXF_POINT: _D3DTEXTUREFILTERTYPE = _D3DTEXTUREFILTERTYPE_D3DTEXF_POINT; // island enum
pub const DXVA2_SampleFieldInterleavedEvenFirst: _DXVA2_SampleFormat = _DXVA2_SampleFormat_DXVA2_SampleFieldInterleavedEvenFirst; // island enum
pub const DXVA2_SampleFieldInterleavedOddFirst: _DXVA2_SampleFormat = _DXVA2_SampleFormat_DXVA2_SampleFieldInterleavedOddFirst; // island enum
pub const DXVA2_SampleFieldSingleEven: _DXVA2_SampleFormat = _DXVA2_SampleFormat_DXVA2_SampleFieldSingleEven; // island enum
pub const DXVA2_SampleFieldSingleOdd: _DXVA2_SampleFormat = _DXVA2_SampleFormat_DXVA2_SampleFieldSingleOdd; // island enum
pub const DXVA2_SampleProgressiveFrame: _DXVA2_SampleFormat = _DXVA2_SampleFormat_DXVA2_SampleProgressiveFrame; // island enum
pub const DXVA2_SampleSubStream: _DXVA2_SampleFormat = _DXVA2_SampleFormat_DXVA2_SampleSubStream; // island enum
pub const MFT_INPUT_STATUS_ACCEPT_DATA: _MFT_INPUT_STATUS_FLAGS = _MFT_INPUT_STATUS_FLAGS_MFT_INPUT_STATUS_ACCEPT_DATA; // island enum
pub const MFT_INPUT_STREAM_OPTIONAL: _MFT_INPUT_STREAM_INFO_FLAGS = _MFT_INPUT_STREAM_INFO_FLAGS_MFT_INPUT_STREAM_OPTIONAL; // island enum
pub const MFT_INPUT_STREAM_REMOVABLE: _MFT_INPUT_STREAM_INFO_FLAGS = _MFT_INPUT_STREAM_INFO_FLAGS_MFT_INPUT_STREAM_REMOVABLE; // island enum
pub const MFT_MESSAGE_COMMAND_DRAIN: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_COMMAND_DRAIN; // island enum
pub const MFT_MESSAGE_COMMAND_FLUSH: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_COMMAND_FLUSH; // island enum
pub const MFT_MESSAGE_NOTIFY_BEGIN_STREAMING: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_BEGIN_STREAMING; // island enum
pub const MFT_MESSAGE_NOTIFY_END_OF_STREAM: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_END_OF_STREAM; // island enum
pub const MFT_MESSAGE_NOTIFY_END_STREAMING: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_END_STREAMING; // island enum
pub const MFT_MESSAGE_NOTIFY_START_OF_STREAM: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_START_OF_STREAM; // island enum
pub const MFT_MESSAGE_SET_D3D_MANAGER: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_SET_D3D_MANAGER; // island enum
pub const MFT_OUTPUT_STATUS_SAMPLE_READY: _MFT_OUTPUT_STATUS_FLAGS = _MFT_OUTPUT_STATUS_FLAGS_MFT_OUTPUT_STATUS_SAMPLE_READY; // island enum
pub const MFT_SET_TYPE_TEST_ONLY: _MFT_SET_TYPE_FLAGS = _MFT_SET_TYPE_FLAGS_MFT_SET_TYPE_TEST_ONLY; // island enum
pub const MFVP_MESSAGE_BEGINSTREAMING: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_BEGINSTREAMING; // island enum
pub const MFVP_MESSAGE_ENDSTREAMING: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_ENDSTREAMING; // island enum
pub const MFVP_MESSAGE_INVALIDATEMEDIATYPE: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_INVALIDATEMEDIATYPE; // island enum
pub const MFVP_MESSAGE_PROCESSINPUTNOTIFY: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_PROCESSINPUTNOTIFY; // island enum
pub const MFVideoInterlace_FieldInterleavedLowerFirst: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_FieldInterleavedLowerFirst; // island enum
pub const MFVideoInterlace_FieldInterleavedUpperFirst: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_FieldInterleavedUpperFirst; // island enum
pub const MFVideoInterlace_FieldSingleLower: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_FieldSingleLower; // island enum
pub const MFVideoInterlace_FieldSingleUpper: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_FieldSingleUpper; // island enum
pub const MFVideoInterlace_Progressive: _MFVideoInterlaceMode = _MFVideoInterlaceMode_MFVideoInterlace_Progressive; // island enum
pub const MF_SERVICE_LOOKUP_GLOBAL: _MF_SERVICE_LOOKUP_TYPE = _MF_SERVICE_LOOKUP_TYPE_MF_SERVICE_LOOKUP_GLOBAL; // island enum
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
extern "C" {
    pub fn abs(x: i32) -> i32; // charmap-rust/src/prelude.rs
    pub fn bsearch( key: *const c_void, base: *const c_void, nmemb: usize, size: usize, compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> i32>, ) -> *mut c_void; // kernel32-rust/src/string/nls_subst.rs
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>); // cmd-rust/src/prelude.rs
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn wine_dbg_sprintf(format: *const ::core::ffi::c_char, ...) -> *const ::core::ffi::c_char; // winegdi-rust/src/types_i386.rs
    pub static IID_IClassFactory: GUID; // cicero-rust/src/types_amd64.rs
    pub static IID_IUnknown: GUID; // cicero-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CopyRect(arg1: LPRECT, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EqualRect(arg1: LPCRECT, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn IsRectEmpty(arg1: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn MsgWaitForMultipleObjects( nCount: DWORD, pHandles: *const HANDLE, fWaitAll: BOOL, dwMilliseconds: DWORD, dwWakeMask: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostThreadMessageW(arg1: DWORD, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::evr::sample::MFCreateVideoSampleAllocator;
pub use crate::evr::mixer::__WINE_DBCL_TRACE;
pub use crate::evr::main::__wine_dbch_evr;
pub use crate::evr::sample::debugstr_time;
pub use crate::evr::mixer::evr_mixer_create;
pub use crate::evr::presenter::evr_presenter_create;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const D3DCREATE_HARDWARE_VERTEXPROCESSING: i32 = 0x40; // clang, presenter.c
pub const D3DCREATE_MULTITHREADED: i32 = 4; // clang, presenter.c
pub const D3DPRESENT_INTERVAL_IMMEDIATE: u32 = 0x80000000; // clang, presenter.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn strmbase_release_typelibs(); // main.c
    pub fn strmbase_renderer_cleanup(filter: *mut strmbase_renderer); // evr.c
    pub fn strmbase_renderer_init(
        filter: *mut strmbase_renderer,
        outer: *mut IUnknown,
        clsid: *const CLSID,
        sink_name: *const WCHAR,
        ops: *const strmbase_renderer_ops,
    ); // evr.c
}
extern "system" {
    pub fn DXVA2CreateDirect3DDeviceManager9(
        pResetToken: *mut UINT,
        ppDXVAManager: *mut *mut IDirect3DDeviceManager9,
    ) -> HRESULT; // presenter.c
    pub fn Direct3DCreate9(sdk_version: UINT) -> *mut IDirect3D9; // presenter.c
    pub fn MFCopyImage(
        dest: *mut BYTE,
        deststride: LONG,
        src: *const BYTE,
        srcstride: LONG,
        width: DWORD,
        lines: DWORD,
    ) -> HRESULT; // evr.c
    pub fn MFCreate2DMediaBuffer(
        width: DWORD,
        height: DWORD,
        fourcc: DWORD,
        bottom_up: BOOL,
        buffer: *mut *mut IMFMediaBuffer,
    ) -> HRESULT; // sample.c
    pub fn MFCreateAttributes(attributes: *mut *mut IMFAttributes, size: UINT32) -> HRESULT; // mixer.c
    pub fn MFCreateDXSurfaceBuffer(
        riid: *const IID,
        surface: *mut IUnknown,
        bottom_up: BOOL,
        buffer: *mut *mut IMFMediaBuffer,
    ) -> HRESULT; // sample.c
    pub fn MFCreateMediaType(type_0: *mut *mut IMFMediaType) -> HRESULT; // evr.c
    pub fn MFCreateSample(sample: *mut *mut IMFSample) -> HRESULT; // sample.c
    pub fn MFFrameRateToAverageTimePerFrame(
        numerator: UINT32,
        denominator: UINT32,
        avgtime: *mut UINT64,
    ) -> HRESULT; // presenter.c
    pub fn MFGetPlaneSize(format: DWORD, width: DWORD, height: DWORD, size: *mut DWORD) -> HRESULT; // presenter.c
    pub fn MFGetStrideForBitmapInfoHeader(format: DWORD, width: DWORD, stride: *mut LONG) -> HRESULT; // presenter.c
    pub fn MFInitMediaTypeFromAMMediaType(
        mediatype: *mut IMFMediaType,
        am_type: *const AM_MEDIA_TYPE,
    ) -> HRESULT; // evr.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_evr };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT);
pub const DXVA2_E_NEW_VIDEO_DEVICE: HRESULT = ((0x80041001u32) as HRESULT);
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT);
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT);
#[inline]
pub unsafe fn IClassFactory_AddRef(This: *mut IClassFactory) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3D9_CreateDevice(This: *mut IDirect3D9, Adapter: UINT, DeviceType: D3DDEVTYPE, hFocusWindow: HWND, BehaviorFlags: DWORD, pPresentationParameters: *mut D3DPRESENT_PARAMETERS, ppReturnedDeviceInterface: *mut *mut IDirect3DDevice9) -> HRESULT { ((*(*This).lpVtbl).CreateDevice.unwrap_unchecked())(This, Adapter, DeviceType, hFocusWindow, BehaviorFlags, pPresentationParameters, ppReturnedDeviceInterface) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3D9_Release(This: *mut IDirect3D9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDevice9_ColorFill(This: *mut IDirect3DDevice9, surface: *mut IDirect3DSurface9, rect: *const RECT, color: D3DCOLOR) -> HRESULT { ((*(*This).lpVtbl).ColorFill.unwrap_unchecked())(This, surface, rect, color) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDevice9_CreateAdditionalSwapChain(This: *mut IDirect3DDevice9, pPresentationParameters: *mut D3DPRESENT_PARAMETERS, pSwapChain: *mut *mut IDirect3DSwapChain9) -> HRESULT { ((*(*This).lpVtbl).CreateAdditionalSwapChain.unwrap_unchecked())(This, pPresentationParameters, pSwapChain) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDevice9_CreateOffscreenPlainSurface(This: *mut IDirect3DDevice9, Width: UINT, Height: UINT, Format: D3DFORMAT, Pool: D3DPOOL, ppSurface: *mut *mut IDirect3DSurface9, pSharedHandle: *mut HANDLE) -> HRESULT { ((*(*This).lpVtbl).CreateOffscreenPlainSurface.unwrap_unchecked())(This, Width, Height, Format, Pool, ppSurface, pSharedHandle) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDevice9_GetRenderTargetData(This: *mut IDirect3DDevice9, pRenderTarget: *mut IDirect3DSurface9, pDestSurface: *mut IDirect3DSurface9) -> HRESULT { ((*(*This).lpVtbl).GetRenderTargetData.unwrap_unchecked())(This, pRenderTarget, pDestSurface) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDevice9_Release(This: *mut IDirect3DDevice9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDevice9_StretchRect(This: *mut IDirect3DDevice9, src_surface: *mut IDirect3DSurface9, src_rect: *const RECT, dst_surface: *mut IDirect3DSurface9, dst_rect: *const RECT, filter: D3DTEXTUREFILTERTYPE) -> HRESULT { ((*(*This).lpVtbl).StretchRect.unwrap_unchecked())(This, src_surface, src_rect, dst_surface, dst_rect, filter) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_CloseDeviceHandle(This: *mut IDirect3DDeviceManager9, hDevice: HANDLE) -> HRESULT { ((*(*This).lpVtbl).CloseDeviceHandle.unwrap_unchecked())(This, hDevice) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_GetVideoService(This: *mut IDirect3DDeviceManager9, hDevice: HANDLE, riid: *const IID, ppService: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).GetVideoService.unwrap_unchecked())(This, hDevice, riid, ppService) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_LockDevice(This: *mut IDirect3DDeviceManager9, hDevice: HANDLE, ppDevice: *mut *mut IDirect3DDevice9, fBlock: BOOL) -> HRESULT { ((*(*This).lpVtbl).LockDevice.unwrap_unchecked())(This, hDevice, ppDevice, fBlock) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_OpenDeviceHandle(This: *mut IDirect3DDeviceManager9, phDevice: *mut HANDLE) -> HRESULT { ((*(*This).lpVtbl).OpenDeviceHandle.unwrap_unchecked())(This, phDevice) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_QueryInterface(This: *mut IDirect3DDeviceManager9, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_Release(This: *mut IDirect3DDeviceManager9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_ResetDevice(This: *mut IDirect3DDeviceManager9, pDevice: *mut IDirect3DDevice9, resetToken: UINT) -> HRESULT { ((*(*This).lpVtbl).ResetDevice.unwrap_unchecked())(This, pDevice, resetToken) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_TestDevice(This: *mut IDirect3DDeviceManager9, hDevice: HANDLE) -> HRESULT { ((*(*This).lpVtbl).TestDevice.unwrap_unchecked())(This, hDevice) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DDeviceManager9_UnlockDevice(This: *mut IDirect3DDeviceManager9, hDevice: HANDLE, fSaveState: BOOL) -> HRESULT { ((*(*This).lpVtbl).UnlockDevice.unwrap_unchecked())(This, hDevice, fSaveState) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_GetDesc(This: *mut IDirect3DSurface9, pDesc: *mut D3DSURFACE_DESC) -> HRESULT { ((*(*This).lpVtbl).GetDesc.unwrap_unchecked())(This, pDesc) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_GetDevice(This: *mut IDirect3DSurface9, ppDevice: *mut *mut IDirect3DDevice9) -> HRESULT { ((*(*This).lpVtbl).GetDevice.unwrap_unchecked())(This, ppDevice) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_LockRect(This: *mut IDirect3DSurface9, locked_rect: *mut D3DLOCKED_RECT, rect: *const RECT, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).LockRect.unwrap_unchecked())(This, locked_rect, rect, flags) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_Release(This: *mut IDirect3DSurface9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSurface9_UnlockRect(This: *mut IDirect3DSurface9) -> HRESULT { ((*(*This).lpVtbl).UnlockRect.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSwapChain9_GetBackBuffer(This: *mut IDirect3DSwapChain9, iBackBuffer: UINT, Type: D3DBACKBUFFER_TYPE, ppBackBuffer: *mut *mut IDirect3DSurface9) -> HRESULT { ((*(*This).lpVtbl).GetBackBuffer.unwrap_unchecked())(This, iBackBuffer, Type, ppBackBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSwapChain9_GetDevice(This: *mut IDirect3DSwapChain9, ppDevice: *mut *mut IDirect3DDevice9) -> HRESULT { ((*(*This).lpVtbl).GetDevice.unwrap_unchecked())(This, ppDevice) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSwapChain9_Present(This: *mut IDirect3DSwapChain9, src_rect: *const RECT, dst_rect: *const RECT, dst_window_override: HWND, dirty_region: *const RGNDATA, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).Present.unwrap_unchecked())(This, src_rect, dst_rect, dst_window_override, dirty_region, flags) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3DSwapChain9_Release(This: *mut IDirect3DSwapChain9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_CreateSurface(This: *mut IDirectXVideoProcessorService, width: UINT, height: UINT, backBuffers: UINT, format: D3DFORMAT, pool: D3DPOOL, usage: DWORD, dxvaType: DWORD, ppSurface: *mut *mut IDirect3DSurface9, pSharedHandle: *mut HANDLE) -> HRESULT { ((*(*This).lpVtbl).CreateSurface.unwrap_unchecked())(This, width, height, backBuffers, format, pool, usage, dxvaType, ppSurface, pSharedHandle) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_CreateVideoProcessor(This: *mut IDirectXVideoProcessorService, VideoProcDeviceGuid: *const GUID, pVideoDesc: *const DXVA2_VideoDesc, RenderTargetFormat: D3DFORMAT, MaxNumSubStreams: UINT, ppVidProcess: *mut *mut IDirectXVideoProcessor) -> HRESULT { ((*(*This).lpVtbl).CreateVideoProcessor.unwrap_unchecked())(This, VideoProcDeviceGuid, pVideoDesc, RenderTargetFormat, MaxNumSubStreams, ppVidProcess) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_GetVideoProcessorCaps(This: *mut IDirectXVideoProcessorService, VideoProcDeviceGuid: *const GUID, pVideoDesc: *const DXVA2_VideoDesc, RenderTargetFormat: D3DFORMAT, pCaps: *mut DXVA2_VideoProcessorCaps) -> HRESULT { ((*(*This).lpVtbl).GetVideoProcessorCaps.unwrap_unchecked())(This, VideoProcDeviceGuid, pVideoDesc, RenderTargetFormat, pCaps) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_GetVideoProcessorDeviceGuids(This: *mut IDirectXVideoProcessorService, pVideoDesc: *const DXVA2_VideoDesc, pCount: *mut UINT, pGuids: *mut *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetVideoProcessorDeviceGuids.unwrap_unchecked())(This, pVideoDesc, pCount, pGuids) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_GetVideoProcessorRenderTargets(This: *mut IDirectXVideoProcessorService, VideoProcDeviceGuid: *const GUID, pVideoDesc: *const DXVA2_VideoDesc, pCount: *mut UINT, pFormats: *mut *mut D3DFORMAT) -> HRESULT { ((*(*This).lpVtbl).GetVideoProcessorRenderTargets.unwrap_unchecked())(This, VideoProcDeviceGuid, pVideoDesc, pCount, pFormats) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessorService_Release(This: *mut IDirectXVideoProcessorService) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessor_Release(This: *mut IDirectXVideoProcessor) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirectXVideoProcessor_VideoProcessBlt(This: *mut IDirectXVideoProcessor, pRenderTarget: *mut IDirect3DSurface9, pBltParams: *const DXVA2_VideoProcessBltParams, pSamples: *const DXVA2_VideoSample, NumSamples: UINT, pHandleComplete: *mut HANDLE) -> HRESULT { ((*(*This).lpVtbl).VideoProcessBlt.unwrap_unchecked())(This, pRenderTarget, pBltParams, pSamples, NumSamples, pHandleComplete) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncCallback_AddRef(This: *mut IMFAsyncCallback) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncCallback_Invoke(This: *mut IMFAsyncCallback, pAsyncResult: *mut IMFAsyncResult) -> HRESULT { ((*(*This).lpVtbl).Invoke.unwrap_unchecked())(This, pAsyncResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncCallback_Release(This: *mut IMFAsyncCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_AddRef(This: *mut IMFAsyncResult) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetObject(This: *mut IMFAsyncResult, ppObject: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObject.unwrap_unchecked())(This, ppObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_Release(This: *mut IMFAsyncResult) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_AddRef(This: *mut IMFAttributes) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_Compare(This: *mut IMFAttributes, pTheirs: *mut IMFAttributes, MatchType: MF_ATTRIBUTES_MATCH_TYPE, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).Compare.unwrap_unchecked())(This, pTheirs, MatchType, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_CompareItem(This: *mut IMFAttributes, guidKey: *const GUID, Value: *const PROPVARIANT, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).CompareItem.unwrap_unchecked())(This, guidKey, Value, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_CopyAllItems(This: *mut IMFAttributes, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_DeleteAllItems(This: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).DeleteAllItems.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_DeleteItem(This: *mut IMFAttributes, guidKey: *const GUID) -> HRESULT { ((*(*This).lpVtbl).DeleteItem.unwrap_unchecked())(This, guidKey) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetAllocatedBlob(This: *mut IMFAttributes, guidKey: *const GUID, ppBuf: *mut *mut UINT8, pcbSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedBlob.unwrap_unchecked())(This, guidKey, ppBuf, pcbSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetAllocatedString(This: *mut IMFAttributes, guidKey: *const GUID, ppwszValue: *mut LPWSTR, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedString.unwrap_unchecked())(This, guidKey, ppwszValue, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetBlob(This: *mut IMFAttributes, guidKey: *const GUID, pBuf: *mut UINT8, cbBufSize: UINT32, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetBlobSize(This: *mut IMFAttributes, guidKey: *const GUID, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlobSize.unwrap_unchecked())(This, guidKey, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetCount(This: *mut IMFAttributes, pcItems: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetCount.unwrap_unchecked())(This, pcItems) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetDouble(This: *mut IMFAttributes, guidKey: *const GUID, pfValue: *mut f64) -> HRESULT { ((*(*This).lpVtbl).GetDouble.unwrap_unchecked())(This, guidKey, pfValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetGUID(This: *mut IMFAttributes, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetItem(This: *mut IMFAttributes, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetItemByIndex(This: *mut IMFAttributes, unIndex: UINT32, pguidKey: *mut GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItemByIndex.unwrap_unchecked())(This, unIndex, pguidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetItemType(This: *mut IMFAttributes, guidKey: *const GUID, pType: *mut MF_ATTRIBUTE_TYPE) -> HRESULT { ((*(*This).lpVtbl).GetItemType.unwrap_unchecked())(This, guidKey, pType) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetString(This: *mut IMFAttributes, guidKey: *const GUID, pwszValue: LPWSTR, cchBufSize: UINT32, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetString.unwrap_unchecked())(This, guidKey, pwszValue, cchBufSize, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetStringLength(This: *mut IMFAttributes, guidKey: *const GUID, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetStringLength.unwrap_unchecked())(This, guidKey, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUINT32(This: *mut IMFAttributes, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUINT64(This: *mut IMFAttributes, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUnknown(This: *mut IMFAttributes, guidKey: *const GUID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetUnknown.unwrap_unchecked())(This, guidKey, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_LockStore(This: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).LockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_Release(This: *mut IMFAttributes) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetBlob(This: *mut IMFAttributes, guidKey: *const GUID, pBuf: *const UINT8, cbBufSize: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetDouble(This: *mut IMFAttributes, guidKey: *const GUID, fValue: f64) -> HRESULT { ((*(*This).lpVtbl).SetDouble.unwrap_unchecked())(This, guidKey, fValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetGUID(This: *mut IMFAttributes, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetItem(This: *mut IMFAttributes, guidKey: *const GUID, Value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).SetItem.unwrap_unchecked())(This, guidKey, Value) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetString(This: *mut IMFAttributes, guidKey: *const GUID, wszValue: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetString.unwrap_unchecked())(This, guidKey, wszValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetUINT32(This: *mut IMFAttributes, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetUINT64(This: *mut IMFAttributes, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetUnknown(This: *mut IMFAttributes, guidKey: *const GUID, pUnknown: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetUnknown.unwrap_unchecked())(This, guidKey, pUnknown) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_UnlockStore(This: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).UnlockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_GetCorrelatedTime(This: *mut IMFClock, reserved: DWORD, clock_time: *mut LONGLONG, system_time: *mut MFTIME) -> HRESULT { ((*(*This).lpVtbl).GetCorrelatedTime.unwrap_unchecked())(This, reserved, clock_time, system_time) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_Release(This: *mut IMFClock) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_Release(This: *mut IMFCollection) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFDesiredSample_GetDesiredSampleTimeAndDuration(This: *mut IMFDesiredSample, sample_time: *mut LONGLONG, sample_duration: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetDesiredSampleTimeAndDuration.unwrap_unchecked())(This, sample_time, sample_duration) } // widl C wrapper
#[inline]
pub unsafe fn IMFDesiredSample_Release(This: *mut IMFDesiredSample) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_GetService(This: *mut IMFGetService, guidService: *const GUID, riid: *const IID, ppvObject: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetService.unwrap_unchecked())(This, guidService, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_QueryInterface(This: *mut IMFGetService, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_Release(This: *mut IMFGetService) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_AddRef(This: *mut IMFMediaBuffer) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_QueryInterface(This: *mut IMFMediaBuffer, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Release(This: *mut IMFMediaBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_AddRef(This: *mut IMFMediaType) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_CopyAllItems(This: *mut IMFMediaType, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetBlob(This: *mut IMFMediaType, guidKey: *const GUID, pBuf: *mut UINT8, cbBufSize: UINT32, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetGUID(This: *mut IMFMediaType, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetMajorType(This: *mut IMFMediaType, pguidMajorType: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetMajorType.unwrap_unchecked())(This, pguidMajorType) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetUINT32(This: *mut IMFMediaType, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetUINT64(This: *mut IMFMediaType, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_IsCompressedFormat(This: *mut IMFMediaType, pfCompressed: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).IsCompressedFormat.unwrap_unchecked())(This, pfCompressed) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_IsEqual(This: *mut IMFMediaType, pIMediaType: *mut IMFMediaType, pdwFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).IsEqual.unwrap_unchecked())(This, pIMediaType, pdwFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_QueryInterface(This: *mut IMFMediaType, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_Release(This: *mut IMFMediaType) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetBlob(This: *mut IMFMediaType, guidKey: *const GUID, pBuf: *const UINT8, cbBufSize: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetGUID(This: *mut IMFMediaType, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetUINT32(This: *mut IMFMediaType, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetUINT64(This: *mut IMFMediaType, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_AddBuffer(This: *mut IMFSample, pBuffer: *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).AddBuffer.unwrap_unchecked())(This, pBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_AddRef(This: *mut IMFSample) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_Compare(This: *mut IMFSample, pTheirs: *mut IMFAttributes, MatchType: MF_ATTRIBUTES_MATCH_TYPE, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).Compare.unwrap_unchecked())(This, pTheirs, MatchType, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_CompareItem(This: *mut IMFSample, guidKey: *const GUID, Value: *const PROPVARIANT, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).CompareItem.unwrap_unchecked())(This, guidKey, Value, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_CopyAllItems(This: *mut IMFSample, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_DeleteAllItems(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).DeleteAllItems.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_DeleteItem(This: *mut IMFSample, guidKey: *const GUID) -> HRESULT { ((*(*This).lpVtbl).DeleteItem.unwrap_unchecked())(This, guidKey) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetAllocatedBlob(This: *mut IMFSample, guidKey: *const GUID, ppBuf: *mut *mut UINT8, pcbSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedBlob.unwrap_unchecked())(This, guidKey, ppBuf, pcbSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetAllocatedString(This: *mut IMFSample, guidKey: *const GUID, ppwszValue: *mut LPWSTR, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetAllocatedString.unwrap_unchecked())(This, guidKey, ppwszValue, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBlob(This: *mut IMFSample, guidKey: *const GUID, pBuf: *mut UINT8, cbBufSize: UINT32, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBlobSize(This: *mut IMFSample, guidKey: *const GUID, pcbBlobSize: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBlobSize.unwrap_unchecked())(This, guidKey, pcbBlobSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBufferByIndex(This: *mut IMFSample, dwIndex: DWORD, ppBuffer: *mut *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).GetBufferByIndex.unwrap_unchecked())(This, dwIndex, ppBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBufferCount(This: *mut IMFSample, pdwBufferCount: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetBufferCount.unwrap_unchecked())(This, pdwBufferCount) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetCount(This: *mut IMFSample, pcItems: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetCount.unwrap_unchecked())(This, pcItems) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetDouble(This: *mut IMFSample, guidKey: *const GUID, pfValue: *mut f64) -> HRESULT { ((*(*This).lpVtbl).GetDouble.unwrap_unchecked())(This, guidKey, pfValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetGUID(This: *mut IMFSample, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetItem(This: *mut IMFSample, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetItemByIndex(This: *mut IMFSample, unIndex: UINT32, pguidKey: *mut GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItemByIndex.unwrap_unchecked())(This, unIndex, pguidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetItemType(This: *mut IMFSample, guidKey: *const GUID, pType: *mut MF_ATTRIBUTE_TYPE) -> HRESULT { ((*(*This).lpVtbl).GetItemType.unwrap_unchecked())(This, guidKey, pType) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleDuration(This: *mut IMFSample, phnsSampleDuration: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleDuration.unwrap_unchecked())(This, phnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleFlags(This: *mut IMFSample, pdwSampleFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetSampleFlags.unwrap_unchecked())(This, pdwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleTime(This: *mut IMFSample, phnsSampleTime: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleTime.unwrap_unchecked())(This, phnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetString(This: *mut IMFSample, guidKey: *const GUID, pwszValue: LPWSTR, cchBufSize: UINT32, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetString.unwrap_unchecked())(This, guidKey, pwszValue, cchBufSize, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetStringLength(This: *mut IMFSample, guidKey: *const GUID, pcchLength: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetStringLength.unwrap_unchecked())(This, guidKey, pcchLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetUINT32(This: *mut IMFSample, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetUINT64(This: *mut IMFSample, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetUnknown(This: *mut IMFSample, guidKey: *const GUID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetUnknown.unwrap_unchecked())(This, guidKey, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_LockStore(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).LockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_QueryInterface(This: *mut IMFSample, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_Release(This: *mut IMFSample) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_RemoveAllBuffers(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).RemoveAllBuffers.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_RemoveBufferByIndex(This: *mut IMFSample, dwIndex: DWORD) -> HRESULT { ((*(*This).lpVtbl).RemoveBufferByIndex.unwrap_unchecked())(This, dwIndex) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetBlob(This: *mut IMFSample, guidKey: *const GUID, pBuf: *const UINT8, cbBufSize: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetBlob.unwrap_unchecked())(This, guidKey, pBuf, cbBufSize) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetDouble(This: *mut IMFSample, guidKey: *const GUID, fValue: f64) -> HRESULT { ((*(*This).lpVtbl).SetDouble.unwrap_unchecked())(This, guidKey, fValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetGUID(This: *mut IMFSample, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetItem(This: *mut IMFSample, guidKey: *const GUID, Value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).SetItem.unwrap_unchecked())(This, guidKey, Value) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleDuration(This: *mut IMFSample, hnsSampleDuration: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleDuration.unwrap_unchecked())(This, hnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleFlags(This: *mut IMFSample, dwSampleFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetSampleFlags.unwrap_unchecked())(This, dwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleTime(This: *mut IMFSample, hnsSampleTime: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleTime.unwrap_unchecked())(This, hnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetString(This: *mut IMFSample, guidKey: *const GUID, wszValue: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetString.unwrap_unchecked())(This, guidKey, wszValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetUINT32(This: *mut IMFSample, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetUINT64(This: *mut IMFSample, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetUnknown(This: *mut IMFSample, guidKey: *const GUID, pUnknown: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetUnknown.unwrap_unchecked())(This, guidKey, pUnknown) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_UnlockStore(This: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).UnlockStore.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookupClient_InitServicePointers(This: *mut IMFTopologyServiceLookupClient, service_lookup: *mut IMFTopologyServiceLookup) -> HRESULT { ((*(*This).lpVtbl).InitServicePointers.unwrap_unchecked())(This, service_lookup) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookupClient_Release(This: *mut IMFTopologyServiceLookupClient) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookupClient_ReleaseServicePointers(This: *mut IMFTopologyServiceLookupClient) -> HRESULT { ((*(*This).lpVtbl).ReleaseServicePointers.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookup_LookupService(This: *mut IMFTopologyServiceLookup, lookup_type: MF_SERVICE_LOOKUP_TYPE, index: DWORD, service: *const GUID, riid: *const IID, objects: *mut *mut ::core::ffi::c_void, num_objects: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).LookupService.unwrap_unchecked())(This, lookup_type, index, service, riid, objects, num_objects) } // widl C wrapper
#[inline]
pub unsafe fn IMFTrackedSample_Release(This: *mut IMFTrackedSample) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTrackedSample_SetAllocator(This: *mut IMFTrackedSample, sample_allocator: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetAllocator.unwrap_unchecked())(This, sample_allocator, state) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_AddRef(This: *mut IMFTransform) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetAttributes(This: *mut IMFTransform, attributes: *mut *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).GetAttributes.unwrap_unchecked())(This, attributes) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetInputCurrentType(This: *mut IMFTransform, id: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetInputCurrentType.unwrap_unchecked())(This, id, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputAvailableType(This: *mut IMFTransform, id: DWORD, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetOutputAvailableType.unwrap_unchecked())(This, id, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_ProcessInput(This: *mut IMFTransform, id: DWORD, sample: *mut IMFSample, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).ProcessInput.unwrap_unchecked())(This, id, sample, flags) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_ProcessMessage(This: *mut IMFTransform, message: MFT_MESSAGE_TYPE, param: ULONG_PTR) -> HRESULT { ((*(*This).lpVtbl).ProcessMessage.unwrap_unchecked())(This, message, param) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_ProcessOutput(This: *mut IMFTransform, flags: DWORD, count: DWORD, samples: *mut MFT_OUTPUT_DATA_BUFFER, status: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).ProcessOutput.unwrap_unchecked())(This, flags, count, samples, status) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_QueryInterface(This: *mut IMFTransform, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_Release(This: *mut IMFTransform) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_SetInputType(This: *mut IMFTransform, id: DWORD, type_: *mut IMFMediaType, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetInputType.unwrap_unchecked())(This, id, type_, flags) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_SetOutputType(This: *mut IMFTransform, id: DWORD, type_: *mut IMFMediaType, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetOutputType.unwrap_unchecked())(This, id, type_, flags) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoDeviceID_GetDeviceID(This: *mut IMFVideoDeviceID, device_id: *mut IID) -> HRESULT { ((*(*This).lpVtbl).GetDeviceID.unwrap_unchecked())(This, device_id) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoDeviceID_Release(This: *mut IMFVideoDeviceID) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoMediaType_GetVideoFormat(This: *mut IMFVideoMediaType) -> *const MFVIDEOFORMAT { ((*(*This).lpVtbl).GetVideoFormat.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoMediaType_IsCompressedFormat(This: *mut IMFVideoMediaType, pfCompressed: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).IsCompressedFormat.unwrap_unchecked())(This, pfCompressed) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoMediaType_Release(This: *mut IMFVideoMediaType) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_AddRef(This: *mut IMFVideoPresenter) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_ProcessMessage(This: *mut IMFVideoPresenter, eMessage: MFVP_MESSAGE_TYPE, ulParam: ULONG_PTR) -> HRESULT { ((*(*This).lpVtbl).ProcessMessage.unwrap_unchecked())(This, eMessage, ulParam) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_QueryInterface(This: *mut IMFVideoPresenter, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_Release(This: *mut IMFVideoPresenter) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorCallback_Release(This: *mut IMFVideoSampleAllocatorCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorCallback_SetCallback(This: *mut IMFVideoSampleAllocatorCallback, callback: *mut IMFVideoSampleAllocatorNotify) -> HRESULT { ((*(*This).lpVtbl).SetCallback.unwrap_unchecked())(This, callback) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorNotify_AddRef(This: *mut IMFVideoSampleAllocatorNotify) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorNotify_NotifyRelease(This: *mut IMFVideoSampleAllocatorNotify) -> HRESULT { ((*(*This).lpVtbl).NotifyRelease.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorNotify_Release(This: *mut IMFVideoSampleAllocatorNotify) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocator_AddRef(This: *mut IMFVideoSampleAllocator) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocator_AllocateSample(This: *mut IMFVideoSampleAllocator, sample: *mut *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).AllocateSample.unwrap_unchecked())(This, sample) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocator_InitializeSampleAllocator(This: *mut IMFVideoSampleAllocator, sample_count: DWORD, media_type: *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).InitializeSampleAllocator.unwrap_unchecked())(This, sample_count, media_type) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocator_QueryInterface(This: *mut IMFVideoSampleAllocator, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocator_Release(This: *mut IMFVideoSampleAllocator) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocator_SetDirectXManager(This: *mut IMFVideoSampleAllocator, manager: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetDirectXManager.unwrap_unchecked())(This, manager) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocator_UninitializeSampleAllocator(This: *mut IMFVideoSampleAllocator) -> HRESULT { ((*(*This).lpVtbl).UninitializeSampleAllocator.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMediaEventSink_Notify(This: *mut IMediaEventSink, EventCode: LONG, EventParam1: LONG_PTR, EventParam2: LONG_PTR) -> HRESULT { ((*(*This).lpVtbl).Notify.unwrap_unchecked())(This, EventCode, EventParam1, EventParam2) } // widl C wrapper
#[inline]
pub unsafe fn IMediaEventSink_Release(This: *mut IMediaEventSink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMediaSample_GetPointer(This: *mut IMediaSample, ppBuffer: *mut *mut BYTE) -> HRESULT { ((*(*This).lpVtbl).GetPointer.unwrap_unchecked())(This, ppBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const MF_E_BUFFERTOOSMALL: HRESULT = ((0xc00d36b1u32) as HRESULT);
pub const MF_E_INVALIDMEDIATYPE: HRESULT = ((0xc00d36b4u32) as HRESULT);
pub const MF_E_INVALIDREQUEST: HRESULT = ((0xc00d36b2u32) as HRESULT);
pub const MF_E_INVALIDSTREAMNUMBER: HRESULT = ((0xc00d36b3u32) as HRESULT);
pub const MF_E_NOTACCEPTING: HRESULT = ((0xc00d36b5u32) as HRESULT);
pub const MF_E_NOT_AVAILABLE: HRESULT = ((0xc00d36d6u32) as HRESULT);
pub const MF_E_NOT_INITIALIZED: HRESULT = ((0xc00d36b6u32) as HRESULT);
pub const MF_E_NO_MORE_TYPES: HRESULT = ((0xc00d36b9u32) as HRESULT);
pub const MF_E_NO_SAMPLE_DURATION: HRESULT = ((0xc00d36c9u32) as HRESULT);
pub const MF_E_NO_SAMPLE_TIMESTAMP: HRESULT = ((0xc00d36c8u32) as HRESULT);
pub const MF_E_SAMPLEALLOCATOR_EMPTY: HRESULT = ((0xc00d4a3eu32) as HRESULT);
pub const MF_E_SHUTDOWN: HRESULT = ((0xc00d3e85u32) as HRESULT);
pub const MF_E_TRANSFORM_NEED_MORE_INPUT: HRESULT = ((0xc00d6d72u32) as HRESULT);
pub const MF_E_TRANSFORM_TYPE_NOT_SET: HRESULT = ((0xc00d6d60u32) as HRESULT);
pub const MF_E_UNSUPPORTED_SERVICE: HRESULT = ((0xc00d36bau32) as HRESULT);
pub type REFCLSID = *const CLSID;
pub type REFGUID = *const GUID;
pub type REFIID = *const IID;
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{IsEqualIID};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/evr/evr_private.h:27: header-inline `debugstr_normalized_rect`
#[allow(dead_code)]
#[inline]
pub unsafe fn debugstr_normalized_rect(rect: *const MFVideoNormalizedRect) -> *const core::ffi::c_char {
    if rect.is_null() {
        return "(null)\0".as_ptr().cast();
    }
    wine_dbg_sprintf(
        "(%.8e,%.8e)-(%.8e,%.8e)\0".as_ptr().cast(),
        (*rect).left as core::ffi::c_double,
        (*rect).top as core::ffi::c_double,
        (*rect).right as core::ffi::c_double,
        (*rect).bottom as core::ffi::c_double,
    )
}
// ==== end port macros ====
