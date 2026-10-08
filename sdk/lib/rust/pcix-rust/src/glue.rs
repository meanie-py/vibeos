//! glue.rs — #define constants harvested from the C headers by
//! tools/rustconv/harvest_consts.py (REAL values, never LLM-guessed).
#![allow(non_upper_case_globals, dead_code)]

use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR};

/// CmiGetNextPartialDescriptor — FORCEINLINE from sdk/include/reactos/drivers/cmreslist.h.
/// Faithful translation: advance past one CM_PARTIAL_RESOURCE_DESCRIPTOR, adding the
/// variable DeviceSpecificData.DataSize when Type == CmResourceTypeDeviceSpecific (5).
#[inline]
pub unsafe fn CmiGetNextPartialDescriptor(
    PartialDescriptor: *const CM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> PCM_PARTIAL_RESOURCE_DESCRIPTOR {
    // Assume the descriptors are the fixed size ones.
    let mut NextDescriptor = PartialDescriptor.add(1) as usize;
    // But check if this is actually a variable-sized descriptor.
    if (*PartialDescriptor).Type == 5
    /* CmResourceTypeDeviceSpecific */
    {
        NextDescriptor += (*PartialDescriptor).u.DeviceSpecificData.DataSize as usize;
    }
    NextDescriptor as PCM_PARTIAL_RESOURCE_DESCRIPTOR
}

pub const ACPI_EVAL_INPUT_BUFFER_SIGNATURE: u32 = 0x42696541;  // 'BieA'
pub const ACPI_METHOD_ARGUMENT_INTEGER: u32 = 0x0;
pub const AGP_BUS_INTERFACE_V1: u32 = 1;
pub const PCI_POOL_TAG: u32 = 0x42696350;  // 'BicP'
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32;
pub const STATUS_DEVICE_POWER_FAILURE: i32 = 0xC000009Eu32 as i32;
pub const STATUS_DEVICE_PROTOCOL_ERROR: i32 = 0xC0000186u32 as i32;
pub const STATUS_FAIL_CHECK: i32 = 0xC0000229u32 as i32;
pub const STATUS_INVALID_DEVICE_STATE: i32 = 0xC0000184u32 as i32;
pub const STATUS_NOINTERFACE: i32 = 0xC00002B9u32 as i32;
pub const STATUS_NO_SUCH_DEVICE: i32 = 0xC000000Eu32 as i32;

// harvested:
pub const CM_RESOURCE_MEMORY_PREFETCHABLE: u32 = 0x0004;
pub const CM_RESOURCE_MEMORY_READ_ONLY: u32 = 0x0001;
pub const CM_RESOURCE_MEMORY_READ_WRITE: u32 = 0x0000;
pub const CM_RESOURCE_PORT_10_BIT_DECODE: u32 = 0x0004;
pub const CM_RESOURCE_PORT_IO: u32 = 0x0001;
pub const CM_RESOURCE_PORT_POSITIVE_DECODE: u32 = 0x0020;
pub const CM_RESOURCE_PORT_WINDOW_DECODE: u32 = 0x0080;
pub const CmResourceTypeBusNumber: u32 = 6;
pub const CmResourceTypeInterrupt: u32 = 2;
pub const CmResourceTypeMemory: u32 = 3;
// xdk/cmtypes.h: CmResourceTypeMemoryLarge, the CM_RESOURCE_MEMORY_LARGE_* flags, and the
// CM_SHARE_DISPOSITION values (enum: Undetermined=0, DeviceExclusive, DriverExclusive, Shared).
// Hand-added: hal_ddk has them but only on x86/x64, and pcix builds for arm64 too.
pub const CmResourceTypeMemoryLarge: u32 = 7;
pub const CM_RESOURCE_MEMORY_LARGE: u32 = 0x0E00;
pub const CM_RESOURCE_MEMORY_LARGE_40: u32 = 0x0200;
pub const CM_RESOURCE_MEMORY_LARGE_48: u32 = 0x0400;
pub const CM_RESOURCE_MEMORY_LARGE_64: u32 = 0x0800;
pub const CmResourceShareDeviceExclusive: u8 = 1;
pub const CmResourceShareDriverExclusive: u8 = 2;
pub const CmResourceTypeNull: u32 = 0;
pub const CmResourceTypePort: u32 = 1;
pub const FILE_DEVICE_BUS_EXTENDER: u32 = 42;
pub const IRP_MN_QUERY_CAPABILITIES: u32 = 0x09;
pub const IRP_MN_QUERY_INTERFACE: u32 = 0x08;
pub const IRP_MN_QUERY_LEGACY_BUS_INFORMATION: u32 = 0x18;
pub const KEY_ALL_ACCESS: u32 = 0xf003f;
pub const KEY_WRITE: u32 = 0x20006;
pub const MESSAGE_RESOURCE_UNICODE: u32 = 1;
pub const PCI_ADDRESS_IO_ADDRESS_MASK: u32 = 0xfffffffc;
pub const PCI_ADDRESS_IO_SPACE: u32 = 0x00000001;
pub const PCI_ADDRESS_MEMORY_ADDRESS_MASK: u32 = 0xfffffff0;
pub const PCI_ADDRESS_MEMORY_PREFETCHABLE: u32 = 0x00000008;
pub const PCI_ADDRESS_MEMORY_TYPE_MASK: u32 = 0x00000006;
pub const PCI_ADDRESS_ROM_ADDRESS_MASK: u32 = 0xfffff800;
pub const PCI_BRIDGE_TYPE: u32 = 0x01;
pub const PCI_BUS_INTERFACE_STANDARD_VERSION: u32 = 1;
pub const PCI_CAPABILITY_ID_AGP: u32 = 0x02;
pub const PCI_CAPABILITY_ID_AGP_TARGET: u32 = 0x0E;
pub const PCI_CAPABILITY_ID_PCIX: u32 = 0x07;
pub const PCI_CAPABILITY_ID_POWER_MANAGEMENT: u32 = 0x01;
pub const PCI_CARDBUS_BRIDGE_TYPE: u32 = 0x02;
pub const PCI_CB_INTRF_VERSION: u32 = 1;
pub const PCI_CLASS_BASE_SYSTEM_DEV: u32 = 0x08;
pub const PCI_CLASS_BRIDGE_DEV: u32 = 0x06;
pub const PCI_CLASS_DISPLAY_CTLR: u32 = 0x03;
pub const PCI_CLASS_MASS_STORAGE_CTLR: u32 = 0x01;
pub const PCI_CLASS_NOT_DEFINED: u32 = 0xff;
pub const PCI_CLASS_PRE_20: u32 = 0x00;
pub const PCI_DEVICE_PRESENT_INTERFACE_VERSION: u32 = 1;
pub const PCI_ENABLE_BRIDGE_ISA: u32 = 0x0004;
pub const PCI_ENABLE_BRIDGE_VGA: u32 = 0x0008;
pub const PCI_ENABLE_BUS_MASTER: u32 = 0x0004;
pub const PCI_ENABLE_IO_SPACE: u32 = 0x0001;
pub const PCI_ENABLE_MEMORY_SPACE: u32 = 0x0002;
pub const PCI_ENABLE_WRITE_AND_INVALIDATE: u32 = 0x0010;
pub const PCI_HACK_BROKEN_SUBTRACTIVE_DECODE: u64 = 0x0000008000000000;
pub const PCI_HACK_CB_SHARE_CMD_BITS: u64 = 0x0000000000010000;
pub const PCI_HACK_CRITICAL_DEVICE: u64 = 0x0000002000000000;
pub const PCI_HACK_DISABLE_IDE_NATIVE_MODE: u64 = 0x0000000800000000;
// Upstream 4dabb9e5825 renamed 0x40000000 from PCI_HACK_DONT_DISABLE_DECODES to
// PCI_HACK_NEVER_POWER_DOWN and gave 0x2 / 0x4000000000 their meanings
// (sdk/include/reactos/drivers/pci/pci.h). Hand-added, values checked against that header.
pub const PCI_HACK_NEVER_POWER_DOWN: u64 = 0x0000000040000000;
pub const PCI_HACK_KEEP_DECODES_ON_STOP: u64 = 0x0000000000000002;
pub const PCI_HACK_NOT_CRITICAL_DEVICE: u64 = 0x0000004000000000;
pub const PCI_HACK_NEVER_DISCONNECT: u64 = 0x0000000000000040;
pub const PCI_HACK_DONT_DISABLE: u64 = 0x0000000000000080;
pub const PCI_HACK_DOUBLE_DECKER: u64 = 0x0000000000001000;
pub const PCI_HACK_ENUM_NO_RESOURCE: u64 = 0x0000000000000010;
pub const PCI_HACK_FAKE_CLASS_CODE: u64 = 0x0000000200000000;
pub const PCI_HACK_NO_ENUM_AT_ALL: u64 = 0x0000000000000008;
pub const PCI_HACK_NO_PM_CAPS: u64 = 0x0000000020000000;
pub const PCI_HACK_NO_REVISION_AFTER_D3: u64 = 0x0000010000000000;
pub const PCI_HACK_NO_SUBSYSTEM: u64 = 0x0000000000400000;
pub const PCI_HACK_NO_SUBSYSTEM_AFTER_D3: u64 = 0x0000000080000000;
pub const PCI_HACK_ONE_CHILD: u64 = 0x0000000000002000;
pub const PCI_HACK_PRESERVE_COMMAND: u64 = 0x0000000000004000;
pub const PCI_HACK_VIDEO_LEGACY_DECODE: u64 = 0x0000000100000000;
pub const PCI_INT_ROUTE_INTRF_STANDARD_VER: u32 = 1;
pub const PCI_INVALID_VENDORID: u32 = 0xFFFF;
pub const PCI_MAX_DEVICES: u32 = 32;
pub const PCI_MAX_FUNCTION: u32 = 8;
pub const PCI_PME_INTRF_STANDARD_VER: u32 = 1;
pub const PCI_ROMADDRESS_ENABLED: u32 = 0x00000001;
pub const PCI_STATUS_CAPABILITIES_LIST: u32 = 0x0010;
pub const PCI_SUBCLASS_BR_CARDBUS: u32 = 0x07;
pub const PCI_SUBCLASS_BR_EISA: u32 = 0x02;
pub const PCI_SUBCLASS_BR_HOST: u32 = 0x00;
pub const PCI_SUBCLASS_BR_ISA: u32 = 0x01;
pub const PCI_SUBCLASS_BR_MCA: u32 = 0x03;
pub const PCI_SUBCLASS_BR_OTHER: u32 = 0x80;
pub const PCI_SUBCLASS_BR_PCI_TO_PCI: u32 = 0x04;
pub const PCI_SUBCLASS_MSC_IDE_CTLR: u32 = 0x01;
pub const PCI_SUBCLASS_PRE_20_VGA: u32 = 0x01;
pub const PCI_SUBCLASS_SYS_INTERRUPT_CTLR: u32 = 0x00;
pub const PCI_SUBCLASS_SYS_OTHER: u32 = 0x80;
pub const PCI_SUBCLASS_VID_VGA_CTLR: u32 = 0x00;
pub const PCI_TYPE0_ADDRESSES: u32 = 6;
pub const PCI_TYPE1_ADDRESSES: u32 = 2;
pub const PCI_TYPE_20BIT: u32 = 2;
pub const PCI_TYPE_64BIT: u32 = 4;
pub const PF_PAE_ENABLED: u32 = 9;
pub const PLUGPLAY_REGKEY_DEVICE: u32 = 1;
pub const PNP_LOCATION_INTERFACE_VERSION: u32 = 1;
pub const REG_FULL_RESOURCE_DESCRIPTOR: u32 = 9;
pub const REG_OPTION_VOLATILE: u32 = 1;
pub const RTL_RANGE_LIST_ADD_IF_CONFLICT: u32 = 0x00000001;
pub const VER_SUITE_DATACENTER: u32 = 0x00000080;

// Hand-added for upstream's ECAM (#9542), Express (#9543), resizable BAR and hot-plug
// (#9592) work. hal_ddk carries these only on x86/x64 and pcix builds for arm64 too;
// values checked against xdk/iotypes.h, xdk/ntbasedef.h, psdk/winnt.h, ntstatus.h and
// sdk/include/reactos/drivers/pci/pci.h.
pub const MAXUCHAR: u32 = 0xff;
pub const PCI_EXTENDED_CONFIG_LENGTH: u32 = 0x1000;
pub const PCI_MAX_BRIDGE_NUMBER: u32 = 0xFF;
pub const PCI_CAPABILITY_ID_PCI_EXPRESS: u32 = 0x10;
pub const PCI_DEVICE_TYPE: u32 = 0x00;
pub const PCI_ENABLE_PARITY: u32 = 0x0040;
pub const PCI_ENABLE_SERR: u32 = 0x0100;
pub const PCI_HACK_NO_EXPRESS_CAP: u64 = 0x0000000000080000;
pub const IO_RESOURCE_PREFERRED: u32 = 0x01;
pub const IO_RESOURCE_ALTERNATIVE: u32 = 0x08;
pub const REG_RESOURCE_REQUIREMENTS_LIST: u32 = 10;
pub const STATUS_DEVICE_CONFIGURATION_ERROR: i32 = 0xC0000182u32 as i32;
