//! mfreadwrite-rust prelude — the single import surface the translated modules use.
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
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const D3DFMT_A8B8G8R8: _D3DFORMAT = _D3DFORMAT_D3DFMT_A8B8G8R8; // island enum
pub const MFMEDIASOURCE_CAN_SEEK: _MFMEDIASOURCE_CHARACTERISTICS = _MFMEDIASOURCE_CHARACTERISTICS_MFMEDIASOURCE_CAN_SEEK; // island enum
pub const MFSTREAMSINK_MARKER_DEFAULT: _MFSTREAMSINK_MARKER_TYPE = _MFSTREAMSINK_MARKER_TYPE_MFSTREAMSINK_MARKER_DEFAULT; // island enum
pub const MFSTREAMSINK_MARKER_ENDOFSEGMENT: _MFSTREAMSINK_MARKER_TYPE = _MFSTREAMSINK_MARKER_TYPE_MFSTREAMSINK_MARKER_ENDOFSEGMENT; // island enum
pub const MFSTREAMSINK_MARKER_TICK: _MFSTREAMSINK_MARKER_TYPE = _MFSTREAMSINK_MARKER_TYPE_MFSTREAMSINK_MARKER_TICK; // island enum
pub const MFT_ENUM_FLAG_HARDWARE: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_HARDWARE; // island enum
pub const MFT_ENUM_FLAG_LOCALMFT: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_LOCALMFT; // island enum
pub const MFT_ENUM_FLAG_SORTANDFILTER: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_SORTANDFILTER; // island enum
pub const MFT_ENUM_FLAG_SYNCMFT: _MFT_ENUM_FLAG = _MFT_ENUM_FLAG_MFT_ENUM_FLAG_SYNCMFT; // island enum
pub const MFT_MESSAGE_COMMAND_DRAIN: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_COMMAND_DRAIN; // island enum
pub const MFT_MESSAGE_COMMAND_FLUSH: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_COMMAND_FLUSH; // island enum
pub const MFT_MESSAGE_NOTIFY_BEGIN_STREAMING: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_BEGIN_STREAMING; // island enum
pub const MFT_MESSAGE_NOTIFY_START_OF_STREAM: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_NOTIFY_START_OF_STREAM; // island enum
pub const MFT_MESSAGE_SET_D3D_MANAGER: _MFT_MESSAGE_TYPE = _MFT_MESSAGE_TYPE_MFT_MESSAGE_SET_D3D_MANAGER; // island enum
pub const MF_ACCESSMODE_WRITE: MF_FILE_ACCESSMODE = MF_FILE_ACCESSMODE_MF_ACCESSMODE_WRITE; // island enum
pub const MF_OBJECT_BYTESTREAM: MF_OBJECT_TYPE = MF_OBJECT_TYPE_MF_OBJECT_BYTESTREAM; // island enum
pub const MF_OBJECT_MEDIASOURCE: MF_OBJECT_TYPE = MF_OBJECT_TYPE_MF_OBJECT_MEDIASOURCE; // island enum
pub const MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED: MF_SOURCE_READER_FLAG = MF_SOURCE_READER_FLAG_MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED; // island enum
pub const MF_SOURCE_READERF_ENDOFSTREAM: MF_SOURCE_READER_FLAG = MF_SOURCE_READER_FLAG_MF_SOURCE_READERF_ENDOFSTREAM; // island enum
pub const MF_SOURCE_READERF_ERROR: MF_SOURCE_READER_FLAG = MF_SOURCE_READER_FLAG_MF_SOURCE_READERF_ERROR; // island enum
pub const MF_SOURCE_READERF_STREAMTICK: MF_SOURCE_READER_FLAG = MF_SOURCE_READER_FLAG_MF_SOURCE_READERF_STREAMTICK; // island enum
pub const MF_SOURCE_READER_CONTROLF_DRAIN: MF_SOURCE_READER_CONTROL_FLAG = MF_SOURCE_READER_CONTROL_FLAG_MF_SOURCE_READER_CONTROLF_DRAIN; // island enum
pub const VT_EMPTY: u16 = 0; // msctf-rust/src/prelude.rs
pub const VT_I8: DWORD = 0x14; // ole32-rust/src/win_consts.rs
pub const VT_UI4: DWORD = 19; // shell32-rust/src/cfs_folder.rs
pub const VT_UNKNOWN: u16 = 13; // msctf-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn abs(x: i32) -> i32; // charmap-rust/src/prelude.rs
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub static IID_IClassFactory: GUID; // cicero-rust/src/types_amd64.rs
    pub static IID_IUnknown: GUID; // cicero-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn InitializeConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE); // kernel32-vista-rust/src/sync.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn PathCchFindExtension(path: PCWSTR, size: usize, extension: *mut *const u16,) -> HRESULT; // kernel32-rust/src/client/pathcch.rs
    pub fn PropVariantClear(pvar: *mut PROPVARIANT) -> HRESULT; // ole32-rust/src/ring/ole2.rs
    pub fn PropVariantCopy(pvarDest: *mut PROPVARIANT, pvarSrc: *const PROPVARIANT,) -> HRESULT; // ole32-rust/src/ring/ole2.rs
    pub fn SleepConditionVariableCS(ConditionVariable: PRTL_CONDITION_VARIABLE, CriticalSection: PRTL_CRITICAL_SECTION, Timeout: DWORD,) -> BOOL; // kernel32-vista-rust/src/sync.rs
    pub fn WakeAllConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE); // kernel32-vista-rust/src/sync.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const MFT_OUTPUT_STREAM_CAN_PROVIDE_SAMPLES: i32 = 0x200; // clang, reader.c
pub const MFT_OUTPUT_STREAM_PROVIDES_SAMPLES: i32 = 0x100; // clang, reader.c
pub const MF_OPENMODE_DELETE_IF_EXIST: i32 = 4; // clang, writer.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CONDITION_VARIABLE: c2rust, reader.c
pub type CONDITION_VARIABLE = RTL_CONDITION_VARIABLE;
// extension_map: c2rust, writer.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct extension_map {
    pub ext: *const WCHAR,
    pub guid: *const GUID,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn MFCreateAlignedMemoryBuffer(
        max_length: DWORD,
        alignment: DWORD,
        buffer: *mut *mut IMFMediaBuffer,
    ) -> HRESULT; // reader.c
    pub fn MFCreateAttributes(attributes: *mut *mut IMFAttributes, size: UINT32) -> HRESULT; // reader.c
    pub fn MFCreateFile(
        accessmode: MF_FILE_ACCESSMODE,
        openmode: MF_FILE_OPENMODE,
        flags: MF_FILE_FLAGS,
        url: LPCWSTR,
        bytestream: *mut *mut IMFByteStream,
    ) -> HRESULT; // writer.c
    pub fn MFCreateMediaType(type_0: *mut *mut IMFMediaType) -> HRESULT; // reader.c
    pub fn MFCreatePresentationClock(clock: *mut *mut IMFPresentationClock) -> HRESULT; // writer.c
    pub fn MFCreateSample(sample: *mut *mut IMFSample) -> HRESULT; // reader.c
    pub fn MFCreateSourceResolver(resolver: *mut *mut IMFSourceResolver) -> HRESULT; // reader.c
    pub fn MFCreateSystemTimeSource(time_source: *mut *mut IMFPresentationTimeSource) -> HRESULT; // writer.c
    pub fn MFCreateVideoSampleAllocatorEx(
        riid: *const IID,
        allocator: *mut *mut ::core::ffi::c_void,
    ) -> HRESULT; // reader.c
    pub fn MFGetPlaneSize(format: DWORD, width: DWORD, height: DWORD, size: *mut DWORD) -> HRESULT; // reader.c
    pub fn MFGetStrideForBitmapInfoHeader(format: DWORD, width: DWORD, stride: *mut LONG) -> HRESULT; // reader.c
    pub fn MFGetSystemTime() -> MFTIME; // writer.c
    pub fn MFLockSharedWorkQueue(
        name: *const WCHAR,
        base_priority: LONG,
        taskid: *mut DWORD,
        queue: *mut DWORD,
    ) -> HRESULT; // reader.c
    pub fn MFPutWorkItem(
        queue: DWORD,
        callback: *mut IMFAsyncCallback,
        state: *mut IUnknown,
    ) -> HRESULT; // reader.c
    pub fn MFTEnumEx(
        category: GUID,
        flags: UINT32,
        input_type: *const MFT_REGISTER_TYPE_INFO,
        output_type: *const MFT_REGISTER_TYPE_INFO,
        activate: *mut *mut *mut IMFActivate,
        pcount: *mut UINT32,
    ) -> HRESULT; // reader.c
    pub fn MFUnlockWorkQueue(queue: DWORD) -> HRESULT; // reader.c
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
pub use crate::mfreadwrite::writer::create_sink_writer_from_sink;
pub use crate::mfreadwrite::writer::create_sink_writer_from_url;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! PropVariantInit { ($p0:expr $(,)?) => { memset((($p0) as _), ((0) as _), ((core::mem::size_of::<PROPVARIANT>()) as _)) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT);
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT);
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
pub unsafe fn IClassFactory_QueryInterface(This: *mut IClassFactory, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_ActivateObject(This: *mut IMFActivate, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).ActivateObject.unwrap_unchecked())(This, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFActivate_Release(This: *mut IMFActivate) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncCallback_AddRef(This: *mut IMFAsyncCallback) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetState(This: *mut IMFAsyncResult, ppunkState: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetState.unwrap_unchecked())(This, ppunkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFAsyncResult_GetStateNoAddRef(This: *mut IMFAsyncResult) -> *mut IUnknown { ((*(*This).lpVtbl).GetStateNoAddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_AddRef(This: *mut IMFAttributes) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetGUID(This: *mut IMFAttributes, guidKey: *const GUID, pguidValue: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetGUID.unwrap_unchecked())(This, guidKey, pguidValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetItem(This: *mut IMFAttributes, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUINT32(This: *mut IMFAttributes, guidKey: *const GUID, punValue: *mut UINT32) -> HRESULT { ((*(*This).lpVtbl).GetUINT32.unwrap_unchecked())(This, guidKey, punValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_GetUnknown(This: *mut IMFAttributes, guidKey: *const GUID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetUnknown.unwrap_unchecked())(This, guidKey, riid, ppv) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_Release(This: *mut IMFAttributes) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFAttributes_SetUINT32(This: *mut IMFAttributes, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_AddRef(This: *mut IMFByteStream) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFByteStream_Release(This: *mut IMFByteStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFCollection_Release(This: *mut IMFCollection) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_GetService(This: *mut IMFGetService, guidService: *const GUID, riid: *const IID, ppvObject: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).GetService.unwrap_unchecked())(This, guidService, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFGetService_Release(This: *mut IMFGetService) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_GetCurrentLength(This: *mut IMFMediaBuffer, pcbCurrentLength: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetCurrentLength.unwrap_unchecked())(This, pcbCurrentLength) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaBuffer_Release(This: *mut IMFMediaBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_GetStatus(This: *mut IMFMediaEvent, phrStatus: *mut HRESULT) -> HRESULT { ((*(*This).lpVtbl).GetStatus.unwrap_unchecked())(This, phrStatus) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_GetType(This: *mut IMFMediaEvent, pmet: *mut MediaEventType) -> HRESULT { ((*(*This).lpVtbl).GetType.unwrap_unchecked())(This, pmet) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_GetValue(This: *mut IMFMediaEvent, pvValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetValue.unwrap_unchecked())(This, pvValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaEvent_Release(This: *mut IMFMediaEvent) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_AddRef(This: *mut IMFMediaSink) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_AddStreamSink(This: *mut IMFMediaSink, stream_sink_id: DWORD, media_type: *mut IMFMediaType, stream_sink: *mut *mut IMFStreamSink) -> HRESULT { ((*(*This).lpVtbl).AddStreamSink.unwrap_unchecked())(This, stream_sink_id, media_type, stream_sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_GetStreamSinkByIndex(This: *mut IMFMediaSink, index: DWORD, sink: *mut *mut IMFStreamSink) -> HRESULT { ((*(*This).lpVtbl).GetStreamSinkByIndex.unwrap_unchecked())(This, index, sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_GetStreamSinkCount(This: *mut IMFMediaSink, count: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamSinkCount.unwrap_unchecked())(This, count) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_QueryInterface(This: *mut IMFMediaSink, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_Release(This: *mut IMFMediaSink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_RemoveStreamSink(This: *mut IMFMediaSink, stream_sink_id: DWORD) -> HRESULT { ((*(*This).lpVtbl).RemoveStreamSink.unwrap_unchecked())(This, stream_sink_id) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSink_SetPresentationClock(This: *mut IMFMediaSink, clock: *mut IMFPresentationClock) -> HRESULT { ((*(*This).lpVtbl).SetPresentationClock.unwrap_unchecked())(This, clock) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_AddRef(This: *mut IMFMediaSource) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_BeginGetEvent(This: *mut IMFMediaSource, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_CreatePresentationDescriptor(This: *mut IMFMediaSource, descriptor: *mut *mut IMFPresentationDescriptor) -> HRESULT { ((*(*This).lpVtbl).CreatePresentationDescriptor.unwrap_unchecked())(This, descriptor) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_EndGetEvent(This: *mut IMFMediaSource, pResult: *mut IMFAsyncResult, ppEvent: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).EndGetEvent.unwrap_unchecked())(This, pResult, ppEvent) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_GetCharacteristics(This: *mut IMFMediaSource, characteristics: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetCharacteristics.unwrap_unchecked())(This, characteristics) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Release(This: *mut IMFMediaSource) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Shutdown(This: *mut IMFMediaSource) -> HRESULT { ((*(*This).lpVtbl).Shutdown.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Start(This: *mut IMFMediaSource, descriptor: *mut IMFPresentationDescriptor, time_format: *const GUID, start_position: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).Start.unwrap_unchecked())(This, descriptor, time_format, start_position) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaSource_Stop(This: *mut IMFMediaSource) -> HRESULT { ((*(*This).lpVtbl).Stop.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_AddRef(This: *mut IMFMediaStream) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_BeginGetEvent(This: *mut IMFMediaStream, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_EndGetEvent(This: *mut IMFMediaStream, pResult: *mut IMFAsyncResult, ppEvent: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).EndGetEvent.unwrap_unchecked())(This, pResult, ppEvent) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_GetStreamDescriptor(This: *mut IMFMediaStream, descriptor: *mut *mut IMFStreamDescriptor) -> HRESULT { ((*(*This).lpVtbl).GetStreamDescriptor.unwrap_unchecked())(This, descriptor) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_Release(This: *mut IMFMediaStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaStream_RequestSample(This: *mut IMFMediaStream, token: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).RequestSample.unwrap_unchecked())(This, token) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_GetCurrentMediaType(This: *mut IMFMediaTypeHandler, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetCurrentMediaType.unwrap_unchecked())(This, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_GetMajorType(This: *mut IMFMediaTypeHandler, type_: *mut GUID) -> HRESULT { ((*(*This).lpVtbl).GetMajorType.unwrap_unchecked())(This, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_GetMediaTypeByIndex(This: *mut IMFMediaTypeHandler, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetMediaTypeByIndex.unwrap_unchecked())(This, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_Release(This: *mut IMFMediaTypeHandler) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaTypeHandler_SetCurrentMediaType(This: *mut IMFMediaTypeHandler, type_: *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).SetCurrentMediaType.unwrap_unchecked())(This, type_) } // widl C wrapper
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
pub unsafe fn IMFMediaType_IsCompressedFormat(This: *mut IMFMediaType, pfCompressed: *mut BOOL) -> HRESULT { ((*(*This).lpVtbl).IsCompressedFormat.unwrap_unchecked())(This, pfCompressed) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_IsEqual(This: *mut IMFMediaType, pIMediaType: *mut IMFMediaType, pdwFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).IsEqual.unwrap_unchecked())(This, pIMediaType, pdwFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_Release(This: *mut IMFMediaType) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetItem(This: *mut IMFMediaType, guidKey: *const GUID, Value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).SetItem.unwrap_unchecked())(This, guidKey, Value) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetUINT32(This: *mut IMFMediaType, guidKey: *const GUID, unValue: UINT32) -> HRESULT { ((*(*This).lpVtbl).SetUINT32.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFMediaType_SetUINT64(This: *mut IMFMediaType, guidKey: *const GUID, unValue: UINT64) -> HRESULT { ((*(*This).lpVtbl).SetUINT64.unwrap_unchecked())(This, guidKey, unValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_Release(This: *mut IMFPresentationClock) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_SetTimeSource(This: *mut IMFPresentationClock, time_source: *mut IMFPresentationTimeSource) -> HRESULT { ((*(*This).lpVtbl).SetTimeSource.unwrap_unchecked())(This, time_source) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationClock_Start(This: *mut IMFPresentationClock, start_offset: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).Start.unwrap_unchecked())(This, start_offset) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_DeselectStream(This: *mut IMFPresentationDescriptor, index: DWORD) -> HRESULT { ((*(*This).lpVtbl).DeselectStream.unwrap_unchecked())(This, index) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_GetItem(This: *mut IMFPresentationDescriptor, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_GetStreamDescriptorByIndex(This: *mut IMFPresentationDescriptor, index: DWORD, selected: *mut BOOL, descriptor: *mut *mut IMFStreamDescriptor) -> HRESULT { ((*(*This).lpVtbl).GetStreamDescriptorByIndex.unwrap_unchecked())(This, index, selected, descriptor) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_GetStreamDescriptorCount(This: *mut IMFPresentationDescriptor, count: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamDescriptorCount.unwrap_unchecked())(This, count) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_Release(This: *mut IMFPresentationDescriptor) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationDescriptor_SelectStream(This: *mut IMFPresentationDescriptor, index: DWORD) -> HRESULT { ((*(*This).lpVtbl).SelectStream.unwrap_unchecked())(This, index) } // widl C wrapper
#[inline]
pub unsafe fn IMFPresentationTimeSource_Release(This: *mut IMFPresentationTimeSource) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFReadWriteClassFactory_AddRef(This: *mut IMFReadWriteClassFactory) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFReadWriteClassFactory_QueryInterface(This: *mut IMFReadWriteClassFactory, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
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
pub unsafe fn IMFSample_GetSampleDuration(This: *mut IMFSample, phnsSampleDuration: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleDuration.unwrap_unchecked())(This, phnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleFlags(This: *mut IMFSample, pdwSampleFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetSampleFlags.unwrap_unchecked())(This, pdwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_GetSampleTime(This: *mut IMFSample, phnsSampleTime: *mut LONGLONG) -> HRESULT { ((*(*This).lpVtbl).GetSampleTime.unwrap_unchecked())(This, phnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_Release(This: *mut IMFSample) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleDuration(This: *mut IMFSample, hnsSampleDuration: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleDuration.unwrap_unchecked())(This, hnsSampleDuration) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleFlags(This: *mut IMFSample, dwSampleFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetSampleFlags.unwrap_unchecked())(This, dwSampleFlags) } // widl C wrapper
#[inline]
pub unsafe fn IMFSample_SetSampleTime(This: *mut IMFSample, hnsSampleTime: LONGLONG) -> HRESULT { ((*(*This).lpVtbl).SetSampleTime.unwrap_unchecked())(This, hnsSampleTime) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkClassFactory_CreateMediaSink(This: *mut IMFSinkClassFactory, stream: *mut IMFByteStream, video_type: *mut IMFMediaType, audio_type: *mut IMFMediaType, sink: *mut *mut IMFMediaSink) -> HRESULT { ((*(*This).lpVtbl).CreateMediaSink.unwrap_unchecked())(This, stream, video_type, audio_type, sink) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkClassFactory_Release(This: *mut IMFSinkClassFactory) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkWriterCallback_OnMarker(This: *mut IMFSinkWriterCallback, stream_index: DWORD, context: *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).OnMarker.unwrap_unchecked())(This, stream_index, context) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkWriterCallback_Release(This: *mut IMFSinkWriterCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkWriterEx_AddRef(This: *mut IMFSinkWriterEx) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkWriterEx_QueryInterface(This: *mut IMFSinkWriterEx, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFSinkWriterEx_Release(This: *mut IMFSinkWriterEx) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceReaderCallback_OnEvent(This: *mut IMFSourceReaderCallback, stream_index: DWORD, event: *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).OnEvent.unwrap_unchecked())(This, stream_index, event) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceReaderCallback_OnFlush(This: *mut IMFSourceReaderCallback, stream_index: DWORD) -> HRESULT { ((*(*This).lpVtbl).OnFlush.unwrap_unchecked())(This, stream_index) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceReaderCallback_OnReadSample(This: *mut IMFSourceReaderCallback, hr: HRESULT, stream_index: DWORD, stream_flags: DWORD, timestamp: LONGLONG, sample: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).OnReadSample.unwrap_unchecked())(This, hr, stream_index, stream_flags, timestamp, sample) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceReaderCallback_Release(This: *mut IMFSourceReaderCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceReaderEx_QueryInterface(This: *mut IMFSourceReaderEx, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceReaderEx_Release(This: *mut IMFSourceReaderEx) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_CreateObjectFromByteStream(This: *mut IMFSourceResolver, stream: *mut IMFByteStream, url: *const WCHAR, flags: DWORD, props: *mut IPropertyStore, obj_type: *mut MF_OBJECT_TYPE, object: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CreateObjectFromByteStream.unwrap_unchecked())(This, stream, url, flags, props, obj_type, object) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_CreateObjectFromURL(This: *mut IMFSourceResolver, url: *const WCHAR, flags: DWORD, props: *mut IPropertyStore, obj_type: *mut MF_OBJECT_TYPE, object: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CreateObjectFromURL.unwrap_unchecked())(This, url, flags, props, obj_type, object) } // widl C wrapper
#[inline]
pub unsafe fn IMFSourceResolver_Release(This: *mut IMFSourceResolver) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_GetItem(This: *mut IMFStreamDescriptor, guidKey: *const GUID, pValue: *mut PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).GetItem.unwrap_unchecked())(This, guidKey, pValue) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_GetMediaTypeHandler(This: *mut IMFStreamDescriptor, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT { ((*(*This).lpVtbl).GetMediaTypeHandler.unwrap_unchecked())(This, handler) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_GetStreamIdentifier(This: *mut IMFStreamDescriptor, identifier: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetStreamIdentifier.unwrap_unchecked())(This, identifier) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamDescriptor_Release(This: *mut IMFStreamDescriptor) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_AddRef(This: *mut IMFStreamSink) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_BeginGetEvent(This: *mut IMFStreamSink, pCallback: *mut IMFAsyncCallback, punkState: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).BeginGetEvent.unwrap_unchecked())(This, pCallback, punkState) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_EndGetEvent(This: *mut IMFStreamSink, pResult: *mut IMFAsyncResult, ppEvent: *mut *mut IMFMediaEvent) -> HRESULT { ((*(*This).lpVtbl).EndGetEvent.unwrap_unchecked())(This, pResult, ppEvent) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_Flush(This: *mut IMFStreamSink) -> HRESULT { ((*(*This).lpVtbl).Flush.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_GetIdentifier(This: *mut IMFStreamSink, identifier: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetIdentifier.unwrap_unchecked())(This, identifier) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_GetMediaTypeHandler(This: *mut IMFStreamSink, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT { ((*(*This).lpVtbl).GetMediaTypeHandler.unwrap_unchecked())(This, handler) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_PlaceMarker(This: *mut IMFStreamSink, marker_type: MFSTREAMSINK_MARKER_TYPE, marker_value: *const PROPVARIANT, context_value: *const PROPVARIANT) -> HRESULT { ((*(*This).lpVtbl).PlaceMarker.unwrap_unchecked())(This, marker_type, marker_value, context_value) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_ProcessSample(This: *mut IMFStreamSink, sample: *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).ProcessSample.unwrap_unchecked())(This, sample) } // widl C wrapper
#[inline]
pub unsafe fn IMFStreamSink_Release(This: *mut IMFStreamSink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_AddRef(This: *mut IMFTransform) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetAttributes(This: *mut IMFTransform, attributes: *mut *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).GetAttributes.unwrap_unchecked())(This, attributes) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetInputAvailableType(This: *mut IMFTransform, id: DWORD, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetInputAvailableType.unwrap_unchecked())(This, id, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetInputCurrentType(This: *mut IMFTransform, id: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetInputCurrentType.unwrap_unchecked())(This, id, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputAvailableType(This: *mut IMFTransform, id: DWORD, index: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetOutputAvailableType.unwrap_unchecked())(This, id, index, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputCurrentType(This: *mut IMFTransform, id: DWORD, type_: *mut *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).GetOutputCurrentType.unwrap_unchecked())(This, id, type_) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputStreamAttributes(This: *mut IMFTransform, id: DWORD, attributes: *mut *mut IMFAttributes) -> HRESULT { ((*(*This).lpVtbl).GetOutputStreamAttributes.unwrap_unchecked())(This, id, attributes) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_GetOutputStreamInfo(This: *mut IMFTransform, id: DWORD, info: *mut MFT_OUTPUT_STREAM_INFO) -> HRESULT { ((*(*This).lpVtbl).GetOutputStreamInfo.unwrap_unchecked())(This, id, info) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_ProcessInput(This: *mut IMFTransform, id: DWORD, sample: *mut IMFSample, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).ProcessInput.unwrap_unchecked())(This, id, sample, flags) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_ProcessMessage(This: *mut IMFTransform, message: MFT_MESSAGE_TYPE, param: ULONG_PTR) -> HRESULT { ((*(*This).lpVtbl).ProcessMessage.unwrap_unchecked())(This, message, param) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_ProcessOutput(This: *mut IMFTransform, flags: DWORD, count: DWORD, samples: *mut MFT_OUTPUT_DATA_BUFFER, status: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).ProcessOutput.unwrap_unchecked())(This, flags, count, samples, status) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_Release(This: *mut IMFTransform) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_SetInputType(This: *mut IMFTransform, id: DWORD, type_: *mut IMFMediaType, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetInputType.unwrap_unchecked())(This, id, type_, flags) } // widl C wrapper
#[inline]
pub unsafe fn IMFTransform_SetOutputType(This: *mut IMFTransform, id: DWORD, type_: *mut IMFMediaType, flags: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetOutputType.unwrap_unchecked())(This, id, type_, flags) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorCallback_Release(This: *mut IMFVideoSampleAllocatorCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorCallback_SetCallback(This: *mut IMFVideoSampleAllocatorCallback, callback: *mut IMFVideoSampleAllocatorNotify) -> HRESULT { ((*(*This).lpVtbl).SetCallback.unwrap_unchecked())(This, callback) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_AllocateSample(This: *mut IMFVideoSampleAllocatorEx, sample: *mut *mut IMFSample) -> HRESULT { ((*(*This).lpVtbl).AllocateSample.unwrap_unchecked())(This, sample) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_InitializeSampleAllocatorEx(This: *mut IMFVideoSampleAllocatorEx, initial_sample_count: DWORD, max_sample_count: DWORD, attributes: *mut IMFAttributes, media_type: *mut IMFMediaType) -> HRESULT { ((*(*This).lpVtbl).InitializeSampleAllocatorEx.unwrap_unchecked())(This, initial_sample_count, max_sample_count, attributes, media_type) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_QueryInterface(This: *mut IMFVideoSampleAllocatorEx, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_Release(This: *mut IMFVideoSampleAllocatorEx) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_SetDirectXManager(This: *mut IMFVideoSampleAllocatorEx, manager: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetDirectXManager.unwrap_unchecked())(This, manager) } // widl C wrapper
#[inline]
pub unsafe fn IMFVideoSampleAllocatorEx_UninitializeSampleAllocator(This: *mut IMFVideoSampleAllocatorEx) -> HRESULT { ((*(*This).lpVtbl).UninitializeSampleAllocator.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IPropertyStore_Release(This: *mut IPropertyStore) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const MF_E_ATTRIBUTENOTFOUND: HRESULT = ((0xc00d36e6u32) as HRESULT);
pub const MF_E_INVALIDINDEX: HRESULT = ((0xc00d36bfu32) as HRESULT);
pub const MF_E_INVALIDMEDIATYPE: HRESULT = ((0xc00d36b4u32) as HRESULT);
pub const MF_E_INVALIDREQUEST: HRESULT = ((0xc00d36b2u32) as HRESULT);
pub const MF_E_INVALIDSTREAMNUMBER: HRESULT = ((0xc00d36b3u32) as HRESULT);
pub const MF_E_MEDIA_SOURCE_NO_STREAMS_SELECTED: HRESULT = ((0xc00d3e9cu32) as HRESULT);
pub const MF_E_NOTACCEPTING: HRESULT = ((0xc00d36b5u32) as HRESULT);
pub const MF_E_TOPO_CODEC_NOT_FOUND: HRESULT = ((0xc00d5212u32) as HRESULT);
pub const MF_E_TRANSFORM_NEED_MORE_INPUT: HRESULT = ((0xc00d6d72u32) as HRESULT);
pub const MF_E_TRANSFORM_STREAM_CHANGE: HRESULT = ((0xc00d6d61u32) as HRESULT);
pub const MF_E_UNSUPPORTED_SERVICE: HRESULT = ((0xc00d36bau32) as HRESULT);
pub type REFGUID = *const GUID;
pub type REFIID = *const IID;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{IsEqualIID, PropVariantInit, max};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/mfreadwrite/mf_private.h:27: header-inline `mf_array_reserve`
#[allow(dead_code)]
#[inline]
pub unsafe fn mf_array_reserve(
    elements: *mut *mut core::ffi::c_void,
    capacity: *mut size_t,
    count: size_t,
    size: size_t,
) -> BOOL {
    if count <= *capacity {
        return (TRUE) as BOOL;
    }

    let max_capacity: size_t = (!(0 as SIZE_T) / (size) as SIZE_T) as _;
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

    let new_elements: *mut core::ffi::c_void = realloc(*elements, new_capacity * size);
    if new_elements.is_null() {
        return (FALSE) as BOOL;
    }

    *elements = new_elements;
    *capacity = new_capacity;

    (TRUE as i32)
}
// ==== end port macros ====
