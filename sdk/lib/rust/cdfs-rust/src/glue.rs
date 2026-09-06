//! glue.rs — #define constants harvested from the C headers by
//! tools/rustconv/harvest_consts.py (REAL values, never LLM-guessed).
#![allow(non_upper_case_globals, dead_code)]

pub const CDFS_BUG_CHECK_ALLOCSUP: u32 = 0x00020000;
pub const CDFS_BUG_CHECK_CDDATA: u32 = 0x00040000;
pub const CDFS_BUG_CHECK_CDINIT: u32 = 0x00050000;
pub const CDFS_BUG_CHECK_CLEANUP: u32 = 0x00060000;
pub const CDFS_BUG_CHECK_CLOSE: u32 = 0x00070000;
pub const CDFS_BUG_CHECK_CREATE: u32 = 0x00080000;
pub const CDFS_BUG_CHECK_DEVCTRL: u32 = 0x00090000;
pub const CDFS_BUG_CHECK_DIRCTRL: u32 = 0x000b0000;
pub const CDFS_BUG_CHECK_DIRSUP: u32 = 0x000c0000;
pub const CDFS_BUG_CHECK_FSCTRL: u32 = 0x000f0000;
pub const CDFS_BUG_CHECK_FSPDISP: u32 = 0x00100000;
pub const CDFS_BUG_CHECK_LOCKCTRL: u32 = 0x00110000;
pub const CDFS_BUG_CHECK_NAMESUP: u32 = 0x00120000;
pub const CDFS_BUG_CHECK_PATHSUP: u32 = 0x00130000;
pub const CDFS_BUG_CHECK_PNP: u32 = 0x00140000;
pub const CDFS_BUG_CHECK_PREFXSUP: u32 = 0x00150000;
pub const CDFS_BUG_CHECK_READ: u32 = 0x00160000;
pub const CDFS_BUG_CHECK_RESRCSUP: u32 = 0x00180000;
pub const CDFS_BUG_CHECK_SHUTDOWN: u32 = 0x001e0000;
pub const CDFS_BUG_CHECK_VERFYSUP: u32 = 0x001b0000;
pub const CDFS_BUG_CHECK_VOLINFO: u32 = 0x001c0000;
pub const CDFS_BUG_CHECK_WORKQUE: u32 = 0x001d0000;
pub const CDFS_BUG_CHECK_WRITE: u32 = 0x00170000;
pub const CDFS_NTC_FCB_DATA: u32 = 0x0305;
pub const CDFS_NTC_FCB_INDEX: u32 = 0x0304;
pub const CDFS_NTC_FCB_NONPAGED: u32 = 0x0306;
pub const CDFS_NTC_FCB_PATH_TABLE: u32 = 0x0303;
pub const CDFS_NTC_IRP_CONTEXT: u32 = 0x0308;
pub const CDFS_NTC_VCB: u32 = 0x0302;
pub const CDFS_RESIDUAL_REFERENCE: u32 = 6;
pub const CD_ATTRIBUTE_ASSOC: u32 = 0x04;
pub const CD_ATTRIBUTE_DIRECTORY: u32 = 0x02;
pub const CD_ATTRIBUTE_HIDDEN: u32 = 0x01;
pub const CD_ATTRIBUTE_MULTI: u32 = 0x80;
pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;
pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
pub const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
pub const FILE_NO_INTERMEDIATE_BUFFERING: u32 = 0x00000008;
pub const FILE_OPENED: u32 = 0x00000001;
pub const FO_CLEANUP_COMPLETE: u32 = 0x00004000;
pub const FO_FILE_MODIFIED: u32 = 0x00001000;
pub const FO_SYNCHRONOUS_IO: u32 = 0x00000002;
pub const FSRTL_VOLUME_MOUNT: u32 = 6;
pub const FSRTL_VOLUME_UNLOCK: u32 = 5;
pub const IO_TYPE_VPB: u32 = 10;
pub const IRP_MN_COMPLETE: u32 = 0x04;
pub const IRP_MN_MDL: u32 = 0x02;
pub const IRP_NOCACHE: u32 = 0x00000001;
pub const IRP_PAGING_IO: u32 = 0x00000002;
pub const SECTOR_SHIFT: u32 = 11;
pub const SECTOR_SIZE: u32 = 2048;
pub const SL_ALLOW_RAW_MOUNT: u32 = 0x01;
pub const STATUS_CANT_WAIT: i32 = 0xC00000D8u32 as i32;
pub const STATUS_DISK_CORRUPT_ERROR: i32 = 0xC0000032u32 as i32;
pub const STATUS_DRIVER_INTERNAL_ERROR: i32 = 0xC0000183u32 as i32;
pub const STATUS_FILE_CLOSED: i32 = 0xC0000128u32 as i32;
pub const STATUS_FILE_CORRUPT_ERROR: i32 = 0xC0000102u32 as i32;
pub const STATUS_FILE_INVALID: i32 = 0xC0000098u32 as i32;
pub const STATUS_NOT_LOCKED: i32 = 0xC000002Au32 as i32;
pub const STATUS_NO_MORE_FILES: i32 = 0x80000006u32 as i32;
pub const STATUS_VERIFY_REQUIRED: i32 = 0x80000016u32 as i32;
pub const STATUS_VOLUME_DISMOUNTED: i32 = 0xC000026Eu32 as i32;
pub const STATUS_WRONG_VOLUME: i32 = 0xC0000012u32 as i32;
pub const SYSTEM_USE_XA_DA: u32 = 0x0040;
pub const SYSTEM_USE_XA_FORM2: u32 = 0x0010;
pub const SYSTEM_XA_SIGNATURE: u32 = 0x4158;
pub const TAG_DIRENT_NAME: u32 = 0x6e646443;  // 'nddC'
pub const TAG_VOL_DESC: u32 = 0x64766443;  // 'dvdC'
pub const TAG_VPB: u32 = 0x70766443; // nolint: const-contradiction (per-driver pool tag, each verified against its own C)
                                     // cdprocs.h:106 'pvdC'
pub const VPB_LOCKED: u32 = 0x0002;
pub const VPB_MOUNTED: u32 = 0x0001;
pub const VPB_REMOVE_PENDING: u32 = 0x0008;
pub const XA_SECTOR_SIZE: u32 = 2352;

// ---- grind-noise type aliases: the translation used the variable-naming convention
// (Fcb/Vcb/Ccb/IrpContext) as TYPE names in places; map them to the real substrate structs.
pub use crate::types::{FCB as Fcb, VCB as Vcb, CCB as Ccb, IRP_CONTEXT as IrpContext};

// cd.h pool aliases: CdPagedPool=PagedPool, CdNonPagedPool=NonPagedPoolNx (->NonPagedPool @0x502).
pub use ntkrnl_sys::ddk::{_POOL_TYPE_PagedPool as CdPagedPool, _POOL_TYPE_NonPagedPool as CdNonPagedPool};
