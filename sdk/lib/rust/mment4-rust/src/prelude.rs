//! mment4-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};
// mmebuddy_debug.h / mmebuddy.h statement macros: DBG without NDEBUG here, so mmebuddy_debug.h compiles the printing bodies.
pub use mmebuddy_macros_rust::{StringLengthToBytes, AllocateWideString, ZeroWideString, CopyWideString};
pub use mmebuddy_macros_rust::{IS_VALID_SOUND_DEVICE_TYPE, IsValidSoundDeviceType, IS_WAVE_DEVICE_TYPE, IS_MIDI_DEVICE_TYPE, IS_AUX_DEVICE_TYPE, IS_MIXER_DEVICE_TYPE, SOUND_DEVICE_TYPE_TO_INDEX, INDEX_TO_SOUND_DEVICE_TYPE};
pub use mmebuddy_macros_rust::AllocateStruct;
pub use mmebuddy_macros_rust::SND_ERR_DBG as SND_ERR;
pub use mmebuddy_macros_rust::SND_WARN_DBG as SND_WARN;
pub use mmebuddy_macros_rust::SND_TRACE_DBG as SND_TRACE;
pub use mmebuddy_macros_rust::SND_ASSERT_DBG as SND_ASSERT;
pub use mmebuddy_macros_rust::DUMP_WAVEHDR_QUEUE_DBG as DUMP_WAVEHDR_QUEUE;
pub use mmebuddy_macros_rust::VALIDATE_MMSYS_PARAMETER_DBG as VALIDATE_MMSYS_PARAMETER;

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
pub const MAX_SOUND_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MAX_SOUND_DEVICE_TYPE; // island enum
pub const MIDI_IN_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MIDI_IN_DEVICE_TYPE; // island enum
pub const MIDI_OUT_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MIDI_OUT_DEVICE_TYPE; // island enum
pub const MIN_SOUND_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MIN_SOUND_DEVICE_TYPE; // island enum
pub const MMSYSERR_ERROR: u32 = 1; // audiosrv-rust/src/types_amd64.rs
pub const MMSYSERR_INVALPARAM: u32 = 11; // audiosrv-rust/src/types_amd64.rs
pub const MMSYSERR_NOERROR: u32 = 0; // audiosrv-rust/src/types_amd64.rs
pub const MMSYSERR_NOMEM: u32 = 7; // audiosrv-rust/src/types_amd64.rs
pub const WAVE_IN_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_WAVE_IN_DEVICE_TYPE; // island enum
pub const WAVE_OUT_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_WAVE_OUT_DEVICE_TYPE; // island enum
extern "C" {
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mment4::registry::OpenSoundDeviceRegKey;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const IOCTL_MIDI_GET_CAPABILITIES: i32 = 0x1D4204; // clang, control.c
pub const IOCTL_WAVE_GET_CAPABILITIES: i32 = 0x1D400C; // clang, control.c
pub const IOCTL_WAVE_QUERY_FORMAT: i32 = 0x1D4004; // clang, control.c
pub const IOCTL_WAVE_SET_FORMAT: i32 = 0x1D8008; // clang, control.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn AllocateMemory(Size: UINT) -> PVOID; // detect.c
    pub fn CloseKernelSoundDevice(Handle: HANDLE) -> MMRESULT; // control.c
    pub fn FreeMemory(Pointer: PVOID); // detect.c
    pub fn GetDigitCount(Number: UINT) -> UINT; // detect.c
    pub fn GetSoundDeviceIdentifier(SoundDevice: PSOUND_DEVICE, Identifier: *mut PVOID) -> MMRESULT; // control.c
    pub fn GetSoundDeviceInstanceHandle(
        SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
        Handle: *mut PVOID,
    ) -> MMRESULT; // control.c
    pub fn GetSoundDeviceType(SoundDevice: PSOUND_DEVICE, DeviceType: PMMDEVICE_TYPE) -> MMRESULT; // control.c
    pub fn IsValidSoundDevice(SoundDevice: PSOUND_DEVICE) -> BOOLEAN; // control.c
    pub fn IsValidSoundDeviceInstance(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE) -> BOOLEAN; // control.c
    pub fn OpenKernelSoundDeviceByName(
        DevicePath: PWSTR,
        ReadOnly: BOOLEAN,
        Handle: PHANDLE,
    ) -> MMRESULT; // detect.c
    pub fn SyncOverlappedDeviceIoControl(
        SoundDeviceInstance: HANDLE,
        IoControlCode: DWORD,
        InBuffer: LPVOID,
        InBufferSize: DWORD,
        OutBuffer: LPVOID,
        OutBufferSize: DWORD,
        BytesTransferred: LPDWORD,
    ) -> MMRESULT; // control.c
    pub fn TranslateInternalMmResult(Result: MMRESULT) -> MMRESULT; // control.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // detect.c
    pub fn wsprintfW(_: LPWSTR, _: LPCWSTR, ...) -> ::core::ffi::c_int; // detect.c
}
extern "system" {
    pub fn RegEnumValueW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpValueName: LPWSTR,
        lpcchValueName: LPDWORD,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // detect.c
    pub fn RegOpenKeyExW(_: HKEY, _: LPCWSTR, _: DWORD, _: REGSAM, _: PHKEY) -> LSTATUS; // detect.c
    pub fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpReserved: LPDWORD,
        lpcSubKeys: LPDWORD,
        lpcbMaxSubKeyLen: LPDWORD,
        lpcbMaxClassLen: LPDWORD,
        lpcValues: LPDWORD,
        lpcbMaxValueNameLen: LPDWORD,
        lpcbMaxValueLen: LPDWORD,
        lpcbSecurityDescriptor: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS; // detect.c
}
pub use crate::prelude::RegEnumValueW as RegEnumValue; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryInfoKeyW as RegQueryInfoKey; // sdk alias
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

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! MMSUCCESS { ($p0:expr $(,)?) => { (($p0 == 0) as c_int) }; }
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub use crate::{MMSUCCESS};
// ==== end SDK header macros ====
