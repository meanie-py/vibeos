//! glue.rs — #define constants + enum-variant aliases the fs_rec C source/LLM
//! reference by bare name. REAL values (FSREC_TAG compiler-evaluated, the rest from
//! the bindgen substrate), never LLM-guessed.
#![allow(non_upper_case_globals, dead_code)]

// fs_rec.h: #define FSREC_TAG 'cRsF'. Multi-char char constant → first char is the
// high byte: ('c'<<24)|('R'<<16)|('s'<<8)|'F' = 0x63527346 (clang/gcc-evaluated).
// Used as the ExAllocatePoolWithTag pool tag for FsRecLoadSync.
pub const FSREC_TAG: u32 = 0x63527346;

// core::ffi re-exports the LLM used by bare name.
pub use core::ffi::c_char;

// DDK #define constants — clang-computed (bindgen --allowlist-var over ntifs.h/
// ntdddisk.h/ntddcdrm.h/ffs.h), NOT hand-derived. Device types / object flags / IRP
// stack flags.
pub const FILE_DEVICE_CD_ROM_FILE_SYSTEM: u32 = 3;
pub const FILE_DEVICE_DISK: u32 = 7;
pub const FILE_DEVICE_DISK_FILE_SYSTEM: u32 = 8;
pub const FILE_FLOPPY_DISKETTE: u32 = 4;
pub const SL_OVERRIDE_VERIFY_VOLUME: u32 = 2;
pub const DO_LOW_PRIORITY_FILESYSTEM: u32 = 0x00010000; // 65536

// NTSTATUS codes — clang-evaluated (negative i32; bindgen folded via enum cast).
pub const STATUS_FS_DRIVER_REQUIRED: i32 = 0xC000019Cu32 as i32;  // -1073741412
pub const STATUS_UNRECOGNIZED_VOLUME: i32 = 0xC000014Fu32 as i32; // -1073741489

// CTL_CODE() IOCTLs — clang-computed exact values (NOT hand-derived).
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000;   // 458752
pub const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x00074004;   // 475140
pub const IOCTL_CDROM_GET_DRIVE_GEOMETRY: u32 = 0x0002404C;  // 147532

// ffs.h: #define DISKMAGIC ((u_int32_t)0x82564557) — FreeBSD disklabel magic.
pub const DISKMAGIC: u32 = 0x82564557;

// IRP_MN_* minor function codes for IRP_MJ_FILE_SYSTEM_CONTROL — clang-computed.
// CRITICAL: every FsRec*FsControl `match (*stack).MinorFunction { … }` uses these as
// arm patterns. If the const isn't in scope the arm silently degrades to an irrefutable
// BINDING (catch-all) and the recognizer treats every IRP as MOUNT_VOLUME, never loading
// the FS — surfaced only as a "unreachable pattern" warning (field manual: match-binding
// hazard). MinorFunction is UCHAR → u8.
pub const IRP_MN_MOUNT_VOLUME: u32 = 1;
pub const IRP_MN_LOAD_FILE_SYSTEM: u32 = 3;

// enum-variant aliases — bindgen prefixes C enum constants as _ENUM_Variant; the C
// source (and the LLM) reference the bare variant name, and several are used in `match`
// arms (where an out-of-scope name silently becomes a catch-all binding — so these MUST
// resolve). Re-export from crate::types with the exact bindgen type (both enums c_uint).
pub use crate::types::{
    _FILE_SYSTEM_TYPE_FS_TYPE_UNUSED as FS_TYPE_UNUSED,
    _FILE_SYSTEM_TYPE_FS_TYPE_VFAT as FS_TYPE_VFAT,
    _FILE_SYSTEM_TYPE_FS_TYPE_NTFS as FS_TYPE_NTFS,
    _FILE_SYSTEM_TYPE_FS_TYPE_CDFS as FS_TYPE_CDFS,
    _FILE_SYSTEM_TYPE_FS_TYPE_UDFS as FS_TYPE_UDFS,
    _FILE_SYSTEM_TYPE_FS_TYPE_EXT as FS_TYPE_EXT,
    _FILE_SYSTEM_TYPE_FS_TYPE_BTRFS as FS_TYPE_BTRFS,
    _FILE_SYSTEM_TYPE_FS_TYPE_REISERFS as FS_TYPE_REISERFS,
    _FILE_SYSTEM_TYPE_FS_TYPE_FFS as FS_TYPE_FFS,
    _FILE_SYSTEM_TYPE_FS_TYPE_FATX as FS_TYPE_FATX,
    _FS_REC_STATE_Pending as Pending,
    _FS_REC_STATE_Loaded as Loaded,
    _FS_REC_STATE_Unloading as Unloading,
};
