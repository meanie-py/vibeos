//! glue.rs — #define constants + enum-variant aliases + driver globals the scsiport C
//! source references by bare name. REAL values (clang-computed / bindgen substrate).
#![allow(non_upper_case_globals, dead_code, non_camel_case_types)]
use ntkrnl_sys::ddk::ULONG;

// ── clang-computed #define consts (from scsi.h/srb.h/cmtypes via -dM dump) ──
pub const CM_RESOURCE_DMA_16: u32 = 1;
pub const CM_RESOURCE_DMA_32: u32 = 2;
pub const CM_RESOURCE_DMA_8: u32 = 0;
pub const CM_RESOURCE_DMA_8_AND_16: u32 = 4;
pub const CM_RESOURCE_INTERRUPT_LATCHED: u32 = 1;
pub const CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE: u32 = 0;
pub const CM_RESOURCE_MEMORY_READ_WRITE: u32 = 0;
pub const CM_RESOURCE_PORT_IO: u32 = 1;
pub const CmResourceTypeDma: u8 = 4;
pub const CmResourceTypeInterrupt: u8 = 2;
pub const CmResourceTypeMemory: u8 = 3;
pub const CmResourceTypePort: u8 = 1;
pub const FILE_DEVICE_CONTROLLER: u32 = 4;
pub const FILE_DEVICE_DISK: u32 = 7;
pub const FILE_WORD_ALIGNMENT: u32 = 1;
pub const IRP_MJ_SCSI: u8 = 15;
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 9;
pub const SP_RETURN_FOUND: u32 = 1;
pub const SP_RETURN_NOT_FOUND: u32 = 0;
pub const SRBEV_SCSI_ASYNC_NOTIFICATION: u32 = 2;
pub const SRB_FLAGS_DATA_IN: u32 = 64;
pub const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 8;
pub const SRB_FUNCTION_EXECUTE_SCSI: u8 = 0;
pub const SRB_STATUS_AUTOSENSE_VALID: u8 = 128;
pub const SRB_STATUS_BAD_FUNCTION: u8 = 34;
pub const SRB_STATUS_BAD_SRB_BLOCK_LENGTH: u8 = 21;
pub const SRB_STATUS_DATA_OVERRUN: u8 = 18;
pub const SRB_STATUS_NO_DEVICE: u8 = 8;
pub const SRB_STATUS_QUEUE_FROZEN: u8 = 64;
pub const SRB_STATUS_SELECTION_TIMEOUT: u8 = 10;
pub const SRB_STATUS_SUCCESS: u8 = 1;
// manual (cast-wrapped macros the resolver skipped):
pub const SP_UNTAGGED: u8 = 0xFF;            // ((UCHAR)~0)
pub const STATUS_ADAPTER_HARDWARE_ERROR: i32 = 0xC00000C2u32 as i32;
pub const STATUS_INTERNAL_ERROR: i32 = 0xC00000E5u32 as i32;
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32;
// miniport debug-print threshold global (scsiport.c: ULONG InternalDebugLevel = 0;)
pub static mut InternalDebugLevel: ULONG = 0;
pub type LPVOID = *mut core::ffi::c_void;

// ── driver-private #defines that the LLM redefined (often with WRONG/placeholder values)
// in multiple modules → E0428. Single source of truth here, real values from scsiport.h
// + the clang -dM dump. Local redefs stripped from the modules.
pub const TAG_SCSIPORT: u32 = 0x49534353;            // 'ISCS'
pub const SP_UNINITIALIZED_VALUE: u32 = 0xFFFFFFFF;  // ((ULONG)~0)
pub const SCSI_PORT_RESET: u32 = 0x00080;
pub const SCSI_PORT_NOTIFICATION_NEEDED: u32 = 0x00004;
pub const SRB_FLAGS_IS_ACTIVE: u32 = 0x00010000;
pub const SCSI_MAXIMUM_PHYSICAL_BREAKS: u32 = 255;
pub const SCSI_MINIMUM_PHYSICAL_BREAKS: u32 = 16;
pub const UNICODE_NULL: u16 = 0;

// STORAGE_PROPERTY_ID / STORAGE_QUERY_TYPE / STORAGE_BUS_TYPE enum values (ntddstor.h —
// not in the substrate; verified from the header, NOT guessed).
pub const StorageDeviceProperty: u32 = 0;
pub const StorageAdapterProperty: u32 = 1;
pub const StorageDeviceIdProperty: u32 = 2;
pub const PropertyStandardQuery: u32 = 0;
pub const PropertyExistsQuery: u32 = 1;
pub const BusTypeScsi: u32 = 1;

// GUID_DEVINTERFACE_STORAGEPORT — defined as data in guid.c (the kept C TU, DEFINE_GUID);
// reference it as an extern (faithful: the Rust + C cohab share the one GUID symbol).
extern "C" {
    pub static GUID_DEVINTERFACE_STORAGEPORT: ntkrnl_sys::ddk::GUID;
}
pub const SRB_STATUS_BUSY: u8 = 0x05;
pub const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32;
