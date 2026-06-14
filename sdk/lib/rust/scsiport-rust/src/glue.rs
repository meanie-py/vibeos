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
// IRP_MN_* PnP minor codes missing from ntkrnl_sys::driver_ddk — without these the
// match arms in fdo/pdo silently became catch-all BINDINGS (the pciidex hazard).
// Values verified vs sdk/include/xdk/iotypes.h.
pub const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
pub const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
pub const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
pub const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
pub const IRP_MN_FILTER_RESOURCE_REQUIREMENTS: u8 = 0x0D;
pub const IRP_MN_READ_CONFIG: u8 = 0x0F;
pub const IRP_MN_WRITE_CONFIG: u8 = 0x10;
pub const IRP_MN_EJECT: u8 = 0x11;
pub const IRP_MN_SET_LOCK: u8 = 0x12;
pub const IRP_MN_QUERY_ID: u8 = 0x13;
pub const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;
pub const IRP_MN_QUERY_BUS_INFORMATION: u8 = 0x15;
pub const IRP_MN_DEVICE_USAGE_NOTIFICATION: u8 = 0x16;
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

// ── scsi.rs dup-const sweep (2026-06-12): the module had 3 conflicting local copies of
// many of these (e.g. SRB_STATUS_SUCCESS 0x00 vs 0x01, SP_UNTAGGED 0, LUNEX_NEED_REQUEST_
// SENSE 0x01/0x02 vs real 0x0004). Values below verified against sdk/include/ddk/srb.h,
// scsi.h, scsiport.h, psdk/ntstatus.h — NOT taken from the LLM output.
pub const SRB_STATUS_PENDING: u8 = 0x00;
pub const SRB_STATUS_ABORTED: u8 = 0x02;
pub const SRB_STATUS_ABORT_FAILED: u8 = 0x03;
pub const SRB_STATUS_ERROR: u8 = 0x04;
pub const SRB_STATUS_INVALID_REQUEST: u8 = 0x06;
pub const SRB_STATUS_INVALID_PATH_ID: u8 = 0x07;
pub const SRB_STATUS_TIMEOUT: u8 = 0x09;
pub const SRB_STATUS_COMMAND_TIMEOUT: u8 = 0x0B;
pub const SRB_STATUS_BUS_RESET: u8 = 0x0E;
pub const SRB_STATUS_PARITY_ERROR: u8 = 0x0F;
pub const SRB_STATUS_REQUEST_SENSE_FAILED: u8 = 0x10;
pub const SRB_STATUS_NO_HBA: u8 = 0x11;
pub const SRB_STATUS_INVALID_LUN: u8 = 0x20;
pub const SRB_STATUS_INVALID_TARGET_ID: u8 = 0x21;
// SRB_FUNCTION_* as u8 to match the SRB.Function field (srb.h values) — match arms use
// these bare. The bindgen u32 copies in types_*.rs are deliberately NOT imported.
pub const SRB_FUNCTION_CLAIM_DEVICE: u8 = 0x01;
pub const SRB_FUNCTION_IO_CONTROL: u8 = 0x02;
pub const SRB_FUNCTION_RELEASE_QUEUE: u8 = 0x04;
pub const SRB_FUNCTION_ATTACH_DEVICE: u8 = 0x05;
pub const SRB_FUNCTION_RELEASE_DEVICE: u8 = 0x06;
pub const SRB_FUNCTION_SHUTDOWN: u8 = 0x07;
pub const SRB_FUNCTION_FLUSH: u8 = 0x08;
pub const SRB_FUNCTION_ABORT_COMMAND: u8 = 0x10;
pub const SRB_FUNCTION_RESET_BUS: u8 = 0x12;
pub const SRB_FUNCTION_FLUSH_QUEUE: u8 = 0x15;
pub const SRB_FUNCTION_LOCK_QUEUE: u8 = 0x18;
pub const SRB_FUNCTION_UNLOCK_QUEUE: u8 = 0x19;
pub const SRB_FLAGS_DISABLE_DISCONNECT: u32 = 0x00000004;
pub const SRB_FLAGS_BYPASS_FROZEN_QUEUE: u32 = 0x00000010;
pub const SRB_FLAGS_DATA_OUT: u32 = 0x00000080;
pub const SRB_FLAGS_UNSPECIFIED_DIRECTION: u32 = 0x000000C0; // DATA_IN | DATA_OUT
pub const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x00000100;
pub const SRB_FLAGS_BYPASS_LOCKED_QUEUE: u32 = 0x00080000;
pub const SCSI_PORT_RESET_REQUEST: u32 = 0x00100;
pub const SCSI_PORT_DISABLE_INTERRUPTS: u32 = 0x04000;
pub const LUNEX_FROZEN_QUEUE: u32 = 0x0001;
pub const LUNEX_NEED_REQUEST_SENSE: u32 = 0x0004;
pub const SCSISTAT_GOOD: u8 = 0x00;
pub const SCSISTAT_CHECK_CONDITION: u8 = 0x02;
pub const SCSISTAT_QUEUE_FULL: u8 = 0x28;
pub const SCSIOP_REQUEST_SENSE: u8 = 0x03;
pub const SCSIOP_READ: u8 = 0x28;
pub const SCSIOP_WRITE: u8 = 0x2A;
pub const SCSI_MAXIMUM_TARGETS: u32 = 8;
pub const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32;
pub const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32;
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32; // LLM had 0xC000009A (= INSUFFICIENT_RESOURCES)
pub const STATUS_NO_SUCH_DEVICE: i32 = 0xC000000Eu32 as i32;
pub const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32;
// SCSI_NOTIFICATION_TYPE bare enum-variant aliases (bindgen prefixes them with the enum
// name; the C source uses the bare variants). Aliased to the bindgen consts — compiler-
// exact, no hand-copied values (recall would have dropped NextLuRequest and shifted the
// tail by one).
pub use crate::types::{
    _SCSI_NOTIFICATION_TYPE_RequestComplete as RequestComplete,
    _SCSI_NOTIFICATION_TYPE_NextRequest as NextRequest,
    _SCSI_NOTIFICATION_TYPE_NextLuRequest as NextLuRequest,
    _SCSI_NOTIFICATION_TYPE_ResetDetected as ResetDetected,
    _SCSI_NOTIFICATION_TYPE_CallDisableInterrupts as CallDisableInterrupts,
    _SCSI_NOTIFICATION_TYPE_CallEnableInterrupts as CallEnableInterrupts,
    _SCSI_NOTIFICATION_TYPE_RequestTimerCall as RequestTimerCall,
    _SCSI_NOTIFICATION_TYPE_BusChangeDetected as BusChangeDetected,
};

// Bare wdm enum-variant aliases (kernel bindgen prefixes variants with the enum
// name; the C source uses them bare). Resolved against ntkrnl_sys bindgen — exact.
pub use ntkrnl_sys::ddk::{_BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs, _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID, _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs, _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID, _DEVICE_RELATION_TYPE_BusRelations as BusRelations, _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation, _DEVICE_TEXT_TYPE_DeviceTextDescription as DeviceTextDescription, _DEVICE_TEXT_TYPE_DeviceTextLocationInformation as DeviceTextLocationInformation, _KINTERRUPT_MODE_LevelSensitive as LevelSensitive, _KINTERRUPT_MODE_Latched as Latched, _INTERFACE_TYPE_MicroChannel as MicroChannel, _INTERFACE_TYPE_Eisa as Eisa, _INTERFACE_TYPE_Isa as Isa, _INTERFACE_TYPE_Internal as Internal, _INTERFACE_TYPE_PCIBus as PCIBus, _BUS_DATA_TYPE_PCIConfiguration as PCIConfiguration, _IO_ALLOCATION_ACTION_KeepObject as KeepObject, _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as DeallocateObjectKeepRegisters, _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as KeyValueFullInformation, _CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive, _CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared, _DMA_WIDTH_Width8Bits as Width8Bits, _DMA_WIDTH_Width16Bits as Width16Bits, _DMA_WIDTH_Width32Bits as Width32Bits};

// PCI_COMMON_CONFIG — the kernel bindgen DROPS the macro-spliced header fields (known
// bug, see feedback_bindgen_drops_macro_spliced_struct_fields / pcix): ddk::_PCI_COMMON_
// CONFIG is header-less. Faithful 256-byte type-0 layout, hand-checked vs pcitypes.h.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PCI_COMMON_CONFIG_FIXED {
    pub VendorID: u16,
    pub DeviceID: u16,
    pub Command: u16,
    pub Status: u16,
    pub RevisionID: u8,
    pub ProgIf: u8,
    pub SubClass: u8,
    pub BaseClass: u8,
    pub CacheLineSize: u8,
    pub LatencyTimer: u8,
    pub HeaderType: u8,
    pub BIST: u8,
    pub BaseAddresses: [u32; 6],
    pub CIS: u32,
    pub SubVendorID: u16,
    pub SubSystemID: u16,
    pub ROMBaseAddress: u32,
    pub CapabilitiesPtr: u8,
    pub Reserved1: [u8; 3],
    pub Reserved2: u32,
    pub InterruptLine: u8,
    pub InterruptPin: u8,
    pub MinimumGrant: u8,
    pub MaximumLatency: u8,
    pub DeviceSpecific: [u8; 192],
}
const _: () = assert!(core::mem::size_of::<PCI_COMMON_CONFIG_FIXED>() == 256);

// dprint_noop — sink for prints the C compiled out (NDEBUG DPRINT). Keeps the call
// sites (and their argument expressions) intact while matching C's silent behavior;
// per-IO DbgPrint over the serial port was slowing boot past the smoke timeout.
pub unsafe extern "C" fn dprint_noop(_format: *const core::ffi::c_char, mut _args: ...) -> i32 { 0 }

// sync_exec — KeSynchronizeExecution that tolerates a no-interrupt adapter.
//
// `KeSynchronizeExecution` (ntoskrnl) dereferences `Interrupt->SynchronizeIrql`
// unconditionally; the C scsiport (scsi.c:1288/1778, scsiport.c StartIo, etc.)
// passes `DeviceExtension->Interrupt[0]` raw, relying on the IRQ always
// connecting. When `IoConnectInterrupt` fails for an adapter — e.g. the empty
// secondary IDE channel once the Winsock catalog grows the system hive and
// perturbs IRQ arbitration — `Interrupt[0]` is NULL and the scan path
// (SpiStartReq -> KeSynchronizeExecution) NULL-derefs in KeSynchronizeExecution+0x18
// -> BUGCHECK 0x1E (STATUS_IN_PAGE_ERROR). With no ISR there is nothing to
// synchronize against, so run the routine directly at the caller's IRQL; the
// device spinlock the caller already holds supplies the serialization. The
// no-interrupt adapter then completes via the IoTimer poll path. See task 79 /
// docs/engineering/translation-debugging-field-manual.md §9.9.
#[inline]
pub unsafe fn sync_exec(
    interrupt: ntkrnl_sys::ddk::PKINTERRUPT,
    routine: ntkrnl_sys::ddk::PKSYNCHRONIZE_ROUTINE,
    context: ntkrnl_sys::ddk::PVOID,
) -> ntkrnl_sys::ddk::BOOLEAN {
    if interrupt.is_null() {
        match routine {
            Some(f) => f(context),
            None => 0,
        }
    } else {
        ntkrnl_sys::ddk::KeSynchronizeExecution(interrupt, routine, context)
    }
}
