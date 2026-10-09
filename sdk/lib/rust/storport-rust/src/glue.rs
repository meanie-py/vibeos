// glue.rs — storport-rust binding helpers
// Only items genuinely missing from the ntkrnl_sys module set.
// NT_SUCCESS, IoCompleteRequest, IoCallDriver, RtlZeroMemory, RtlCopyMemory,
// InitializeListHead, InsertHeadList, RemoveEntryList, IsListEmpty,
// ExAllocatePoolWithTag — all live in ntkrnl_sys::base / ::driver_ddk / ::rtl / ::ex.

use ntkrnl_sys::ddk::{
    GUID, LIST_ENTRY, NTSTATUS, PDEVICE_OBJECT, PIRP, PUCHAR, PULONG, PUSHORT, UCHAR, ULONG,
    USHORT, PVOID,
};

// ============================================================================
// Missing STATUS codes (not in ntkrnl_sys::base or ::constants)
// ============================================================================
pub const STATUS_NO_SUCH_DEVICE: NTSTATUS  = 0xC000000E_u32 as i32;
pub const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185_u32 as i32;
// STATUS_NOT_IMPLEMENTED lives in ntkrnl_sys::base — do not redefine here.

// ============================================================================
// IRP_MJ_SCSI — not in ntkrnl_sys::constants, but in hal_bindgen only
// ============================================================================
pub const IRP_MJ_SCSI: u8 = 0x0F;

// ============================================================================
// Memory tags (from precomp.h)
// ============================================================================
pub const TAG_GLOBAL_DATA:     u32 = u32::from_le_bytes(*b"DGtS");
pub const TAG_INIT_DATA:       u32 = u32::from_le_bytes(*b"DItS");
pub const TAG_MINIPORT_DATA:   u32 = u32::from_le_bytes(*b"DMtS");
pub const TAG_ACCRESS_RANGE:   u32 = u32::from_le_bytes(*b"RAtS");
pub const TAG_RESOURCE_LIST:   u32 = u32::from_be_bytes(*b"LRtS"); // C literal 'LRtS' -- drivers/storage/port/storport/precomp.h
pub const TAG_ADDRESS_MAPPING: u32 = u32::from_le_bytes(*b"MAtS");
pub const TAG_INQUIRY_DATA:    u32 = u32::from_le_bytes(*b"QItS");
pub const TAG_SENSE_DATA:      u32 = u32::from_le_bytes(*b"NStS");

// ============================================================================
// SP_UNINITIALIZED_VALUE (storport.h: #define SP_UNINITIALIZED_VALUE ((ULONG)~0))
// ============================================================================
pub const SP_UNINITIALIZED_VALUE: ULONG = !0u32;

// ============================================================================
// DEVICE_STATE short-name aliases (defined in crate::types / types_*.rs)
// ============================================================================
pub use crate::types::DEVICE_STATE_dsStopped as dsStopped;
pub use crate::types::DEVICE_STATE_dsStarted as dsStarted;

// ============================================================================
// EXTENSION_TYPE short-name aliases (defined in crate::types / types_*.rs)
// These use full-name exports to avoid shadowing common parameter names.
// ============================================================================
pub use crate::types::EXTENSION_TYPE_DriverExtension;
pub use crate::types::EXTENSION_TYPE_FdoExtension;
pub use crate::types::EXTENSION_TYPE_PdoExtension;

// ============================================================================
// INTERFACE_TYPE short-name aliases
// ============================================================================
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined;
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCIBus    as PCIBus;
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCMCIABus as PCMCIABus;
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_PNPISABus as PNPISABus;

// ============================================================================
// KINTERRUPT_MODE short-name aliases
// ============================================================================
pub use ntkrnl_sys::ddk::_KINTERRUPT_MODE_LevelSensitive as LevelSensitive;
pub use ntkrnl_sys::ddk::_KINTERRUPT_MODE_Latched        as Latched;

// ============================================================================
// DMA_WIDTH short-name aliases
// ============================================================================
pub use ntkrnl_sys::ddk::_DMA_WIDTH_Width8Bits  as Width8Bits;
pub use ntkrnl_sys::ddk::_DMA_WIDTH_Width16Bits as Width16Bits;
pub use ntkrnl_sys::ddk::_DMA_WIDTH_Width32Bits as Width32Bits;

// ============================================================================
// DEVICE_RELATION_TYPE short-name aliases
// ============================================================================
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_BusRelations     as BusRelations;
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_RemovalRelations  as RemovalRelations;

// ============================================================================
// DEVICE_REGISTRY_PROPERTY short-name aliases
// (BusNumber omitted — name clashes with common function parameter names)
// ============================================================================
pub use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyBusTypeGuid   as DevicePropertyBusTypeGuid;

// ============================================================================
// MEMORY_CACHING_TYPE short-name aliases
// ============================================================================
pub use ntkrnl_sys::ddk::_MEMORY_CACHING_TYPE_MmCached    as MmCached;
pub use ntkrnl_sys::ddk::_MEMORY_CACHING_TYPE_MmNonCached as MmNonCached;

// ============================================================================
// CM_SHARE_DISPOSITION alias
// ============================================================================
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared;

// ============================================================================
// SharedSystemTime — address of InterruptTime in KUSER_SHARED_DATA (x86: 0x7ffe0008)
// ============================================================================
pub const SharedSystemTime: usize = 0x7ffe0008; // nolint: const-contradiction (user-mode alias of KUSER_SHARED_DATA+0x8, mapped in every process; the island has the kernel-mode address)

// ============================================================================
// SRB_STATUS macro: strips the queue-frozen bit (0x40) before comparison.
// C: #define SRB_STATUS(Status) ((Status) & ~SRB_STATUS_QUEUE_FROZEN)
// ============================================================================
#[inline(always)]
pub fn SRB_STATUS(status: u8) -> u8 {
    status & !0x40u8
}

// ============================================================================
// GUID constants from wdmguid.h
// ============================================================================
pub static GUID_BUS_INTERFACE_STANDARD: GUID = GUID {
    Data1: 0x496b8280,
    Data2: 0x6f25,
    Data3: 0x11d0,
    Data4: [0xbe, 0xaf, 0x08, 0x00, 0x2b, 0xe2, 0x09, 0x2f],
};
pub static GUID_BUS_TYPE_PCMCIA: GUID = GUID {
    Data1: 0x09343630,
    Data2: 0xaf9f,
    Data3: 0x11d0,
    Data4: [0x92, 0xE9, 0x00, 0x00, 0xf8, 0x1e, 0x1b, 0x30],
};
pub static GUID_BUS_TYPE_PCI: GUID = GUID {
    Data1: 0xc8ebdfb0,
    Data2: 0xb510,
    Data3: 0x11d0,
    Data4: [0x80, 0xe5, 0x00, 0xa0, 0xc9, 0x25, 0x42, 0xe3],
};
pub static GUID_BUS_TYPE_ISAPNP: GUID = GUID {
    Data1: 0xe676f854,
    Data2: 0xd87d,
    Data3: 0x11d0,
    Data4: [0x92, 0xb2, 0x00, 0xa0, 0xc9, 0x05, 0x5f, 0xc5],
};

// ============================================================================
// RtlAssert — DBG assert macro; no-op in free (non-debug) builds.
// DBG=0 is the norm here: ntoskrnl.lib has no RtlAssert export in free builds,
// so the extern decl would produce an undefined-symbol link error.
// The translation calls it faithfully; we just make it a no-op inline so the
// call sites compile + link without omitting any logic.
// Signature matches the C prototype: RtlAssert(expr, file, line, msg).
// ============================================================================
#[inline(always)]
pub unsafe extern "system" fn RtlAssert(
    _FailedAssertion: PVOID,
    _FileName: PVOID,
    _LineNumber: ULONG,
    _Message: *mut i8,
) {}

// ============================================================================
// PORT I/O and REGISTER (MMIO) access
//
// On i386 and arm64 these are HAL exports — resolved at link time via the
// extern "system" blocks below.
//
// On amd64 (x86_64) the HAL does NOT export them as symbols; they are
// compiler intrinsics in MSVC.  We provide inline-asm / volatile-ptr
// equivalents that are faithful to the DDK semantics.
// ============================================================================

// ---------------------------------------------------------------------------
// PORT I/O — HAL extern (i386 / arm64)
// ---------------------------------------------------------------------------
#[cfg(not(target_arch = "x86_64"))]
extern "system" {
    pub fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR;
    pub fn READ_PORT_USHORT(Port: PUSHORT) -> USHORT;
    pub fn READ_PORT_ULONG(Port: PULONG) -> ULONG;
    pub fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR);
    pub fn WRITE_PORT_USHORT(Port: PUSHORT, Value: USHORT);
    pub fn WRITE_PORT_ULONG(Port: PULONG, Value: ULONG);
    pub fn READ_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG);
    pub fn READ_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG);
    pub fn READ_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG);
    pub fn WRITE_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG);
    pub fn WRITE_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG);
    pub fn WRITE_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG);
}

// ---------------------------------------------------------------------------
// PORT I/O — inline asm (amd64 only)
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR {
    let v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") (Port as usize as u16), options(nostack, preserves_flags));
    v
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_USHORT(Port: PUSHORT) -> USHORT {
    let v: u16;
    core::arch::asm!("in ax, dx", out("ax") v, in("dx") (Port as usize as u16), options(nostack, preserves_flags));
    v
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_ULONG(Port: PULONG) -> ULONG {
    let v: u32;
    core::arch::asm!("in eax, dx", out("eax") v, in("dx") (Port as usize as u16), options(nostack, preserves_flags));
    v
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR) {
    core::arch::asm!("out dx, al", in("dx") (Port as usize as u16), in("al") Value, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_USHORT(Port: PUSHORT, Value: USHORT) {
    core::arch::asm!("out dx, ax", in("dx") (Port as usize as u16), in("ax") Value, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_ULONG(Port: PULONG, Value: ULONG) {
    core::arch::asm!("out dx, eax", in("dx") (Port as usize as u16), in("eax") Value, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), READ_PORT_UCHAR(Port));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), READ_PORT_USHORT(Port));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), READ_PORT_ULONG(Port));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        WRITE_PORT_UCHAR(Port, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG) {
    for i in 0..Count as isize {
        WRITE_PORT_USHORT(Port, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG) {
    for i in 0..Count as isize {
        WRITE_PORT_ULONG(Port, core::ptr::read(Buffer.offset(i)));
    }
}

// ---------------------------------------------------------------------------
// REGISTER (MMIO) — HAL extern (i386 / arm64)
// ---------------------------------------------------------------------------
#[cfg(not(target_arch = "x86_64"))]
extern "system" {
    pub fn READ_REGISTER_UCHAR(Register: *mut UCHAR) -> UCHAR;
    pub fn READ_REGISTER_USHORT(Register: *mut USHORT) -> USHORT;
    pub fn READ_REGISTER_ULONG(Register: *mut ULONG) -> ULONG;
    pub fn READ_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG);
    pub fn READ_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG);
    pub fn READ_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG);
    pub fn WRITE_REGISTER_UCHAR(Register: *mut UCHAR, Value: UCHAR);
    pub fn WRITE_REGISTER_USHORT(Register: *mut USHORT, Value: USHORT);
    pub fn WRITE_REGISTER_ULONG(Register: *mut ULONG, Value: ULONG);
    pub fn WRITE_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG);
    pub fn WRITE_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG);
    pub fn WRITE_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG);
}

// ---------------------------------------------------------------------------
// REGISTER (MMIO) — volatile pointer (amd64 only)
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_UCHAR(Register: *mut UCHAR) -> UCHAR {
    core::ptr::read_volatile(Register)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_USHORT(Register: *mut USHORT) -> USHORT {
    core::ptr::read_volatile(Register)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_ULONG(Register: *mut ULONG) -> ULONG {
    core::ptr::read_volatile(Register)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_UCHAR(Register: *mut UCHAR, Value: UCHAR) {
    core::ptr::write_volatile(Register, Value)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_USHORT(Register: *mut USHORT, Value: USHORT) {
    core::ptr::write_volatile(Register, Value)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_ULONG(Register: *mut ULONG, Value: ULONG) {
    core::ptr::write_volatile(Register, Value)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), core::ptr::read_volatile(Register));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), core::ptr::read_volatile(Register));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), core::ptr::read_volatile(Register));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write_volatile(Register, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write_volatile(Register, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write_volatile(Register, core::ptr::read(Buffer.offset(i)));
    }
}
