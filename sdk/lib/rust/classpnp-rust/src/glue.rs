//! glue.rs — #define constants harvested from the C headers by
//! tools/rustconv/harvest_consts.py (REAL values, never LLM-guessed).
#![allow(non_upper_case_globals, dead_code)]

pub const DO_VERIFY_VOLUME: u32 = 0x00000002;
pub const DUID_HARDWARE_IDS_ONLY: u32 = 0;
pub const DUID_INCLUDE_SOFTWARE_IDS: u32 = 1;
pub const FILE_DEVICE_DISK: u32 = 7;
pub const FILE_PORTABLE_DEVICE: u32 = 0x00004000;
pub const IO_WRN_FAILURE_PREDICTED: i32 = 0x80040034u32 as i32;
pub const IRP_MJ_SCSI: u32 = 0x0f;
pub const KEY_ALL_ACCESS: u32 = 0xf003f;
pub const PLUGPLAY_REGKEY_DEVICE: u32 = 1;
pub const REG_OPTION_NON_VOLATILE: u32 = 0;
pub const RTL_QUERY_REGISTRY_TYPECHECK: u32 = 0x00000100;
pub const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x02;
pub const SL_WRITE_THROUGH: u32 = 0x04;
pub const STATUS_DATA_ERROR: i32 = 0xC000003Eu32 as i32;
pub const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32;
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32;
pub const STATUS_DEVICE_PROTOCOL_ERROR: i32 = 0xC0000186u32 as i32;
pub const STATUS_INVALID_DEVICE_STATE: i32 = 0xC0000184u32 as i32;
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32;
pub const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32;
pub const STATUS_MEDIA_CHANGED: i32 = 0x8000001Cu32 as i32;
pub const STATUS_VERIFY_REQUIRED: i32 = 0x80000016u32 as i32;
pub const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
pub const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
pub const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
pub const VPB_MOUNTED: u32 = 0x0001;

// harvested:
pub const DPFLTR_INFO_LEVEL: u32 = 3;
pub const DUID_VERSION_1: u32 = 1;
pub const FILE_DEVICE_CD_ROM: u32 = 2;
pub const FILE_FLOPPY_DISKETTE: u32 = 0x00000004;
pub const IO_TYPE_IRP: u32 = 6;
pub const IO_WARNING_DISK_SURPRISE_REMOVED: i32 = 0x8004009Du32 as i32;
pub const IRP_MN_CHANGE_SINGLE_INSTANCE: u32 = 0x02;
pub const IRP_MN_CHANGE_SINGLE_ITEM: u32 = 0x03;
pub const IRP_MN_EXECUTE_METHOD: u32 = 0x09;
pub const IRP_MN_QUERY_SINGLE_INSTANCE: u32 = 0x01;
pub const IRP_MN_REGINFO: u32 = 0x08;
pub const IRP_PAGING_IO: u32 = 0x00000002;
pub const IRP_SYNCHRONOUS_PAGING_IO: u32 = 0x00000040;
pub const KEY_WRITE: u32 = 0x20006;
pub const MAXUCHAR: u32 = 0xff;
pub const MAXUSHORT: u32 = 0xffff;
pub const RTL_QUERY_REGISTRY_TYPECHECK_SHIFT: u32 = 24;
pub const SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED: u32 = 0x00000001;
pub const SCSI_SENSE_OPTIONS_NONE: u32 = 0x00000000;
pub const SL_KEY_SPECIFIED: u32 = 0x01;
pub const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32;
pub const STATUS_FT_READ_FROM_COPY: i32 = 0x40000035u32 as i32;
pub const STATUS_INTEGER_OVERFLOW: i32 = 0xC0000095u32 as i32;
pub const STATUS_INTERNAL_ERROR: i32 = 0xC00000E5u32 as i32;
pub const STATUS_INVALID_BUFFER_SIZE: i32 = 0xC0000206u32 as i32;
pub const STATUS_INVALID_LEVEL: i32 = 0xC0000148u32 as i32;
pub const STATUS_INVALID_USER_BUFFER: i32 = 0xC00000E8u32 as i32;
pub const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32;
pub const STATUS_NONEXISTENT_SECTOR: i32 = 0xC0000015u32 as i32;
pub const STATUS_TRIM_READ_ZERO_NOT_SUPPORTED: i32 = 0xC0000472u32 as i32;
pub const STATUS_WMI_GUID_NOT_FOUND: i32 = 0xC0000295u32 as i32;
pub const STATUS_WMI_INSTANCE_NOT_FOUND: i32 = 0xC0000296u32 as i32;
pub const TRACE_LEVEL_FATAL: u32 = 1; // storswtr.h:26-30
pub const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
pub const USHORT_MAX: u32 = 0xffff;
pub const VER_NT_DOMAIN_CONTROLLER: u32 = 0x0000002;
pub const VER_NT_SERVER: u32 = 0x0000003;
pub const WMIREG_ACTION_REGISTER: u32 = 1;
pub const WMIREG_FLAG_INSTANCE_LIST: u32 = 0x00000004;
pub const WMIREG_FLAG_INSTANCE_PDO: u32 = 0x00000020;
pub const WNODE_FLAG_EVENT_ITEM: u32 = 0x00000008;
pub const WNODE_FLAG_FIXED_INSTANCE_SIZE: u32 = 0x00000010;
pub const WNODE_FLAG_SINGLE_INSTANCE: u32 = 0x00000002;
pub const WNODE_FLAG_STATIC_INSTANCE_NAMES: u32 = 0x00000080;
pub const WNODE_FLAG_TOO_SMALL: u32 = 0x00000020;

// harvested:
pub const CLASS_TAG_AUTORUN_DISABLE: u32 = 0x414c6353;  // 'ALcS'
pub const CLASS_TAG_FAILURE_PREDICT: u32 = 0x664c6353;  // 'fLcS'
pub const CLASS_TAG_FILE_OBJECT_EXTENSION: u32 = 0x464c6353;  // 'FLcS'
pub const CLASS_TAG_LB_PROVISIONING: u32 = 0x504c6353;  // 'PLcS'
pub const CLASS_TAG_LOCK_TRACKING: u32 = 0x544c6353;  // 'TLcS'
pub const CLASS_TAG_MEDIA_CHANGE_DETECTION: u32 = 0x4d4c6353;  // 'MLcS'
pub const CLASS_TAG_MODE_DATA: u32 = 0x6f4c6353;  // 'oLcS'
pub const CLASS_TAG_WMI: u32 = 0x774c6353;  // 'wLcS'

// WPP_DEFINE_BIT trace flags from storswtr.h (sequential; WPP disabled in this build)
pub const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_POWER: u32 = 2; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_RW: u32 = 3; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_QUEUE: u32 = 5; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_WMI: u32 = 6; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_TIMER: u32 = 7; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_LOCK: u32 = 9; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_DEBUG1: u32 = 10; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_DEBUG2: u32 = 11; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_MCN: u32 = 12; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_ISR: u32 = 13; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_ENUM: u32 = 14; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_LOGOTEST: u32 = 15; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_DUMP: u32 = 16; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_SCSI: u32 = 17; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// Win8+ pool types absent from ntkrnl-sys POOL_TYPE enum (NonPagedPoolNx = NonPagedPool|0x200)
// classp.h: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
pub const NonPagedPoolNx: u32 = 0;
// classp.h: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
pub const NonPagedPoolNxCacheAligned: u32 = 4;
// ddk/srb.h:55 == ((UCHAR) ~0) == 0xFF.
pub const SP_UNTAGGED: u32 = 0xFF;

// --- enum-variant aliases (C members used bare/wrong-prefix; real names in types_*.rs) ---
pub use crate::types::CLASS_FUNCTION_SUPPORT_Supported as Supported;
pub use crate::types::CLASS_FUNCTION_SUPPORT_NotSupported as NotSupported;
pub use crate::types::CLASS_FUNCTION_SUPPORT_SupportUnknown as SupportUnknown;
pub use crate::types::FAILURE_PREDICTION_METHOD_FailurePredictionNone as FailurePredictionNone;

// --- IOCTL codes (CTL_CODE-computed, verified by investigator) ---
pub const IOCTL_SCSI_GET_ADDRESS: u32 = 0x00041018;
pub const IOCTL_STORAGE_ENABLE_IDLE_POWER: u32 = 0x002D1C80;
pub const IOCTL_STORAGE_FIRMWARE_GET_INFO: u32 = 0x002D1C00;
pub const IOCTL_STORAGE_PERSISTENT_RESERVE_IN: u32 = 0x002D5018;
pub const IOCTL_STORAGE_PERSISTENT_RESERVE_OUT: u32 = 0x002DD01C;
pub const IOCTL_STORAGE_POWER_ACTIVE: u32 = 0x002D1C88;
pub const IOCTL_STORAGE_POWER_IDLE: u32 = 0x002D1C8C;
pub const IOCTL_STORAGE_PREDICT_FAILURE: u32 = 0x002D1100;

// --- sizeof / arithmetic #defines as size_of (arch-correct via the size asserts) ---
pub const CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE: usize = core::mem::size_of::<crate::types::_STORAGE_REQUEST_BLOCK>() + core::mem::size_of::<crate::types::_STOR_ADDR_BTL8>() + core::mem::size_of::<crate::types::_SRBEX_DATA_SCSI_CDB16>();
pub const CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE: usize = core::mem::size_of::<crate::types::_STORAGE_REQUEST_BLOCK>() + core::mem::size_of::<crate::types::_STOR_ADDR_BTL8>();
pub const SENSE_BUFFER_SIZE_EX: usize = core::mem::size_of::<crate::types::_SENSE_DATA_EX>();
pub const STORAGE_HW_FIRMWARE_REQUEST_FLAG_CONTROLLER: u32 = 0x00000001;
pub const SCSI_DISK_DRIVER_INTERNAL: u32 = 0x2D;

// harvested:
pub const CLASSPNP_POOL_TAG_FIRMWARE: u32 = 0x77466353;  // 'wFcS'
pub const CLASSPNP_POOL_TAG_LOG_MESSAGE: u32 = 0x6d6c6353;  // 'mlcS'
pub const CLASSPNP_POOL_TAG_TOKEN_OPERATION: u32 = 0x6f546353;  // 'oTcS'
pub const CLASS_TAG_PRIVATE_DATA: u32 = 0x43506353;  // 'CPcS'
pub const CLASS_TAG_SENSE2: u32 = 0x32536353;  // '2ScS'
pub const CLASS_TAG_WORKING_SET: u32 = 0x73576353;  // 'sWcS'
pub const DISK_SPINUP_BACKTRACE_LENGTH: u32 = 0x18;
pub const NUMBER_OF_DISK_SPINUP_TRACES: u32 = 0x10;

// C min()/max() macros — generic; same-type args resolve here, mixed-type calls still
// surface as E0308 (a real translation cast the LLM owes), but the name resolves.
#[inline] pub fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline] pub fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }

// c2rust-named enum consts (the anchor emitted <ENUMTAG>_<Variant>; bindgen names differ).
// Aliases keep the LLM's reference site unchanged while pointing at the real bindgen const.
pub use crate::types::CLASS_FUNCTION_SUPPORT_NotSupported as _CLASS_FUNCTION_SUPPORT_STATE_NotSupported;
pub use crate::types::CLASS_FUNCTION_SUPPORT_Supported as _CLASS_FUNCTION_SUPPORT_STATE_Supported;
pub use crate::types::MEDIA_LOCK_TYPE_SecureMediaLock as _MEDIA_LOCK_TYPE_SecureMediaLock;
pub use crate::types::FAILURE_PREDICTION_METHOD_FailurePredictionNone as _PREDICT_FAILURE_METHOD_FailurePredictionNone;
pub use crate::types::_SRBEXDATATYPE_SrbExDataTypePnP as _SRBEX_DATA_TYPE_SrbExDataTypePnP;
pub use crate::types::_SRBEXDATATYPE_SrbExDataTypePower as _SRBEX_DATA_TYPE_SrbExDataTypePower;
pub use crate::types::_SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdb16;
pub use crate::types::_SRBEXDATATYPE_SrbExDataTypeScsiCdb32 as _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdb32;
pub use crate::types::_SRBEXDATATYPE_SrbExDataTypeScsiCdbVar as _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdbVar;
pub use crate::types::_SRBEXDATATYPE_SrbExDataTypeWmi as _SRBEX_DATA_TYPE_SrbExDataTypeWmi;
// STORAGE_QUERY_TYPE enum (ntddstor.h) not emitted by bindgen here; faithful literal values.
pub const _STORAGE_PROPERTY_ID_PropertyStandardQuery: u32 = 0;
pub const _STORAGE_PROPERTY_ID_PropertyExistsQuery: u32 = 1;

// --- IOCTLs used as match arms in ClassDeviceControl (class_part_b.rs) that had NO
//     definition -> bind-all catch-alls (first arm swallowed EVERY ioctl; found via the
//     USB-disk START 0xC000000D, 2026-07-06). CTL_CODE-resolved from the C headers.
pub const IOCTL_MOUNTDEV_QUERY_UNIQUE_ID: u32 = 0x004D0000;          // CTL_CODE('M',0,BUFFERED,ANY)  [ddk/mountdev.h]
pub const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: u32 = 0x004D0008;        // CTL_CODE('M',2,BUFFERED,ANY)  [ddk/mountmgr.h]
pub const IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME: u32 = 0x004D000C; // CTL_CODE('M',3,BUFFERED,ANY) [ddk/mountdev.h]
pub const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x002D1080;         // CTL_CODE(0x2D,0x420,BUFFERED,ANY)   [psdk/winioctl.h]
pub const IOCTL_STORAGE_FIND_NEW_DEVICES: u32 = 0x002D4818;          // CTL_CODE(0x2D,0x206,BUFFERED,READ)  [psdk/winioctl.h]
pub const IOCTL_STORAGE_GET_HOTPLUG_INFO: u32 = 0x002D0C14;          // CTL_CODE(0x2D,0x305,BUFFERED,ANY)   [psdk/ntddstor.h]
pub const IOCTL_STORAGE_SET_HOTPLUG_INFO: u32 = 0x002DCC18;          // CTL_CODE(0x2D,0x306,BUFFERED,RW)    [psdk/ntddstor.h]
pub const IOCTL_STORAGE_MCN_CONTROL: u32 = 0x002D0944;               // CTL_CODE(0x2D,0x251,BUFFERED,ANY)   [psdk/winioctl.h]
pub const IOCTL_STORAGE_READ_CAPACITY: u32 = 0x002D5140;             // CTL_CODE(0x2D,0x450,BUFFERED,READ)  [psdk/ntddstor.h]
pub const IOCTL_STORAGE_EVENT_NOTIFICATION: u32 = 0x002D1C90;        // CTL_CODE(0x2D,0x724,BUFFERED,ANY)   [psdk/ntddstor.h]
pub const IOCTL_STORAGE_CHECK_PRIORITY_HINT_SUPPORT: u32 = 0x002D1880; // CTL_CODE(0x2D,0x620,BUFFERED,ANY) [psdk/ntddstor.h]
pub const IOCTL_STORAGE_FIRMWARE_DOWNLOAD: u32 = 0x002DDC04;         // CTL_CODE(0x2D,0x701,BUFFERED,RW)    [psdk/ntddstor.h]
pub const IOCTL_STORAGE_GET_LB_PROVISIONING_MAP_RESOURCES: u32 = 0x002D5408; // CTL_CODE(0x2D,0x502,BUFFERED,READ) [psdk/ntddstor.h]
pub const IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES: u32 = 0x002D9404; // CTL_CODE(0x2D,0x501,BUFFERED,WRITE) [psdk/ntddstor.h]
// GESN notification-class codes matched against NOTIFICATION_EVENT_STATUS_HEADER
// .NotificationClass() (UCHAR) in autorun.rs -- same bind-all hazard.
pub const NOTIFICATION_OPERATIONAL_CHANGE_CLASS_EVENTS: u8 = 0x1;    // [ddk/scsi.h]
pub const NOTIFICATION_EXTERNAL_REQUEST_CLASS_EVENTS: u8 = 0x3;      // [ddk/scsi.h]
pub const NOTIFICATION_MEDIA_STATUS_CLASS_EVENTS: u8 = 0x4;          // [ddk/scsi.h]
pub const NOTIFICATION_DEVICE_BUSY_CLASS_EVENTS: u8 = 0x6;           // [ddk/scsi.h]

/// classp.h spells the CLASSP_REG_* registry names as WIDE literals
/// (`#define CLASSP_REG_SUBKEY_NAME (L"Classpnp")`), but the type substrate
/// (`crate::types`) carries them as ASCII byte strings. Call sites that did
/// `CLASSP_REG_X.as_ptr() as PWSTR` handed the registry "Classpnp\0" read as
/// UTF-16 (0x6C43 0x7361 ...): every override lookup silently kept its default
/// and ClassSetDeviceParameter created a junk-named subkey. These are the
/// UTF-16 forms, same names, for those sites. (power.rs / clntirp.rs /
/// autorun.rs already carry their own wide copies.)
pub mod regw {
    const fn widen<const N: usize>(s: &[u8; N]) -> [u16; N] {
        let mut out = [0u16; N];
        let mut i = 0;
        while i < N {
            out[i] = s[i] as u16;
            i += 1;
        }
        out
    }
    pub static CLASSP_REG_SUBKEY_NAME: [u16; crate::types::CLASSP_REG_SUBKEY_NAME.len()] = widen(crate::types::CLASSP_REG_SUBKEY_NAME);
    pub static CLASSP_REG_REMOVAL_POLICY_VALUE_NAME: [u16; crate::types::CLASSP_REG_REMOVAL_POLICY_VALUE_NAME.len()] = widen(crate::types::CLASSP_REG_REMOVAL_POLICY_VALUE_NAME);
    pub static CLASSP_REG_WRITE_CACHE_VALUE_NAME: [u16; crate::types::CLASSP_REG_WRITE_CACHE_VALUE_NAME.len()] = widen(crate::types::CLASSP_REG_WRITE_CACHE_VALUE_NAME);
    pub static CLASSP_REG_PERF_RESTORE_VALUE_NAME: [u16; crate::types::CLASSP_REG_PERF_RESTORE_VALUE_NAME.len()] = widen(crate::types::CLASSP_REG_PERF_RESTORE_VALUE_NAME);
    pub static CLASSP_REG_QERR_OVERRIDE_MODE: [u16; crate::types::CLASSP_REG_QERR_OVERRIDE_MODE.len()] = widen(crate::types::CLASSP_REG_QERR_OVERRIDE_MODE);
    pub static CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED: [u16; crate::types::CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED.len()] = widen(crate::types::CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED);
    pub static CLASSP_REG_DISBALE_IDLE_POWER_NAME: [u16; crate::types::CLASSP_REG_DISBALE_IDLE_POWER_NAME.len()] = widen(crate::types::CLASSP_REG_DISBALE_IDLE_POWER_NAME);
    pub static CLASSP_REG_COPY_OFFLOAD_MAX_TARGET_DURATION: [u16; crate::types::CLASSP_REG_COPY_OFFLOAD_MAX_TARGET_DURATION.len()] = widen(crate::types::CLASSP_REG_COPY_OFFLOAD_MAX_TARGET_DURATION);
}
