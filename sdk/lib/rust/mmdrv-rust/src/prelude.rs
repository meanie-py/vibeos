//! mmdrv-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const AuxDevice: DeviceType = DeviceType_AuxDevice; // island enum
pub const MidiInDevice: DeviceType = DeviceType_MidiInDevice; // island enum
pub const MidiOutDevice: DeviceType = DeviceType_MidiOutDevice; // island enum
pub const WaveInDevice: DeviceType = DeviceType_WaveInDevice; // island enum
pub const WaveOutDevice: DeviceType = DeviceType_WaveOutDevice; // island enum
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DefDriverProc( arg1: DWORD_PTR, arg2: HDRVR, arg3: UINT, arg4: LPARAM, arg5: LPARAM, ) -> LRESULT; // sndrec32-rust/src/types_i386.rs
    pub fn DeviceIoControl( hDevice: *mut c_void, dwIoControlCode: u32, lpInBuffer: *mut c_void, nInBufferSize: u32, lpOutBuffer: *mut c_void, nOutBufferSize: u32, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThread() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetOverlappedResult( hFile: *mut ::core::ffi::c_void, lpOverlapped: LPOVERLAPPED, lpNumberOfBytesTransferred: LPDWORD, bWait: ::core::ffi::c_int, ) -> BOOL; // winlogon-sys/src/bindgen_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetThreadPriority(arg1: HANDLE, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObjectEx(arg1: HANDLE, arg2: DWORD, arg3: BOOL) -> DWORD; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mmdrv::common::CloseDevice;
pub use crate::mmdrv::common::ErrorToMmResult;
pub use crate::mmdrv::common::GetDeviceCapabilities;
pub use crate::mmdrv::common::GetDeviceCount;
pub use crate::mmdrv::session::HandleBySessionThread;
pub use crate::mmdrv::common::OpenDevice;
pub use crate::mmdrv::wave_io::PerformWaveIO;
pub use crate::mmdrv::wave::QueryWaveFormat;
pub use crate::mmdrv::wave::WaveThread;
pub use crate::mmdrv::wave::WriteWaveBuffer;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const IOCTL_MIDI_GET_CAPABILITIES: i32 = 0x1D4204; // clang, common.c
pub const IOCTL_WAVE_GET_CAPABILITIES: i32 = 0x1D400C; // clang, common.c
pub const IOCTL_WAVE_GET_PITCH: i32 = 0x1D8028; // clang, wave.c
pub const IOCTL_WAVE_GET_PLAYBACK_RATE: i32 = 0x1D8030; // clang, wave.c
pub const IOCTL_WAVE_SET_FORMAT: i32 = 0x1D8008; // clang, wave.c
pub const IOCTL_WAVE_SET_PITCH: i32 = 0x1D8024; // clang, wave.c
pub const IOCTL_WAVE_SET_PLAYBACK_RATE: i32 = 0x1D802C; // clang, wave.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // kernel.c
    pub fn wsprintfW(_: LPWSTR, _: LPCWSTR, ...) -> ::core::ffi::c_int; // kernel.c
}
extern "system" {
    pub fn CreateEventW(
        _: LPSECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE; // kernel.c
    pub fn CreateFileW(
        _: LPCWSTR,
        _: DWORD,
        _: DWORD,
        _: LPSECURITY_ATTRIBUTES,
        _: DWORD,
        _: DWORD,
        _: HANDLE,
    ) -> HANDLE; // kernel.c
    pub fn DriverCallback(
        dwCallBack: DWORD,
        uFlags: UINT,
        hDev: HDRVR,
        wMsg: UINT,
        dwUser: DWORD,
        dwParam1: DWORD,
        dwParam2: DWORD,
    ) -> BOOL; // mme.c
    pub fn mmTaskCreate(lpfn: LPTASKCALLBACK, lph: *mut HANDLE, dwInst: DWORD) -> UINT; // session.c
}
pub use crate::prelude::CreateEventW as CreateEvent; // sdk alias
pub use crate::prelude::CreateFileW as CreateFile; // sdk alias
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
// ==== end UNICODE TCHAR family ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/mmdrv/mmdrv.h:56: #define IsAuxDevice(devicetype) ( devicetype == AuxDevice )
#[allow(unused_macros)]
macro_rules! IsAuxDevice {
    ($devicetype:expr $(,)?) => {
        (($devicetype) == AuxDevice)
    };
}
#[allow(unused_imports)]
pub(crate) use IsAuxDevice;
// dll/win32/mmdrv/mmdrv.h:53: #define IsMidiDevice(devicetype) ( ( devicetype == MidiOutDevice ) || ( devicetype == MidiInDevice ) )
#[allow(unused_macros)]
macro_rules! IsMidiDevice {
    ($devicetype:expr $(,)?) => {
        ((($devicetype) == MidiOutDevice) || (($devicetype) == MidiInDevice))
    };
}
#[allow(unused_imports)]
pub(crate) use IsMidiDevice;
// dll/win32/mmdrv/mmdrv.h:50: #define IsWaveDevice(devicetype) ( ( devicetype == WaveOutDevice ) || ( devicetype == WaveInDevice ) )
#[allow(unused_macros)]
macro_rules! IsWaveDevice {
    ($devicetype:expr $(,)?) => {
        ((($devicetype) == WaveOutDevice) || (($devicetype) == WaveInDevice))
    };
}
#[allow(unused_imports)]
pub(crate) use IsWaveDevice;
// ==== end port macros ====
