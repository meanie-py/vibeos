//! kernel_ext — kernel imports that the bindgen substrate couldn't provide correctly.
//!
//! `ScsiPortGetBusDataByOffset` is declared with C++ linkage in uniata's headers, so
//! bindgen emitted it as a `_Z`-mangled decl that would NOT resolve to scsiport.sys's
//! C export (`_ScsiPortGetBusDataByOffset@28` on i386). The mangled decl is stripped
//! from types_*.rs by strip_cpp_mangled_externs.py; here is the correct C-linkage one.
//! `extern "system"` decorates per arch (stdcall on i386, cdecl on amd64/arm64), exactly
//! like its correctly-bound twin `ScsiPortSetBusDataByOffset` in the substrate.

use ntkrnl_sys::ddk::{PVOID, ULONG};

extern "system" {
    /// Read PCI config space at a byte offset (SCSI miniport API, scsiport.sys export).
    /// BusDataType is `PCIConfiguration`(=4); typed ULONG to match ScsiPortSetBusDataByOffset.
    pub fn ScsiPortGetBusDataByOffset(
        HwDeviceExtension: PVOID,
        BusDataType: ULONG,
        BusNumber: ULONG,
        SlotNumber: ULONG,
        Buffer: PVOID,
        Offset: ULONG,
        Length: ULONG,
    ) -> ULONG;
}
