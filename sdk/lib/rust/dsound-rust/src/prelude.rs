//! dsound-rust prelude — the single import surface the translated modules use.
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
// win32macros-rust with the other macros. DPRINT is off, but its arguments still
// type-check (DBG_TYPECHECK: a never-called closure), so a stale one is a compile error
// rather than silently swallowed tokens.
#[macro_export] macro_rules! DPRINT {
    ($($arg:tt)*) => { ::win32macros_rust::DBG_TYPECHECK!($($arg)*) };
}
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_TYPECHECK, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlSecureZeroMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, SecureZeroMemory, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_ERR_, WINE_FIXME, WINE_FIXME_, WINE_TRACE, WINE_TRACE_, WINE_WARN, WINE_WARN_, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_fourcc, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, strlenW, tolowerW, wide_bytes, wine_dbgstr_a, wine_dbgstr_fourcc, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_rect, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const DIRECTSOUNDDEVICE_DATAFLOW_CAPTURE: DIRECTSOUNDDEVICE_DATAFLOW = DIRECTSOUNDDEVICE_DATAFLOW_DIRECTSOUNDDEVICE_DATAFLOW_CAPTURE; // island enum // gen:38550a7d6248
pub const DIRECTSOUNDDEVICE_DATAFLOW_RENDER: DIRECTSOUNDDEVICE_DATAFLOW = DIRECTSOUNDDEVICE_DATAFLOW_DIRECTSOUNDDEVICE_DATAFLOW_RENDER; // island enum // gen:5f90ddd7ff1b
pub const DIRECTSOUNDDEVICE_TYPE_VXD: DIRECTSOUNDDEVICE_TYPE = DIRECTSOUNDDEVICE_TYPE_DIRECTSOUNDDEVICE_TYPE_VXD; // island enum // gen:7b288eb0b257
pub const DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1; // island enum // gen:20d63edb1c30
pub const DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A; // island enum // gen:9b6ebed8bfa0
pub const DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W; // island enum // gen:fab800b7f792
pub const DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1; // island enum // gen:d12ec6a39ffc
pub const DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A; // island enum // gen:176efd86fadb
pub const DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W; // island enum // gen:66bdb24a1500
pub const DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A; // island enum // gen:e5d5b85eb470
pub const DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W: DSPROPERTY_DIRECTSOUNDDEVICE = DSPROPERTY_DIRECTSOUNDDEVICE_DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W; // island enum // gen:1d58ef756fa4
pub const HKEY_CURRENT_USER: usize = 0x8000_0001; // rapps-rust/src/rapps/appdb_cpp_h.rs // gen:e72774d412c2
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // winegdi-rust/src/types_i386.rs // gen:87d449b869a7
    pub fn _assert(message: *const i8, file: *const i8, line: u32); // comctl32-rust/src/prelude.rs // gen:babfe4f9f3dc
    pub fn acos(x: f64) -> f64; // calc-rust/src/prelude.rs // gen:3ef33e3182dd
    pub fn log(x: f64) -> f64; // calc-rust/src/prelude.rs // gen:09d82f3d19bb
    pub fn log10(x: f64) -> f64; // calc-rust/src/prelude.rs // gen:9f652d43da42
    pub fn pow(x: f64, y: f64) -> f64; // beepmidi-rust/src/prelude.rs // gen:8218403ae7c2
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs // gen:3291f684b033
    pub fn sqrt(x: f64) -> f64; // calc-rust/src/prelude.rs // gen:c7f92507f753
    pub fn wine_dbg_sprintf(format: *const ::core::ffi::c_char, ...) -> *const ::core::ffi::c_char; // winegdi-rust/src/types_i386.rs // gen:64f2d13938b7
    pub static GUID_NULL: IID; // evr-rust/src/types_amd64.rs // gen:baf72c332eee
    pub static KSDATAFORMAT_SUBTYPE_IEEE_FLOAT: GUID; // audiosrv-rust/src/types_amd64.rs // gen:81206c3b004a
    pub static KSDATAFORMAT_SUBTYPE_PCM: GUID; // audiosrv-rust/src/types_amd64.rs // gen:5d5a69983a23
}
extern "system" {
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:7893eb6ab4e4
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:111127023c7f
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:b4d73724c7aa
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs // gen:b82f9257e472
    pub fn GetModuleFileNameA(hModule: HINSTANCE, lpFilename: LPSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:51bffb082f29
    pub fn GetModuleHandleExW(arg1: DWORD, arg2: LPCWSTR, arg3: *mut HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e24f313d56d9
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:8b50fa1e44f3
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs // gen:f624a448a1f6
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:3aaa7317691b
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:cd40b76d9b6b
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:13245079d396
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:4cc9a88dd118
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:d32c25758174
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:198f85e2baee
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:4862658b57c4
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:dcf787d63821
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:0abd4a9770ae
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::dsound::sound3d::DSOUND_Calc3DBuffer;
pub use crate::dsound::mixer::DSOUND_CheckEvent;
pub use crate::dsound::primary::DSOUND_CopyFormat;
pub use crate::dsound::duplex::DSOUND_FullDuplexCreate;
pub use crate::dsound::mixer::DSOUND_MixToTemporary;
pub use crate::dsound::primary::DSOUND_PrimaryCreate;
pub use crate::dsound::primary::DSOUND_PrimaryDestroy;
pub use crate::dsound::primary::DSOUND_PrimaryGetPosition;
pub use crate::dsound::primary::DSOUND_PrimaryPlay;
pub use crate::dsound::primary::DSOUND_PrimaryStop;
pub use crate::dsound::mixer::DSOUND_RecalcFormat;
pub use crate::dsound::mixer::DSOUND_RecalcVolPan;
pub use crate::dsound::primary::DSOUND_ReopenDevice;
pub use crate::dsound::primary::DSOUND_fraglen;
pub use crate::dsound::mixer::DSOUND_secpos_to_bufpos;
pub use crate::dsound::mixer::DSOUND_timer;
pub use crate::dsound::dsound::DirectSoundDevice_AddBuffer;
pub use crate::dsound::dsound::DirectSoundDevice_RemoveBuffer;
pub use crate::dsound::dsound_main::GetDeviceID;
pub use crate::dsound::sound3d::IDirectSound3DBufferImpl_Create;
pub use crate::dsound::sound3d::IDirectSound3DBufferImpl_Destroy;
pub use crate::dsound::sound3d::IDirectSound3DListenerImpl_Create;
pub use crate::dsound::propset::IKsPrivatePropertySetImpl_Create;
pub use crate::dsound::dsound_main::mmErr;
pub use crate::dsound::primary::primarybuffer_SetFormat;
pub use crate::dsound::primary::primarybuffer_create;
pub use crate::dsound::primary::primarybuffer_destroy;
pub use crate::dsound::dsound_main::setup_dsound_options;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const M_PI: f64 = 3.14159265358979323846; // clang, sound3d.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// IDirectSound8_IDirectSound: c2rust, dsound.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSound8_IDirectSound {
    pub lpVtbl: *const IDirectSoundVtbl,
    pub ref_: LONG,
    pub pds: LPDIRECTSOUND8,
}
// IDirectSound8_IDirectSound8: c2rust, dsound.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSound8_IDirectSound8 {
    pub lpVtbl: *const IDirectSound8Vtbl,
    pub ref_: LONG,
    pub pds: LPDIRECTSOUND8,
}
// IDirectSound8_IUnknown: c2rust, dsound.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSound8_IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
    pub ref_: LONG,
    pub pds: LPDIRECTSOUND8,
}
// IDirectSoundCaptureImpl: c2rust, capture.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSoundCaptureImpl {
    pub lpVtbl: *const IDirectSoundCaptureVtbl,
    pub ref_: LONG,
    pub device: *mut DirectSoundCaptureDevice,
}
// IDirectSoundCaptureNotifyImpl: c2rust, capture.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSoundCaptureNotifyImpl {
    pub lpVtbl: *const IDirectSoundNotifyVtbl,
    pub ref_: LONG,
    pub dscb: *mut IDirectSoundCaptureBufferImpl,
}
// IDirectSoundImpl: c2rust, dsound.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSoundImpl {
    pub ref_: LONG,
    pub device: *mut DirectSoundDevice,
    pub pUnknown: LPUNKNOWN,
    pub pDS: LPDIRECTSOUND,
    pub pDS8: LPDIRECTSOUND8,
}
// IDirectSoundNotifyImpl: c2rust, buffer.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSoundNotifyImpl {
    pub lpVtbl: *const IDirectSoundNotifyVtbl,
    pub ref_: LONG,
    pub dsb: *mut IDirectSoundBufferImpl,
}
// IDirectSound_IDirectSound: c2rust, dsound.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSound_IDirectSound {
    pub lpVtbl: *const IDirectSoundVtbl,
    pub ref_: LONG,
    pub pds: LPDIRECTSOUND8,
}
// IDirectSound_IUnknown: c2rust, dsound.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDirectSound_IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
    pub ref_: LONG,
    pub pds: LPDIRECTSOUND8,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn atoi(_Str: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // dsound_main.c
    pub fn strcat(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // dsound_main.c
    pub fn strcmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // dsound_main.c
    pub fn strrchr(
        _Str: *const ::core::ffi::c_char,
        _Ch: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // dsound_main.c
}
pub use win32macros_rust::WINE_ERR as ERR; // sdk alias
pub use win32macros_rust::WINE_FIXME as FIXME; // sdk alias
pub use crate::types::IID_IDirectSoundCapture as IID_IDirectSoundCapture8; // sdk alias
pub use win32macros_rust::WINE_TRACE as TRACE; // sdk alias
pub use win32macros_rust::WINE_WARN as WARN; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub static mut __wine_dbch_dsound: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x73\x6f\x75\x6e\x64\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_dsound };
// ==== end wine debug channels ====


// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! FillMemory { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { memset((($p0) as _), (($p2) as _), (($p1) as _)) }; } // gen:69249fdb75eb
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; } // gen:aa85287a8771
#[macro_export] macro_rules! le16 { ($p0:expr $(,)?) => { $p0 }; } // gen:5c6291c1d09d
#[macro_export] macro_rules! le32 { ($p0:expr $(,)?) => { $p0 }; } // gen:9ccefef092c0
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT); // gen:1197fd83f980
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT); // gen:01676cfea511
pub const DSERR_ALLOCATED: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((10) as c_uint))) as HRESULT); // gen:81db1c652736
pub const DSERR_ALREADYINITIALIZED: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((130) as c_uint))) as HRESULT); // gen:4ece15f86a8c
pub const DSERR_BADFORMAT: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((100) as c_uint))) as HRESULT); // gen:72d503ac1a9d
pub const DSERR_BUFFERLOST: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((150) as c_uint))) as HRESULT); // gen:786261f7f8c1
pub const DSERR_CONTROLUNAVAIL: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((30) as c_uint))) as HRESULT); // gen:3bd21e18b289
pub const DSERR_GENERIC: HRESULT = ((0x80004005u32) as HRESULT); // gen:01c4ca5f04d8
pub const DSERR_INVALIDCALL: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((50) as c_uint))) as HRESULT); // gen:030fce417af2
pub const DSERR_INVALIDPARAM: HRESULT = ((0x80070057u32) as HRESULT); // gen:7e4abd1680aa
pub const DSERR_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT); // gen:0b20026adc7f
pub const DSERR_NODRIVER: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((120) as c_uint))) as HRESULT); // gen:c7583ebde000
pub const DSERR_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:621550fcf803
pub const DSERR_PRIOLEVELNEEDED: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((70) as c_uint))) as HRESULT); // gen:c5053c333b80
pub const DSERR_UNINITIALIZED: HRESULT = (((((((1) as c_uint) << 31) | (((0x878) as c_uint) << 16)) | ((170) as c_uint))) as HRESULT); // gen:6c95e4a5e437
pub const DSERR_UNSUPPORTED: HRESULT = ((0x80004001u32) as HRESULT); // gen:5d84f81146d1
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const E_PROP_ID_UNSUPPORTED: HRESULT = ((0x80070490u32) as HRESULT); // gen:598429036114
#[inline] // gen:bd5d7911e44d
pub unsafe fn IDirectSound3DBuffer_AddRef(This: *mut IDirectSound3DBuffer) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:684b771e572f
pub unsafe fn IDirectSound3DListener_AddRef(This: *mut IDirectSound3DListener) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ce18fc326319
pub unsafe fn IDirectSound8_Compact(This: *mut IDirectSound8) -> HRESULT { ((*(*This).lpVtbl).Compact.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2b11d1e55487
pub unsafe fn IDirectSound8_CreateSoundBuffer(This: *mut IDirectSound8, lpcDSBufferDesc: LPCDSBUFFERDESC, lplpDirectSoundBuffer: LPLPDIRECTSOUNDBUFFER, pUnkOuter: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).CreateSoundBuffer.unwrap_unchecked())(This, lpcDSBufferDesc, lplpDirectSoundBuffer, pUnkOuter) } // widl C wrapper
#[inline] // gen:0a77b338a61b
pub unsafe fn IDirectSound8_GetCaps(This: *mut IDirectSound8, lpDSCaps: LPDSCAPS) -> HRESULT { ((*(*This).lpVtbl).GetCaps.unwrap_unchecked())(This, lpDSCaps) } // widl C wrapper
#[inline] // gen:12670fc5c59d
pub unsafe fn IDirectSound8_GetSpeakerConfig(This: *mut IDirectSound8, lpdwSpeakerConfig: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetSpeakerConfig.unwrap_unchecked())(This, lpdwSpeakerConfig) } // widl C wrapper
#[inline] // gen:76b062bc7880
pub unsafe fn IDirectSound8_Initialize(This: *mut IDirectSound8, lpcGuid: LPCGUID) -> HRESULT { ((*(*This).lpVtbl).Initialize.unwrap_unchecked())(This, lpcGuid) } // widl C wrapper
#[inline] // gen:8ce0f56c31e8
pub unsafe fn IDirectSound8_Release(This: *mut IDirectSound8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:9eccaa65a03f
pub unsafe fn IDirectSound8_SetCooperativeLevel(This: *mut IDirectSound8, hwnd: HWND, dwLevel: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetCooperativeLevel.unwrap_unchecked())(This, hwnd, dwLevel) } // widl C wrapper
#[inline] // gen:aaf4b5e332f9
pub unsafe fn IDirectSound8_SetSpeakerConfig(This: *mut IDirectSound8, dwSpeakerConfig: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetSpeakerConfig.unwrap_unchecked())(This, dwSpeakerConfig) } // widl C wrapper
#[inline] // gen:12af71645c85
pub unsafe fn IDirectSound8_VerifyCertification(This: *mut IDirectSound8, pdwCertified: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).VerifyCertification.unwrap_unchecked())(This, pdwCertified) } // widl C wrapper
#[inline] // gen:2c69f42876fc
pub unsafe fn IDirectSoundBuffer8_AddRef(This: *mut IDirectSoundBuffer8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:dddec607271a
pub unsafe fn IDirectSoundBuffer8_Release(This: *mut IDirectSoundBuffer8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:8b8bcb04f5f3
pub unsafe fn IDirectSoundBuffer_AddRef(This: *mut IDirectSoundBuffer) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:c666a9aa90cd
pub unsafe fn IDirectSoundBuffer_GetCurrentPosition(This: *mut IDirectSoundBuffer, lpdwCurrentPlayCursor: LPDWORD, lpdwCurrentWriteCursor: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetCurrentPosition.unwrap_unchecked())(This, lpdwCurrentPlayCursor, lpdwCurrentWriteCursor) } // widl C wrapper
#[inline] // gen:f1c9f901df1f
pub unsafe fn IDirectSoundBuffer_QueryInterface(This: *mut IDirectSoundBuffer, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:4ad4ac66aa8a
pub unsafe fn IDirectSoundBuffer_Release(This: *mut IDirectSoundBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2ded13c55e4b
pub unsafe fn IDirectSoundCaptureBuffer8_AddRef(This: *mut IDirectSoundCaptureBuffer8) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:43b4915449b7
pub unsafe fn IDirectSoundCaptureBuffer_AddRef(This: *mut IDirectSoundCaptureBuffer) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:22e6f028b523
pub unsafe fn IDirectSoundCaptureBuffer_QueryInterface(This: *mut IDirectSoundCaptureBuffer, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:2b50e9113c45
pub unsafe fn IDirectSoundCaptureBuffer_Release(This: *mut IDirectSoundCaptureBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1a183f4913ed
pub unsafe fn IDirectSoundCapture_AddRef(This: *mut IDirectSoundCapture) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:31aa038effde
pub unsafe fn IDirectSoundCapture_CreateCaptureBuffer(This: *mut IDirectSoundCapture, lpcDSCBufferDesc: LPCDSCBUFFERDESC, lplpDSCaptureBuffer: *mut LPDIRECTSOUNDCAPTUREBUFFER, pUnk: LPUNKNOWN) -> HRESULT { ((*(*This).lpVtbl).CreateCaptureBuffer.unwrap_unchecked())(This, lpcDSCBufferDesc, lplpDSCaptureBuffer, pUnk) } // widl C wrapper
#[inline] // gen:a6058ca93610
pub unsafe fn IDirectSoundCapture_Initialize(This: *mut IDirectSoundCapture, lpcGUID: LPCGUID) -> HRESULT { ((*(*This).lpVtbl).Initialize.unwrap_unchecked())(This, lpcGUID) } // widl C wrapper
#[inline] // gen:fc6442dc16da
pub unsafe fn IDirectSoundCapture_Release(This: *mut IDirectSoundCapture) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:afcc87e27f71
pub unsafe fn IDirectSoundFullDuplex_QueryInterface(This: *mut IDirectSoundFullDuplex, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:3bf17c2ed825
pub unsafe fn IDirectSoundNotify_AddRef(This: *mut IDirectSoundNotify) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6a109dada528
pub unsafe fn IDirectSoundNotify_Release(This: *mut IDirectSoundNotify) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:fdf406488aba
pub unsafe fn IDirectSound_Initialize(This: *mut IDirectSound, lpcGuid: LPCGUID) -> HRESULT { ((*(*This).lpVtbl).Initialize.unwrap_unchecked())(This, lpcGuid) } // widl C wrapper
#[inline] // gen:44a97cdad89d
pub unsafe fn IDirectSound_Release(This: *mut IDirectSound) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e92ae6eb15d1
pub unsafe fn IDsCaptureDriverBuffer_GetPosition(This: *mut IDsCaptureDriverBuffer, lpdwCurrentPlayCursor: LPDWORD, lpdwCurrentWriteCursor: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetPosition.unwrap_unchecked())(This, lpdwCurrentPlayCursor, lpdwCurrentWriteCursor) } // widl C wrapper
#[inline] // gen:296cea027e44
pub unsafe fn IDsCaptureDriverBuffer_Lock(This: *mut IDsCaptureDriverBuffer, ppvAudio1: *mut LPVOID, pdwLen1: LPDWORD, ppvAudio2: *mut LPVOID, pdwLen2: LPDWORD, dwWritePosition: DWORD, dwWriteLen: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).Lock.unwrap_unchecked())(This, ppvAudio1, pdwLen1, ppvAudio2, pdwLen2, dwWritePosition, dwWriteLen, dwFlags) } // widl C wrapper
#[inline] // gen:849c93ca617c
pub unsafe fn IDsCaptureDriverBuffer_QueryInterface(This: *mut IDsCaptureDriverBuffer, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:1cba2afb1077
pub unsafe fn IDsCaptureDriverBuffer_Release(This: *mut IDsCaptureDriverBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:951d3fa97655
pub unsafe fn IDsCaptureDriverBuffer_Start(This: *mut IDsCaptureDriverBuffer, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).Start.unwrap_unchecked())(This, dwFlags) } // widl C wrapper
#[inline] // gen:1581c95e5985
pub unsafe fn IDsCaptureDriverBuffer_Stop(This: *mut IDsCaptureDriverBuffer) -> HRESULT { ((*(*This).lpVtbl).Stop.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:2b068083be14
pub unsafe fn IDsCaptureDriverBuffer_Unlock(This: *mut IDsCaptureDriverBuffer, pvAudio1: LPVOID, dwLen1: DWORD, pvAudio2: LPVOID, dwLen2: DWORD) -> HRESULT { ((*(*This).lpVtbl).Unlock.unwrap_unchecked())(This, pvAudio1, dwLen1, pvAudio2, dwLen2) } // widl C wrapper
#[inline] // gen:eba45aae8867
pub unsafe fn IDsCaptureDriver_Close(This: *mut IDsCaptureDriver) -> HRESULT { ((*(*This).lpVtbl).Close.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:0cb6fb41a72d
pub unsafe fn IDsCaptureDriver_CreateCaptureBuffer(This: *mut IDsCaptureDriver, pwfx: LPWAVEFORMATEX, dwFlags: DWORD, dwCardAddress: DWORD, pdwcbBufferSize: LPDWORD, ppbBuffer: *mut LPBYTE, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).CreateCaptureBuffer.unwrap_unchecked())(This, pwfx, dwFlags, dwCardAddress, pdwcbBufferSize, ppbBuffer, ppvObj) } // widl C wrapper
#[inline] // gen:b53d7241b307
pub unsafe fn IDsCaptureDriver_Release(This: *mut IDsCaptureDriver) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ade977a9fa07
pub unsafe fn IDsDriverBuffer_GetPosition(This: *mut IDsDriverBuffer, lpdwCurrentPlayCursor: LPDWORD, lpdwCurrentWriteCursor: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).GetPosition.unwrap_unchecked())(This, lpdwCurrentPlayCursor, lpdwCurrentWriteCursor) } // widl C wrapper
#[inline] // gen:54f65f946d75
pub unsafe fn IDsDriverBuffer_Lock(This: *mut IDsDriverBuffer, ppvAudio1: *mut LPVOID, pdwLen1: LPDWORD, pdwAudio2: *mut LPVOID, pdwLen2: LPDWORD, dwWritePosition: DWORD, dwWriteLen: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).Lock.unwrap_unchecked())(This, ppvAudio1, pdwLen1, pdwAudio2, pdwLen2, dwWritePosition, dwWriteLen, dwFlags) } // widl C wrapper
#[inline] // gen:e8a12df5111f
pub unsafe fn IDsDriverBuffer_Play(This: *mut IDsDriverBuffer, dwReserved1: DWORD, dwReserved2: DWORD, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).Play.unwrap_unchecked())(This, dwReserved1, dwReserved2, dwFlags) } // widl C wrapper
#[inline] // gen:38f79a52021a
pub unsafe fn IDsDriverBuffer_Release(This: *mut IDsDriverBuffer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1ea20029d4de
pub unsafe fn IDsDriverBuffer_SetFormat(This: *mut IDsDriverBuffer, pwfxToSet: LPWAVEFORMATEX) -> HRESULT { ((*(*This).lpVtbl).SetFormat.unwrap_unchecked())(This, pwfxToSet) } // widl C wrapper
#[inline] // gen:534b0732b896
pub unsafe fn IDsDriverBuffer_SetPosition(This: *mut IDsDriverBuffer, dwNewPosition: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetPosition.unwrap_unchecked())(This, dwNewPosition) } // widl C wrapper
#[inline] // gen:bffbb2ddc754
pub unsafe fn IDsDriverBuffer_SetVolumePan(This: *mut IDsDriverBuffer, pDsVolumePan: PDSVOLUMEPAN) -> HRESULT { ((*(*This).lpVtbl).SetVolumePan.unwrap_unchecked())(This, pDsVolumePan) } // widl C wrapper
#[inline] // gen:5f0964b43edb
pub unsafe fn IDsDriverBuffer_Stop(This: *mut IDsDriverBuffer) -> HRESULT { ((*(*This).lpVtbl).Stop.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:fa95d3335169
pub unsafe fn IDsDriverBuffer_Unlock(This: *mut IDsDriverBuffer, pvAudio1: LPVOID, dwLen1: DWORD, pvAudio2: LPVOID, dwLen2: DWORD) -> HRESULT { ((*(*This).lpVtbl).Unlock.unwrap_unchecked())(This, pvAudio1, dwLen1, pvAudio2, dwLen2) } // widl C wrapper
#[inline] // gen:0ef0ff6da88c
pub unsafe fn IDsDriverNotify_Release(This: *mut IDsDriverNotify) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ff124b830149
pub unsafe fn IDsDriverNotify_SetNotificationPositions(This: *mut IDsDriverNotify, dwPositionNotifies: DWORD, pcPositionNotifies: LPCDSBPOSITIONNOTIFY) -> HRESULT { ((*(*This).lpVtbl).SetNotificationPositions.unwrap_unchecked())(This, dwPositionNotifies, pcPositionNotifies) } // widl C wrapper
#[inline] // gen:6d6e44568ab4
pub unsafe fn IDsDriverPropertySet_Get(This: *mut IDsDriverPropertySet, pDsProperty: PDSPROPERTY, pPropertyParams: LPVOID, cbPropertyParams: ULONG, pPropertyData: LPVOID, cbPropertyData: ULONG, pcbReturnedData: PULONG) -> HRESULT { ((*(*This).lpVtbl).Get.unwrap_unchecked())(This, pDsProperty, pPropertyParams, cbPropertyParams, pPropertyData, cbPropertyData, pcbReturnedData) } // widl C wrapper
#[inline] // gen:6bdec17d8522
pub unsafe fn IDsDriverPropertySet_QuerySupport(This: *mut IDsDriverPropertySet, PropertySetId: *const GUID, PropertyId: ULONG, pSupport: PULONG) -> HRESULT { ((*(*This).lpVtbl).QuerySupport.unwrap_unchecked())(This, PropertySetId, PropertyId, pSupport) } // widl C wrapper
#[inline] // gen:a99313888cb4
pub unsafe fn IDsDriverPropertySet_Release(This: *mut IDsDriverPropertySet) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:454656a463a2
pub unsafe fn IDsDriverPropertySet_Set(This: *mut IDsDriverPropertySet, pDsProperty: PDSPROPERTY, pPropertyParams: LPVOID, cbPropertyParams: ULONG, pPropertyData: LPVOID, cbPropertyData: ULONG) -> HRESULT { ((*(*This).lpVtbl).Set.unwrap_unchecked())(This, pDsProperty, pPropertyParams, cbPropertyParams, pPropertyData, cbPropertyData) } // widl C wrapper
#[inline] // gen:7a2367ac4f1b
pub unsafe fn IDsDriver_Close(This: *mut IDsDriver) -> HRESULT { ((*(*This).lpVtbl).Close.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:31423ef15717
pub unsafe fn IDsDriver_CreateSoundBuffer(This: *mut IDsDriver, pwfx: LPWAVEFORMATEX, dwFlags: DWORD, dwCardAddress: DWORD, pdwcbBufferSize: LPDWORD, ppbBuffer: *mut LPBYTE, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).CreateSoundBuffer.unwrap_unchecked())(This, pwfx, dwFlags, dwCardAddress, pdwcbBufferSize, ppbBuffer, ppvObj) } // widl C wrapper
#[inline] // gen:47f43840e61f
pub unsafe fn IDsDriver_DuplicateSoundBuffer(This: *mut IDsDriver, pIDsDriverBuffer: PIDSDRIVERBUFFER, ppvObj: *mut LPVOID) -> HRESULT { ((*(*This).lpVtbl).DuplicateSoundBuffer.unwrap_unchecked())(This, pIDsDriverBuffer, ppvObj) } // widl C wrapper
#[inline] // gen:76c9c6219f44
pub unsafe fn IDsDriver_GetCaps(This: *mut IDsDriver, pDsDrvCaps: PDSDRIVERCAPS) -> HRESULT { ((*(*This).lpVtbl).GetCaps.unwrap_unchecked())(This, pDsDrvCaps) } // widl C wrapper
#[inline] // gen:ab813c029b83
pub unsafe fn IDsDriver_GetDriverDesc(This: *mut IDsDriver, pDsDriverDesc: PDSDRIVERDESC) -> HRESULT { ((*(*This).lpVtbl).GetDriverDesc.unwrap_unchecked())(This, pDsDriverDesc) } // widl C wrapper
#[inline] // gen:9f6744d822fb
pub unsafe fn IDsDriver_Open(This: *mut IDsDriver) -> HRESULT { ((*(*This).lpVtbl).Open.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:7f5840c095ec
pub unsafe fn IDsDriver_QueryInterface(This: *mut IDsDriver, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:2b116bc8805f
pub unsafe fn IDsDriver_Release(This: *mut IDsDriver) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:a7775d7092fc
pub unsafe fn IKsPropertySet_AddRef(This: *mut IKsPropertySet) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:aa234a85a98a
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub type REFCLSID = *const CLSID; // gen:4a91e3a9efb2
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{FillMemory, IsEqualIID, le16, le32};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/directx/wine/dsound/dsound_main.c:611: #define INIT_GUID(guid, l, w1, w2, b1, b2, b3, b4, b5, b6, b7, b8) guid.Data1 = l; guid.Data2 = w1; guid.Data3 = w2; guid.Data4[0] = b1; guid.Data4[1] = b2; guid.Data4[2] = b3; guid.Data4[3] = b4; guid.Data4[4] = b5; guid.Data4[5] = b6; guid.Data4[6] = b7; guid.Data4[7] = b8; // gen:5087c482e51d
#[allow(unused_macros)]
macro_rules! INIT_GUID {
    ($guid:expr, $l:expr, $w1:expr, $w2:expr, $b1:expr, $b2:expr, $b3:expr, $b4:expr, $b5:expr, $b6:expr, $b7:expr, $b8:expr $(,)?) => {
        {
            $guid.Data1 = $l;
            $guid.Data2 = $w1;
            $guid.Data3 = $w2;
            $guid.Data4[0] = $b1;
            $guid.Data4[1] = $b2;
            $guid.Data4[2] = $b3;
            $guid.Data4[3] = $b4;
            $guid.Data4[4] = $b5;
            $guid.Data4[5] = $b6;
            $guid.Data4[6] = $b7;
            $guid.Data4[7] = $b8;
        }
    };
}
#[allow(unused_imports)]
pub(crate) use INIT_GUID;
// ==== end port macros ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static IID_IDsDriverNotify: GUID = GUID { Data1: 0x00363EF44, Data2: 0x3B57, Data3: 0x11D3, Data4: [0xAC, 0x79, 0x00, 0x10, 0x5A, 0x01, 0x7f, 0xe1] };
#[no_mangle]
pub static IID_IDsDriverPropertySet: GUID = GUID { Data1: 0x0F6F2E8E0, Data2: 0xD842, Data3: 0x11D0, Data4: [0x8F, 0x75, 0x00, 0xC0, 0x4F, 0xC2, 0x8A, 0xCA] };
// ==== end INITGUID storage ====
