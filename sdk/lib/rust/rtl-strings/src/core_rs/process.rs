//! Constants + FFI struct layouts for `sdk/lib/rtl/process.c`.
//!
//! Wave PRC is translate-only: 9 fns of `Zw*`/`Nt*` syscall + `Rtl*`
//! helper plumbing (process create + small encode/decode + critical
//! flag + os-version reads). No algorithmic content to host in
//! `core_rs`. Faithful to ReactOS `sdk/lib/rtl/process.c`.

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::thread as thr;

// --- NTSTATUS values used here.
pub use super::thread::{STATUS_NOT_IMPLEMENTED, STATUS_UNSUCCESSFUL};

// --- Memory-protection + section + share + handle flags.
pub use super::thread::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE};

pub const PAGE_EXECUTE: u32 = 0x10;
pub const SEC_IMAGE: u32 = 0x0100_0000;
pub const SECTION_ALL_ACCESS: u32 = 0x000F_001F;

// --- File / object access masks.
pub const SYNCHRONIZE: u32 = 0x0010_0000;
pub const FILE_READ_DATA: u32 = 0x0001;
pub const FILE_EXECUTE: u32 = 0x0020;
pub const FILE_SHARE_READ: u32 = 0x0001;
pub const FILE_SHARE_DELETE: u32 = 0x0004;
pub const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x0000_0020;
pub const FILE_NON_DIRECTORY_FILE: u32 = 0x0000_0040;
pub const OBJ_INHERIT: u32 = 0x0000_0002;
pub const OBJ_CASE_INSENSITIVE: u32 = 0x0000_0040;

// --- PROCESS_ALL_ACCESS — per memory `process_all_access_constant`:
// kernel ntoskrnl-rust uses NT 5.x value (Vista+ extended is 0x001F_FFFF).
pub const PROCESS_ALL_ACCESS: u32 = 0x001F_0FFF;

// --- Duplicate options (winnt.h).
pub const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;
pub const DUPLICATE_SAME_ATTRIBUTES: u32 = 0x0000_0004;

// --- PEB.NtGlobalFlag bits (pstypes.h).
pub use super::thread::FLG_ENABLE_SYSTEM_CRIT_BREAKS;
pub const FLG_ENABLE_CSRDEBUG: u32 = 0x0002_0000;

// --- Process / Section info classes.
pub const PROCESS_BASIC_INFORMATION_CLASS: u32 = 0;
pub const PROCESS_BREAK_ON_TERMINATION_CLASS: u32 = 29;
pub const PROCESS_COOKIE_CLASS: u32 = 36;
pub const SECTION_IMAGE_INFORMATION_CLASS: u32 = 1;

// --- RTL_USER_PROCESS_PARAMETERS.Flags bit (rtltypes.h).
pub const RTL_USER_PROCESS_PARAMETERS_RESERVE_1MB: u32 = 0x0000_0040;

// --- `_WIN32_WINNT_WS03` baseline used by RtlRosGetAppcompatVersion default.
pub const WIN32_WINNT_WS03: u32 = 0x0502;

// --- KUSER_SHARED_DATA.Cookie offset (ndk/ketypes.h:686). Same on
// all 3 arches (0x330) — RtlEncodeSystemPointer reads it via the
// fixed-VA `KI_USER_SHARED_DATA` mapping.
pub const KUSER_SHARED_COOKIE_OFFSET: usize = 0x330;

#[cfg(target_arch = "x86")]
pub const KI_USER_SHARED_DATA_VA: usize = 0xFFDF0000;
#[cfg(target_arch = "x86_64")]
pub const KI_USER_SHARED_DATA_VA: usize = 0xFFFFF780_00000000;
#[cfg(target_arch = "aarch64")]
pub const KI_USER_SHARED_DATA_VA: usize = 0xFFFFF780_00000000;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
pub const KI_USER_SHARED_DATA_VA: usize = 0;

// --- struct layouts -----------------------------------------------------

/// `IO_STATUS_BLOCK` (ntddk.h). The union is one pointer-sized slot
/// (`Status`/`Pointer`) followed by ULONG_PTR `Information`.
#[repr(C)]
pub struct IO_STATUS_BLOCK {
    pub StatusOrPointer: usize,
    pub Information: usize,
}

/// `PROCESS_BASIC_INFORMATION` (ntpsapi.h).
#[repr(C)]
pub struct PROCESS_BASIC_INFORMATION {
    pub ExitStatus: i32,
    pub PebBaseAddress: *mut u8,
    pub AffinityMask: usize,
    pub BasePriority: i32,
    pub UniqueProcessId: usize,
    pub InheritedFromUniqueProcessId: usize,
}

/// `SECTION_IMAGE_INFORMATION` (ntddk.h). The fields after
/// `CommittedStackSize` are not accessed by Wave PRC, but we
/// keep the layout so the struct passes through `ZwQuerySection`
/// at the correct size.
#[repr(C)]
pub struct SECTION_IMAGE_INFORMATION {
    pub TransferAddress: *mut u8,
    pub ZeroBits: u32,
    pub MaximumStackSize: usize,
    pub CommittedStackSize: usize,
    pub SubSystemType: u32,
    pub SubSystemMinorVersion: u16,
    pub SubSystemMajorVersion: u16,
    pub GpValue: u32,
    pub ImageCharacteristics: u16,
    pub DllCharacteristics: u16,
    pub Machine: u16,
    pub ImageContainsCode: u8,
    pub ImageFlags: u8,
    pub LoaderFlags: u32,
    pub ImageFileSize: u32,
    pub CheckSum: u32,
}

/// `UNICODE_STRING` layout-match (shared with `cabi::mod.rs` but
/// duplicated here so `core_rs::process` is self-contained).
#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}

/// `STRING` (counted ANSI string).
#[repr(C)]
pub struct STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u8,
}

/// `CURDIR` (rtltypes.h).
#[repr(C)]
pub struct CURDIR {
    pub DosPath: UNICODE_STRING,
    pub Handle: *mut u8,
}

/// `RTL_DRIVE_LETTER_CURDIR` (rtltypes.h).
#[repr(C)]
pub struct RTL_DRIVE_LETTER_CURDIR {
    pub Flags: u16,
    pub Length: u16,
    pub TimeStamp: u32,
    pub DosPath: STRING,
}

pub const RTL_MAX_DRIVE_LETTERS: usize = 32;

/// `RTL_USER_PROCESS_PARAMETERS` (rtltypes.h:1551, NTDDI_WS03 baseline).
#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS {
    pub MaximumLength: u32,
    pub Length: u32,
    pub Flags: u32,
    pub DebugFlags: u32,
    pub ConsoleHandle: *mut u8,
    pub ConsoleFlags: u32,
    pub StandardInput: *mut u8,
    pub StandardOutput: *mut u8,
    pub StandardError: *mut u8,
    pub CurrentDirectory: CURDIR,
    pub DllPath: UNICODE_STRING,
    pub ImagePathName: UNICODE_STRING,
    pub CommandLine: UNICODE_STRING,
    pub Environment: *mut u16,
    pub StartingX: u32,
    pub StartingY: u32,
    pub CountX: u32,
    pub CountY: u32,
    pub CountCharsX: u32,
    pub CountCharsY: u32,
    pub FillAttribute: u32,
    pub WindowFlags: u32,
    pub ShowWindowFlags: u32,
    pub WindowTitle: UNICODE_STRING,
    pub DesktopInfo: UNICODE_STRING,
    pub ShellInfo: UNICODE_STRING,
    pub RuntimeData: UNICODE_STRING,
    pub CurrentDirectories: [RTL_DRIVE_LETTER_CURDIR; RTL_MAX_DRIVE_LETTERS],
}

/// `CLIENT_ID` (re-exported from `thread`).
pub use super::thread::CLIENT_ID;

/// `RTL_USER_PROCESS_INFORMATION` (rtltypes.h:1589).
#[repr(C)]
pub struct RTL_USER_PROCESS_INFORMATION {
    pub Size: u32,
    pub ProcessHandle: *mut u8,
    pub ThreadHandle: *mut u8,
    pub ClientId: CLIENT_ID,
    pub ImageInformation: SECTION_IMAGE_INFORMATION,
}

// Suppress the unused-import warning for `thr` — its `pub use`s are
// what we want, but the bare alias is unused.
#[doc(hidden)]
const _USE_THR: usize = core::mem::size_of::<thr::INITIAL_TEB>();
