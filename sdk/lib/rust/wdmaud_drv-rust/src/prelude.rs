//! wdmaud_drv-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};
// mmebuddy_debug.h / mmebuddy.h statement macros: NDEBUG is defined here, so mmebuddy_debug.h compiles the quiet bodies.
pub use mmebuddy_macros_rust::{StringLengthToBytes, AllocateWideString, ZeroWideString, CopyWideString};
pub use mmebuddy_macros_rust::{IS_VALID_SOUND_DEVICE_TYPE, IsValidSoundDeviceType, IS_WAVE_DEVICE_TYPE, IS_MIDI_DEVICE_TYPE, IS_AUX_DEVICE_TYPE, IS_MIXER_DEVICE_TYPE, SOUND_DEVICE_TYPE_TO_INDEX, INDEX_TO_SOUND_DEVICE_TYPE};
pub use mmebuddy_macros_rust::AllocateStruct;
pub use mmebuddy_macros_rust::SND_ERR_NDEBUG as SND_ERR;
pub use mmebuddy_macros_rust::SND_WARN_NDEBUG as SND_WARN;
pub use mmebuddy_macros_rust::SND_TRACE_NDEBUG as SND_TRACE;
pub use mmebuddy_macros_rust::SND_ASSERT_NDEBUG as SND_ASSERT;
pub use mmebuddy_macros_rust::DUMP_WAVEHDR_QUEUE_NDEBUG as DUMP_WAVEHDR_QUEUE;
pub use mmebuddy_macros_rust::VALIDATE_MMSYS_PARAMETER_NDEBUG as VALIDATE_MMSYS_PARAMETER;

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const AUX_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_AUX_DEVICE_TYPE; // island enum
pub const KSRESET_BEGIN: KSRESET = KSRESET_KSRESET_BEGIN; // island enum
pub const KSRESET_END: KSRESET = KSRESET_KSRESET_END; // island enum
pub const KSSTATE_ACQUIRE: KSSTATE = KSSTATE_KSSTATE_ACQUIRE; // island enum
pub const KSSTATE_PAUSE: KSSTATE = KSSTATE_KSSTATE_PAUSE; // island enum
pub const KSSTATE_RUN: KSSTATE = KSSTATE_KSSTATE_RUN; // island enum
pub const KSSTATE_STOP: KSSTATE = KSSTATE_KSSTATE_STOP; // island enum
pub const MAX_SOUND_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MAX_SOUND_DEVICE_TYPE; // island enum
pub const MIDI_IN_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MIDI_IN_DEVICE_TYPE; // island enum
pub const MIDI_OUT_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MIDI_OUT_DEVICE_TYPE; // island enum
pub const MIN_SOUND_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MIN_SOUND_DEVICE_TYPE; // island enum
pub const MIXER_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_MIXER_DEVICE_TYPE; // island enum
pub const MM_STATUS_MORE_ENTRIES: MIXER_STATUS = MIXER_STATUS_MM_STATUS_MORE_ENTRIES; // island enum
pub const MM_STATUS_NO_MEMORY: MIXER_STATUS = MIXER_STATUS_MM_STATUS_NO_MEMORY; // island enum
pub const MM_STATUS_NO_MORE_DEVICES: MIXER_STATUS = MIXER_STATUS_MM_STATUS_NO_MORE_DEVICES; // island enum
pub const MM_STATUS_SUCCESS: MIXER_STATUS = MIXER_STATUS_MM_STATUS_SUCCESS; // island enum
pub const MM_STATUS_UNSUCCESSFUL: MIXER_STATUS = MIXER_STATUS_MM_STATUS_UNSUCCESSFUL; // island enum
pub const WAVE_IN_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_WAVE_IN_DEVICE_TYPE; // island enum
pub const WAVE_OUT_DEVICE_TYPE: SOUND_DEVICE_TYPE = SOUND_DEVICE_TYPE_WAVE_OUT_DEVICE_TYPE; // island enum
extern "C" {
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DefDriverProc( arg1: DWORD_PTR, arg2: HDRVR, arg3: UINT, arg4: LPARAM, arg5: LPARAM, ) -> LRESULT; // sndrec32-rust/src/types_i386.rs
    pub fn DeviceIoControl( hDevice: *mut c_void, dwIoControlCode: u32, lpInBuffer: *mut c_void, nInBufferSize: u32, lpOutBuffer: *mut c_void, nOutBufferSize: u32, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetOverlappedResult( hFile: *mut ::core::ffi::c_void, lpOverlapped: LPOVERLAPPED, lpNumberOfBytesTransferred: LPDWORD, bWait: ::core::ffi::c_int, ) -> BOOL; // winlogon-sys/src/bindgen_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReadFileEx( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: LPOVERLAPPED, arg5: LPOVERLAPPED_COMPLETION_ROUTINE, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetupDiDestroyDeviceInfoList(arg1: HDEVINFO) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupDiEnumDeviceInterfaces( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: *const GUID, arg4: DWORD, arg5: PSP_DEVICE_INTERFACE_DATA, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupDiGetClassDevsW( arg1: *const GUID, arg2: PCWSTR, arg3: HWND, arg4: DWORD, ) -> HDEVINFO; // desk-rust/src/types_i386.rs
    pub fn SetupDiGetDeviceInterfaceDetailW( DeviceInfoSet: HDEVINFO, DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA, DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W, DeviceInterfaceDetailDataSize: DWORD, RequiredSize: PDWORD, DeviceInfoData: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupDiOpenDeviceInterfaceRegKey( arg1: HDEVINFO, arg2: PSP_DEVICE_INTERFACE_DATA, arg3: DWORD, arg4: REGSAM, ) -> HKEY; // desk-rust/src/types_i386.rs
    pub fn WaitForMultipleObjects( nCount: DWORD, lpHandles: *const HANDLE, bWaitAll: BOOL, dwMilliseconds: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObjectEx(arg1: HANDLE, arg2: DWORD, arg3: BOOL) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WriteFileEx( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPOVERLAPPED, arg5: LPOVERLAPPED_COMPLETION_ROUTINE, ) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::wdmaud_drv::legacy::KernelHandle;
pub use crate::wdmaud_drv::mmixer::WdmAudCleanupByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudCloseSoundDeviceByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudCommitWaveBufferByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudGetCapabilitiesByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudGetDeviceInterfaceStringByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudGetNumWdmDevsByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudGetVolumeByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudGetWavePositionByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudOpenSoundDeviceByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudQueryMixerInfoByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudResetStreamByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudSetMixerDeviceFormatByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudSetVolumeByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudSetWaveDeviceFormatByMMixer;
pub use crate::wdmaud_drv::mmixer::WdmAudSetWaveStateByMMixer;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const IOCTL_CLOSE_WDMAUD: i32 = 0x3D0004; // clang, legacy.c
pub const IOCTL_GETCAPABILITIES: i32 = 0x3D001C; // clang, legacy.c
pub const IOCTL_GETCONTROLDETAILS: i32 = 0x3D0038; // clang, legacy.c
pub const IOCTL_GETFRAMESIZE: i32 = 0x3D0028; // clang, legacy.c
pub const IOCTL_GETLINECONTROLS: i32 = 0x3D0030; // clang, legacy.c
pub const IOCTL_GETLINEINFO: i32 = 0x3D002C; // clang, legacy.c
pub const IOCTL_GETNUMDEVS_TYPE: i32 = 0x3D0008; // clang, legacy.c
pub const IOCTL_GETPOS: i32 = 0x3D0024; // clang, legacy.c
pub const IOCTL_GET_MIXER_EVENT: i32 = 0x3D0040; // clang, legacy.c
pub const IOCTL_KS_READ_STREAM: i32 = 0x2F4017; // clang, mmixer.c
pub const IOCTL_KS_WRITE_STREAM: i32 = 0x2F8013; // clang, mmixer.c
pub const IOCTL_OPEN_WDMAUD: i32 = 0x3D0000; // clang, legacy.c
pub const IOCTL_QUERYDEVICEINTERFACESTRING: i32 = 0x3D003C; // clang, legacy.c
pub const IOCTL_RESET_STREAM: i32 = 0x3D0044; // clang, legacy.c
pub const IOCTL_SETCONTROLDETAILS: i32 = 0x3D0034; // clang, legacy.c
pub const IOCTL_SETDEVICE_STATE: i32 = 0x3D000C; // clang, legacy.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn CleanupEntrypointMutexes(); // wdmaud.c
    pub fn GetSoundDeviceFromInstance(
        SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
        SoundDevice: *mut PSOUND_DEVICE,
    ) -> MMRESULT; // mmixer.c
    pub fn GetSoundDeviceIdentifier(SoundDevice: PSOUND_DEVICE, Identifier: *mut PVOID) -> MMRESULT; // legacy.c
    pub fn GetSoundDeviceInstanceHandle(
        SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
        Handle: *mut PVOID,
    ) -> MMRESULT; // mixer.c
    pub fn GetSoundDeviceType(SoundDevice: PSOUND_DEVICE, DeviceType: PMMDEVICE_TYPE) -> MMRESULT; // mmixer.c
    pub fn InitEntrypointMutexes() -> MMRESULT; // wdmaud.c
    pub fn ListSoundDevice(
        DeviceType: MMDEVICE_TYPE,
        Identifier: PVOID,
        SoundDevice: *mut PSOUND_DEVICE,
    ) -> MMRESULT; // wdmaud.c
    pub fn MMixerGetCapabilities(
        MixerContext_0: PMIXER_CONTEXT,
        MixerIndex: ULONG,
        MixerCaps: LPMIXERCAPSW,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerGetControlDetails(
        MixerContext_0: PMIXER_CONTEXT,
        MixerHandle: HANDLE,
        MixerId: ULONG,
        Flags: ULONG,
        MixerControlDetails: LPMIXERCONTROLDETAILS,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerGetCount(MixerContext_0: PMIXER_CONTEXT) -> ULONG; // mmixer.c
    pub fn MMixerGetLineControls(
        MixerContext_0: PMIXER_CONTEXT,
        MixerHandle: HANDLE,
        MixerId: ULONG,
        Flags: ULONG,
        MixerLineControls: LPMIXERLINECONTROLSW,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerGetLineInfo(
        MixerContext_0: PMIXER_CONTEXT,
        MixerHandle: HANDLE,
        MixerId: ULONG,
        Flags: ULONG,
        MixerLine: LPMIXERLINEW,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerGetWaveInCount(MixerContext_0: PMIXER_CONTEXT) -> ULONG; // mmixer.c
    pub fn MMixerGetWaveOutCount(MixerContext_0: PMIXER_CONTEXT) -> ULONG; // mmixer.c
    pub fn MMixerGetWavePosition(
        MixerContext_0: PMIXER_CONTEXT,
        PinHandle: HANDLE,
        Position: PDWORD,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerInitialize(
        MixerContext_0: PMIXER_CONTEXT,
        EnumFunction: PMIXER_ENUM,
        EnumContext: PVOID,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerOpen(
        MixerContext_0: PMIXER_CONTEXT,
        MixerId: ULONG,
        MixerEventContext: PVOID,
        MixerEventRoutine: PMIXER_EVENT,
        MixerHandle: PHANDLE,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerOpenWave(
        MixerContext_0: PMIXER_CONTEXT,
        DeviceIndex: ULONG,
        bWaveIn: ULONG,
        WaveFormat: LPWAVEFORMATEX,
        CreateCallback: PIN_CREATE_CALLBACK,
        Context: PVOID,
        PinHandle: PHANDLE,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerSetControlDetails(
        MixerContext_0: PMIXER_CONTEXT,
        MixerHandle: HANDLE,
        MixerId: ULONG,
        Flags: ULONG,
        MixerControlDetails: LPMIXERCONTROLDETAILS,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerSetMidiStatus(
        MixerContext_0: PMIXER_CONTEXT,
        PinHandle: HANDLE,
        State: KSSTATE,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerSetWaveResetState(
        MixerContext_0: PMIXER_CONTEXT,
        PinHandle: HANDLE,
        bBegin: ULONG,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerSetWaveStatus(
        MixerContext_0: PMIXER_CONTEXT,
        PinHandle: HANDLE,
        State: KSSTATE,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerWaveInCapabilities(
        MixerContext_0: PMIXER_CONTEXT,
        DeviceIndex: ULONG,
        Caps: LPWAVEINCAPSW,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn MMixerWaveOutCapabilities(
        MixerContext_0: PMIXER_CONTEXT,
        DeviceIndex: ULONG,
        Caps: LPWAVEOUTCAPSW,
    ) -> MIXER_STATUS; // mmixer.c
    pub fn SetSoundDeviceFunctionTable(
        SoundDevice: PSOUND_DEVICE,
        FunctionTable: PMMFUNCTION_TABLE,
    ) -> MMRESULT; // wdmaud.c
    pub fn SyncOverlappedDeviceIoControl(
        SoundDeviceInstance: HANDLE,
        IoControlCode: DWORD,
        InBuffer: LPVOID,
        InBufferSize: DWORD,
        OutBuffer: LPVOID,
        OutBufferSize: DWORD,
        BytesTransferred: LPDWORD,
    ) -> MMRESULT; // legacy.c
    pub fn TranslateInternalMmResult(Result: MMRESULT) -> MMRESULT; // wdmaud.c
    pub fn UnlistAllSoundDevices(); // wdmaud.c
    pub fn src_delete(state: *mut SRC_STATE) -> *mut SRC_STATE; // mixer.c
    pub fn src_float_to_int_array(
        in_0: *const ::core::ffi::c_float,
        out: *mut ::core::ffi::c_int,
        len: ::core::ffi::c_int,
    ); // mixer.c
    pub fn src_float_to_short_array(
        in_0: *const ::core::ffi::c_float,
        out: *mut ::core::ffi::c_short,
        len: ::core::ffi::c_int,
    ); // mixer.c
    pub fn src_int_to_float_array(
        in_0: *const ::core::ffi::c_int,
        out: *mut ::core::ffi::c_float,
        len: ::core::ffi::c_int,
    ); // mixer.c
    pub fn src_new(
        converter_type: ::core::ffi::c_int,
        channels: ::core::ffi::c_int,
        error: *mut ::core::ffi::c_int,
    ) -> *mut SRC_STATE; // mixer.c
    pub fn src_process(state: *mut SRC_STATE, data: *mut SRC_DATA) -> ::core::ffi::c_int; // mixer.c
    pub fn src_short_to_float_array(
        in_0: *const ::core::ffi::c_short,
        out: *mut ::core::ffi::c_float,
        len: ::core::ffi::c_int,
    ); // mixer.c
}
extern "system" {
    pub fn DriverCallback(
        dwCallBack: DWORD_PTR,
        uFlags: DWORD,
        hDev: HDRVR,
        wMsg: DWORD,
        dwUser: DWORD_PTR,
        dwParam1: DWORD_PTR,
        dwParam2: DWORD_PTR,
    ) -> BOOL; // mmixer.c
}
pub use crate::prelude::CreateEventW as CreateEvent; // sdk alias
pub use crate::prelude::SetupDiGetClassDevsW as SetupDiGetClassDevs; // sdk alias
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
pub use crate::{MMSUCCESS};
// ==== end SDK header macros ====
