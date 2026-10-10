//! setuplib-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1 — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier).
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
// DPRINT1 PRINTS: this tree builds -DDBG=1, where reactos/debug.h defines it as
// `if (DbgPrint("(%s:%d) " fmt, __RELFILE__, __LINE__, ...)) DbgPrint("(%s:%d) DbgPrint() failed!\n", ..)`.
// The prefix carries the Rust file, not the C one. The translations spell the format three
// ways (a str literal, a byte literal, a pointer), so it goes through `DprintFmt`.
#[macro_export] macro_rules! DPRINT1 {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut __dprint1_buf = [0u8; 512];
        let __dprint1_fmt = $crate::prelude::DprintFmt::fmt_ptr($fmt, &mut __dprint1_buf);
        let __dprint1_file = concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char;
        $crate::prelude::DbgPrint(b"(%s:%d) \0".as_ptr() as *const core::ffi::c_char, __dprint1_file, line!());
        if $crate::prelude::DbgPrint(__dprint1_fmt $(, $arg)*) != 0 {
            $crate::prelude::DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                __dprint1_file,
                line!(),
            );
        }
    }};
}
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

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { ((($s) as i32) >= 0) as u8 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    ($addr:expr, $ty:ty, $field:ident) => {
        (($addr) as usize - core::mem::offset_of!($ty, $field)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust rather than per module.
pub use win32macros_rust::{ARRAYSIZE, ASSERT, C_ASSERT, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, PAGE_ROUND_UP, PRIMARYLANGID, RGB, RTL_CONSTANT_STRING, RTL_FIELD_SIZE, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ARCH_NEC98x86: _ARCHITECTURE_TYPE = _ARCHITECTURE_TYPE_ARCH_NEC98x86; // island enum
pub const ARCH_PcAT: _ARCHITECTURE_TYPE = _ARCHITECTURE_TYPE_ARCH_PcAT; // island enum
pub const ARCH_Efi: _ARCHITECTURE_TYPE = _ARCHITECTURE_TYPE_ARCH_Efi; // island enum
pub const BS_CreateAlways: _BOOT_STORE_OPENMODE = _BOOT_STORE_OPENMODE_BS_CreateAlways; // island enum
pub const BS_CreateNew: _BOOT_STORE_OPENMODE = _BOOT_STORE_OPENMODE_BS_CreateNew; // island enum
pub const BS_OpenAlways: _BOOT_STORE_OPENMODE = _BOOT_STORE_OPENMODE_BS_OpenAlways; // island enum
pub const BS_OpenExisting: _BOOT_STORE_OPENMODE = _BOOT_STORE_OPENMODE_BS_OpenExisting; // island enum
pub const BS_ReadAccess: _BOOT_STORE_ACCESS = _BOOT_STORE_ACCESS_BS_ReadAccess; // island enum
pub const BS_ReadWriteAccess: _BOOT_STORE_ACCESS = _BOOT_STORE_ACCESS_BS_ReadWriteAccess; // island enum
pub const BS_RecreateExisting: _BOOT_STORE_OPENMODE = _BOOT_STORE_OPENMODE_BS_RecreateExisting; // island enum
pub const BS_WriteAccess: _BOOT_STORE_ACCESS = _BOOT_STORE_ACCESS_BS_WriteAccess; // island enum
pub const BldrTypeMax: _BOOT_STORE_TYPE = _BOOT_STORE_TYPE_BldrTypeMax; // island enum
pub const ChangeSystemPartition: _FSVOLNOTIFY = _FSVOLNOTIFY_ChangeSystemPartition; // island enum
pub const CodePageInfoUpdate: _REGISTRY_STATUS = _REGISTRY_STATUS_CodePageInfoUpdate; // island enum
pub const DisplaySettingsUpdate: _REGISTRY_STATUS = _REGISTRY_STATUS_DisplaySettingsUpdate; // island enum
pub const ERROR_ADDING_CODEPAGE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_ADDING_CODEPAGE; // island enum
pub const ERROR_ADDING_KBLAYOUTS: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_ADDING_KBLAYOUTS; // island enum
pub const ERROR_CABINET_SECTION: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_CABINET_SECTION; // island enum
pub const ERROR_COPY_QUEUE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_COPY_QUEUE; // island enum
pub const ERROR_CORRUPT_TXTSETUPSIF: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_CORRUPT_TXTSETUPSIF; // island enum
pub const ERROR_CREATE_DIR: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_CREATE_DIR; // island enum
pub const ERROR_CREATE_HIVE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_CREATE_HIVE; // island enum
pub const ERROR_CREATE_INSTALL_DIR: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_CREATE_INSTALL_DIR; // island enum
pub const ERROR_FIND_REGISTRY: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_FIND_REGISTRY; // island enum
pub const ERROR_IMPORT_HIVE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_IMPORT_HIVE; // island enum
pub const ERROR_INITIALIZE_REGISTRY: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_INITIALIZE_REGISTRY; // island enum
pub const ERROR_INSTALL_BOOTCODE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_INSTALL_BOOTCODE; // island enum
pub const ERROR_INVALID_CABINET_INF: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_INVALID_CABINET_INF; // island enum
pub const ERROR_LOAD_TXTSETUPSIF: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_LOAD_TXTSETUPSIF; // island enum
pub const ERROR_NEW_PARTITION: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_NEW_PARTITION; // island enum
pub const ERROR_NO_SOURCE_DRIVE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_NO_SOURCE_DRIVE; // island enum
pub const ERROR_ONLY_ONE_EXTENDED: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_ONLY_ONE_EXTENDED; // island enum
pub const ERROR_PARTITION_TABLE_FULL: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_PARTITION_TABLE_FULL; // island enum
pub const ERROR_SIGNATURE_TXTSETUPSIF: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_SIGNATURE_TXTSETUPSIF; // island enum
pub const ERROR_TXTSETUP_SECTION: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_TXTSETUP_SECTION; // island enum
pub const ERROR_UPDATE_DISPLAY_SETTINGS: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_UPDATE_DISPLAY_SETTINGS; // island enum
pub const ERROR_UPDATE_GEOID: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_UPDATE_GEOID; // island enum
pub const ERROR_UPDATE_KBSETTINGS: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_UPDATE_KBSETTINGS; // island enum
pub const ERROR_UPDATE_LOCALESETTINGS: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_UPDATE_LOCALESETTINGS; // island enum
pub const ERROR_WRITE_BOOT: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_WRITE_BOOT; // island enum
pub const FILE_GENERIC_READ: u32 = 1179785; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const FILE_GENERIC_WRITE: u32 = 1179926; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const FMIFS_FLOPPY: FMIFS_MEDIA_FLAG = FMIFS_MEDIA_FLAG_FMIFS_FLOPPY; // island enum
pub const FMIFS_REMOVABLE: FMIFS_MEDIA_FLAG = FMIFS_MEDIA_FLAG_FMIFS_REMOVABLE; // island enum
pub const FSCTL_DISMOUNT_VOLUME: ULONG = 0x00090020; // ntkrnl-sys/src/constants.rs
pub const FSCTL_LOCK_VOLUME: ULONG = 0x00090018; // ntkrnl-sys/src/constants.rs
pub const FSCTL_UNLOCK_VOLUME: ULONG = 0x0009001c; // ntkrnl-sys/src/constants.rs
pub const FSVOLNOTIFY_CHECKERROR: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_CHECKERROR; // island enum
pub const FSVOLNOTIFY_ENDCHECK: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDCHECK; // island enum
pub const FSVOLNOTIFY_ENDFORMAT: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDFORMAT; // island enum
pub const FSVOLNOTIFY_ENDQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDQUEUE; // island enum
pub const FSVOLNOTIFY_ENDSUBQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDSUBQUEUE; // island enum
pub const FSVOLNOTIFY_FORMATERROR: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_FORMATERROR; // island enum
pub const FSVOLNOTIFY_PARTITIONERROR: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_PARTITIONERROR; // island enum
pub const FSVOLNOTIFY_STARTCHECK: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTCHECK; // island enum
pub const FSVOLNOTIFY_STARTFORMAT: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTFORMAT; // island enum
pub const FSVOLNOTIFY_STARTQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTQUEUE; // island enum
pub const FSVOLNOTIFY_STARTSUBQUEUE: _FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTSUBQUEUE; // island enum
pub const FSVOL_ABORT: _FSVOL_OP = _FSVOL_OP_FSVOL_ABORT; // island enum
pub const FSVOL_CHECK: _FSVOL_OP = _FSVOL_OP_FSVOL_CHECK; // island enum
pub const FSVOL_DOIT: _FSVOL_OP = _FSVOL_OP_FSVOL_DOIT; // island enum
pub const FSVOL_FORMAT: _FSVOL_OP = _FSVOL_OP_FSVOL_FORMAT; // island enum
pub const FSVOL_RETRY: _FSVOL_OP = _FSVOL_OP_FSVOL_RETRY; // island enum
pub const FSVOL_SKIP: _FSVOL_OP = _FSVOL_OP_FSVOL_SKIP; // island enum
pub const Formatted: _FORMATSTATE = _FORMATSTATE_Formatted; // island enum
pub const FreeLdr: _BOOT_STORE_TYPE = _BOOT_STORE_TYPE_FreeLdr; // island enum
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as HKEY; // userenv-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // dhcpcsvc-rust/src/prelude.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY; // dhcpcsvc-rust/src/prelude.rs
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as HKEY; // userenv-rust/src/prelude.rs
pub const INSERT_AFTER: INSERTION_TYPE = INSERTION_TYPE_INSERT_AFTER; // island enum
pub const INSERT_BEFORE: INSERTION_TYPE = INSERTION_TYPE_INSERT_BEFORE; // island enum
pub const INSERT_FIRST: INSERTION_TYPE = INSERTION_TYPE_INSERT_FIRST; // island enum
pub const INSERT_LAST: INSERTION_TYPE = INSERTION_TYPE_INSERT_LAST; // island enum
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000; // disk-rust/src/glue.rs
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: u32 = 0x000700A0; // partmgr-rust/src/glue.rs
pub const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x00074004; // fs_rec-rust/src/glue.rs
pub const IOCTL_DISK_GET_PARTITION_INFO_EX: u32 = 0x00070048; // partmgr-rust/src/glue.rs
pub const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: u32 = 0x004D0008; // classpnp-rust/src/glue.rs
pub const IOCTL_SCSI_GET_ADDRESS: u32 = 0x00041018; // classpnp-rust/src/glue.rs
pub const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x002D1080; // classpnp-rust/src/glue.rs
pub const ImportRegHive: _REGISTRY_STATUS = _REGISTRY_STATUS_ImportRegHive; // island enum
pub const KeybLayouts: _REGISTRY_STATUS = _REGISTRY_STATUS_KeybLayouts; // island enum
pub const KeybSettingsUpdate: _REGISTRY_STATUS = _REGISTRY_STATUS_KeybSettingsUpdate; // island enum
pub const LocaleSettingsUpdate: _REGISTRY_STATUS = _REGISTRY_STATUS_LocaleSettingsUpdate; // island enum
pub const NOT_AN_ERROR: _ERROR_NUMBER = _ERROR_NUMBER_NOT_AN_ERROR; // island enum
pub const NtLdr: _BOOT_STORE_TYPE = _BOOT_STORE_TYPE_NtLdr; // island enum
pub const RegHiveUpdate: _REGISTRY_STATUS = _REGISTRY_STATUS_RegHiveUpdate; // island enum
pub const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000007Bu32 as NTSTATUS; // kernel32-rust/src/client/loader.rs
pub const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC000_0106u32 as NTSTATUS; // mup-rust/src/glue.rs
pub const STATUS_REGISTRY_RECOVERED: i32 = 0x4000_0009; // cmlib-rust/src/glue.rs
pub const STATUS_RESOURCE_TYPE_NOT_FOUND: i32 = 0xC000008Au32 as i32; // rtl-actctx/src/shim.rs
pub const STATUS_UNRECOGNIZED_VOLUME: i32 = 0xC000014Fu32 as i32; // fs_rec-rust/src/glue.rs
pub const ULONG_MAX: u32 = 4294967295; // msv1_0-rust/src/types_amd64.rs
pub const UNICODE_STRING_MAX_BYTES: USHORT = 65534_u64 as USHORT; // win32k-sys/src/bindgen_amd64.rs
pub const Unformatted: _FORMATSTATE = _FORMATSTATE_Unformatted; // island enum
pub const UnknownFormat: _FORMATSTATE = _FORMATSTATE_UnknownFormat; // island enum
extern "C" {
    pub fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32; // kernel32-rust/src/client/dllmain.rs
    pub fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32; // freeldr-rust/src/bootmgr.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32; // msvcrt-difftest/tests/classify.rs
    pub fn _wtoi(s: *const u16) -> c_int; // advapi32-rust/src/prelude.rs
    pub fn isalnum(c: i32) -> i32; // kernel32-rust/src/string/nls_subst.rs
    pub fn isspace(c: i32) -> i32; // freeldr-rust/src/inffile.rs
    pub fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32; // freeldr-rust/src/hwdisk.rs
    pub fn strchr(s: *const i8, c: i32) -> *mut i8; // freeldr-rust/src/frldr.rs
    pub fn strtoul(s: PCSTR, endptr: *mut PCSTR, base: i32) -> u32; // freeldr-rust/src/drivemap.rs
    pub fn tolower(c: i32) -> i32; // freeldr-rust/src/pxe.rs
    pub fn towlower(c: u16) -> u16; // msvcrt-difftest/tests/classify.rs
    pub fn towupper(c: c_int) -> c_int; // dnsapi-rust/src/prelude.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // acpi-rust/src/buspdo.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // kernel32-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // inflib-rust/src/infget.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // concfg-rust/src/font.rs
}
// ==== end port resolve ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/setup/lib/utils/ntverrsrc.c:90: #define DWORD_ALIGN(base, ptr) ( (ULONG_PTR)(base) + ((((ULONG_PTR)(ptr) - (ULONG_PTR)(base)) + 3) & ~3) )
#[allow(unused_macros)]
macro_rules! DWORD_ALIGN {
    ($base:expr, $ptr:expr $(,)?) => {
        (($base as ULONG_PTR) + (((($ptr as ULONG_PTR) - ($base as ULONG_PTR)) + 3) & !3))
    };
}
#[allow(unused_imports)]
pub(crate) use DWORD_ALIGN;
// base/setup/lib/utils/filesup.h:80: #define DoesDirExist(RootDirectory, DirName) DoesPathExist((RootDirectory), (DirName), TRUE)
#[allow(unused_macros)]
macro_rules! DoesDirExist {
    ($RootDirectory:expr, $DirName:expr $(,)?) => {
        (DoesPathExist($RootDirectory, $DirName, TRUE as BOOLEAN))
    };
}
#[allow(unused_imports)]
pub(crate) use DoesDirExist;
// base/setup/lib/utils/filesup.h:83: #define DoesFileExist(RootDirectory, FileName) DoesPathExist((RootDirectory), (FileName), FALSE)
#[allow(unused_macros)]
macro_rules! DoesFileExist {
    ($RootDirectory:expr, $FileName:expr $(,)?) => {
        (DoesPathExist($RootDirectory, $FileName, FALSE as BOOLEAN))
    };
}
#[allow(unused_imports)]
pub(crate) use DoesFileExist;
// base/setup/lib/utils/partlist.h:258: #define GetDiskSizeInBytes(DiskEntry) ((DiskEntry)->SectorCount.QuadPart * (DiskEntry)->BytesPerSector)
#[allow(unused_macros)]
macro_rules! GetDiskSizeInBytes {
    ($DiskEntry:expr $(,)?) => {
        ((*$DiskEntry).SectorCount.QuadPart * (*$DiskEntry).BytesPerSector)
    };
}
#[allow(unused_imports)]
pub(crate) use GetDiskSizeInBytes;
// base/setup/lib/utils/partlist.c:2595: #define GetLogicalPartitionCount(DiskEntry) (((DiskEntry)->DiskStyle == PARTITION_STYLE_MBR) ? GetPartitionCount(&(DiskEntry)->LogicalPartListHead) : 0)
#[allow(unused_macros)]
macro_rules! GetLogicalPartitionCount {
    ($DiskEntry:expr $(,)?) => {
        (if (*$DiskEntry).DiskStyle == PARTITION_STYLE_MBR { GetPartitionCount(&mut (*$DiskEntry).LogicalPartListHead) } else { 0 })
    };
}
#[allow(unused_imports)]
pub(crate) use GetLogicalPartitionCount;
// base/setup/lib/utils/partlist.h:252: #define GetPartEntryOffsetInBytes(PartEntry) ((PartEntry)->StartSector.QuadPart * (PartEntry)->DiskEntry->BytesPerSector)
#[allow(unused_macros)]
macro_rules! GetPartEntryOffsetInBytes {
    ($PartEntry:expr $(,)?) => {
        (((*$PartEntry).StartSector.QuadPart) * ((*(*$PartEntry).DiskEntry).BytesPerSector))
    };
}
#[allow(unused_imports)]
pub(crate) use GetPartEntryOffsetInBytes;
// base/setup/lib/utils/partlist.h:255: #define GetPartEntrySizeInBytes(PartEntry) ((PartEntry)->SectorCount.QuadPart * (PartEntry)->DiskEntry->BytesPerSector)
#[allow(unused_macros)]
macro_rules! GetPartEntrySizeInBytes {
    ($PartEntry:expr $(,)?) => {
        (((*$PartEntry).SectorCount.QuadPart) * ((*(*$PartEntry).DiskEntry).BytesPerSector))
    };
}
#[allow(unused_imports)]
pub(crate) use GetPartEntrySizeInBytes;
// base/setup/lib/registry.c:86: #define GetPredefKeyIndex(HKey) ((ULONG_PTR)(HKey) & 0x0FFFFFFF)
#[allow(unused_macros)]
macro_rules! GetPredefKeyIndex {
    ($HKey:expr $(,)?) => {
        (($HKey as ULONG_PTR) & 0x0FFFFFFF)
    };
}
#[allow(unused_imports)]
pub(crate) use GetPredefKeyIndex;
// base/setup/lib/utils/partlist.c:2592: #define GetPrimaryPartitionCount(DiskEntry) GetPartitionCount(&(DiskEntry)->PrimaryPartListHead)
#[allow(unused_macros)]
macro_rules! GetPrimaryPartitionCount {
    ($DiskEntry:expr $(,)?) => {
        (GetPartitionCount(&mut (*$DiskEntry).PrimaryPartListHead))
    };
}
#[allow(unused_imports)]
pub(crate) use GetPrimaryPartitionCount;
// base/setup/lib/spapisup/infsupp.h:152: header-inline `INF_FreeData`
#[allow(dead_code)]
#[inline]
pub unsafe fn INF_FreeData(InfData: PCWSTR) {
    UNREFERENCED_PARAMETER!(InfData);
}
// base/setup/lib/setuplib.c:897: #define IS_PATH_SEPARATOR(c) ((c) == L'\\' || (c) == L'/')
#[allow(unused_macros)]
macro_rules! IS_PATH_SEPARATOR {
    ($c:expr $(,)?) => {
        (($c == (b'\\' as u16)) || ($c == (b'/' as u16)))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_PATH_SEPARATOR;
// base/setup/lib/utils/volutil.h:30: #define IS_RAWFS(fs) ((fs)[0] == 'R' && (fs)[1] == 'A' && (fs)[2] == 'W' && (fs)[3] == 0)
#[allow(unused_macros)]
macro_rules! IS_RAWFS {
    ($fs:expr $(,)?) => {
        (($fs[0] as u8 == b'R') && ($fs[1] as u8 == b'A') && ($fs[2] as u8 == b'W') && ($fs[3] as u8 == 0))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_RAWFS;
// base/setup/lib/setuplib.h:206: #define IS_VALID_INSTALL_PATH_CHAR(c) (isalnum(c) || (c) == L'.' || (c) == L'\\' || (c) == L'-' || (c) == L'_')
#[allow(unused_macros)]
macro_rules! IS_VALID_INSTALL_PATH_CHAR {
    ($c:expr $(,)?) => {
        // setuplib.h: isalnum takes the promoted int; the rest compare WCHARs.
        ((isalnum(($c) as i32) != 0) || (($c) as u16 == b'.' as u16) || (($c) as u16 == b'\\' as u16) || (($c) as u16 == b'-' as u16) || (($c) as u16 == b'_' as u16))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_VALID_INSTALL_PATH_CHAR;
// base/setup/lib/utils/linklist.h:10: #define InsertAscendingList(ListHead, NewEntry, Type, ListEntryField, SortField) do { PLIST_ENTRY current = (ListHead)->Flink; while (current != (ListHead)) { if (CONTAINING_RECORD(current, Type, ListEntryField)->SortField >= (NewEntry)->SortField) { break; } current = current->Flink; } InsertTailList(current, &((NewEntry)->ListEntryField)); } while (0)
#[allow(unused_macros)]
macro_rules! InsertAscendingList {
    ($ListHead:expr, $NewEntry:expr, $Type:ty, $ListEntryField:ident, $SortField:ident $(,)?) => {
        {
            let mut current = (*$ListHead).Flink;
            while current != $ListHead {
                if (*CONTAINING_RECORD!(current, $Type, $ListEntryField)).$SortField >= (*$NewEntry).$SortField {
                    break;
                }
                current = (*current).Flink;
            }
            InsertTailList(current, &mut (*$NewEntry).$ListEntryField);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use InsertAscendingList;
// base/setup/lib/utils/partlist.h:22: #define IsOEMPartition(PartitionType) ( ((PartitionType) == PARTITION_EISA) || ((PartitionType) == PARTITION_HIBERNATION) || ((PartitionType) == PARTITION_DIAGNOSTIC) || ((PartitionType) == PARTITION_DELL) || ((PartitionType) == PARTITION_IBM) )
#[allow(unused_macros)]
macro_rules! IsOEMPartition {
    ($PartitionType:expr $(,)?) => {
        (($PartitionType == PARTITION_EISA) || ($PartitionType == PARTITION_HIBERNATION) || ($PartitionType == PARTITION_DIAGNOSTIC) || ($PartitionType == PARTITION_DELL) || ($PartitionType == PARTITION_IBM))
    };
}
#[allow(unused_imports)]
pub(crate) use IsOEMPartition;
// base/setup/lib/registry.c:83: #define IsPredefKey(HKey) (((ULONG_PTR)(HKey) & 0xF0000000) == 0x80000000)
#[allow(unused_macros)]
macro_rules! IsPredefKey {
    ($HKey:expr $(,)?) => {
        ((($HKey as ULONG_PTR) & 0xF0000000) == 0x80000000)
    };
}
#[allow(unused_imports)]
pub(crate) use IsPredefKey;
// base/setup/lib/utils/volutil.h:36: #define IsUnformatted(VolInfo) IS_RAWFS((VolInfo)->FileSystem)
#[allow(unused_macros)]
macro_rules! IsUnformatted {
    ($VolInfo:expr $(,)?) => {
        (IS_RAWFS!((*$VolInfo).FileSystem))
    };
}
#[allow(unused_imports)]
pub(crate) use IsUnformatted;
// base/setup/lib/utils/volutil.h:33: #define IsUnknown(VolInfo) (!*(VolInfo)->FileSystem)
#[allow(unused_macros)]
macro_rules! IsUnknown {
    ($VolInfo:expr $(,)?) => {
        ((*$VolInfo).FileSystem[0] == 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IsUnknown;
// base/setup/lib/utils/bldrsup.h:58: #define MAKESTRKEY(i) ((ULONG_PTR)(i))
#[allow(unused_macros)]
macro_rules! MAKESTRKEY {
    ($i:expr $(,)?) => {
        ($i as ULONG_PTR)
    };
}
#[allow(unused_imports)]
pub(crate) use MAKESTRKEY;
// base/setup/lib/spapisup/fileqsup.h:137: #define SpFileQueueClose (SpFileExports.SpFileQueueClose)
#[allow(unused_macros)]
macro_rules! SpFileQueueClose {
    ($($arg:expr),* $(,)?) => {
        ((SpFileExports.SpFileQueueClose.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpFileQueueClose;
// base/setup/lib/spapisup/fileqsup.h:141: #define SpFileQueueCommit (SpFileExports.SpFileQueueCommit)
#[allow(unused_macros)]
macro_rules! SpFileQueueCommit {
    ($($arg:expr),* $(,)?) => {
        ((SpFileExports.SpFileQueueCommit.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpFileQueueCommit;
// base/setup/lib/spapisup/fileqsup.h:138: #define SpFileQueueCopy (SpFileExports.SpFileQueueCopy)
#[allow(unused_macros)]
macro_rules! SpFileQueueCopy {
    ($($arg:expr),* $(,)?) => {
        ((SpFileExports.SpFileQueueCopy.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpFileQueueCopy;
// base/setup/lib/spapisup/fileqsup.h:136: #define SpFileQueueOpen (SpFileExports.SpFileQueueOpen)
#[allow(unused_macros)]
macro_rules! SpFileQueueOpen {
    ($($arg:expr),* $(,)?) => {
        ((SpFileExports.SpFileQueueOpen.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpFileQueueOpen;
// base/setup/lib/spapisup/infsupp.h:139: #define SpInfCloseInfFile (SpInfExports.SpInfCloseInfFile)
#[allow(unused_macros)]
macro_rules! SpInfCloseInfFile {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfCloseInfFile.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfCloseInfFile;
// base/setup/lib/spapisup/infsupp.h:140: #define SpInfFindFirstLine (SpInfExports.SpInfFindFirstLine)
#[allow(unused_macros)]
macro_rules! SpInfFindFirstLine {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfFindFirstLine.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfFindFirstLine;
// base/setup/lib/spapisup/infsupp.h:141: #define SpInfFindNextLine (SpInfExports.SpInfFindNextLine)
#[allow(unused_macros)]
macro_rules! SpInfFindNextLine {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfFindNextLine.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfFindNextLine;
// base/setup/lib/spapisup/infsupp.h:143: #define SpInfGetBinaryField (SpInfExports.SpInfGetBinaryField)
#[allow(unused_macros)]
macro_rules! SpInfGetBinaryField {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfGetBinaryField.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfGetBinaryField;
// base/setup/lib/spapisup/infsupp.h:147: #define SpInfGetField (SpInfExports.SpInfGetField)
#[allow(unused_macros)]
macro_rules! SpInfGetField {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfGetField.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfGetField;
// base/setup/lib/spapisup/infsupp.h:142: #define SpInfGetFieldCount (SpInfExports.SpInfGetFieldCount)
#[allow(unused_macros)]
macro_rules! SpInfGetFieldCount {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfGetFieldCount.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfGetFieldCount;
// base/setup/lib/spapisup/infsupp.h:144: #define SpInfGetIntField (SpInfExports.SpInfGetIntField)
#[allow(unused_macros)]
macro_rules! SpInfGetIntField {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfGetIntField.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfGetIntField;
// base/setup/lib/spapisup/infsupp.h:145: #define SpInfGetMultiSzField (SpInfExports.SpInfGetMultiSzField)
#[allow(unused_macros)]
macro_rules! SpInfGetMultiSzField {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfGetMultiSzField.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfGetMultiSzField;
// base/setup/lib/spapisup/infsupp.h:146: #define SpInfGetStringField (SpInfExports.SpInfGetStringField)
#[allow(unused_macros)]
macro_rules! SpInfGetStringField {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfGetStringField.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfGetStringField;
// base/setup/lib/spapisup/infsupp.h:148: #define SpInfOpenInfFile (SpInfExports.SpInfOpenInfFile)
#[allow(unused_macros)]
macro_rules! SpInfOpenInfFile {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfOpenInfFile.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfOpenInfFile;
// base/setup/lib/utils/filesup.h:121: #define UnMapAndCloseFile(FileHandle, SectionHandle, BaseAddress) do { UnMapFile((SectionHandle), (BaseAddress)); NtClose(FileHandle); } while (0)
#[allow(unused_macros)]
macro_rules! UnMapAndCloseFile {
    ($FileHandle:expr, $SectionHandle:expr, $BaseAddress:expr $(,)?) => {
        {
            UnMapFile($SectionHandle, $BaseAddress);
            NtClose($FileHandle);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use UnMapAndCloseFile;
// base/setup/lib/utils/ntverrsrc.c:96: #define VersionInfo32_Children(ver) (PCVS_VERSION_INFO_STRUCT32)( VersionInfo32_Value( ver ) + ( ( (ver)->wValueLength * ((ver)->wType? 2 : 1) + 3 ) & ~3 ) )
#[allow(unused_macros)]
macro_rules! VersionInfo32_Children {
    ($ver:expr $(,)?) => {
        ((VersionInfo32_Value!($ver) + ((((*$ver).wValueLength as ULONG_PTR) * (if (*$ver).wType != 0 { 2 } else { 1 }) + 3) & !3)) as PCVS_VERSION_INFO_STRUCT32)
    };
}
#[allow(unused_imports)]
pub(crate) use VersionInfo32_Children;
// base/setup/lib/utils/ntverrsrc.c:101: #define VersionInfo32_Next(ver) (PVS_VERSION_INFO_STRUCT32)( (ULONG_PTR)ver + (((ver)->wLength + 3) & ~3) )
#[allow(unused_macros)]
macro_rules! VersionInfo32_Next {
    ($ver:expr $(,)?) => {
        ((($ver as ULONG_PTR) + ((((*$ver).wLength as ULONG_PTR) + 3) & !3)) as PVS_VERSION_INFO_STRUCT32)
    };
}
#[allow(unused_imports)]
pub(crate) use VersionInfo32_Next;
// base/setup/lib/utils/ntverrsrc.c:93: #define VersionInfo32_Value(ver) DWORD_ALIGN( (ver), (ver)->szKey + wcslen((ver)->szKey) + 1 )
#[allow(unused_macros)]
macro_rules! VersionInfo32_Value {
    ($ver:expr $(,)?) => {
        (DWORD_ALIGN!($ver, (*$ver).szKey.as_ptr().add(wcslen((*$ver).szKey.as_ptr()) + 1)))
    };
}
#[allow(unused_imports)]
pub(crate) use VersionInfo32_Value;
// ==== end port macros ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::setuplib::partlist::AssignDriveLetters;
pub use crate::setuplib::filesup::ConcatPaths;
pub use crate::setuplib::partlist::DestroyRegion;
pub use crate::setuplib::filesup::DoesPathExist;
pub use crate::setuplib::partlist::GetActiveDiskPartition;
pub use crate::setuplib::partlist::GetPartitionCount;
pub use crate::setuplib::partlist::GetSystemDisk;
pub use crate::setuplib::partlist::InitPartitionDeviceName;
pub use crate::setuplib::partlist::InitVolume;
pub use crate::setuplib::partlist::InitVolumeDeviceName;
pub use crate::setuplib::partlist::InitializePartitionEntry;
pub use crate::setuplib::partlist::IsPartitionActive;
pub use crate::setuplib::partlist::IsSuperFloppy;
pub use crate::setuplib::filesup::UnMapFile;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ERROR_SYSTEM_PARTITION_NOT_FOUND: i32 = 0x2D; // clang, setuplib.c
pub const IOCTL_DISK_GET_DRIVE_LAYOUT: i32 = 0x7400C; // clang, partlist.c
pub const IOCTL_DISK_SET_DRIVE_LAYOUT: i32 = 0x7C010; // clang, partlist.c
pub const STATUS_NOT_SAME_DEVICE: NTSTATUS = 0xC00000D4_u32 as NTSTATUS; // clang, filesup.c
pub const STATUS_PARTITION_FAILURE: NTSTATUS = 0xC0000172_u32 as NTSTATUS; // clang, fsutil.c
pub const STATUS_REGISTRY_HIVE_RECOVERED: NTSTATUS = 0x8000002A_u32 as NTSTATUS; // clang, regutil.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// DISK_GEOMETRY_EX_INTERNAL: c2rust, bootsup.c
pub type DISK_GEOMETRY_EX_INTERNAL = _DISK_GEOMETRY_EX_INTERNAL;
// FILE_SYSTEM: c2rust, fsutil.c
pub type FILE_SYSTEM = _FILE_SYSTEM;
// GPT_PARTITION_TYPE: c2rust, partinfo.c
pub type GPT_PARTITION_TYPE = _GPT_PARTITION_TYPE;
// HIVE_LIST_ENTRY: c2rust, registry.c
pub type HIVE_LIST_ENTRY = _HIVE_LIST_ENTRY;
// HIVE_UPDATE_STATE: c2rust, registry.c
pub type HIVE_UPDATE_STATE = _HIVE_UPDATE_STATE;
// MBR_PARTITION_TYPE: c2rust, partinfo.c
pub type MBR_PARTITION_TYPE = _MBR_PARTITION_TYPE;
// PDISK_GEOMETRY_EX_INTERNAL: c2rust, bootsup.c
pub type PDISK_GEOMETRY_EX_INTERNAL = *mut _DISK_GEOMETRY_EX_INTERNAL;
// ROOT_KEY: c2rust, registry.c
pub type ROOT_KEY = _ROOT_KEY;
// _DISK_GEOMETRY_EX_INTERNAL: c2rust, bootsup.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _DISK_GEOMETRY_EX_INTERNAL {
    pub Geometry: DISK_GEOMETRY,
    pub DiskSize: LARGE_INTEGER,
    pub Partition: DISK_PARTITION_INFO,
}
// _FILE_SYSTEM: c2rust, fsutil.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _FILE_SYSTEM {
    pub FileSystemName: PCWSTR,
    pub FormatFunc: PULIB_FORMAT,
    pub ChkdskFunc: PULIB_CHKDSK,
}
// _GPT_PARTITION_TYPE: c2rust, partinfo.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _GPT_PARTITION_TYPE {
    pub Guid: GUID,
    pub Description: PCSTR,
}
// _HIVE_LIST_ENTRY: c2rust, registry.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _HIVE_LIST_ENTRY {
    pub HiveName: PCWSTR,
    pub HiveRegistryPath: PCWSTR,
    pub PredefKeyHandle: HANDLE,
    pub RegSymLink: PCWSTR,
    pub State: HIVE_UPDATE_STATE,
}
// _HIVE_UPDATE_STATE: c2rust, registry.c
pub type _HIVE_UPDATE_STATE = ::core::ffi::c_uint;
// _MBR_PARTITION_TYPE: c2rust, partinfo.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MBR_PARTITION_TYPE {
    pub Type: UCHAR,
    pub Description: PCSTR,
}
// _ROOT_KEY: c2rust, registry.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _ROOT_KEY {
    pub Name: PCWSTR,
    pub MountPoint: PCWSTR,
    pub Handle: HANDLE,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn VfatChkdsk(
        DriveRoot: PUNICODE_STRING,
        Callback: PFMIFSCALLBACK,
        FixErrors: BOOLEAN,
        Verbose: BOOLEAN,
        CheckOnlyIfDirty: BOOLEAN,
        ScanDrive: BOOLEAN,
        pUnknown1: PVOID,
        pUnknown2: PVOID,
        pUnknown3: PVOID,
        pUnknown4: PVOID,
        ExitStatus: PULONG,
    ) -> BOOLEAN; // fsutil.c
    pub fn VfatFormat(
        DriveRoot: PUNICODE_STRING,
        Callback: PFMIFSCALLBACK,
        QuickFormat: BOOLEAN,
        BackwardCompatible: BOOLEAN,
        MediaType: MEDIA_TYPE,
        Label: PUNICODE_STRING,
        ClusterSize: ULONG,
    ) -> BOOLEAN; // fsutil.c
}
pub use crate::setuplib::fsutil::InstallFatBootCode as InstallFat12BootCode; // base/setup/lib/fsutil.h:90
pub use crate::setuplib::fsutil::InstallFatBootCode as InstallFat16BootCode; // base/setup/lib/fsutil.h:91
// ==== end port externs ====

/// `ndk/rtlfuncs.h:182` -- FORCEINLINE in the header, so no DLL exports it; ntdll-sys's
/// bindgen `extern` for it is a link error. This definition shadows the glob import.
#[inline]
pub unsafe fn RtlInitEmptyUnicodeString(
    UnicodeString: PUNICODE_STRING,
    Buffer: PWCHAR,
    BufferSize: USHORT,
) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

/// A DPRINT1 format, NUL-terminated: used in place if it already is, else copied (truncated)
/// into the caller's stack buffer.
pub trait DprintFmt {
    fn fmt_ptr(self, buf: &mut [u8; 512]) -> *const core::ffi::c_char;
}
fn dprint_terminated(bytes: &[u8], buf: &mut [u8; 512]) -> *const core::ffi::c_char {
    if bytes.last() == Some(&0) {
        return bytes.as_ptr() as *const core::ffi::c_char;
    }
    let n = bytes.len().min(buf.len() - 1);
    buf[..n].copy_from_slice(&bytes[..n]);
    buf[n] = 0;
    buf.as_ptr() as *const core::ffi::c_char
}
impl DprintFmt for &'static str {
    fn fmt_ptr(self, buf: &mut [u8; 512]) -> *const core::ffi::c_char {
        dprint_terminated(self.as_bytes(), buf)
    }
}
impl<const N: usize> DprintFmt for &'static [u8; N] {
    fn fmt_ptr(self, buf: &mut [u8; 512]) -> *const core::ffi::c_char {
        dprint_terminated(self, buf)
    }
}
impl DprintFmt for *const i8 {
    fn fmt_ptr(self, _: &mut [u8; 512]) -> *const core::ffi::c_char {
        self as *const core::ffi::c_char
    }
}
impl DprintFmt for *const u8 {
    fn fmt_ptr(self, _: &mut [u8; 512]) -> *const core::ffi::c_char {
        self as *const core::ffi::c_char
    }
}
impl DprintFmt for *mut i8 {
    fn fmt_ptr(self, _: &mut [u8; 512]) -> *const core::ffi::c_char {
        self as *const core::ffi::c_char
    }
}
