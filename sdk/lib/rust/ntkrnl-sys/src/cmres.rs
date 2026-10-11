//! `xdk/cmtypes.h`'s resource-descriptor constants, on every arch.
//!
//! The kernel island carries no `#define` constants (it is glob-imported next to each driver's
//! own island, which usually has these too, and a name arriving through two globs is an
//! ambiguity error at every use). The x86 HAL island has them; arm64 has no HAL island, so its
//! consumers named them by hand. This module is imported by name and collides with nothing.
//! Typed as the descriptor fields that hold them: `Type` is a UCHAR, `Flags` a USHORT.

use crate::ddk::{UCHAR, ULONGLONG, USHORT};

pub const CmResourceTypeNull: UCHAR = 0;
pub const CmResourceTypePort: UCHAR = 1;
pub const CmResourceTypeInterrupt: UCHAR = 2;
pub const CmResourceTypeMemory: UCHAR = 3;
pub const CmResourceTypeDma: UCHAR = 4;
pub const CmResourceTypeDeviceSpecific: UCHAR = 5;
pub const CmResourceTypeBusNumber: UCHAR = 6;
pub const CmResourceTypeNonArbitrated: UCHAR = 128;
pub const CmResourceTypeConfigData: UCHAR = 128;
pub const CmResourceTypeDevicePrivate: UCHAR = 129;
pub const CmResourceTypePcCardConfig: UCHAR = 130;
pub const CmResourceTypeMfCardConfig: UCHAR = 131;
pub const CmResourceTypeMemoryLarge: UCHAR = 7;
pub const CmResourceTypeMaximum: UCHAR = 8;

pub const CM_RESOURCE_PORT_MEMORY: USHORT = 0x0000;
pub const CM_RESOURCE_PORT_IO: USHORT = 0x0001;
pub const CM_RESOURCE_PORT_10_BIT_DECODE: USHORT = 0x0004;
pub const CM_RESOURCE_PORT_12_BIT_DECODE: USHORT = 0x0008;
pub const CM_RESOURCE_PORT_16_BIT_DECODE: USHORT = 0x0010;
pub const CM_RESOURCE_PORT_POSITIVE_DECODE: USHORT = 0x0020;
pub const CM_RESOURCE_PORT_PASSIVE_DECODE: USHORT = 0x0040;
pub const CM_RESOURCE_PORT_WINDOW_DECODE: USHORT = 0x0080;
pub const CM_RESOURCE_PORT_BAR: USHORT = 0x0100;
pub const CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE: USHORT = 0x0000;
pub const CM_RESOURCE_INTERRUPT_LATCHED: USHORT = 0x0001;
pub const CM_RESOURCE_INTERRUPT_MESSAGE: USHORT = 0x0002;
pub const CM_RESOURCE_INTERRUPT_POLICY_INCLUDED: USHORT = 0x0004;
pub const CM_RESOURCE_INTERRUPT_SECONDARY_INTERRUPT: USHORT = 0x0010;
pub const CM_RESOURCE_INTERRUPT_WAKE_HINT: USHORT = 0x0020;
pub const CM_RESOURCE_INTERRUPT_LEVEL_LATCHED_BITS: USHORT = 0x0001;
pub const CM_RESOURCE_MEMORY_READ_WRITE: USHORT = 0x0000;
pub const CM_RESOURCE_MEMORY_READ_ONLY: USHORT = 0x0001;
pub const CM_RESOURCE_MEMORY_WRITE_ONLY: USHORT = 0x0002;
pub const CM_RESOURCE_MEMORY_WRITEABILITY_MASK: USHORT = 0x0003;
pub const CM_RESOURCE_MEMORY_PREFETCHABLE: USHORT = 0x0004;
pub const CM_RESOURCE_MEMORY_COMBINEDWRITE: USHORT = 0x0008;
pub const CM_RESOURCE_MEMORY_24: USHORT = 0x0010;
pub const CM_RESOURCE_MEMORY_CACHEABLE: USHORT = 0x0020;
pub const CM_RESOURCE_MEMORY_WINDOW_DECODE: USHORT = 0x0040;
pub const CM_RESOURCE_MEMORY_BAR: USHORT = 0x0080;
pub const CM_RESOURCE_MEMORY_COMPAT_FOR_INACCESSIBLE_RANGE: USHORT = 0x0100;
pub const CM_RESOURCE_MEMORY_LARGE: USHORT = 0x0E00;
pub const CM_RESOURCE_MEMORY_LARGE_40: USHORT = 0x0200;
pub const CM_RESOURCE_MEMORY_LARGE_48: USHORT = 0x0400;
pub const CM_RESOURCE_MEMORY_LARGE_64: USHORT = 0x0800;
pub const CM_RESOURCE_MEMORY_LARGE_40_MAXLEN: ULONGLONG = 0x000000FFFFFFFF00;
pub const CM_RESOURCE_MEMORY_LARGE_48_MAXLEN: ULONGLONG = 0x0000FFFFFFFF0000;
pub const CM_RESOURCE_MEMORY_LARGE_64_MAXLEN: ULONGLONG = 0xFFFFFFFF00000000;
pub const CM_RESOURCE_DMA_8: USHORT = 0x0000;
pub const CM_RESOURCE_DMA_16: USHORT = 0x0001;
pub const CM_RESOURCE_DMA_32: USHORT = 0x0002;
pub const CM_RESOURCE_DMA_8_AND_16: USHORT = 0x0004;
pub const CM_RESOURCE_DMA_BUS_MASTER: USHORT = 0x0008;
pub const CM_RESOURCE_DMA_TYPE_A: USHORT = 0x0010;
pub const CM_RESOURCE_DMA_TYPE_B: USHORT = 0x0020;
pub const CM_RESOURCE_DMA_TYPE_F: USHORT = 0x0040;
pub const CM_RESOURCE_DMA_V3: USHORT = 0x0080;
