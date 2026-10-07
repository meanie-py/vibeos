//! diskpart-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1/UNIMPLEMENTED — no-op MACROS, never out-of-line
// fns (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, DPRINT1, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// crt/errno.h: `#define errno (*_errno())`, `_CRTIMP extern int *__cdecl _errno(void);`.
// A read `errno == ERANGE` is `*_errno() == ERANGE`, a write `errno = 0` is `*_errno() = 0`.
// crt/stdio.h: `#define stdin (&_iob[0])`; `__iob_func()` returns `_iob` (the CRT's own accessor
// for the same array), so `&_iob[0]` is `__iob_func().add(0)`.
extern "C" {
    pub fn _errno() -> *mut c_int;
    pub fn __iob_func() -> *mut FILE;
}
/// crt/stdio.h: `#define stdin (&_iob[0])`.
#[inline(always)]
pub unsafe fn stdin() -> *mut FILE {
    __iob_func()
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
pub use win32macros_rust::{ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, RGB, RTL_CONSTANT_STRING, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::diskpart::partlist::AlignDown;
pub use crate::diskpart::mountmgr::AssignDriveLetter;
pub use crate::diskpart::mountmgr::AssignNextDriveLetter;
pub use crate::diskpart::misc::CreateGUID;
pub use crate::diskpart::partlist::CreatePartitionList;
pub use crate::diskpart::misc::CreateSignature;
pub use crate::diskpart::partlist::CreateVolumeList;
pub use crate::diskpart::partlist::CurrentDisk;
pub use crate::diskpart::partlist::CurrentPartition;
pub use crate::diskpart::partlist::CurrentVolume;
pub use crate::diskpart::mountmgr::DeleteDriveLetter;
pub use crate::diskpart::partlist::DestroyPartitionList;
pub use crate::diskpart::partlist::DestroyVolumeList;
pub use crate::diskpart::partlist::DiskListHead;
pub use crate::diskpart::partlist::DismountVolume;
pub use crate::diskpart::mountmgr::GetAutomountState;
pub use crate::diskpart::partlist::GetNextUnpartitionedEntry;
pub use crate::diskpart::partlist::GetPrevUnpartitionedEntry;
pub use crate::diskpart::partlist::GetPrimaryPartitionCount;
pub use crate::diskpart::partlist::GetVolumeFromPartition;
pub use crate::diskpart::misc::HasPrefix;
pub use crate::diskpart::interpreter::InterpretMain;
pub use crate::diskpart::interpreter::InterpretScript;
pub use crate::diskpart::list::ListDisk;
pub use crate::diskpart::list::ListPartition;
pub use crate::diskpart::list::ListVolume;
pub use crate::diskpart::guid::PARTITION_MSFT_RESERVED_GUID;
pub use crate::diskpart::guid::PARTITION_SYSTEM_GUID;
pub use crate::diskpart::misc::PrintBusType;
pub use crate::diskpart::list::PrintDisk;
pub use crate::diskpart::misc::PrintGUID;
pub use crate::diskpart::list::PrintVolume;
pub use crate::diskpart::partlist::ReadLayoutBuffer;
pub use crate::diskpart::partlist::RemoveVolume;
pub use crate::diskpart::misc::RoundingDivide;
pub use crate::diskpart::partlist::ScanForUnpartitionedGptDiskSpace;
pub use crate::diskpart::partlist::ScanForUnpartitionedMbrDiskSpace;
pub use crate::diskpart::mountmgr::ScrubAutomount;
pub use crate::diskpart::select::SelectDisk;
pub use crate::diskpart::select::SelectPartition;
pub use crate::diskpart::select::SelectVolume;
pub use crate::diskpart::mountmgr::SetAutomountState;
pub use crate::diskpart::misc::StringToGUID;
pub use crate::diskpart::uniqueid::UniqueIdDisk;
pub use crate::diskpart::partlist::UpdateGptDiskLayout;
pub use crate::diskpart::partlist::UpdateMbrDiskLayout;
pub use crate::diskpart::partlist::VolumeListHead;
pub use crate::diskpart::partlist::WriteGptPartitions;
pub use crate::diskpart::partlist::WriteMbrPartitions;
pub use crate::diskpart::interpreter::cmds;
pub use crate::diskpart::remove::remove_main;
pub use crate::diskpart::rescan::rescan_main;
pub use crate::diskpart::setid::setid_main;
// ==== end cross-TU definitions ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ANYSIZE_ARRAY: u32 = 1; // ntdll-sys/src/bindgen_amd64.rs
pub const AnsiText: _CON_STREAM_MODE = _CON_STREAM_MODE_AnsiText; // island enum
pub const BusTypeMax: _STORAGE_BUS_TYPE = _STORAGE_BUS_TYPE_BusTypeMax; // island enum
pub const CmResourceTypeDeviceSpecific: u32 = 5; // bootvid-rust/src/pc.rs
pub const DONE: CALLBACKCOMMAND = CALLBACKCOMMAND_DONE; // island enum
pub const Disabled: _MOUNTMGR_AUTO_MOUNT_STATE = _MOUNTMGR_AUTO_MOUNT_STATE_Disabled; // island enum
pub const ERANGE: u32 = 34; // msvcrt-sys/src/bindgen_amd64.rs
pub const ERROR_MORE_DATA: u32 = 234; // msvcrt-sys/src/bindgen_amd64.rs
pub const ERROR_SUCCESS: u32 = 0; // msvcrt-sys/src/bindgen_amd64.rs
pub const ERROR_UNRECOGNIZED_VOLUME: u32 = 1005; // msvcrt-sys/src/bindgen_amd64.rs
pub const EXIT_EXIT: _EXIT_CODE = _EXIT_CODE_EXIT_EXIT; // island enum
pub const EXIT_SUCCESS: _EXIT_CODE = _EXIT_CODE_EXIT_SUCCESS; // island enum
pub const EXIT_SYNTAX: _EXIT_CODE = _EXIT_CODE_EXIT_SYNTAX; // island enum
pub const Enabled: _MOUNTMGR_AUTO_MOUNT_STATE = _MOUNTMGR_AUTO_MOUNT_STATE_Enabled; // island enum
pub const FALSE: BOOL = 0; // advapi32-rust/src/prelude.rs
pub const FILE_DIRECTORY_FILE: u32 = 1; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_OPEN_FOR_BACKUP_INTENT: u32 = 16384; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_READ_ATTRIBUTES: u32 = 128; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_READ_DATA: u32 = 1; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_REMOVABLE_MEDIA: u32 = 1; // ntdll-sys/src/bindgen_amd64.rs
pub const FILE_SHARE_READ: u32 = 1; // msvcrt-sys/src/bindgen_amd64.rs
pub const FILE_SHARE_WRITE: u32 = 2; // msvcrt-sys/src/bindgen_amd64.rs
pub const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 32; // ntdll-sys/src/bindgen_amd64.rs
pub const FMIFS_HARDDISK: FMIFS_MEDIA_FLAG = FMIFS_MEDIA_FLAG_FMIFS_HARDDISK; // island enum
pub const FORMAT_MESSAGE_FROM_HMODULE: u32 = 2048; // desk-rust/src/types_amd64.rs
pub const FSCTL_DISMOUNT_VOLUME: ULONG = 0x00090020; // ntkrnl-sys/src/constants.rs
pub const FSCTL_LOCK_VOLUME: ULONG = 0x00090018; // ntkrnl-sys/src/constants.rs
pub const FSCTL_UNLOCK_VOLUME: ULONG = 0x0009001c; // ntkrnl-sys/src/constants.rs
pub const GENERIC_READ: u32 = 2147483648; // msvcrt-sys/src/bindgen_amd64.rs
pub const GENERIC_WRITE: u32 = 1073741824; // msvcrt-sys/src/bindgen_amd64.rs
pub const GPT_ATTRIBUTE_PLATFORM_REQUIRED: u32 = 1; // cdrom-rust/src/types_amd64.rs
pub const HEAP_ZERO_MEMORY: u32 = 8; // msvcrt-sys/src/bindgen_amd64.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY; // dhcpcsvc-rust/src/prelude.rs
pub const INVALID_CP: UINT = 0xFFFFFFFF; // concfg-rust/src/font.rs
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000; // disk-rust/src/glue.rs
pub const IOCTL_MOUNTMGR_DELETE_POINTS: u32 = 0x006DC004; // partmgr-rust/src/glue.rs
pub const IOCTL_SCSI_GET_ADDRESS: u32 = 0x00041018; // classpnp-rust/src/glue.rs
pub const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400; // scsidisk-rust/src/lib.rs
pub const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x00560000; // partmgr-rust/src/glue.rs
pub const KEY_READ: DWORD = 0x20019; // shell32-rust/src/cfs_folder.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // msvcrt-sys/src/bindgen_amd64.rs
pub const MAX_PATH: u32 = 260; // msvcrt-sys/src/bindgen_amd64.rs
pub const OBJ_CASE_INSENSITIVE: u32 = 64; // ntdll-sys/src/bindgen_amd64.rs
pub const PARTITION_ENTRY_UNUSED: u32 = 0; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_EXTENDED: u32 = 5; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_FAT32: u32 = 11; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_FAT32_XINT13: u32 = 12; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_FAT_12: u32 = 1; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_FAT_16: u32 = 4; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_GPT: u32 = 238; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_HUGE: u32 = 6; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_IFS: u32 = 7; // cdrom-rust/src/types_amd64.rs
pub const PARTITION_STYLE_GPT: u32 = 1; // fstub-rust/src/disksup.rs
pub const PARTITION_STYLE_MBR: u32 = 0; // fstub-rust/src/disksup.rs
pub const PARTITION_STYLE_RAW: u32 = 2; // fstub-rust/src/disksup.rs
pub const PARTITION_XINT13: u32 = 14; // cdrom-rust/src/types_amd64.rs
pub const PROGRESS: CALLBACKCOMMAND = CALLBACKCOMMAND_PROGRESS; // island enum
pub const Preformatted: _FORMATSTATE = _FORMATSTATE_Preformatted; // island enum
pub const PropertyStandardQuery: _STORAGE_QUERY_TYPE = _STORAGE_QUERY_TYPE_PropertyStandardQuery; // island enum
pub const REG_FULL_RESOURCE_DESCRIPTOR: u32 = 9; // advapi32-sys/src/bindgen_amd64.rs
pub const REG_SZ: u32 = 1; // advapi32-sys/src/bindgen_amd64.rs
pub const RTL_QUERY_REGISTRY_DIRECT: u32 = 32; // ntdll-sys/src/bindgen_amd64.rs
pub const RTL_REGISTRY_ABSOLUTE: u32 = 0; // ntdll-sys/src/bindgen_amd64.rs
pub const RTL_REGISTRY_DEVICEMAP: u32 = 4; // ntdll-sys/src/bindgen_amd64.rs
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32; // rtl-strings/src/core_rs/env.rs
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as NTSTATUS; // kernel32-rust/src/console/history.rs
pub const STATUS_SUCCESS: i32 = 0; // network/src/lib.rs
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS; // msfs-rust/src/glue.rs
pub const STD_ERROR_HANDLE: u32 = 4294967284; // msvcrt-sys/src/bindgen_amd64.rs
pub const STD_INPUT_HANDLE: u32 = 4294967286; // msvcrt-sys/src/bindgen_amd64.rs
pub const STD_OUTPUT_HANDLE: u32 = 4294967285; // msvcrt-sys/src/bindgen_amd64.rs
pub const SYNCHRONIZE: u32 = 1048576; // ntdll-sys/src/bindgen_amd64.rs
pub const StorageDeviceProperty: _STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceProperty; // island enum
pub const TRUE: BOOL = 1; // advapi32-rust/src/prelude.rs
pub const Unformatted: _FORMATSTATE = _FORMATSTATE_Unformatted; // island enum
pub const UnknownFormat: _FORMATSTATE = _FORMATSTATE_UnknownFormat; // island enum
pub const VOLUME_TYPE_CDROM: _VOLUME_TYPE = _VOLUME_TYPE_VOLUME_TYPE_CDROM; // island enum
pub const VOLUME_TYPE_PARTITION: _VOLUME_TYPE = _VOLUME_TYPE_VOLUME_TYPE_PARTITION; // island enum
pub const VOLUME_TYPE_REMOVABLE: _VOLUME_TYPE = _VOLUME_TYPE_VOLUME_TYPE_REMOVABLE; // island enum
pub const VOLUME_TYPE_UNKNOWN: _VOLUME_TYPE = _VOLUME_TYPE_VOLUME_TYPE_UNKNOWN; // island enum
extern "C" {
    pub fn ConPrintf(Stream: *mut CON_STREAM, szStr: *const u16, ...) -> i32; // conutils-rust/src/conutils/outstream.rs
    pub fn ConPuts(Stream: *mut CON_STREAM, szText: *const u16) -> i32; // conutils-rust/src/conutils/screen.rs
    pub fn ConResPrintf(Stream: *mut CON_STREAM, uID: u32, ...) -> i32; // conutils-rust/src/conutils/outstream.rs
    pub fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32; // arm64input-cargo/src/main.rs
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // rtl-actctx/src/shim.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // kernel32-rust/src/client/vdm.rs
    pub fn _wcstoi64(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> i64; // cnt-rust/src/cabi/generated.rs
    pub fn _wcstoui64(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> u64; // cnt-rust/src/cabi/generated.rs
    pub fn _wtoi(s: *const u16) -> c_int; // advapi32-rust/src/prelude.rs
    pub fn fclose(stream: *mut FILE) -> c_int; // dnsrslvr-rust/src/prelude.rs
    pub fn towupper(c: c_int) -> c_int; // dnsapi-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // acpi-rust/src/buspdo.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // kernel32-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // concfg-rust/src/font.rs
}
extern "system" {
    pub fn DeviceIoControl( hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: LPVOID, nInBufferSize: DWORD, lpOutBuffer: LPVOID, nOutBufferSize: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED, ) -> BOOL; // iphlpapi-rust/src/prelude.rs
    pub fn FindFirstVolumeW(arg1: LPWSTR, arg2: DWORD) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn FindNextVolumeW(arg1: HANDLE, arg2: LPWSTR, arg3: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindVolumeClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: LPDWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetVolumeInformationW( arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD, arg4: PDWORD, arg5: PDWORD, arg6: PDWORD, arg7: LPWSTR, arg8: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetVolumePathNamesForVolumeNameW( arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD, arg4: PDWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn K32LoadStringW(hInstance: HINSTANCE, uID: u32, lpBuffer: PWSTR, cchBufferMax: i32) -> i32; // conutils-rust/src/conutils/pager.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn NtClose(Handle: HANDLE) -> NTSTATUS; // inflib-rust/src/infrosgen.rs
    pub fn NtDeviceIoControlFile( FileHandle: HANDLE, Event: HANDLE, ApcRoutine: PIO_APC_ROUTINE, ApcContext: PVOID, IoStatusBlock: PIO_STATUS_BLOCK, IoControlCode: ULONG, InputBuffer: PVOID, InputBufferLength: ULONG, OutputBuffer: PVOID, OutputBufferLength: ULONG, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn NtFsControlFile( FileHandle: HANDLE, Event: HANDLE, ApcRoutine: PIO_APC_ROUTINE, ApcContext: PVOID, IoStatusBlock: PIO_STATUS_BLOCK, FsControlCode: ULONG, InputBuffer: PVOID, InputBufferLength: ULONG, OutputBuffer: PVOID, OutputBufferLength: ULONG, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn NtOpenFile( FileHandle: PHANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES, IoStatusBlock: PIO_STATUS_BLOCK, ShareAccess: ULONG, OpenOptions: ULONG, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn NtQuerySystemInformation( SystemInformationClass: SYSTEM_INFORMATION_CLASS, SystemInformation: PVOID, SystemInformationLength: ULONG, ReturnLength: PULONG, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn NtQueryVolumeInformationFile( FileHandle: HANDLE, IoStatusBlock: PIO_STATUS_BLOCK, FsInformation: PVOID, Length: ULONG, FsInformationClass: FS_INFORMATION_CLASS, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn NtReadFile( FileHandle: HANDLE, Event: HANDLE, ApcRoutine: PIO_APC_ROUTINE, ApcContext: PVOID, IoStatusBlock: PIO_STATUS_BLOCK, Buffer: PVOID, Length: ULONG, ByteOffset: PLARGE_INTEGER, Key: PULONG, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn NtWriteFile( FileHandle: HANDLE, Event: HANDLE, ApcRoutine: PIO_APC_ROUTINE, ApcContext: PVOID, IoStatusBlock: PIO_STATUS_BLOCK, Buffer: PVOID, Length: ULONG, ByteOffset: PLARGE_INTEGER, Key: PULONG, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn QueryAvailableFileSystemFormat( Index: DWORD, FileSystem: PWCHAR, Major: *mut UCHAR, Minor: *mut UCHAR, LatestVersion: *mut BOOLEAN, ) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn QueryDosDeviceW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RtlDosPathNameToNtPathName_U( DosPathName: PCWSTR, NtPathName: PUNICODE_STRING, NtFileNamePart: *mut PCWSTR, DirectoryInfo: PRTL_RELATIVE_NAME_U, ) -> BOOLEAN; // ntdll-sys/src/bindgen_amd64.rs
    pub fn RtlFreeUnicodeString(UnicodeString: PUNICODE_STRING); // gdi32-rust/src/objects/dc.rs
    pub fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR); // gdi32-rust/src/objects/dc.rs
    pub fn RtlQueryRegistryValues( RelativeTo: ULONG, Path: PCWSTR, QueryTable: PRTL_QUERY_REGISTRY_TABLE, Context: PVOID, Environment: PVOID, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn RtlTimeToTimeFields(Time: PLARGE_INTEGER, TimeFields: PTIME_FIELDS); // ntdll-sys/src/bindgen_amd64.rs
    pub fn RtlUnicodeStringToInteger( String: PCUNICODE_STRING, Base: ULONG, Value: PULONG, ) -> NTSTATUS; // ntdll-sys/src/bindgen_amd64.rs
    pub fn SetConsoleTitleW(lpConsoleTitle: LPCWSTR) -> BOOL; // winsrv-sys/src/bindgen_amd64.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const IDS_ACTIVE_ALREADY: i32 = 0x3EA; // clang, active.c
pub const IDS_ACTIVE_FAIL: i32 = 0x3E8; // clang, active.c
pub const IDS_ACTIVE_NO_MBR: i32 = 0x3EB; // clang, active.c
pub const IDS_ACTIVE_SUCCESS: i32 = 0x3E9; // clang, active.c
pub const IDS_APP_CURR_COMPUTER: i32 = 3; // clang, diskpart.c
pub const IDS_APP_HEADER: i32 = 0; // clang, diskpart.c
pub const IDS_APP_LEAVING: i32 = 4; // clang, diskpart.c
pub const IDS_APP_LICENSE: i32 = 2; // clang, diskpart.c
pub const IDS_APP_PROMPT: i32 = 5; // clang, interpreter.c
pub const IDS_APP_USAGE: i32 = 1; // clang, diskpart.c
pub const IDS_ASSIGN_ALREADY_ASSIGNED: i32 = 0x3FE; // clang, assign.c
pub const IDS_ASSIGN_FAIL: i32 = 0x3FC; // clang, assign.c
pub const IDS_ASSIGN_INVALID_LETTER: i32 = 0x3FF; // clang, assign.c
pub const IDS_ASSIGN_NO_MORE_LETTER: i32 = 0x400; // clang, assign.c
pub const IDS_ASSIGN_SUCCESS: i32 = 0x3FD; // clang, assign.c
pub const IDS_ASSIGN_SYSTEM_VOLUME: i32 = 0x401; // clang, assign.c
pub const IDS_AUTOMOUNT_DISABLED: i32 = 0x3F3; // clang, automount.c
pub const IDS_AUTOMOUNT_ENABLED: i32 = 0x3F2; // clang, automount.c
pub const IDS_AUTOMOUNT_SCRUBBED: i32 = 0x3F4; // clang, automount.c
pub const IDS_BUSTYPE_OTHER: i32 = 0x1401; // clang, misc.c
pub const IDS_BUSTYPE_UNKNOWN: i32 = 0x13EC; // clang, misc.c
pub const IDS_CLEAN_FAIL: i32 = 0x406; // clang, clean.c
pub const IDS_CLEAN_SUCCESS: i32 = 0x407; // clang, clean.c
pub const IDS_CLEAN_SYSTEM: i32 = 0x408; // clang, clean.c
pub const IDS_CONVERT_GPT_ALREADY: i32 = 0x410; // clang, convert.c
pub const IDS_CONVERT_GPT_NOT_EMPTY: i32 = 0x411; // clang, convert.c
pub const IDS_CONVERT_GPT_SUCCESS: i32 = 0x412; // clang, convert.c
pub const IDS_CONVERT_MBR_ALREADY: i32 = 0x413; // clang, convert.c
pub const IDS_CONVERT_MBR_NOT_EMPTY: i32 = 0x414; // clang, convert.c
pub const IDS_CONVERT_MBR_SUCCESS: i32 = 0x415; // clang, convert.c
pub const IDS_CREATE_PARTITION_FAIL: i32 = 0x41A; // clang, create.c
pub const IDS_CREATE_PARTITION_INVALID_STYLE: i32 = 0x41C; // clang, create.c
pub const IDS_CREATE_PARTITION_SUCCESS: i32 = 0x41B; // clang, create.c
pub const IDS_DELETE_PARTITION_FAIL: i32 = 0x42E; // clang, delete.c
pub const IDS_DELETE_PARTITION_SUCCESS: i32 = 0x42F; // clang, delete.c
pub const IDS_DELETE_PARTITION_SYSTEM: i32 = 0x430; // clang, delete.c
pub const IDS_DETAIL_DISK_DESCRIPTION: i32 = 0x452; // clang, detail.c
pub const IDS_DETAIL_DISK_ID: i32 = 0x453; // clang, detail.c
pub const IDS_DETAIL_DISK_STATUS: i32 = 0x455; // clang, detail.c
pub const IDS_DETAIL_DISK_TYPE: i32 = 0x454; // clang, detail.c
pub const IDS_DETAIL_INFO_BOOT_DSK: i32 = 0x45C; // clang, detail.c
pub const IDS_DETAIL_INFO_LUN_ID: i32 = 0x458; // clang, detail.c
pub const IDS_DETAIL_INFO_PATH: i32 = 0x456; // clang, detail.c
pub const IDS_DETAIL_INFO_TARGET: i32 = 0x457; // clang, detail.c
pub const IDS_DETAIL_NO_DISKS: i32 = 0x471; // clang, detail.c
pub const IDS_DETAIL_NO_VOLUME: i32 = 0x472; // clang, detail.c
pub const IDS_DETAIL_PARTITION_ACTIVE: i32 = 0x46D; // clang, detail.c
pub const IDS_DETAIL_PARTITION_ATTRIBUTE: i32 = 0x470; // clang, detail.c
pub const IDS_DETAIL_PARTITION_HIDDEN: i32 = 0x46C; // clang, detail.c
pub const IDS_DETAIL_PARTITION_NUMBER: i32 = 0x46A; // clang, detail.c
pub const IDS_DETAIL_PARTITION_OFFSET: i32 = 0x46E; // clang, detail.c
pub const IDS_DETAIL_PARTITION_REQUIRED: i32 = 0x46F; // clang, detail.c
pub const IDS_DETAIL_PARTITION_TYPE: i32 = 0x46B; // clang, detail.c
pub const IDS_ERROR_INVALID_ARGS: i32 = 0x138A; // clang, assign.c
pub const IDS_ERROR_MSG_BAD_ARG: i32 = 0x1389; // clang, diskpart.c
pub const IDS_ERROR_MSG_NO_SCRIPT: i32 = 0x1388; // clang, diskpart.c
pub const IDS_FILESYSTEMS_CLUSTERSIZE: i32 = 0x495; // clang, filesystems.c
pub const IDS_FILESYSTEMS_CURRENT: i32 = 0x492; // clang, filesystems.c
pub const IDS_FILESYSTEMS_DEFAULT: i32 = 0x497; // clang, filesystems.c
pub const IDS_FILESYSTEMS_FORMATTING: i32 = 0x493; // clang, filesystems.c
pub const IDS_FILESYSTEMS_SERIAL_NUMBER: i32 = 0x496; // clang, filesystems.c
pub const IDS_FILESYSTEMS_TYPE: i32 = 0x494; // clang, filesystems.c
pub const IDS_FORMAT_FAIL: i32 = 0x49C; // clang, format.c
pub const IDS_FORMAT_PROGRESS: i32 = 0x49E; // clang, format.c
pub const IDS_FORMAT_SUCCESS: i32 = 0x49D; // clang, format.c
pub const IDS_GPT_FAIL: i32 = 0x4A6; // clang, gpt.c
pub const IDS_GPT_SUCCESS: i32 = 0x4A7; // clang, gpt.c
pub const IDS_HELP_ACTIVE: i32 = 0x3A; // clang, interpreter.c
pub const IDS_HELP_ASSIGN: i32 = 0x3C; // clang, interpreter.c
pub const IDS_HELP_AUTOMOUNT: i32 = 0x3F; // clang, interpreter.c
pub const IDS_HELP_CLEAN: i32 = 0x41; // clang, interpreter.c
pub const IDS_HELP_CONVERT: i32 = 0x43; // clang, interpreter.c
pub const IDS_HELP_CONVERT_GPT: i32 = 0x44; // clang, interpreter.c
pub const IDS_HELP_CONVERT_MBR: i32 = 0x45; // clang, interpreter.c
pub const IDS_HELP_CREATE: i32 = 0x46; // clang, interpreter.c
pub const IDS_HELP_CREATE_PARTITION: i32 = 0x47; // clang, interpreter.c
pub const IDS_HELP_CREATE_PARTITION_EFI: i32 = 0x48; // clang, interpreter.c
pub const IDS_HELP_CREATE_PARTITION_EXTENDED: i32 = 0x49; // clang, interpreter.c
pub const IDS_HELP_CREATE_PARTITION_LOGICAL: i32 = 0x4A; // clang, interpreter.c
pub const IDS_HELP_CREATE_PARTITION_MSR: i32 = 0x4B; // clang, interpreter.c
pub const IDS_HELP_CREATE_PARTITION_PRIMARY: i32 = 0x4C; // clang, interpreter.c
pub const IDS_HELP_DELETE: i32 = 0x4F; // clang, interpreter.c
pub const IDS_HELP_DELETE_PARTITION: i32 = 0x51; // clang, interpreter.c
pub const IDS_HELP_DETAIL: i32 = 0x54; // clang, interpreter.c
pub const IDS_HELP_DETAIL_DISK: i32 = 0x55; // clang, interpreter.c
pub const IDS_HELP_DETAIL_PARTITION: i32 = 0x56; // clang, interpreter.c
pub const IDS_HELP_DETAIL_VOLUME: i32 = 0x57; // clang, interpreter.c
pub const IDS_HELP_EXIT: i32 = 0x58; // clang, interpreter.c
pub const IDS_HELP_FILESYSTEMS: i32 = 0x5B; // clang, interpreter.c
pub const IDS_HELP_FORMAT: i32 = 0x5C; // clang, interpreter.c
pub const IDS_HELP_FORMAT_STRING: i32 = 0x4B0; // clang, help.c
pub const IDS_HELP_GPT: i32 = 0x5D; // clang, interpreter.c
pub const IDS_HELP_HELP: i32 = 0x5E; // clang, interpreter.c
pub const IDS_HELP_INACTIVE: i32 = 0x60; // clang, interpreter.c
pub const IDS_HELP_LIST: i32 = 0x61; // clang, interpreter.c
pub const IDS_HELP_LIST_DISK: i32 = 0x62; // clang, interpreter.c
pub const IDS_HELP_LIST_PARTITION: i32 = 0x63; // clang, interpreter.c
pub const IDS_HELP_LIST_VOLUME: i32 = 0x64; // clang, interpreter.c
pub const IDS_HELP_REM: i32 = 0x6A; // clang, interpreter.c
pub const IDS_HELP_REMOVE: i32 = 0x6B; // clang, interpreter.c
pub const IDS_HELP_RESCAN: i32 = 0x6D; // clang, interpreter.c
pub const IDS_HELP_SELECT: i32 = 0x70; // clang, interpreter.c
pub const IDS_HELP_SELECT_DISK: i32 = 0x71; // clang, interpreter.c
pub const IDS_HELP_SELECT_PARTITION: i32 = 0x72; // clang, interpreter.c
pub const IDS_HELP_SELECT_VOLUME: i32 = 0x73; // clang, interpreter.c
pub const IDS_HELP_SETID: i32 = 0x75; // clang, interpreter.c
pub const IDS_HELP_UNIQUEID: i32 = 0x77; // clang, interpreter.c
pub const IDS_HELP_UNIQUEID_DISK: i32 = 0x78; // clang, interpreter.c
pub const IDS_INACTIVE_ALREADY: i32 = 0x4BC; // clang, inactive.c
pub const IDS_INACTIVE_FAIL: i32 = 0x4BA; // clang, inactive.c
pub const IDS_INACTIVE_NO_MBR: i32 = 0x4BD; // clang, inactive.c
pub const IDS_INACTIVE_SUCCESS: i32 = 0x4BB; // clang, inactive.c
pub const IDS_INFO_BOOT: i32 = 0x27; // clang, list.c
pub const IDS_INFO_SYSTEM: i32 = 0x28; // clang, list.c
pub const IDS_LIST_DISK_FORMAT: i32 = 0xCE6; // clang, list.c
pub const IDS_LIST_DISK_HEAD: i32 = 0xCE4; // clang, detail.c
pub const IDS_LIST_DISK_LINE: i32 = 0xCE5; // clang, detail.c
pub const IDS_LIST_PARTITION_FORMAT: i32 = 0xCE9; // clang, list.c
pub const IDS_LIST_PARTITION_HEAD: i32 = 0xCE7; // clang, list.c
pub const IDS_LIST_PARTITION_LINE: i32 = 0xCE8; // clang, list.c
pub const IDS_LIST_PARTITION_NONE: i32 = 0xCEB; // clang, list.c
pub const IDS_LIST_PARTITION_NO_DISK: i32 = 0xCEA; // clang, list.c
pub const IDS_LIST_VOLUME_FORMAT: i32 = 0xCEE; // clang, list.c
pub const IDS_LIST_VOLUME_HEAD: i32 = 0xCEC; // clang, detail.c
pub const IDS_LIST_VOLUME_LINE: i32 = 0xCED; // clang, detail.c
pub const IDS_PARTITION_TYPE_EXTENDED: i32 = 0x1450; // clang, list.c
pub const IDS_PARTITION_TYPE_LOGICAL: i32 = 0x1451; // clang, list.c
pub const IDS_PARTITION_TYPE_PRIMARY: i32 = 0x1452; // clang, list.c
pub const IDS_PARTITION_TYPE_RESERVED: i32 = 0x1453; // clang, list.c
pub const IDS_PARTITION_TYPE_SYSTEM: i32 = 0x1454; // clang, list.c
pub const IDS_PARTITION_TYPE_UNKNOWN: i32 = 0x1455; // clang, list.c
pub const IDS_PARTITION_TYPE_UNUSED: i32 = 0x1456; // clang, list.c
pub const IDS_REMOVE_FAIL: i32 = 0xFA0; // clang, remove.c
pub const IDS_REMOVE_NO_LETTER: i32 = 0xFA2; // clang, remove.c
pub const IDS_REMOVE_SUCCESS: i32 = 0xFA1; // clang, remove.c
pub const IDS_REMOVE_WRONG_LETTER: i32 = 0xFA3; // clang, remove.c
pub const IDS_RESCAN_END: i32 = 0x1005; // clang, rescan.c
pub const IDS_RESCAN_START: i32 = 0x1004; // clang, rescan.c
pub const IDS_SELECT_DISK: i32 = 0x1131; // clang, select.c
pub const IDS_SELECT_DISK_ENUM_FINISHED: i32 = 0x1134; // clang, select.c
pub const IDS_SELECT_DISK_ENUM_NO_START: i32 = 0x1133; // clang, select.c
pub const IDS_SELECT_DISK_INVALID: i32 = 0x1132; // clang, select.c
pub const IDS_SELECT_NO_DISK: i32 = 0x1130; // clang, active.c
pub const IDS_SELECT_NO_PARTITION: i32 = 0x1135; // clang, active.c
pub const IDS_SELECT_NO_VOLUME: i32 = 0x1139; // clang, assign.c
pub const IDS_SELECT_PARTITION: i32 = 0x1136; // clang, select.c
pub const IDS_SELECT_PARTITION_INVALID: i32 = 0x1138; // clang, select.c
pub const IDS_SELECT_PARTITION_NO_DISK: i32 = 0x1137; // clang, detail.c
pub const IDS_SELECT_VOLUME: i32 = 0x113A; // clang, select.c
pub const IDS_SELECT_VOLUME_INVALID: i32 = 0x113B; // clang, select.c
pub const IDS_SETID_FAIL: i32 = 0x1162; // clang, setid.c
pub const IDS_SETID_INVALID_FORMAT: i32 = 0x1164; // clang, setid.c
pub const IDS_SETID_INVALID_TYPE: i32 = 0x1165; // clang, setid.c
pub const IDS_SETID_SUCCESS: i32 = 0x1163; // clang, setid.c
pub const IDS_STATUS_NO: i32 = 0x20; // clang, detail.c
pub const IDS_STATUS_ONLINE: i32 = 0x24; // clang, detail.c
pub const IDS_STATUS_YES: i32 = 0x1F; // clang, detail.c
pub const IDS_UNIQUID_DISK_INVALID_STYLE: i32 = 0x1194; // clang, uniqueid.c
pub const IDS_UNIT_B: i32 = 0x1490; // clang, list.c
pub const IDS_UNIT_GB: i32 = 0x148D; // clang, list.c
pub const IDS_UNIT_KB: i32 = 0x148F; // clang, list.c
pub const IDS_UNIT_MB: i32 = 0x148E; // clang, list.c
pub const IDS_UNIT_TB: i32 = 0x148C; // clang, list.c
pub const IDS_VOLUME_TYPE_DVD: i32 = 0x1482; // clang, list.c
pub const IDS_VOLUME_TYPE_PARTITION: i32 = 0x1483; // clang, list.c
pub const IDS_VOLUME_TYPE_REMOVABLE: i32 = 0x1484; // clang, list.c
pub const IDS_VOLUME_TYPE_UNKNOWN: i32 = 0x1485; // clang, list.c
pub const IOCTL_DISK_CREATE_DISK: i32 = 0x7C058; // clang, convert.c
pub const IOCTL_DISK_GET_DRIVE_LAYOUT_EX: i32 = 0x70050; // clang, partlist.c
pub const IOCTL_DISK_SET_DRIVE_LAYOUT_EX: i32 = 0x7C054; // clang, partlist.c
pub const IOCTL_MOUNTMGR_CREATE_POINT: ULONG = 0x6DC000; // clang, mountmgr.c
pub const IOCTL_MOUNTMGR_QUERY_AUTO_MOUNT: ULONG = 0x6D003C; // clang, mountmgr.c
pub const IOCTL_MOUNTMGR_SCRUB_REGISTRY: ULONG = 0x6DC038; // clang, mountmgr.c
pub const IOCTL_MOUNTMGR_SET_AUTO_MOUNT: ULONG = 0x6DC040; // clang, mountmgr.c
pub const MAX_ARGS_COUNT: i32 = 0x100; // clang, interpreter.c
pub const MAX_STRING_SIZE: i32 = 0x400; // clang, diskpart.c
pub const MSG_COMMAND_ACTIVE: DWORD = 0x40002710; // clang, interpreter.c
pub const MSG_COMMAND_ASSIGN: DWORD = 0x40002712; // clang, interpreter.c
pub const MSG_COMMAND_AUTOMOUNT: DWORD = 0x40002715; // clang, interpreter.c
pub const MSG_COMMAND_CLEAN: DWORD = 0x40002717; // clang, interpreter.c
pub const MSG_COMMAND_CONVERT_GPT: DWORD = 0x4000271A; // clang, interpreter.c
pub const MSG_COMMAND_CONVERT_MBR: DWORD = 0x4000271B; // clang, interpreter.c
pub const MSG_COMMAND_CREATE_PARTITION_EFI: DWORD = 0x4000271C; // clang, interpreter.c
pub const MSG_COMMAND_CREATE_PARTITION_EXTENDED: DWORD = 0x4000271D; // clang, interpreter.c
pub const MSG_COMMAND_CREATE_PARTITION_LOGICAL: DWORD = 0x4000271E; // clang, interpreter.c
pub const MSG_COMMAND_CREATE_PARTITION_MSR: DWORD = 0x4000271F; // clang, interpreter.c
pub const MSG_COMMAND_CREATE_PARTITION_PRIMARY: DWORD = 0x40002720; // clang, interpreter.c
pub const MSG_COMMAND_DELETE_PARTITION: DWORD = 0x40002722; // clang, interpreter.c
pub const MSG_COMMAND_DETAIL_DISK: DWORD = 0x40002724; // clang, interpreter.c
pub const MSG_COMMAND_DETAIL_PARTITION: DWORD = 0x40002725; // clang, interpreter.c
pub const MSG_COMMAND_DETAIL_VOLUME: DWORD = 0x40002726; // clang, interpreter.c
pub const MSG_COMMAND_EXIT: DWORD = 0x40002728; // clang, interpreter.c
pub const MSG_COMMAND_FILESYSTEMS: DWORD = 0x4000272B; // clang, interpreter.c
pub const MSG_COMMAND_FORMAT: DWORD = 0x4000272C; // clang, interpreter.c
pub const MSG_COMMAND_GPT: DWORD = 0x4000272D; // clang, interpreter.c
pub const MSG_COMMAND_HELP: DWORD = 0x4000272E; // clang, interpreter.c
pub const MSG_COMMAND_INACTIVE: DWORD = 0x40002730; // clang, interpreter.c
pub const MSG_COMMAND_LIST_DISK: DWORD = 0x40002731; // clang, interpreter.c
pub const MSG_COMMAND_LIST_PARTITION: DWORD = 0x40002732; // clang, interpreter.c
pub const MSG_COMMAND_LIST_VOLUME: DWORD = 0x40002733; // clang, interpreter.c
pub const MSG_COMMAND_REM: DWORD = 0x40002739; // clang, interpreter.c
pub const MSG_COMMAND_REMOVE: DWORD = 0x4000273A; // clang, interpreter.c
pub const MSG_COMMAND_RESCAN: DWORD = 0x4000273C; // clang, interpreter.c
pub const MSG_COMMAND_SELECT_DISK: DWORD = 0x4000273F; // clang, interpreter.c
pub const MSG_COMMAND_SELECT_PARTITION: DWORD = 0x40002740; // clang, interpreter.c
pub const MSG_COMMAND_SELECT_VOLUME: DWORD = 0x40002741; // clang, interpreter.c
pub const MSG_COMMAND_SETID: DWORD = 0x40002743; // clang, interpreter.c
pub const MSG_COMMAND_UNIQUEID_DISK: DWORD = 0x40002745; // clang, interpreter.c
pub const MSG_NONE: i32 = 0; // clang, help.c
pub const SIZE_10GB: u64 = 0x280000000; // clang, list.c
pub const SIZE_10KB: u64 = 0x2800; // clang, filesystems.c
pub const SIZE_10MB: u64 = 0xA00000; // clang, filesystems.c
pub const SIZE_10TB: u64 = 0xA0000000000; // clang, list.c
pub const SIZE_1GB: u64 = 0x40000000; // clang, list.c
pub const SIZE_1KB: u64 = 0x400; // clang, filesystems.c
pub const SIZE_1MB: u64 = 0x100000; // clang, filesystems.c
pub const SIZE_1TB: u64 = 0x10000000000; // clang, list.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// EXIT_CODE: c2rust, active.c
pub type EXIT_CODE = _EXIT_CODE;
// STORAGE_BUS_TYPE: c2rust, misc.c
pub type STORAGE_BUS_TYPE = _STORAGE_BUS_TYPE;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wsprintfW(_: LPWSTR, _: LPCWSTR, ...) -> ::core::ffi::c_int; // filesystems.c
}
extern "system" {
    pub fn FreeLibrary(_: HMODULE) -> BOOL; // format.c
    pub fn GetLastError() -> DWORD; // partlist.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // detail.c
    pub fn GetProcAddress(_: HINSTANCE, _: LPCSTR) -> FARPROC; // format.c
    pub fn GetProcessHeap() -> HANDLE; // mountmgr.c
    pub fn GetStdHandle(_: DWORD) -> HANDLE; // diskpart.c
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // format.c
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // format.c
    pub fn Sleep(_: DWORD); // diskpart.c
    pub fn SystemFunction036(_: PVOID, _: ULONG) -> BOOLEAN; // misc.c
}
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::SystemFunction036 as RtlGenRandom; // sdk alias
pub use crate::prelude::wsprintfW as wsprintf; // sdk alias
// ==== end port externs ====

/// conutils' stream object (sdk/lib/conutils/stream.h). diskpart only passes pointers to
/// it; the definition belongs to conutils-rust, which the exe links but this crate does
/// not depend on, so it is opaque here.
#[repr(C)]
pub struct CON_STREAM {
    _opaque: [u8; 0],
}
pub type PCON_STREAM = *mut CON_STREAM;

extern "C" {
    // conutils-rust's #[no_mangle] instances (stream.h: `extern CON_STREAM csStdOut;`).
    pub static mut csStdIn: CON_STREAM;
    pub static mut csStdOut: CON_STREAM;
    pub static mut csStdErr: CON_STREAM;
}

/// stream.h: `#define StdOut (&csStdOut)`.
pub const StdOut: PCON_STREAM = core::ptr::addr_of_mut!(csStdOut);
/// stream.h: `#define StdErr (&csStdErr)`.
pub const StdErr: PCON_STREAM = core::ptr::addr_of_mut!(csStdErr);
/// stream.h: `#define StdIn (&csStdIn)`.
pub const StdIn: PCON_STREAM = core::ptr::addr_of_mut!(csStdIn);

/// stream.h: `#define ConInitStdStreams() ConInitStdStreamsAndMode(AnsiText, INVALID_CP)`,
/// the non-USE_CRT arm (diskpart does not define USE_CRT): each standard stream on its
/// Win32 handle. A statement macro, so a function is the faithful port.
pub unsafe fn ConInitStdStreams() {
    ConStreamInit(StdIn, GetStdHandle(STD_INPUT_HANDLE), AnsiText, INVALID_CP);
    ConStreamInit(StdOut, GetStdHandle(STD_OUTPUT_HANDLE), AnsiText, INVALID_CP);
    ConStreamInit(StdErr, GetStdHandle(STD_ERROR_HANDLE), AnsiText, INVALID_CP);
}

// Hand-written, from the C prototypes (outside the generated blocks, so `port resolve` treats
// these names as provided and never copies a tree-consensus variant beside them).
extern "C" {
    // conutils/stream.h (diskpart does not define USE_CRT).
    pub fn ConStreamInit(Stream: PCON_STREAM, Handle: PVOID, Mode: CON_STREAM_MODE, CacheCodePage: UINT) -> BOOL;
    // conutils/outstream.h
    pub fn ConMsgPuts(Stream: PCON_STREAM, dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD) -> INT;
    pub fn ConResPuts(Stream: PCON_STREAM, uID: UINT) -> INT;
    // crt/stdio.h
    pub fn _wfopen(_Filename: *const WCHAR, _Mode: *const WCHAR) -> *mut FILE;
    pub fn fgetws(_Dst: *mut WCHAR, _SizeInWords: c_int, _File: *mut FILE) -> *mut WCHAR;
}
extern "system" {
    // winnls.h
    pub fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: c_int, lpWideCharStr: LPWSTR, cchWideChar: c_int) -> c_int;
}
