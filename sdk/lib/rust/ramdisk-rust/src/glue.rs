//! ramdisk glue — c2rust phantom-extern macros + substrate import bridges.
//! Filled in during integration/compile-fix (mirrors serial/storport pattern).
#![allow(unused_imports)]

pub use crate::types::*;

// ---------------------------------------------------------------------------
// bindgen enum-constant aliases.  bindgen renames C enum constants to
// `_ENUM_TYPE_Name`; the faithful translation uses the bare C name.  Re-export
// under the bare name so the call sites compile unchanged.
// ---------------------------------------------------------------------------
pub use ntkrnl_sys::ddk::{
    _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs,
    _BUS_QUERY_ID_TYPE_BusQueryContainerID as BusQueryContainerID,
    _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID,
    _BUS_QUERY_ID_TYPE_BusQueryDeviceSerialNumber as BusQueryDeviceSerialNumber,
    _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs,
    _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID,
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
    _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue,
    _POWER_STATE_TYPE_DevicePowerState as DevicePowerState,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyInstallState as DevicePropertyInstallState,
    _DEVICE_TEXT_TYPE_DeviceTextDescription as DeviceTextDescription,
    _DEVICE_TEXT_TYPE_DeviceTextLocationInformation as DeviceTextLocationInformation,
    _MEDIA_TYPE_FixedMedia as FixedMedia,
    _MEDIA_TYPE_RemovableMedia as RemovableMedia,
    _DEVICE_INSTALL_STATE_InstallStateInstalled as InstallStateInstalled,
    _INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined,
    _INTERFACE_TYPE_PNPBus as PNPBus,
    _MEMORY_CACHING_TYPE_MmCached as MmCached,
    _MM_PAGE_PRIORITY_NormalPagePriority as NormalPagePriority,
};

// bindgen enum *type* aliases (named without the leading `_`).
pub use ntkrnl_sys::ddk::{
    _DEVICE_INSTALL_STATE as DEVICE_INSTALL_STATE,
    _DEVICE_RELATION_TYPE as DEVICE_RELATION_TYPE,
    _DEVICE_TEXT_TYPE as DEVICE_TEXT_TYPE,
};

use ntkrnl_sys::ddk::{UCHAR, USHORT, ULONG, ULONG32, ULONGLONG, NTSTATUS, GUID, WCHAR, BOOLEAN, LONGLONG, PVOID, PCSTR};
// Core ddk types the translation references unqualified in its inline struct defs.
pub use ntkrnl_sys::ddk::{_UNICODE_STRING, IO_REMOVE_LOCK, PIO_REMOVE_LOCK, FAST_MUTEX, PNP_BUS_INFORMATION};

// ---------------------------------------------------------------------------
// Constants from headers NOT carried in the kernel-facing ntkrnl_sys modules
// (only present in hal_bindgen, which is not in scope).  Values verbatim from
// the SDK headers.
// ---------------------------------------------------------------------------
pub const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000Eu32 as NTSTATUS;
pub const IRP_MJ_SCSI: u32 = 0x0f;
pub const FILE_DEVICE_DISK: u32 = 0x00000007;
pub const FILE_DEVICE_CD_ROM: u32 = 0x00000002;
pub const MDL_MAPPED_TO_SYSTEM_VA: u32 = 0x0001;
pub const MDL_SOURCE_IS_NONPAGED_POOL: u32 = 0x0004;

// PnP IRP_MN minor codes (wdm.h) NOT carried in driver_ddk (which already
// provides START/STOP/REMOVE/RELATIONS/POWER as `u8`).  Match driver_ddk's `u8`.
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
pub const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
pub const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
pub const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
pub const IRP_MN_EJECT: u8 = 0x11;
pub const IRP_MN_QUERY_ID: u8 = 0x13;
pub const IRP_MN_QUERY_BUS_INFORMATION: u8 = 0x15;

/// C `UNICODE_NULL` is `(WCHAR)0`; the driver also uses `sizeof(UNICODE_NULL)`,
/// so expose it as BOTH a typed value and a type alias (= WCHAR).
pub const UNICODE_NULL: WCHAR = 0;
pub type UNICODE_NULL = WCHAR;

// ntddvol.h: DEFINE_GUID(GUID_DEVINTERFACE_VOLUME, 0x53f5630d,0xb6bf,0x11d0, 0x94,0xf2,0x00,0xa0,0xc9,0x1e,0xfb,0x8b)
pub static GUID_DEVINTERFACE_VOLUME: GUID = GUID {
    Data1: 0x53f5630d,
    Data2: 0xb6bf,
    Data3: 0x11d0,
    Data4: [0x94, 0xf2, 0x00, 0xa0, 0xc9, 0x1e, 0xfb, 0x8b],
};

use ntkrnl_sys::ddk::{PDRIVER_DISPATCH, PIO_COMPLETION_ROUTINE, PDRIVER_ADD_DEVICE,
    PDRIVER_UNLOAD, PDEVICE_OBJECT as _PDEVOBJ, PIRP as _PIRP, PDRIVER_OBJECT as _PDRVOBJ,
    NTSTATUS as _NTSTATUS, PVOID as _PVOID};

/// Transmute an `extern "system"` driver-dispatch routine into the bindgen
/// `PDRIVER_DISPATCH` slot type (carries the per-arch ABI — `stdcall` on i386).
#[inline]
pub unsafe fn as_dispatch(
    f: unsafe extern "system" fn(_PDEVOBJ, _PIRP) -> _NTSTATUS,
) -> PDRIVER_DISPATCH {
    core::mem::transmute(f as *const ())
}

/// IoSetCompletionRoutine / completion-slot routine.
#[inline]
pub unsafe fn as_completion(
    f: unsafe extern "system" fn(_PDEVOBJ, _PIRP, _PVOID) -> _NTSTATUS,
) -> PIO_COMPLETION_ROUTINE {
    core::mem::transmute(f as *const ())
}

/// DriverExtension->AddDevice slot.
#[inline]
pub unsafe fn as_add_device(
    f: unsafe extern "system" fn(_PDRVOBJ, _PDEVOBJ) -> _NTSTATUS,
) -> PDRIVER_ADD_DEVICE {
    core::mem::transmute(f as *const ())
}

/// DriverObject->DriverUnload slot.
#[inline]
pub unsafe fn as_unload(
    f: unsafe extern "system" fn(_PDRVOBJ),
) -> PDRIVER_UNLOAD {
    core::mem::transmute(f as *const ())
}

/// C macro `UNIMPLEMENTED_DBGBREAK(...)` = DPRINT(UNIMPLEMENTED) + DbgBreakPoint;
/// a debug no-op in retail.  Accepts any arity (the C variant is variadic).
#[macro_export]
macro_rules! UNIMPLEMENTED_DBGBREAK {
    ($($t:tt)*) => {{ }};
}

/// C macro `RD_TRACE(s)` is `((void)0)` — a pure no-op.
#[macro_export]
macro_rules! RD_TRACE {
    ($($t:tt)*) => {{ }};
}

/// NUL-terminated UTF-16 (C `L"..."`) compile-time literal → `*const u16`.
/// Backed by a hidden `static`, so the pointer outlives the call (the Rust
/// translation's `"x".encode_utf16().collect::<Vec<u16>>().as_ptr()` both needs
/// std `Vec` and is a use-after-free; this is the faithful C `L"x"` form).
#[macro_export]
macro_rules! wstr {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        static W: [u16; N] = {
            let b = $s.as_bytes();
            let mut a = [0u16; N];
            let mut i = 0;
            while i < b.len() {
                a[i] = b[i] as u16;
                i += 1;
            }
            a
        };
        W.as_ptr()
    }};
}

// ---------------------------------------------------------------------------
// Driver-private struct layouts pulled from headers NOT in the ntoskrnl
// bindgen surface (ntddcdrm.h, ntddvol.h, fs_rec/fs_rec.h).  These are only
// used by ramdisk for `size_of`/`offset_of`/zeroed scratch buffers; layouts
// reproduce the C headers faithfully.
// ---------------------------------------------------------------------------

// ntddcdrm.h: MAXIMUM_NUMBER_TRACKS = 100
#[repr(C)]
pub struct TRACK_DATA {
    pub Reserved: UCHAR,
    /// C bitfield `UCHAR Control:4; UCHAR Adr:4;` — Control = low nibble, Adr = high nibble.
    pub Control_Adr: UCHAR,
    pub TrackNumber: UCHAR,
    pub Reserved1: UCHAR,
    pub Address: [UCHAR; 4],
}
impl TRACK_DATA {
    #[inline]
    pub fn set_Control(&mut self, v: u32) {
        self.Control_Adr = (self.Control_Adr & 0xF0) | ((v as u8) & 0x0F);
    }
    #[inline]
    pub fn set_Adr(&mut self, v: u32) {
        self.Control_Adr = (self.Control_Adr & 0x0F) | (((v as u8) & 0x0F) << 4);
    }
    #[inline]
    pub fn Control(&self) -> u32 { (self.Control_Adr & 0x0F) as u32 }
    #[inline]
    pub fn Adr(&self) -> u32 { ((self.Control_Adr >> 4) & 0x0F) as u32 }
}
pub type PTRACK_DATA = *mut TRACK_DATA;

#[repr(C)]
pub struct CDROM_TOC {
    pub Length: [UCHAR; 2],
    pub FirstTrack: UCHAR,
    pub LastTrack: UCHAR,
    pub TrackData: [TRACK_DATA; 100],
}
pub type PCDROM_TOC = *mut CDROM_TOC;

// ntddvol.h
#[repr(C)]
pub struct VOLUME_GET_GPT_ATTRIBUTES_INFORMATION {
    pub GptAttributes: ULONGLONG,
}
pub type PVOLUME_GET_GPT_ATTRIBUTES_INFORMATION = *mut VOLUME_GET_GPT_ATTRIBUTES_INFORMATION;

// fs_rec/fs_rec.h
#[repr(C)]
pub struct PACKED_BIOS_PARAMETER_BLOCK {
    pub BytesPerSector: [UCHAR; 2],
    pub SectorsPerCluster: [UCHAR; 1],
    pub ReservedSectors: [UCHAR; 2],
    pub Fats: [UCHAR; 1],
    pub RootEntries: [UCHAR; 2],
    pub Sectors: [UCHAR; 2],
    pub Media: [UCHAR; 1],
    pub SectorsPerFat: [UCHAR; 2],
    pub SectorsPerTrack: [UCHAR; 2],
    pub Heads: [UCHAR; 2],
    pub HiddenSectors: [UCHAR; 4],
    pub LargeSectors: [UCHAR; 4],
}
pub type PPACKED_BIOS_PARAMETER_BLOCK = *mut PACKED_BIOS_PARAMETER_BLOCK;

#[repr(C)]
pub struct PACKED_BOOT_SECTOR {
    pub Jump: [UCHAR; 3],
    pub Oem: [UCHAR; 8],
    pub PackedBpb: PACKED_BIOS_PARAMETER_BLOCK,
    pub PhysicalDriveNumber: UCHAR,
    pub CurrentHead: UCHAR,
    pub Signature: UCHAR,
    pub Id: [UCHAR; 4],
    pub VolumeLabel: [UCHAR; 11],
    pub SystemId: [UCHAR; 8],
}
pub type PPACKED_BOOT_SECTOR = *mut PACKED_BOOT_SECTOR;

#[repr(C)]
pub struct BIOS_PARAMETER_BLOCK {
    pub BytesPerSector: USHORT,
    pub SectorsPerCluster: UCHAR,
    pub ReservedSectors: USHORT,
    pub Fats: UCHAR,
    pub RootEntries: USHORT,
    pub Sectors: USHORT,
    pub Media: UCHAR,
    pub SectorsPerFat: USHORT,
    pub SectorsPerTrack: USHORT,
    pub Heads: USHORT,
    pub HiddenSectors: ULONG32,
    pub LargeSectors: ULONG32,
    pub LargeSectorsPerFat: ULONG32,
    pub ExtendedFlags: USHORT, // union{ USHORT; bitfield } — first member
    pub FsVersion: USHORT,
    pub RootDirFirstCluster: ULONG32,
    pub FsInfoSector: USHORT,
    pub BackupBootSector: USHORT,
}
pub type PBIOS_PARAMETER_BLOCK = *mut BIOS_PARAMETER_BLOCK;

// ===========================================================================
// Header macros / inlines / GUID data referenced by ramdisk.c that are NOT
// ntoskrnl exports — translated faithfully (link-critical for the cohab .sys;
// cargo compiles the original extern decls fine but they have no symbol).
// ===========================================================================

// BYTE_OFFSET(Va) = (ULONG)((ULONG_PTR)Va & (PAGE_SIZE - 1))   [PAGE_SIZE = 4096]
#[inline]
pub unsafe fn BYTE_OFFSET(Va: LONGLONG) -> ULONG {
    (Va & 0xFFF) as ULONG
}

// ExInitializeFastMutex — FORCEINLINE from exfuncs.h; ntkrnl_sys provides it
// (the bindgen ddk extern of the same name has no symbol → must route here).
pub use ntkrnl_sys::inline::ex::ExInitializeFastMutex;

// IO_REMOVE_LOCK inline wrappers (wdm.h FORCEINLINE → the *Ex export with the
// struct size; __FILE__/__LINE__ are debug-only, null/0 is faithful in retail).
#[inline]
pub unsafe fn IoInitializeRemoveLock(
    Lock: *mut core::ffi::c_void, AllocateTag: u32, MaxLockedMinutes: u32, HighWatermark: u32,
) {
    ntkrnl_sys::ddk::IoInitializeRemoveLockEx(
        Lock as PIO_REMOVE_LOCK, AllocateTag, MaxLockedMinutes, HighWatermark,
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}
#[inline]
pub unsafe fn IoAcquireRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) -> NTSTATUS {
    ntkrnl_sys::ddk::IoAcquireRemoveLockEx(
        RemoveLock, Tag, core::ptr::null::<core::ffi::c_char>() as PCSTR, 0,
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}
#[inline]
pub unsafe fn IoReleaseRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    ntkrnl_sys::ddk::IoReleaseRemoveLockEx(
        RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}
#[inline]
pub unsafe fn IoReleaseRemoveLockAndWait(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx(
        RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}

// IsRecognizedPartition / IsFTPartition (ntdddisk.h, __REACTOS__ variant).
const PARTITION_FAT_12: u8 = 0x01;
const PARTITION_FAT_16: u8 = 0x04;
const PARTITION_HUGE: u8 = 0x06;
const PARTITION_IFS: u8 = 0x07;
const PARTITION_FAT32: u8 = 0x0B;
const PARTITION_FAT32_XINT13: u8 = 0x0C;
const PARTITION_XINT13: u8 = 0x0E;
const PARTITION_NTFT: u8 = 0x80;
const VALID_NTFT: u8 = 0xC0;
const PARTITION_OLD_LINUX: u8 = 0x43;
const PARTITION_LINUX: u8 = 0x83;
const PARTITION_ISO9660: u8 = 0x96;
const PARTITION_FREEBSD: u8 = 0xA5;
const PARTITION_OPENBSD: u8 = 0xA6;
const PARTITION_NETBSD: u8 = 0xA9;

#[inline]
unsafe fn IsFTPartition(t: u8) -> bool {
    (t & PARTITION_NTFT) != 0
        && ((t & !VALID_NTFT) == PARTITION_HUGE
            || (t & !VALID_NTFT) == PARTITION_IFS
            || (t & !VALID_NTFT) == PARTITION_FAT32
            || (t & !VALID_NTFT) == PARTITION_FAT32_XINT13)
}
#[inline]
pub unsafe fn IsRecognizedPartition(PartitionType: u8) -> BOOLEAN {
    let t = PartitionType;
    (IsFTPartition(t)
        || t == PARTITION_FAT_12
        || t == PARTITION_FAT_16
        || t == PARTITION_HUGE
        || t == PARTITION_IFS
        || t == PARTITION_FAT32
        || t == PARTITION_FAT32_XINT13
        || t == PARTITION_XINT13
        || t == PARTITION_LINUX
        || t == PARTITION_OLD_LINUX
        || t == PARTITION_ISO9660
        || t == PARTITION_FREEBSD
        || t == PARTITION_OPENBSD
        || t == PARTITION_NETBSD) as BOOLEAN
}

// FatUnpackBios (fs_rec.h) — CopyUcharN = N-byte copy; on LE the packed byte
// arrays read back as little-endian scalars (faithful on i386/amd64/arm64).
#[inline]
pub unsafe fn FatUnpackBios(Bios: *mut BIOS_PARAMETER_BLOCK, PackedBios: *mut PACKED_BIOS_PARAMETER_BLOCK) {
    let p = &*PackedBios;
    let b = &mut *Bios;
    b.BytesPerSector    = u16::from_le_bytes(p.BytesPerSector);
    b.SectorsPerCluster = p.SectorsPerCluster[0];
    b.ReservedSectors   = u16::from_le_bytes(p.ReservedSectors);
    b.Fats              = p.Fats[0];
    b.RootEntries       = u16::from_le_bytes(p.RootEntries);
    b.Sectors           = u16::from_le_bytes(p.Sectors);
    b.Media             = p.Media[0];
    b.SectorsPerFat     = u16::from_le_bytes(p.SectorsPerFat);
    b.SectorsPerTrack   = u16::from_le_bytes(p.SectorsPerTrack);
    b.Heads             = u16::from_le_bytes(p.Heads);
    b.HiddenSectors     = u32::from_le_bytes(p.HiddenSectors);
    b.LargeSectors      = u32::from_le_bytes(p.LargeSectors);
}

// GUID_BUS_TYPE_RAMDISK (ntddrdsk.h DEFINE_GUID, emitted by initguid.h in C).
// Private + #[no_mangle]: supplies the symbol the types_*.rs bindgen extern
// references, without glob-importing a name that collides with that extern.
#[no_mangle]
static GUID_BUS_TYPE_RAMDISK: GUID = GUID {
    Data1: 0x9D6D66A6,
    Data2: 0x0B0C,
    Data3: 0x4563,
    Data4: [0x90, 0x77, 0xA0, 0xE9, 0xA7, 0x95, 0x5A, 0xE4],
};

// KeLoaderBlock — cross-DLL kernel DATA import (ntoskrnl.spec `@ extern KeLoaderBlock`).
// globals.rs declares it via a plain extern static, which is for the IN-KERNEL local
// symbol; a cohab DRIVER importing it links on amd64 but i386/arm64 PE need the raw-dylib
// data-import thunk (__imp_). ramdisk is the first cohab driver to import kernel data.
// ramdisk.rs shadows the `globals::*` / `glue::*` globs with an explicit `use` of THIS.
#[link(name = "ntoskrnl", kind = "raw-dylib")]
extern "C" {
    pub static mut KeLoaderBlock: ntkrnl_sys::ddk::PLOADER_PARAMETER_BLOCK;
}
