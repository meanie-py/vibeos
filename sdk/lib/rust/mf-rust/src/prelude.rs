//! mf-rust prelude — the single import surface the translated modules use.
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

// ================= hand: CRT / winnt.h / wine/debug.h inlines the island cannot carry ===========
extern "C" {
    // msvcrt exports the underscored name; oldnames supplies `wcsdup` to the C build.
    #[link_name = "_wcsdup"]
    pub fn wcsdup(s: *const u16) -> *mut u16;
}
extern "system" {
    // winuser.h is outside the island's allowlist; evr.c's one call.
    pub fn GetClientRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
}
/// float.h
pub const FLT_MAX: f32 = f32::MAX;
#[inline(always)]
pub unsafe fn InterlockedCompareExchangePointer(target: *mut *mut c_void, exchange: *mut c_void, comparand: *mut c_void) -> *mut c_void {
    use core::sync::atomic::{AtomicPtr, Ordering::SeqCst};
    match (*(target as *const AtomicPtr<c_void>)).compare_exchange(comparand, exchange, SeqCst, SeqCst) {
        Ok(v) | Err(v) => v,
    }
}
#[inline(always)]
pub unsafe fn InterlockedCompareExchange64(target: *mut i64, exchange: i64, comparand: i64) -> i64 {
    use core::sync::atomic::{AtomicI64, Ordering::SeqCst};
    match (*(target as *const AtomicI64)).compare_exchange(comparand, exchange, SeqCst, SeqCst) {
        Ok(v) | Err(v) => v,
    }
}
/// wine/debug.h's inline.
pub unsafe fn wine_dbgstr_longlong(ll: ULONGLONG) -> *const ::core::ffi::c_char {
    if ll >> 32 != 0 {
        wine_dbg_sprintf(b"%lx%08lx\0".as_ptr() as *const ::core::ffi::c_char, (ll >> 32) as u32, ll as u32)
    } else {
        wine_dbg_sprintf(b"%lx\0".as_ptr() as *const ::core::ffi::c_char, ll as u32)
    }
}

// HAND (moved out of the macros block so the fix sticks): mf_private.h:25 header-inline `mf_array_reserve`
#[allow(dead_code)]
#[inline]
pub unsafe fn mf_array_reserve(
    elements: *mut *mut c_void,
    capacity: *mut size_t,
    count: size_t,
    size: size_t,
) -> BOOL {
    if count <= *capacity {
        return (TRUE) as BOOL;
    }

    let max_capacity: size_t = !(0 as size_t) / size;
    if count > max_capacity {
        return (FALSE) as BOOL;
    }

    let mut new_capacity: size_t = max!(4, *capacity);
    while new_capacity < count && new_capacity <= max_capacity / 2 {
        new_capacity *= 2;
    }
    if new_capacity < count {
        new_capacity = max_capacity;
    }

    let new_elements = realloc(*elements, new_capacity * size);
    if new_elements.is_null() {
        return (FALSE) as BOOL;
    }

    *elements = new_elements;
    *capacity = new_capacity;

    (TRUE as i32)
}
extern "C" {
    // sdk/lib/mfuuid (DEFINE_GUID), linked by the C target.
    pub static CLSID_FileSchemePlugin: GUID;
}

// HAND (moved out of the macros block so the fix sticks): mf_private.h:61 header-inline `debugstr_time`
#[allow(dead_code)]
#[inline]
pub unsafe fn debugstr_time(time: LONGLONG) -> *const core::ffi::c_char {
    let mut abstime: ULONGLONG = if time >= 0 {
        time as ULONGLONG
    } else {
        (-(time as i128)) as ULONGLONG
    };
    let mut buffer = [0u8; 23];
    let mut rev = [0u8; 23];
    let mut i: usize = 0;
    let mut j: usize = 0;

    while abstime != 0 || i <= 8 {
        buffer[i] = b'0' + (abstime % 10) as u8;
        i += 1;
        abstime /= 10;
        if i == 7 {
            buffer[i] = b'.';
            i += 1;
        }
    }
    if time < 0 {
        buffer[i] = b'-';
        i += 1;
    }

    while i != 0 {
        i -= 1;
        rev[j] = buffer[i];
        j += 1;
    }
    while rev[j - 1] == b'0' && rev[j - 2] != b'.' {
        j -= 1;
    }
    rev[j] = 0;

    wine_dbg_sprintf(c"%s".as_ptr(), rev.as_ptr() as *const core::ffi::c_char)
}

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const AUDCLNT_BUFFERFLAGS_SILENT: _AUDCLNT_BUFFERFLAGS = _AUDCLNT_BUFFERFLAGS_AUDCLNT_BUFFERFLAGS_SILENT; // island enum
pub const AUDCLNT_SHAREMODE_SHARED: _AUDCLNT_SHAREMODE = _AUDCLNT_SHAREMODE_AUDCLNT_SHAREMODE_SHARED; // island enum
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const MFCLOCK_CHARACTERISTICS_FLAG_FREQUENCY_10MHZ: _MFCLOCK_CHARACTERISTICS_FLAGS = _MFCLOCK_CHARACTERISTICS_FLAGS_MFCLOCK_CHARACTERISTICS_FLAG_FREQUENCY_10MHZ; // island enum
pub const MFCLOCK_STATE_INVALID: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_INVALID; // island enum
pub const MFCLOCK_STATE_PAUSED: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_PAUSED; // island enum
pub const MFCLOCK_STATE_RUNNING: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_RUNNING; // island enum
pub const MFCLOCK_STATE_STOPPED: _MFCLOCK_STATE = _MFCLOCK_STATE_MFCLOCK_STATE_STOPPED; // island enum
pub const MFMEDIASOURCE_CAN_PAUSE: _MFMEDIASOURCE_CHARACTERISTICS = _MFMEDIASOURCE_CHARACTERISTICS_MFMEDIASOURCE_CAN_PAUSE; // island enum
pub const MFMEDIASOURCE_CAN_SEEK: _MFMEDIASOURCE_CHARACTERISTICS = _MFMEDIASOURCE_CHARACTERISTICS_MFMEDIASOURCE_CAN_SEEK; // island enum
pub const MFMEDIASOURCE_DOES_NOT_USE_NETWORK: _MFMEDIASOURCE_CHARACTERISTICS = _MFMEDIASOURCE_CHARACTERISTICS_MFMEDIASOURCE_DOES_NOT_USE_NETWORK; // island enum
pub const MFRATE_FORWARD: _MFRATE_DIRECTION = _MFRATE_DIRECTION_MFRATE_FORWARD; // island enum
pub const MFRATE_REVERSE: _MFRATE_DIRECTION = _MFRATE_DIRECTION_MFRATE_REVERSE; // island enum
pub const MFSESSION_GETFULLTOPOLOGY_CURRENT: MFSESSION_GETFULLTOPOLOGY_FLAGS = MFSESSION_GETFULLTOPOLOGY_FLAGS_MFSESSION_GETFULLTOPOLOGY_CURRENT; // island enum
pub const MFSESSION_SETTOPOLOGY_CLEAR_CURRENT: MFSESSION_SETTOPOLOGY_FLAGS = MFSESSION_SETTOPOLOGY_FLAGS_MFSESSION_SETTOPOLOGY_CLEAR_CURRENT; // island enum
pub const MFSESSION_SETTOPOLOGY_IMMEDIATE: MFSESSION_SETTOPOLOGY_FLAGS = MFSESSION_SETTOPOLOGY_FLAGS_MFSESSION_SETTOPOLOGY_IMMEDIATE; // island enum
pub const MFSESSION_SETTOPOLOGY_NORESOLUTION: MFSESSION_SETTOPOLOGY_FLAGS = MFSESSION_SETTOPOLOGY_FLAGS_MFSESSION_SETTOPOLOGY_NORESOLUTION; // island enum
pub const MFSHUTDOWN_COMPLETED: _MFSHUTDOWN_STATUS = _MFSHUTDOWN_STATUS_MFSHUTDOWN_COMPLETED; // island enum
pub const MFSTREAMSINK_MARKER_ENDOFSEGMENT: _MFSTREAMSINK_MARKER_TYPE = _MFSTREAMSINK_MARKER_TYPE_MFSTREAMSINK_MARKER_ENDOFSEGMENT; // island enum
pub const MFTIMER_RELATIVE: MFTIMER_FLAGS = MFTIMER_FLAGS_MFTIMER_RELATIVE; // island enum
pub const MFTOPOLOGY_DXVA_DEFAULT: MFTOPOLOGY_DXVA_MODE = MFTOPOLOGY_DXVA_MODE_MFTOPOLOGY_DXVA_DEFAULT; // island enum
pub const MFTOPOLOGY_DXVA_FULL: MFTOPOLOGY_DXVA_MODE = MFTOPOLOGY_DXVA_MODE_MFTOPOLOGY_DXVA_FULL; // island enum
pub const MFT_ENUM_FLAG_ALL: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_ALL; // island enum
pub const MFT_INPUT_STATUS_ACCEPT_DATA: _MFT_INPUT_STATUS_FLAGS = _MFT_INPUT_STATUS_FLAGS_MFT_INPUT_STATUS_ACCEPT_DATA; // island enum
pub const MFT_MESSAGE_COMMAND_DRAIN: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_COMMAND_DRAIN; // island enum
pub const MFT_MESSAGE_COMMAND_FLUSH: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_COMMAND_FLUSH; // island enum
pub const MFT_MESSAGE_NOTIFY_BEGIN_STREAMING: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_BEGIN_STREAMING; // island enum
pub const MFT_MESSAGE_NOTIFY_END_OF_STREAM: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_END_OF_STREAM; // island enum
pub const MFT_MESSAGE_NOTIFY_END_STREAMING: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_END_STREAMING; // island enum
pub const MFT_MESSAGE_NOTIFY_START_OF_STREAM: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_START_OF_STREAM; // island enum
pub const MFT_MESSAGE_SET_D3D_MANAGER: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_SET_D3D_MANAGER; // island enum
pub const MFT_OUTPUT_DATA_BUFFER_FORMAT_CHANGE: _MFT_OUTPUT_DATA_BUFFER_FLAGS = _MFT_OUTPUT_DATA_BUFFER_FLAGS_MFT_OUTPUT_DATA_BUFFER_FORMAT_CHANGE; // island enum
pub const MFT_OUTPUT_DATA_BUFFER_NO_SAMPLE: _MFT_OUTPUT_DATA_BUFFER_FLAGS = _MFT_OUTPUT_DATA_BUFFER_FLAGS_MFT_OUTPUT_DATA_BUFFER_NO_SAMPLE; // island enum
pub const MFT_OUTPUT_STATUS_SAMPLE_READY: _MFT_OUTPUT_STATUS_FLAGS = _MFT_OUTPUT_STATUS_FLAGS_MFT_OUTPUT_STATUS_SAMPLE_READY; // island enum
pub const MFT_OUTPUT_STREAM_CAN_PROVIDE_SAMPLES: _MFT_OUTPUT_STREAM_INFO_FLAGS = _MFT_OUTPUT_STREAM_INFO_FLAGS_MFT_OUTPUT_STREAM_CAN_PROVIDE_SAMPLES; // island enum
pub const MFT_OUTPUT_STREAM_PROVIDES_SAMPLES: _MFT_OUTPUT_STREAM_INFO_FLAGS = _MFT_OUTPUT_STREAM_INFO_FLAGS_MFT_OUTPUT_STREAM_PROVIDES_SAMPLES; // island enum
pub const MFT_SET_TYPE_TEST_ONLY: _MFT_SET_TYPE_FLAGS = _MFT_SET_TYPE_FLAGS_MFT_SET_TYPE_TEST_ONLY; // island enum
pub const MFVP_MESSAGE_BEGINSTREAMING: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_BEGINSTREAMING; // island enum
pub const MFVP_MESSAGE_ENDSTREAMING: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_ENDSTREAMING; // island enum
pub const MFVP_MESSAGE_FLUSH: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_FLUSH; // island enum
pub const MFVP_MESSAGE_INVALIDATEMEDIATYPE: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_INVALIDATEMEDIATYPE; // island enum
pub const MFVP_MESSAGE_PROCESSINPUTNOTIFY: _MFVP_MESSAGE_TYPE = _MFVP_MESSAGE_TYPE_MFVP_MESSAGE_PROCESSINPUTNOTIFY; // island enum
pub const MF_ACCESSMODE_READ: MF_FILE_ACCESSMODE = MF_FILE_ACCESSMODE_MF_ACCESSMODE_READ; // island enum
pub const MF_ACCESSMODE_READWRITE: MF_FILE_ACCESSMODE = MF_FILE_ACCESSMODE_MF_ACCESSMODE_READWRITE; // island enum
pub const MF_ATTRIBUTES_MATCH_OUR_ITEMS: _MF_ATTRIBUTES_MATCH_TYPE = _MF_ATTRIBUTES_MATCH_TYPE_MF_ATTRIBUTES_MATCH_OUR_ITEMS; // island enum
pub const MF_CONNECT_ALLOW_CONVERTER: _MF_CONNECT_METHOD = _MF_CONNECT_METHOD_MF_CONNECT_ALLOW_CONVERTER; // island enum
pub const MF_CONNECT_ALLOW_DECODER: _MF_CONNECT_METHOD = _MF_CONNECT_METHOD_MF_CONNECT_ALLOW_DECODER; // island enum
pub const MF_CONNECT_AS_OPTIONAL: _MF_CONNECT_METHOD = _MF_CONNECT_METHOD_MF_CONNECT_AS_OPTIONAL; // island enum
pub const MF_CONNECT_AS_OPTIONAL_BRANCH: _MF_CONNECT_METHOD = _MF_CONNECT_METHOD_MF_CONNECT_AS_OPTIONAL_BRANCH; // island enum
pub const MF_CONNECT_DIRECT: _MF_CONNECT_METHOD = _MF_CONNECT_METHOD_MF_CONNECT_DIRECT; // island enum
pub const MF_CONNECT_RESOLVE_INDEPENDENT_OUTPUTTYPES: _MF_CONNECT_METHOD = _MF_CONNECT_METHOD_MF_CONNECT_RESOLVE_INDEPENDENT_OUTPUTTYPES; // island enum
pub const MF_OBJECT_BYTESTREAM: MF_OBJECT_TYPE = MF_OBJECT_TYPE_MF_OBJECT_BYTESTREAM; // island enum
pub const MF_OBJECT_INVALID: MF_OBJECT_TYPE = MF_OBJECT_TYPE_MF_OBJECT_INVALID; // island enum
pub const MF_TOPOLOGY_MAX: MF_TOPOLOGY_TYPE = MF_TOPOLOGY_TYPE_MF_TOPOLOGY_MAX; // island enum
pub const MF_TOPOLOGY_OUTPUT_NODE: MF_TOPOLOGY_TYPE = MF_TOPOLOGY_TYPE_MF_TOPOLOGY_OUTPUT_NODE; // island enum
pub const MF_TOPOLOGY_SOURCESTREAM_NODE: MF_TOPOLOGY_TYPE = MF_TOPOLOGY_TYPE_MF_TOPOLOGY_SOURCESTREAM_NODE; // island enum
pub const MF_TOPOLOGY_TEE_NODE: MF_TOPOLOGY_TYPE = MF_TOPOLOGY_TYPE_MF_TOPOLOGY_TEE_NODE; // island enum
pub const MF_TOPOLOGY_TRANSFORM_NODE: MF_TOPOLOGY_TYPE = MF_TOPOLOGY_TYPE_MF_TOPOLOGY_TRANSFORM_NODE; // island enum
pub const MF_TOPOSTATUS_ENDED: MF_TOPOSTATUS = MF_TOPOSTATUS_MF_TOPOSTATUS_ENDED; // island enum
pub const MF_TOPOSTATUS_INVALID: MF_TOPOSTATUS = MF_TOPOSTATUS_MF_TOPOSTATUS_INVALID; // island enum
pub const MF_TOPOSTATUS_READY: MF_TOPOSTATUS = MF_TOPOSTATUS_MF_TOPOSTATUS_READY; // island enum
pub const MF_TOPOSTATUS_STARTED_SOURCE: MF_TOPOSTATUS = MF_TOPOSTATUS_MF_TOPOSTATUS_STARTED_SOURCE; // island enum
pub const STGM_READ: u32 = 0; // appwiz-rust/src/prelude.rs
pub const VT_EMPTY: u16 = 0; // mfreadwrite-rust/src/prelude.rs
pub const VT_I4: DWORD = 3; // ctfmon-rust/src/prelude.rs
pub const VT_I8: DWORD = 0x14; // mfreadwrite-rust/src/prelude.rs
pub const VT_LPWSTR: u16 = 31; // oleaut32-rust/src/consts.rs
pub const VT_R4: u16 = 4; // oleaut32-rust/src/consts.rs
pub const VT_UNKNOWN: u16 = 13; // mfreadwrite-rust/src/prelude.rs
pub const VT_VECTOR: u16 = 4096; // oleaut32-rust/src/consts.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_WARN: __wine_debug_class = 2; // appwiz-rust/src/prelude.rs
pub const eCapture: _EDataFlow = _EDataFlow_eCapture; // island enum
pub const eConsole: _ERole = _ERole_eConsole; // island enum
pub const eMultimedia: _ERole = _ERole_eMultimedia; // island enum
pub const eRender: _EDataFlow = _EDataFlow_eRender; // island enum
extern "C" {
    pub fn bsearch( key: *const c_void, base: *const c_void, nmemb: usize, size: usize, compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> i32>, ) -> *mut c_void; // evr-rust/src/prelude.rs
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn fabsf(x: f32) -> f32; // cnt-rust/src/cabi/math.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>); // cmd-rust/src/prelude.rs
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn wine_dbg_sprintf(format: *const ::core::ffi::c_char, ...) -> *const ::core::ffi::c_char; // winegdi-rust/src/types_i386.rs
    pub static IID_IPersistPropertyBag: GUID; // wmic-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemRealloc(ptr: LPVOID, size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn PathCreateFromUrlW( pszUrl: LPCWSTR, pszPath: LPWSTR, pcchPath: LPDWORD, dwFlags: DWORD, ) -> HRESULT; // certmgr-rust/src/types_i386.rs
    pub fn PropVariantClear(pvar: *mut PROPVARIANT) -> HRESULT; // mfreadwrite-rust/src/prelude.rs
    pub fn PropVariantCopy(pvarDest: *mut PROPVARIANT, pvarSrc: *const PROPVARIANT,) -> HRESULT; // mfreadwrite-rust/src/prelude.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const AUDCLNT_STREAMFLAGS_CROSSPROCESS: i32 = 0x10000; // clang, sar.c
pub const AUDCLNT_STREAMFLAGS_EVENTCALLBACK: i32 = 0x40000; // clang, sar.c
pub const AUDCLNT_STREAMFLAGS_NOPERSIST: i32 = 0x80000; // clang, sar.c
pub const MF_FILEFLAGS_NONE: i32 = 0; // clang, scheme_handler.c
pub const MF_OPENMODE_FAIL_IF_NOT_EXIST: i32 = 0; // clang, scheme_handler.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// clock_command: c2rust, clock.c
pub type clock_command = ::core::ffi::c_uint;
// clock_notification: c2rust, clock.c
pub type clock_notification = ::core::ffi::c_uint;
// connect_method: c2rust, topology_loader.c
pub type connect_method = ::core::ffi::c_uint;
// sink_state: c2rust, samplegrabber.c
pub type sink_state = ::core::ffi::c_uint;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // main.c
}
extern "system" {
    pub fn MFAddPeriodicCallback(
        callback: MFPERIODICCALLBACK,
        context: *mut IUnknown,
        key: *mut DWORD,
    ) -> HRESULT; // clock.c
    pub fn MFCalculateImageSize(
        subtype: *const GUID,
        width: UINT32,
        height: UINT32,
        size: *mut UINT32,
    ) -> HRESULT; // copier.c
    pub fn MFCancelWorkItem(key: MFWORKITEM_KEY) -> HRESULT; // sar.c
    pub fn MFCreateAlignedMemoryBuffer(
        max_length: DWORD,
        alignment: DWORD,
        buffer: *mut *mut IMFMediaBuffer,
    ) -> HRESULT; // session.c
    pub fn MFCreateAsyncResult(
        object: *mut IUnknown,
        callback: *mut IMFAsyncCallback,
        state: *mut IUnknown,
        result: *mut *mut IMFAsyncResult,
    ) -> HRESULT; // clock.c
    pub fn MFCreateAttributes(attributes: *mut *mut IMFAttributes, size: UINT32) -> HRESULT; // copier.c
    pub fn MFCreateCollection(collection: *mut *mut IMFCollection) -> HRESULT; // main.c
    pub fn MFCreateEventQueue(queue: *mut *mut IMFMediaEventQueue) -> HRESULT; // evr.c
    pub fn MFCreateFile(
        accessmode: MF_FILE_ACCESSMODE,
        openmode: MF_FILE_OPENMODE,
        flags: MF_FILE_FLAGS,
        url: LPCWSTR,
        bytestream: *mut *mut IMFByteStream,
    ) -> HRESULT; // scheme_handler.c
    pub fn MFCreateMFByteStreamOnStream(
        stream: *mut IStream,
        bytestream: *mut *mut IMFByteStream,
    ) -> HRESULT; // scheme_handler.c
    pub fn MFCreateMediaEvent(
        type_0: MediaEventType,
        extended_type: *const GUID,
        status: HRESULT,
        value: *const PROPVARIANT,
        event: *mut *mut IMFMediaEvent,
    ) -> HRESULT; // session.c
    pub fn MFCreateMediaType(type_0: *mut *mut IMFMediaType) -> HRESULT; // copier.c
    pub fn MFCreateSample(sample: *mut *mut IMFSample) -> HRESULT; // session.c
    pub fn MFCreateSourceResolver(resolver: *mut *mut IMFSourceResolver) -> HRESULT; // scheme_handler.c
    pub fn MFCreateSystemTimeSource(time_source: *mut *mut IMFPresentationTimeSource) -> HRESULT; // session.c
    pub fn MFCreateVideoSampleAllocator(
        riid: *const IID,
        allocator: *mut *mut ::core::ffi::c_void,
    ) -> HRESULT; // evr.c
    pub fn MFGetSystemTime() -> MFTIME; // clock.c
    pub fn MFInitMediaTypeFromWaveFormatEx(
        mediatype: *mut IMFMediaType,
        format: *const WAVEFORMATEX,
        size: UINT32,
    ) -> HRESULT; // sar.c
    pub fn MFInvokeCallback(result: *mut IMFAsyncResult) -> HRESULT; // clock.c
    pub fn MFPutWaitingWorkItem(
        event: HANDLE,
        priority: LONG,
        result: *mut IMFAsyncResult,
        key: *mut MFWORKITEM_KEY,
    ) -> HRESULT; // sar.c
    pub fn MFPutWorkItem(
        queue: DWORD,
        callback: *mut IMFAsyncCallback,
        state: *mut IUnknown,
    ) -> HRESULT; // session.c
    pub fn MFPutWorkItemEx(queue: DWORD, result: *mut IMFAsyncResult) -> HRESULT; // clock.c
    pub fn MFRemovePeriodicCallback(key: DWORD) -> HRESULT; // clock.c
    pub fn MFTEnumEx(
        category: GUID,
        flags: UINT32,
        input_type: *const MFT_REGISTER_TYPE_INFO,
        output_type: *const MFT_REGISTER_TYPE_INFO,
        activate: *mut *mut *mut IMFActivate,
        pcount: *mut UINT32,
    ) -> HRESULT; // main.c
    pub fn MFTRegisterLocalByCLSID(
        clsid: *const CLSID,
        category: *const GUID,
        name: LPCWSTR,
        flags: UINT32,
        input_count: UINT32,
        input_types: *const MFT_REGISTER_TYPE_INFO,
        output_count: UINT32,
        output_types: *const MFT_REGISTER_TYPE_INFO,
    ) -> HRESULT; // session.c
    pub fn URLOpenBlockingStreamW(
        _: LPUNKNOWN,
        _: LPCWSTR,
        _: *mut LPSTREAM,
        _: DWORD,
        _: LPBINDSTATUSCALLBACK,
    ) -> HRESULT; // scheme_handler.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_mfplat: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6d\x66\x70\x6c\x61\x74\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_mfplat };
// ==== end wine debug channels ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mf::topology_loader::MFCreateTopoLoader;
pub use crate::mf::topology::MFCreateTopology;
pub use crate::mf::main::MFGetService;
pub use crate::mf::topology::__WINE_DBCL_TRACE;
pub use crate::mf::main::create_activation_object;
pub use crate::mf::sac::enum_audio_capture_sources;
pub use crate::mf::scheme_handler::file_scheme_handler_construct;
pub use crate::mf::topology_loader::stream_sink_get_device_manager;
pub use crate::mf::topology::topology_node_get_object;
pub use crate::mf::topology::topology_node_get_type;
pub use crate::mf::topology::topology_node_get_type_handler;
pub use crate::mf::topology_loader::topology_node_init_media_type;
pub use crate::mf::topology_loader::topology_node_is_d3d_aware;
pub use crate::mf::topology_loader::topology_node_set_device_manager;
pub use crate::mf::topology_loader::topology_node_transform_async_get_attributes;
pub use crate::mf::scheme_handler::urlmon_scheme_handler_construct;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! PropVariantInit { ($p0:expr $(,)?) => { memset((($p0) as _), ((0) as _), ((core::mem::size_of::<PROPVARIANT>()) as _)) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT);
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT);
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT);
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
#[inline]
pub unsafe fn IAudioClient_GetBufferSize(This: *mut IAudioClient, pNumBufferFrames: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetBufferSize.unwrap_unchecked())(This, pNumBufferFrames) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_GetCurrentPadding(This: *mut IAudioClient, pNumPaddingFrames: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetCurrentPadding.unwrap_unchecked())(This, pNumPaddingFrames) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_GetMixFormat(This: *mut IAudioClient, ppDeviceFormat: *mut *mut WAVEFORMATEX) -> HRESULT { ((*(*This).lpVtbl).GetMixFormat.unwrap_unchecked())(This, ppDeviceFormat) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_GetService(This: *mut IAudioClient, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).GetService.unwrap_unchecked())(This, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_Initialize(This: *mut IAudioClient, ShareMode: AUDCLNT_SHAREMODE, StreamFlags: DWORD, hnsBufferDuration: REFERENCE_TIME, hnsPeriodicity: REFERENCE_TIME, pFormat: *const WAVEFORMATEX, AudioSessionGuid: LPCGUID) -> HRESULT { ((*(*This).lpVtbl).Initialize.unwrap_unchecked())(This, ShareMode, StreamFlags, hnsBufferDuration, hnsPeriodicity, pFormat, AudioSessionGuid) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_Release(This: *mut IAudioClient) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_Reset(This: *mut IAudioClient) -> HRESULT { ((*(*This).lpVtbl).Reset.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_SetEventHandle(This: *mut IAudioClient, eventHandle: HANDLE) -> HRESULT { ((*(*This).lpVtbl).SetEventHandle.unwrap_unchecked())(This, eventHandle) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_Start(This: *mut IAudioClient) -> HRESULT { ((*(*This).lpVtbl).Start.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClient_Stop(This: *mut IAudioClient) -> HRESULT { ((*(*This).lpVtbl).Stop.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClock_GetFrequency(This: *mut IAudioClock, pu64Frequency: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetFrequency.unwrap_unchecked())(This, pu64Frequency) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClock_GetPosition(This: *mut IAudioClock, pu64Position: *mut UINT64, pu64QPCPosition: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetPosition.unwrap_unchecked())(This, pu64Position, pu64QPCPosition) } // widl C wrapper
#[inline]
pub unsafe fn IAudioClock_Release(This: *mut IAudioClock) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IAudioRenderClient_GetBuffer(This: *mut IAudioRenderClient, NumFramesRequested: UINT32, ppData: *mut *mut BYTE) -> HRESULT { ((*(*This).lpVtbl).GetBuffer.unwrap_unchecked())(This, NumFramesRequested, ppData) } // widl C wrapper
#[inline]
pub unsafe fn IAudioRenderClient_Release(This: *mut IAudioRenderClient) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IAudioRenderClient_ReleaseBuffer(This: *mut IAudioRenderClient, NumFramesWritten: UINT32, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).ReleaseBuffer.unwrap_unchecked())(This, NumFramesWritten, dwFlags) } // widl C wrapper
#[inline]
pub unsafe fn IAudioStreamVolume_GetAllVolumes(This: *mut IAudioStreamVolume, dwCount: UINT32, pfVolumes: *mut f32) -> HRESULT { ((*(*This).lpVtbl).GetAllVolumes.unwrap_unchecked())(This, dwCount, pfVolumes) } // widl C wrapper
#[inline]
pub unsafe fn IAudioStreamVolume_GetChannelCount(This: *mut IAudioStreamVolume, pdwCount: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetChannelCount.unwrap_unchecked())(This, pdwCount) } // widl C wrapper
#[inline]
pub unsafe fn IAudioStreamVolume_GetChannelVolume(This: *mut IAudioStreamVolume, dwIndex: UINT32, pfLevel: *mut f32) -> HRESULT { ((*(*This).lpVtbl).GetChannelVolume.unwrap_unchecked())(This, dwIndex, pfLevel) } // widl C wrapper
#[inline]
pub unsafe fn IAudioStreamVolume_Release(This: *mut IAudioStreamVolume) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IAudioStreamVolume_SetAllVolumes(This: *mut IAudioStreamVolume, dwCount: UINT32, pfVolumes: *const f32) -> HRESULT { ((*(*This).lpVtbl).SetAllVolumes.unwrap_unchecked())(This, dwCount, pfVolumes) } // widl C wrapper
#[inline]
pub unsafe fn IAudioStreamVolume_SetChannelVolume(This: *mut IAudioStreamVolume, dwIndex: UINT32, fLevel: f32) -> HRESULT { ((*(*This).lpVtbl).SetChannelVolume.unwrap_unchecked())(This, dwIndex, fLevel) } // widl C wrapper
#[inline]
pub unsafe fn IClassFactory_AddRef(This: *mut IClassFactory) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IClassFactory_QueryInterface(This: *mut IClassFactory, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_ActivateObject(This: *mut IMFActivate, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).ActivateObject.unwrap_unchecked())(This, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_AddRef(This: *mut IMFActivate) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_GetGUID(This: *mut IMFActivate, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_Release(This: *mut IMFActivate) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_SetGUID(This: *mut IMFActivate, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_SetString(This: *mut IMFActivate, guidKey: *const GUID, wszValue: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetString.unwrap_unchecked())(This, guidKey, wszValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_SetUINT64(This: *mut IMFActivate, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_ShutdownObject(This: *mut IMFActivate) -> HRESULT { ((*(*This).lpVtbl).ShutdownObject.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncCallback_AddRef(This: *mut IMFAsyncCallback) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_AddRef(This: *mut IMFAsyncResult) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetObject(This: *mut IMFAsyncResult, ppObject: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObject.unwrap_unchecked())(This, ppObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetState(This: *mut IMFAsyncResult, ppunkState: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetState.unwrap_unchecked())(This, ppunkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetStateNoAddRef(This: *mut IMFAsyncResult) -> *mut IUnknown { ((*(*This).lpVtbl).GetStateNoAddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetStatus(This: *mut IMFAsyncResult) -> HRESULT { ((*(*This).lpVtbl).GetStatus.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_Release(This: *mut IMFAsyncResult) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_SetStatus(This: *mut IMFAsyncResult, hrStatus: HRESULT) -> HRESULT { ((*(*This).lpVtbl).SetStatus.unwrap_unchecked())(This, hrStatus) } // widl C wrapper
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
pub unsafe fn IMFAudioPolicy_AddRef(This: *mut IMFAudioPolicy) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAudioStreamVolume_AddRef(This: *mut IMFAudioStreamVolume) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_QueryInterface(This: *mut IMFByteStream, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_Release(This: *mut IMFByteStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockConsumer_Release(This: *mut IMFClockConsumer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockConsumer_SetPresentationClock(This: *mut IMFClockConsumer, clock: *mut IMFPresentationClock) -> HRESULT { ((*(*This).lpVtbl).SetPresentationClock.unwrap_unchecked())(This, clock) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockStateSink_AddRef(This: *mut IMFClockStateSink) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockStateSink_OnClockPause(This: *mut IMFClockStateSink, hnsSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockPause.unwrap_unchecked())(This, hnsSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockStateSink_OnClockRestart(This: *mut IMFClockStateSink, hnsSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockRestart.unwrap_unchecked())(This, hnsSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockStateSink_OnClockSetRate(This: *mut IMFClockStateSink, hnsSystemTime: MFTIME, flRate: f32) -> HRESULT { ((*(*This).lpVtbl).OnClockSetRate.unwrap_unchecked())(This, hnsSystemTime, flRate) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockStateSink_OnClockStart(This: *mut IMFClockStateSink, hnsSystemTime: MFTIME, llClockStartOffset: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).OnClockStart.unwrap_unchecked())(This, hnsSystemTime, llClockStartOffset) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockStateSink_OnClockStop(This: *mut IMFClockStateSink, hnssSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockStop.unwrap_unchecked())(This, hnssSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFClockStateSink_Release(This: *mut IMFClockStateSink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_AddRef(This: *mut IMFClock) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_GetProperties(This: *mut IMFClock, props: *mut MFCLOCK_PROPERTIES) -> HRESULT { ((*(*This).lpVtbl).GetProperties.unwrap_unchecked())(This, props) } // widl C wrapper
#[inline]
pub unsafe fn IMFClock_Release(This: *mut IMFClock) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_AddElement(This: *mut IMFCollection, pUnkElement: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).AddElement.unwrap_unchecked())(This, pUnkElement) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_AddRef(This: *mut IMFCollection) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_GetElement(This: *mut IMFCollection, dwElementIndex: DWORD, ppUnkElement: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetElement.unwrap_unchecked())(This, dwElementIndex, ppUnkElement) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_Release(This: *mut IMFCollection) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_RemoveElement(This: *mut IMFCollection, dwElementIndex: DWORD, ppUnkElement: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).RemoveElement.unwrap_unchecked())(This, dwElementIndex, ppUnkElement) } // widl C wrapper
#[inline]
pub unsafe fn IMFFinalizableMediaSink_BeginFinalize(This: *mut IMFFinalizableMediaSink, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginFinalize.unwrap_unchecked())(This, callback, state) } // widl C wrapper
#[inline]
pub unsafe fn IMFFinalizableMediaSink_EndFinalize(This: *mut IMFFinalizableMediaSink, result: *mut IMFAsyncResult) -> HRESULT { ((*(*This).lpVtbl).EndFinalize.unwrap_unchecked())(This, result) } // widl C wrapper
#[inline]
pub unsafe fn IMFFinalizableMediaSink_Release(This: *mut IMFFinalizableMediaSink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_GetService(This: *mut IMFGetService, guidService: *const GUID, riid: *const IID, ppvObject: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetService.unwrap_unchecked())(This, guidService, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_QueryInterface(This: *mut IMFGetService, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_Release(This: *mut IMFGetService) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFLocalMFTRegistration_AddRef(This: *mut IMFLocalMFTRegistration) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFLocalMFTRegistration_QueryInterface(This: *mut IMFLocalMFTRegistration, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Lock(This: *mut IMFMediaBuffer, ppbBuffer: *mut *mut BYTE, pcbMaxLength: *mut DWORD, pcbCurrentLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).Lock.unwrap_unchecked())(This, ppbBuffer, pcbMaxLength, pcbCurrentLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Release(This: *mut IMFMediaBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_SetCurrentLength(This: *mut IMFMediaBuffer, cbCurrentLength: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetCurrentLength.unwrap_unchecked())(This, cbCurrentLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Unlock(This: *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).Unlock.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_BeginGetEvent(This: *mut IMFMediaEventGenerator, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_EndGetEvent(This: *mut IMFMediaEventGenerator, pResult: *mut IMFAsyncResult, ppEvent: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).EndGetEvent.unwrap_unchecked())(This, pResult, ppEvent) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_QueryInterface(This: *mut IMFMediaEventGenerator, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventGenerator_Release(This: *mut IMFMediaEventGenerator) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_BeginGetEvent(This: *mut IMFMediaEventQueue, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, callback, state) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_EndGetEvent(This: *mut IMFMediaEventQueue, result: *mut IMFAsyncResult, event: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).EndGetEvent.unwrap_unchecked())(This, result, event) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_GetEvent(This: *mut IMFMediaEventQueue, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).GetEvent.unwrap_unchecked())(This, flags, event) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_QueueEvent(This: *mut IMFMediaEventQueue, event: *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).QueueEvent.unwrap_unchecked())(This, event) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_QueueEventParamVar(This: *mut IMFMediaEventQueue, met: MediaEventType, type_: *const GUID, status: HRESULT, value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).QueueEventParamVar.unwrap_unchecked())(This, met, type_, status, value) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_Release(This: *mut IMFMediaEventQueue) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEventQueue_Shutdown(This: *mut IMFMediaEventQueue) -> HRESULT { ((*(*This).lpVtbl).Shutdown.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_GetStatus(This: *mut IMFMediaEvent, phrStatus: *mut HRESULT) -> HRESULT { ((*(*This).lpVtbl).GetStatus.unwrap_unchecked())(This, phrStatus) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_GetType(This: *mut IMFMediaEvent, pmet: *mut MediaEventType) -> HRESULT { ((*(*This).lpVtbl).GetType.unwrap_unchecked())(This, pmet) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_GetUINT32(This: *mut IMFMediaEvent, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_GetValue(This: *mut IMFMediaEvent, pvValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetValue.unwrap_unchecked())(This, pvValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_Release(This: *mut IMFMediaEvent) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_SetUINT32(This: *mut IMFMediaEvent, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_SetUINT64(This: *mut IMFMediaEvent, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSession_AddRef(This: *mut IMFMediaSession) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSession_QueryInterface(This: *mut IMFMediaSession, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSession_Release(This: *mut IMFMediaSession) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSession_Shutdown(This: *mut IMFMediaSession) -> HRESULT { ((*(*This).lpVtbl).Shutdown.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSinkPreroll_NotifyPreroll(This: *mut IMFMediaSinkPreroll, start_time: MFTIME) -> HRESULT { ((*(*This).lpVtbl).NotifyPreroll.unwrap_unchecked())(This, start_time) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSinkPreroll_Release(This: *mut IMFMediaSinkPreroll) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_AddRef(This: *mut IMFMediaSink) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_AddStreamSink(This: *mut IMFMediaSink, stream_sink_id: DWORD, media_type: *mut IMFMediaType, stream_sink: *mut *mut IMFStreamSink) -> HRESULT { ((*(*This).lpVtbl).AddStreamSink.unwrap_unchecked())(This, stream_sink_id, media_type, stream_sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_GetCharacteristics(This: *mut IMFMediaSink, characteristics: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetCharacteristics.unwrap_unchecked())(This, characteristics) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_GetStreamSinkById(This: *mut IMFMediaSink, stream_sink_id: DWORD, sink: *mut *mut IMFStreamSink) -> HRESULT { ((*(*This).lpVtbl).GetStreamSinkById.unwrap_unchecked())(This, stream_sink_id, sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_GetStreamSinkByIndex(This: *mut IMFMediaSink, index: DWORD, sink: *mut *mut IMFStreamSink) -> HRESULT { ((*(*This).lpVtbl).GetStreamSinkByIndex.unwrap_unchecked())(This, index, sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_GetStreamSinkCount(This: *mut IMFMediaSink, count: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamSinkCount.unwrap_unchecked())(This, count) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_QueryInterface(This: *mut IMFMediaSink, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_Release(This: *mut IMFMediaSink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_SetPresentationClock(This: *mut IMFMediaSink, clock: *mut IMFPresentationClock) -> HRESULT { ((*(*This).lpVtbl).SetPresentationClock.unwrap_unchecked())(This, clock) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_Shutdown(This: *mut IMFMediaSink) -> HRESULT { ((*(*This).lpVtbl).Shutdown.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_AddRef(This: *mut IMFMediaSource) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_BeginGetEvent(This: *mut IMFMediaSource, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_GetCharacteristics(This: *mut IMFMediaSource, characteristics: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetCharacteristics.unwrap_unchecked())(This, characteristics) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Pause(This: *mut IMFMediaSource) -> HRESULT { ((*(*This).lpVtbl).Pause.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Release(This: *mut IMFMediaSource) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Start(This: *mut IMFMediaSource, descriptor: *mut IMFPresentationDescriptor, time_format: *const GUID, start_position: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).Start.unwrap_unchecked())(This, descriptor, time_format, start_position) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Stop(This: *mut IMFMediaSource) -> HRESULT { ((*(*This).lpVtbl).Stop.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_AddRef(This: *mut IMFMediaStream) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_BeginGetEvent(This: *mut IMFMediaStream, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_GetMediaSource(This: *mut IMFMediaStream, source: *mut *mut IMFMediaSource) -> HRESULT { ((*(*This).lpVtbl).GetMediaSource.unwrap_unchecked())(This, source) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_GetStreamDescriptor(This: *mut IMFMediaStream, descriptor: *mut *mut IMFStreamDescriptor) -> HRESULT { ((*(*This).lpVtbl).GetStreamDescriptor.unwrap_unchecked())(This, descriptor) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_RequestSample(This: *mut IMFMediaStream, token: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).RequestSample.unwrap_unchecked())(This, token) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_AddRef(This: *mut IMFMediaTypeHandler) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_GetCurrentMediaType(This: *mut IMFMediaTypeHandler, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetCurrentMediaType.unwrap_unchecked())(This, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_GetMediaTypeByIndex(This: *mut IMFMediaTypeHandler, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetMediaTypeByIndex.unwrap_unchecked())(This, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_IsMediaTypeSupported(This: *mut IMFMediaTypeHandler, in_type: *mut IMFMediaType, out_type: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).IsMediaTypeSupported.unwrap_unchecked())(This, in_type, out_type) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_Release(This: *mut IMFMediaTypeHandler) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_SetCurrentMediaType(This: *mut IMFMediaTypeHandler, type_: *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).SetCurrentMediaType.unwrap_unchecked())(This, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_AddRef(This: *mut IMFMediaType) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_Compare(This: *mut IMFMediaType, pTheirs: *mut IMFAttributes, MatchType: MF_ATTRIBUTES_MATCH_TYPE, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).Compare.unwrap_unchecked())(This, pTheirs, MatchType, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_CompareItem(This: *mut IMFMediaType, guidKey: *const GUID, Value: *const PROPVARIANT, pbResult: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).CompareItem.unwrap_unchecked())(This, guidKey, Value, pbResult) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_CopyAllItems(This: *mut IMFMediaType, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetGUID(This: *mut IMFMediaType, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetItem(This: *mut IMFMediaType, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetMajorType(This: *mut IMFMediaType, pguidMajorType: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetMajorType.unwrap_unchecked())(This, pguidMajorType) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetUINT32(This: *mut IMFMediaType, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_GetUINT64(This: *mut IMFMediaType, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_IsEqual(This: *mut IMFMediaType, pIMediaType: *mut IMFMediaType, pdwFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).IsEqual.unwrap_unchecked())(This, pIMediaType, pdwFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_Release(This: *mut IMFMediaType) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetGUID(This: *mut IMFMediaType, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetItem(This: *mut IMFMediaType, guidKey: *const GUID, Value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).SetItem.unwrap_unchecked())(This, guidKey, Value) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_AddClockStateSink(This: *mut IMFPresentationClock, state_sink: *mut IMFClockStateSink) -> HRESULT { ((*(*This).lpVtbl).AddClockStateSink.unwrap_unchecked())(This, state_sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_AddRef(This: *mut IMFPresentationClock) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_GetTime(This: *mut IMFPresentationClock, time: *mut MFTIME) -> HRESULT { ((*(*This).lpVtbl).GetTime.unwrap_unchecked())(This, time) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_GetTimeSource(This: *mut IMFPresentationClock, time_source: *mut *mut IMFPresentationTimeSource) -> HRESULT { ((*(*This).lpVtbl).GetTimeSource.unwrap_unchecked())(This, time_source) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_Pause(This: *mut IMFPresentationClock) -> HRESULT { ((*(*This).lpVtbl).Pause.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_QueryInterface(This: *mut IMFPresentationClock, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_Release(This: *mut IMFPresentationClock) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_RemoveClockStateSink(This: *mut IMFPresentationClock, state_sink: *mut IMFClockStateSink) -> HRESULT { ((*(*This).lpVtbl).RemoveClockStateSink.unwrap_unchecked())(This, state_sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_SetTimeSource(This: *mut IMFPresentationClock, time_source: *mut IMFPresentationTimeSource) -> HRESULT { ((*(*This).lpVtbl).SetTimeSource.unwrap_unchecked())(This, time_source) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_Start(This: *mut IMFPresentationClock, start_offset: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).Start.unwrap_unchecked())(This, start_offset) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_Stop(This: *mut IMFPresentationClock) -> HRESULT { ((*(*This).lpVtbl).Stop.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_GetStreamDescriptorByIndex(This: *mut IMFPresentationDescriptor, index: DWORD, selected: *mut BOOL, descriptor: *mut *mut IMFStreamDescriptor) -> HRESULT { ((*(*This).lpVtbl).GetStreamDescriptorByIndex.unwrap_unchecked())(This, index, selected, descriptor) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_GetStreamDescriptorCount(This: *mut IMFPresentationDescriptor, count: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamDescriptorCount.unwrap_unchecked())(This, count) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_GetUINT64(This: *mut IMFPresentationDescriptor, guidKey: *const GUID, punValue: *mut UINT64) -> HRESULT { ((*(*This).lpVtbl).GetUINT64.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_Release(This: *mut IMFPresentationDescriptor) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_AddRef(This: *mut IMFPresentationTimeSource) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_GetClockCharacteristics(This: *mut IMFPresentationTimeSource, characteristics: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetClockCharacteristics.unwrap_unchecked())(This, characteristics) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_GetCorrelatedTime(This: *mut IMFPresentationTimeSource, reserved: DWORD, clock_time: *mut LONGLONG, system_time: *mut MFTIME) -> HRESULT { ((*(*This).lpVtbl).GetCorrelatedTime.unwrap_unchecked())(This, reserved, clock_time, system_time) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_GetProperties(This: *mut IMFPresentationTimeSource, props: *mut MFCLOCK_PROPERTIES) -> HRESULT { ((*(*This).lpVtbl).GetProperties.unwrap_unchecked())(This, props) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_GetUnderlyingClock(This: *mut IMFPresentationTimeSource, clock: *mut *mut IMFClock) -> HRESULT { ((*(*This).lpVtbl).GetUnderlyingClock.unwrap_unchecked())(This, clock) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_QueryInterface(This: *mut IMFPresentationTimeSource, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_Release(This: *mut IMFPresentationTimeSource) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_AddRef(This: *mut IMFQualityManager) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_NotifyPresentationClock(This: *mut IMFQualityManager, clock: *mut IMFPresentationClock) -> HRESULT { ((*(*This).lpVtbl).NotifyPresentationClock.unwrap_unchecked())(This, clock) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_NotifyProcessInput(This: *mut IMFQualityManager, node: *mut IMFTopologyNode, input_index: LONG, sample: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).NotifyProcessInput.unwrap_unchecked())(This, node, input_index, sample) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_NotifyProcessOutput(This: *mut IMFQualityManager, node: *mut IMFTopologyNode, output_index: LONG, sample: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).NotifyProcessOutput.unwrap_unchecked())(This, node, output_index, sample) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_NotifyQualityEvent(This: *mut IMFQualityManager, object: *mut IUnknown, event: *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).NotifyQualityEvent.unwrap_unchecked())(This, object, event) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_NotifyTopology(This: *mut IMFQualityManager, topology: *mut IMFTopology) -> HRESULT { ((*(*This).lpVtbl).NotifyTopology.unwrap_unchecked())(This, topology) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_QueryInterface(This: *mut IMFQualityManager, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_Release(This: *mut IMFQualityManager) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFQualityManager_Shutdown(This: *mut IMFQualityManager) -> HRESULT { ((*(*This).lpVtbl).Shutdown.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFRateControl_GetRate(This: *mut IMFRateControl, thin: *mut BOOL, rate: *mut f32) -> HRESULT { ((*(*This).lpVtbl).GetRate.unwrap_unchecked())(This, thin, rate) } // widl C wrapper
#[inline]
pub unsafe fn IMFRateControl_Release(This: *mut IMFRateControl) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFRateControl_SetRate(This: *mut IMFRateControl, thin: BOOL, rate: f32) -> HRESULT { ((*(*This).lpVtbl).SetRate.unwrap_unchecked())(This, thin, rate) } // widl C wrapper
#[inline]
pub unsafe fn IMFRateSupport_GetFastestRate(This: *mut IMFRateSupport, direction: MFRATE_DIRECTION, thin: BOOL, rate: *mut f32) -> HRESULT { ((*(*This).lpVtbl).GetFastestRate.unwrap_unchecked())(This, direction, thin, rate) } // widl C wrapper
#[inline]
pub unsafe fn IMFRateSupport_GetSlowestRate(This: *mut IMFRateSupport, direction: MFRATE_DIRECTION, thin: BOOL, rate: *mut f32) -> HRESULT { ((*(*This).lpVtbl).GetSlowestRate.unwrap_unchecked())(This, direction, thin, rate) } // widl C wrapper
#[inline]
pub unsafe fn IMFRateSupport_IsRateSupported(This: *mut IMFRateSupport, thin: BOOL, rate: f32, nearest_supported_rate: *mut f32) -> HRESULT { ((*(*This).lpVtbl).IsRateSupported.unwrap_unchecked())(This, thin, rate, nearest_supported_rate) } // widl C wrapper
#[inline]
pub unsafe fn IMFRateSupport_Release(This: *mut IMFRateSupport) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback2_OnProcessSampleEx(This: *mut IMFSampleGrabberSinkCallback2, major_type: *const GUID, sample_flags: DWORD, sample_time: LONGLONG, sample_duration: LONGLONG, buffer: *const BYTE, sample_size: DWORD, attributes: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).OnProcessSampleEx.unwrap_unchecked())(This, major_type, sample_flags, sample_time, sample_duration, buffer, sample_size, attributes) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback2_Release(This: *mut IMFSampleGrabberSinkCallback2) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_AddRef(This: *mut IMFSampleGrabberSinkCallback) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_OnClockPause(This: *mut IMFSampleGrabberSinkCallback, hnsSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockPause.unwrap_unchecked())(This, hnsSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_OnClockSetRate(This: *mut IMFSampleGrabberSinkCallback, hnsSystemTime: MFTIME, flRate: f32) -> HRESULT { ((*(*This).lpVtbl).OnClockSetRate.unwrap_unchecked())(This, hnsSystemTime, flRate) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_OnClockStart(This: *mut IMFSampleGrabberSinkCallback, hnsSystemTime: MFTIME, llClockStartOffset: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).OnClockStart.unwrap_unchecked())(This, hnsSystemTime, llClockStartOffset) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_OnClockStop(This: *mut IMFSampleGrabberSinkCallback, hnssSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockStop.unwrap_unchecked())(This, hnssSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_OnProcessSample(This: *mut IMFSampleGrabberSinkCallback, major_type: *const GUID, sample_flags: DWORD, sample_time: LONGLONG, sample_duration: LONGLONG, buffer: *const BYTE, sample_size: DWORD) -> HRESULT { ((*(*This).lpVtbl).OnProcessSample.unwrap_unchecked())(This, major_type, sample_flags, sample_time, sample_duration, buffer, sample_size) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_OnSetPresentationClock(This: *mut IMFSampleGrabberSinkCallback, clock: *mut IMFPresentationClock) -> HRESULT { ((*(*This).lpVtbl).OnSetPresentationClock.unwrap_unchecked())(This, clock) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_OnShutdown(This: *mut IMFSampleGrabberSinkCallback) -> HRESULT { ((*(*This).lpVtbl).OnShutdown.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_QueryInterface(This: *mut IMFSampleGrabberSinkCallback, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFSampleGrabberSinkCallback_Release(This: *mut IMFSampleGrabberSinkCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_AddBuffer(This: *mut IMFSample, pBuffer: *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).AddBuffer.unwrap_unchecked())(This, pBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_AddRef(This: *mut IMFSample) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_ConvertToContiguousBuffer(This: *mut IMFSample, ppBuffer: *mut *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).ConvertToContiguousBuffer.unwrap_unchecked())(This, ppBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_CopyAllItems(This: *mut IMFSample, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_CopyToBuffer(This: *mut IMFSample, pBuffer: *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).CopyToBuffer.unwrap_unchecked())(This, pBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBufferByIndex(This: *mut IMFSample, dwIndex: DWORD, ppBuffer: *mut *mut IMFMediaBuffer) -> HRESULT { ((*(*This).lpVtbl).GetBufferByIndex.unwrap_unchecked())(This, dwIndex, ppBuffer) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetBufferCount(This: *mut IMFSample, pdwBufferCount: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetBufferCount.unwrap_unchecked())(This, pdwBufferCount) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleDuration(This: *mut IMFSample, phnsSampleDuration: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleDuration.unwrap_unchecked())(This, phnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleFlags(This: *mut IMFSample, pdwSampleFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetSampleFlags.unwrap_unchecked())(This, pdwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleTime(This: *mut IMFSample, phnsSampleTime: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleTime.unwrap_unchecked())(This, phnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetTotalLength(This: *mut IMFSample, pcbTotalLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetTotalLength.unwrap_unchecked())(This, pcbTotalLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_Release(This: *mut IMFSample) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleDuration(This: *mut IMFSample, hnsSampleDuration: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleDuration.unwrap_unchecked())(This, hnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleFlags(This: *mut IMFSample, dwSampleFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetSampleFlags.unwrap_unchecked())(This, dwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleTime(This: *mut IMFSample, hnsSampleTime: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleTime.unwrap_unchecked())(This, hnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSchemeHandler_AddRef(This: *mut IMFSchemeHandler) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSchemeHandler_QueryInterface(This: *mut IMFSchemeHandler, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFSchemeHandler_Release(This: *mut IMFSchemeHandler) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSequencerSource_AddRef(This: *mut IMFSequencerSource) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSequencerSource_QueryInterface(This: *mut IMFSequencerSource, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFSequencerSource_Release(This: *mut IMFSequencerSource) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFShutdown_Release(This: *mut IMFShutdown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFShutdown_Shutdown(This: *mut IMFShutdown) -> HRESULT { ((*(*This).lpVtbl).Shutdown.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSimpleAudioVolume_AddRef(This: *mut IMFSimpleAudioVolume) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkClassFactory_CreateMediaSink(This: *mut IMFSinkClassFactory, stream: *mut IMFByteStream, video_type: *mut IMFMediaType, audio_type: *mut IMFMediaType, sink: *mut *mut IMFMediaSink) -> HRESULT { ((*(*This).lpVtbl).CreateMediaSink.unwrap_unchecked())(This, stream, video_type, audio_type, sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkClassFactory_Release(This: *mut IMFSinkClassFactory) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_AddRef(This: *mut IMFSourceResolver) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_CreateObjectFromByteStream(This: *mut IMFSourceResolver, stream: *mut IMFByteStream, url: *const WCHAR, flags: DWORD, props: *mut IPropertyStore, obj_type: *mut MF_OBJECT_TYPE, object: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CreateObjectFromByteStream.unwrap_unchecked())(This, stream, url, flags, props, obj_type, object) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_Release(This: *mut IMFSourceResolver) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_GetMediaTypeHandler(This: *mut IMFStreamDescriptor, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT { ((*(*This).lpVtbl).GetMediaTypeHandler.unwrap_unchecked())(This, handler) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_GetStreamIdentifier(This: *mut IMFStreamDescriptor, identifier: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamIdentifier.unwrap_unchecked())(This, identifier) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_GetUINT32(This: *mut IMFStreamDescriptor, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_Release(This: *mut IMFStreamDescriptor) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_AddRef(This: *mut IMFStreamSink) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_BeginGetEvent(This: *mut IMFStreamSink, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_Flush(This: *mut IMFStreamSink) -> HRESULT { ((*(*This).lpVtbl).Flush.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_GetMediaSink(This: *mut IMFStreamSink, sink: *mut *mut IMFMediaSink) -> HRESULT { ((*(*This).lpVtbl).GetMediaSink.unwrap_unchecked())(This, sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_GetMediaTypeHandler(This: *mut IMFStreamSink, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT { ((*(*This).lpVtbl).GetMediaTypeHandler.unwrap_unchecked())(This, handler) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_PlaceMarker(This: *mut IMFStreamSink, marker_type: MFSTREAMSINK_MARKER_TYPE, marker_value: *const PROPVARIANT, context_value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).PlaceMarker.unwrap_unchecked())(This, marker_type, marker_value, context_value) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_ProcessSample(This: *mut IMFStreamSink, sample: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).ProcessSample.unwrap_unchecked())(This, sample) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_QueryInterface(This: *mut IMFStreamSink, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_QueueEvent(This: *mut IMFStreamSink, met: MediaEventType, guidExtendedType: *const GUID, hrStatus: HRESULT, pvValue: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).QueueEvent.unwrap_unchecked())(This, met, guidExtendedType, hrStatus, pvValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_Release(This: *mut IMFStreamSink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTimer_CancelTimer(This: *mut IMFTimer, key: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CancelTimer.unwrap_unchecked())(This, key) } // widl C wrapper
#[inline]
pub unsafe fn IMFTimer_Release(This: *mut IMFTimer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTimer_SetTimer(This: *mut IMFTimer, flags: DWORD, time: LONGLONG, callback: *mut IMFAsyncCallback, state: *mut IUnknown, key: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetTimer.unwrap_unchecked())(This, flags, time, callback, state, key) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopoLoader_AddRef(This: *mut IMFTopoLoader) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopoLoader_Load(This: *mut IMFTopoLoader, input_topology: *mut IMFTopology, output_topology: *mut *mut IMFTopology, current_topology: *mut IMFTopology) -> HRESULT { ((*(*This).lpVtbl).Load.unwrap_unchecked())(This, input_topology, output_topology, current_topology) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopoLoader_Release(This: *mut IMFTopoLoader) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNodeAttributeEditor_AddRef(This: *mut IMFTopologyNodeAttributeEditor) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_AddRef(This: *mut IMFTopologyNode) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_CloneFrom(This: *mut IMFTopologyNode, node: *mut IMFTopologyNode) -> HRESULT { ((*(*This).lpVtbl).CloneFrom.unwrap_unchecked())(This, node) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_ConnectOutput(This: *mut IMFTopologyNode, output_index: DWORD, node: *mut IMFTopologyNode, input_index: DWORD) -> HRESULT { ((*(*This).lpVtbl).ConnectOutput.unwrap_unchecked())(This, output_index, node, input_index) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_CopyAllItems(This: *mut IMFTopologyNode, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_DeleteItem(This: *mut IMFTopologyNode, guidKey: *const GUID) -> HRESULT { ((*(*This).lpVtbl).DeleteItem.unwrap_unchecked())(This, guidKey) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_DisconnectOutput(This: *mut IMFTopologyNode, index: DWORD) -> HRESULT { ((*(*This).lpVtbl).DisconnectOutput.unwrap_unchecked())(This, index) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetInput(This: *mut IMFTopologyNode, input_index: DWORD, node: *mut *mut IMFTopologyNode, output_index: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetInput.unwrap_unchecked())(This, input_index, node, output_index) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetInputCount(This: *mut IMFTopologyNode, count: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetInputCount.unwrap_unchecked())(This, count) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetInputPrefType(This: *mut IMFTopologyNode, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetInputPrefType.unwrap_unchecked())(This, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetItem(This: *mut IMFTopologyNode, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetNodeType(This: *mut IMFTopologyNode, type_: *mut MF_TOPOLOGY_TYPE) -> HRESULT { ((*(*This).lpVtbl).GetNodeType.unwrap_unchecked())(This, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetObject(This: *mut IMFTopologyNode, object: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObject.unwrap_unchecked())(This, object) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetOutput(This: *mut IMFTopologyNode, output_index: DWORD, node: *mut *mut IMFTopologyNode, input_index: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetOutput.unwrap_unchecked())(This, output_index, node, input_index) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetOutputCount(This: *mut IMFTopologyNode, count: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetOutputCount.unwrap_unchecked())(This, count) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetOutputPrefType(This: *mut IMFTopologyNode, output_index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetOutputPrefType.unwrap_unchecked())(This, output_index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetTopoNodeID(This: *mut IMFTopologyNode, id: *mut TOPOID) -> HRESULT { ((*(*This).lpVtbl).GetTopoNodeID.unwrap_unchecked())(This, id) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetUINT32(This: *mut IMFTopologyNode, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_GetUnknown(This: *mut IMFTopologyNode, guidKey: *const GUID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetUnknown.unwrap_unchecked())(This, guidKey, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_Release(This: *mut IMFTopologyNode) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_SetGUID(This: *mut IMFTopologyNode, guidKey: *const GUID, guidValue: *const GUID) -> HRESULT { ((*(*This).lpVtbl).SetGUID.unwrap_unchecked())(This, guidKey, guidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_SetInputPrefType(This: *mut IMFTopologyNode, index: DWORD, type_: *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).SetInputPrefType.unwrap_unchecked())(This, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_SetObject(This: *mut IMFTopologyNode, object: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetObject.unwrap_unchecked())(This, object) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_SetOutputPrefType(This: *mut IMFTopologyNode, index: DWORD, type_: *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).SetOutputPrefType.unwrap_unchecked())(This, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_SetUINT32(This: *mut IMFTopologyNode, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_SetUINT64(This: *mut IMFTopologyNode, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyNode_SetUnknown(This: *mut IMFTopologyNode, guidKey: *const GUID, pUnknown: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetUnknown.unwrap_unchecked())(This, guidKey, pUnknown) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookupClient_InitServicePointers(This: *mut IMFTopologyServiceLookupClient, service_lookup: *mut IMFTopologyServiceLookup) -> HRESULT { ((*(*This).lpVtbl).InitServicePointers.unwrap_unchecked())(This, service_lookup) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookupClient_Release(This: *mut IMFTopologyServiceLookupClient) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookupClient_ReleaseServicePointers(This: *mut IMFTopologyServiceLookupClient) -> HRESULT { ((*(*This).lpVtbl).ReleaseServicePointers.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopologyServiceLookup_AddRef(This: *mut IMFTopologyServiceLookup) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_AddNode(This: *mut IMFTopology, node: *mut IMFTopologyNode) -> HRESULT { ((*(*This).lpVtbl).AddNode.unwrap_unchecked())(This, node) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_AddRef(This: *mut IMFTopology) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_Clear(This: *mut IMFTopology) -> HRESULT { ((*(*This).lpVtbl).Clear.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_CloneFrom(This: *mut IMFTopology, topology: *mut IMFTopology) -> HRESULT { ((*(*This).lpVtbl).CloneFrom.unwrap_unchecked())(This, topology) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_CopyAllItems(This: *mut IMFTopology, pDest: *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).CopyAllItems.unwrap_unchecked())(This, pDest) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_GetNode(This: *mut IMFTopology, index: WORD, node: *mut *mut IMFTopologyNode) -> HRESULT { ((*(*This).lpVtbl).GetNode.unwrap_unchecked())(This, index, node) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_GetNodeByID(This: *mut IMFTopology, id: TOPOID, node: *mut *mut IMFTopologyNode) -> HRESULT { ((*(*This).lpVtbl).GetNodeByID.unwrap_unchecked())(This, id, node) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_GetNodeCount(This: *mut IMFTopology, nodes: *mut WORD) -> HRESULT { ((*(*This).lpVtbl).GetNodeCount.unwrap_unchecked())(This, nodes) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_GetOutputNodeCollection(This: *mut IMFTopology, collection: *mut *mut IMFCollection) -> HRESULT { ((*(*This).lpVtbl).GetOutputNodeCollection.unwrap_unchecked())(This, collection) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_GetTopologyID(This: *mut IMFTopology, id: *mut TOPOID) -> HRESULT { ((*(*This).lpVtbl).GetTopologyID.unwrap_unchecked())(This, id) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_GetUINT32(This: *mut IMFTopology, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_Release(This: *mut IMFTopology) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTopology_RemoveNode(This: *mut IMFTopology, node: *mut IMFTopologyNode) -> HRESULT { ((*(*This).lpVtbl).RemoveNode.unwrap_unchecked())(This, node) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_AddInputStreams(This: *mut IMFTransform, streams: DWORD, ids: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).AddInputStreams.unwrap_unchecked())(This, streams, ids) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_AddRef(This: *mut IMFTransform) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_DeleteInputStream(This: *mut IMFTransform, id: DWORD) -> HRESULT { ((*(*This).lpVtbl).DeleteInputStream.unwrap_unchecked())(This, id) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetAttributes(This: *mut IMFTransform, attributes: *mut *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).GetAttributes.unwrap_unchecked())(This, attributes) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetInputAvailableType(This: *mut IMFTransform, id: DWORD, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetInputAvailableType.unwrap_unchecked())(This, id, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetInputCurrentType(This: *mut IMFTransform, id: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetInputCurrentType.unwrap_unchecked())(This, id, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetInputStreamAttributes(This: *mut IMFTransform, id: DWORD, attributes: *mut *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).GetInputStreamAttributes.unwrap_unchecked())(This, id, attributes) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputAvailableType(This: *mut IMFTransform, id: DWORD, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetOutputAvailableType.unwrap_unchecked())(This, id, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputCurrentType(This: *mut IMFTransform, id: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetOutputCurrentType.unwrap_unchecked())(This, id, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputStreamInfo(This: *mut IMFTransform, id: DWORD, info: *mut MFT_OUTPUT_STREAM_INFO) -> HRESULT { ((*(*This).lpVtbl).GetOutputStreamInfo.unwrap_unchecked())(This, id, info) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetStreamCount(This: *mut IMFTransform, inputs: *mut DWORD, outputs: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamCount.unwrap_unchecked())(This, inputs, outputs) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetStreamIDs(This: *mut IMFTransform, input_size: DWORD, inputs: *mut DWORD, output_size: DWORD, outputs: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamIDs.unwrap_unchecked())(This, input_size, inputs, output_size, outputs) } // widl C wrapper
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
pub unsafe fn IMFVideoDisplayControl_Release(This: *mut IMFVideoDisplayControl) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoDisplayControl_SetVideoPosition(This: *mut IMFVideoDisplayControl, source: *const MFVideoNormalizedRect, dest: *const RECT) -> HRESULT { ((*(*This).lpVtbl).SetVideoPosition.unwrap_unchecked())(This, source, dest) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoDisplayControl_SetVideoWindow(This: *mut IMFVideoDisplayControl, window: HWND) -> HRESULT { ((*(*This).lpVtbl).SetVideoWindow.unwrap_unchecked())(This, window) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_AddRef(This: *mut IMFVideoPresenter) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_OnClockPause(This: *mut IMFVideoPresenter, hnsSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockPause.unwrap_unchecked())(This, hnsSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_OnClockRestart(This: *mut IMFVideoPresenter, hnsSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockRestart.unwrap_unchecked())(This, hnsSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_OnClockSetRate(This: *mut IMFVideoPresenter, hnsSystemTime: MFTIME, flRate: f32) -> HRESULT { ((*(*This).lpVtbl).OnClockSetRate.unwrap_unchecked())(This, hnsSystemTime, flRate) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_OnClockStart(This: *mut IMFVideoPresenter, hnsSystemTime: MFTIME, llClockStartOffset: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).OnClockStart.unwrap_unchecked())(This, hnsSystemTime, llClockStartOffset) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_OnClockStop(This: *mut IMFVideoPresenter, hnssSystemTime: MFTIME) -> HRESULT { ((*(*This).lpVtbl).OnClockStop.unwrap_unchecked())(This, hnssSystemTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_ProcessMessage(This: *mut IMFVideoPresenter, eMessage: MFVP_MESSAGE_TYPE, ulParam: ULONG_PTR) -> HRESULT { ((*(*This).lpVtbl).ProcessMessage.unwrap_unchecked())(This, eMessage, ulParam) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_QueryInterface(This: *mut IMFVideoPresenter, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoPresenter_Release(This: *mut IMFVideoPresenter) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoRenderer_QueryInterface(This: *mut IMFVideoRenderer, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorCallback_Release(This: *mut IMFVideoSampleAllocatorCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorCallback_SetCallback(This: *mut IMFVideoSampleAllocatorCallback, callback: *mut IMFVideoSampleAllocatorNotify) -> HRESULT { ((*(*This).lpVtbl).SetCallback.unwrap_unchecked())(This, callback) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorNotify_AddRef(This: *mut IMFVideoSampleAllocatorNotify) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
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
pub unsafe fn IMMDeviceCollection_GetCount(This: *mut IMMDeviceCollection, pcDevices: *mut UINT) -> HRESULT { ((*(*This).lpVtbl).GetCount.unwrap_unchecked())(This, pcDevices) } // widl C wrapper
#[inline]
pub unsafe fn IMMDeviceCollection_Item(This: *mut IMMDeviceCollection, nDevice: UINT, ppdevice: *mut *mut IMMDevice) -> HRESULT { ((*(*This).lpVtbl).Item.unwrap_unchecked())(This, nDevice, ppdevice) } // widl C wrapper
#[inline]
pub unsafe fn IMMDeviceCollection_Release(This: *mut IMMDeviceCollection) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMMDeviceEnumerator_EnumAudioEndpoints(This: *mut IMMDeviceEnumerator, dataFlow: EDataFlow, dwStateMask: DWORD, ppDevices: *mut *mut IMMDeviceCollection) -> HRESULT { ((*(*This).lpVtbl).EnumAudioEndpoints.unwrap_unchecked())(This, dataFlow, dwStateMask, ppDevices) } // widl C wrapper
#[inline]
pub unsafe fn IMMDeviceEnumerator_GetDefaultAudioEndpoint(This: *mut IMMDeviceEnumerator, dataFlow: EDataFlow, role: ERole, ppEndpoint: *mut *mut IMMDevice) -> HRESULT { ((*(*This).lpVtbl).GetDefaultAudioEndpoint.unwrap_unchecked())(This, dataFlow, role, ppEndpoint) } // widl C wrapper
#[inline]
pub unsafe fn IMMDeviceEnumerator_GetDevice(This: *mut IMMDeviceEnumerator, pwstrId: LPCWSTR, ppDevice: *mut *mut IMMDevice) -> HRESULT { ((*(*This).lpVtbl).GetDevice.unwrap_unchecked())(This, pwstrId, ppDevice) } // widl C wrapper
#[inline]
pub unsafe fn IMMDeviceEnumerator_Release(This: *mut IMMDeviceEnumerator) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMMDevice_Activate(This: *mut IMMDevice, iid: *const IID, dwClsCtx: DWORD, pActivationParams: *mut PROPVARIANT, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).Activate.unwrap_unchecked())(This, iid, dwClsCtx, pActivationParams, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMMDevice_GetId(This: *mut IMMDevice, ppstrId: *mut LPWSTR) -> HRESULT { ((*(*This).lpVtbl).GetId.unwrap_unchecked())(This, ppstrId) } // widl C wrapper
#[inline]
pub unsafe fn IMMDevice_OpenPropertyStore(This: *mut IMMDevice, stgmAccess: DWORD, ppProperties: *mut *mut IPropertyStore) -> HRESULT { ((*(*This).lpVtbl).OpenPropertyStore.unwrap_unchecked())(This, stgmAccess, ppProperties) } // widl C wrapper
#[inline]
pub unsafe fn IMMDevice_Release(This: *mut IMMDevice) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMediaEventSink_AddRef(This: *mut IMediaEventSink) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPersist_GetClassID(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT { ((*(*This).lpVtbl).GetClassID.unwrap_unchecked())(This, pClassID) } // widl C wrapper
#[inline]
pub unsafe fn IPersist_Release(This: *mut IPersist) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_AddRef(This: *mut IPropertyStore) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_GetValue(This: *mut IPropertyStore, key: *const PROPERTYKEY, pv: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetValue.unwrap_unchecked())(This, key, pv) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_Release(This: *mut IPropertyStore) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ISimpleAudioVolume_GetMasterVolume(This: *mut ISimpleAudioVolume, pfLevel: *mut f32) -> HRESULT { ((*(*This).lpVtbl).GetMasterVolume.unwrap_unchecked())(This, pfLevel) } // widl C wrapper
#[inline]
pub unsafe fn ISimpleAudioVolume_GetMute(This: *mut ISimpleAudioVolume, pbMute: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).GetMute.unwrap_unchecked())(This, pbMute) } // widl C wrapper
#[inline]
pub unsafe fn ISimpleAudioVolume_Release(This: *mut ISimpleAudioVolume) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ISimpleAudioVolume_SetMasterVolume(This: *mut ISimpleAudioVolume, fLevel: f32, EventContext: LPCGUID) -> HRESULT { ((*(*This).lpVtbl).SetMasterVolume.unwrap_unchecked())(This, fLevel, EventContext) } // widl C wrapper
#[inline]
pub unsafe fn ISimpleAudioVolume_SetMute(This: *mut ISimpleAudioVolume, bMute: BOOL, EventContext: LPCGUID) -> HRESULT { ((*(*This).lpVtbl).SetMute.unwrap_unchecked())(This, bMute, EventContext) } // widl C wrapper
#[inline]
pub unsafe fn IStream_Release(This: *mut IStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const MF_E_CLOCK_NO_TIME_SOURCE: HRESULT = ((0xc00d9c41u32) as HRESULT);
pub const MF_E_CLOCK_STATE_ALREADY_SET: HRESULT = ((0xc00d9c42u32) as HRESULT);
pub const MF_E_INVALIDINDEX: HRESULT = ((0xc00d36bfu32) as HRESULT);
pub const MF_E_INVALIDMEDIATYPE: HRESULT = ((0xc00d36b4u32) as HRESULT);
pub const MF_E_INVALIDREQUEST: HRESULT = ((0xc00d36b2u32) as HRESULT);
pub const MF_E_INVALIDSTREAMNUMBER: HRESULT = ((0xc00d36b3u32) as HRESULT);
pub const MF_E_INVALIDTYPE: HRESULT = ((0xc00d36bdu32) as HRESULT);
pub const MF_E_INVALID_POSITION: HRESULT = ((0xc00d36e5u32) as HRESULT);
pub const MF_E_INVALID_STATE_TRANSITION: HRESULT = ((0xc00d3e82u32) as HRESULT);
pub const MF_E_INVALID_TIMESTAMP: HRESULT = ((0xc00d36c0u32) as HRESULT);
pub const MF_E_NOTACCEPTING: HRESULT = ((0xc00d36b5u32) as HRESULT);
pub const MF_E_NOT_FOUND: HRESULT = ((0xc00d36d5u32) as HRESULT);
pub const MF_E_NOT_INITIALIZED: HRESULT = ((0xc00d36b6u32) as HRESULT);
pub const MF_E_NO_AUDIO_PLAYBACK_DEVICE: HRESULT = ((0xc00d4e84u32) as HRESULT);
pub const MF_E_NO_CLOCK: HRESULT = ((0xc00d36d7u32) as HRESULT);
pub const MF_E_NO_MORE_TYPES: HRESULT = ((0xc00d36b9u32) as HRESULT);
pub const MF_E_REVERSE_UNSUPPORTED: HRESULT = ((0xc00d36d2u32) as HRESULT);
pub const MF_E_SESSION_PAUSEWHILESTOPPED: HRESULT = ((0xc00d36fcu32) as HRESULT);
pub const MF_E_SHUTDOWN: HRESULT = ((0xc00d3e85u32) as HRESULT);
pub const MF_E_STREAMSINKS_FIXED: HRESULT = ((0xc00d4a3bu32) as HRESULT);
pub const MF_E_STREAMSINK_REMOVED: HRESULT = ((0xc00d4a38u32) as HRESULT);
pub const MF_E_THINNING_UNSUPPORTED: HRESULT = ((0xc00d36d1u32) as HRESULT);
pub const MF_E_TOPO_CODEC_NOT_FOUND: HRESULT = ((0xc00d5212u32) as HRESULT);
pub const MF_E_TOPO_MISSING_PRESENTATION_DESCRIPTOR: HRESULT = ((0xc00d5217u32) as HRESULT);
pub const MF_E_TOPO_MISSING_SOURCE: HRESULT = ((0xc00d521au32) as HRESULT);
pub const MF_E_TOPO_MISSING_STREAM_DESCRIPTOR: HRESULT = ((0xc00d5218u32) as HRESULT);
pub const MF_E_TOPO_SINK_ACTIVATES_UNSUPPORTED: HRESULT = ((0xc00d521bu32) as HRESULT);
pub const MF_E_TOPO_STREAM_DESCRIPTOR_NOT_SELECTED: HRESULT = ((0xc00d5219u32) as HRESULT);
pub const MF_E_TOPO_UNSUPPORTED: HRESULT = ((0xc00d5214u32) as HRESULT);
pub const MF_E_TRANSCODE_NO_MATCHING_ENCODER: HRESULT = ((0xc00da412u32) as HRESULT);
pub const MF_E_TRANSFORM_NEED_MORE_INPUT: HRESULT = ((0xc00d6d72u32) as HRESULT);
pub const MF_E_TRANSFORM_NOT_POSSIBLE_FOR_CURRENT_MEDIATYPE_COMBINATION: HRESULT = ((0xc00d6d6fu32) as HRESULT);
pub const MF_E_TRANSFORM_STREAM_CHANGE: HRESULT = ((0xc00d6d61u32) as HRESULT);
pub const MF_E_TRANSFORM_TYPE_NOT_SET: HRESULT = ((0xc00d6d60u32) as HRESULT);
pub const MF_E_UNEXPECTED: HRESULT = ((0xc00d36bbu32) as HRESULT);
pub const MF_E_UNSUPPORTED_RATE: HRESULT = ((0xc00d36d0u32) as HRESULT);
pub const MF_E_UNSUPPORTED_SERVICE: HRESULT = ((0xc00d36bau32) as HRESULT);
pub const MF_S_CLOCK_STOPPED: HRESULT = ((0x000d9c44) as HRESULT);
pub type REFGUID = *const GUID;
pub type REFIID = *const IID;
pub type REFPROPVARIANT = *const PROPVARIANT;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{IsEqualIID, PropVariantInit, max, min};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// ==== end port macros ====
