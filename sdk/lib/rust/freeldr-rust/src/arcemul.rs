use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::VIDEODISPLAYMODE;
use ntkrnl_sys::ddk::{PCONFIGURATION_COMPONENT_DATA, PCSTR, TIMEINFO};
use core::ffi::c_void;

// arch/arcemul.c is the OWNER of the global MachVtbl. The C file defined
// `MACHVTBL MachVtbl;` and the ARC dispatch helpers (ArcGetTime / MachHwDetect
// / …) that call through it. Cohabitation flip: this Rust file replaces the
// C; we must (a) provide the global symbol `MachVtbl` (#[no_mangle]) so other
// units (machpc/meminit/…) can `extern { static mut MachVtbl }` into us, and
// (b) define MACHVTBL as the full machine.h struct (the keystone's MACHVTBL
// is deliberately opaque for back-compat with the old layout; the full
// definition lives at the OWNER site).
//
// Layout matches include/machine.h tagMACHVTBL exactly (30 fn-pointer slots).
// machpc.rs has the same file-local shadow — keep both struct defs in lockstep.
#[repr(C)]
pub struct MACHVTBL {
    pub ConsPutChar: Option<unsafe extern "C" fn(i32)>,
    pub ConsKbHit: Option<unsafe extern "C" fn() -> BOOLEAN>,
    pub ConsGetCh: Option<unsafe extern "C" fn() -> i32>,
    pub VideoClearScreen: Option<unsafe extern "C" fn(UCHAR)>,
    pub VideoSetDisplayMode: Option<unsafe extern "C" fn(PCSTR, BOOLEAN) -> VIDEODISPLAYMODE>,
    pub VideoGetDisplaySize: Option<unsafe extern "C" fn(PULONG, PULONG, PULONG)>,
    pub VideoGetBufferSize: Option<unsafe extern "C" fn() -> ULONG>,
    pub VideoGetFontsFromFirmware: Option<unsafe extern "C" fn(PULONG)>,
    pub VideoSetTextCursorPosition: Option<unsafe extern "C" fn(UCHAR, UCHAR)>,
    pub VideoHideShowTextCursor: Option<unsafe extern "C" fn(BOOLEAN)>,
    pub VideoPutChar: Option<unsafe extern "C" fn(i32, UCHAR, ::core::ffi::c_uint, ::core::ffi::c_uint)>,
    pub VideoCopyOffScreenBufferToVRAM: Option<unsafe extern "C" fn(PVOID)>,
    pub VideoIsPaletteFixed: Option<unsafe extern "C" fn() -> BOOLEAN>,
    pub VideoSetPaletteColor: Option<unsafe extern "C" fn(UCHAR, UCHAR, UCHAR, UCHAR)>,
    pub VideoGetPaletteColor: Option<unsafe extern "C" fn(UCHAR, *mut UCHAR, *mut UCHAR, *mut UCHAR)>,
    pub VideoSync: Option<unsafe extern "C" fn()>,
    pub Beep: Option<unsafe extern "C" fn()>,
    pub PrepareForReactOS: Option<unsafe extern "C" fn()>,
    pub GetMemoryDescriptor: Option<unsafe extern "C" fn(*mut FREELDR_MEMORY_DESCRIPTOR) -> *mut FREELDR_MEMORY_DESCRIPTOR>,
    pub GetMemoryMap: Option<unsafe extern "C" fn(PULONG) -> PFREELDR_MEMORY_DESCRIPTOR>,
    pub GetExtendedBIOSData: Option<unsafe extern "C" fn(PULONG, PULONG)>,
    pub GetFloppyCount: Option<unsafe extern "C" fn() -> UCHAR>,
    pub DiskReadLogicalSectors: Option<unsafe extern "C" fn(UCHAR, ULONGLONG, ULONG, PVOID) -> BOOLEAN>,
    pub DiskGetDriveGeometry: Option<unsafe extern "C" fn(UCHAR, PGEOMETRY) -> BOOLEAN>,
    pub DiskGetCacheableBlockCount: Option<unsafe extern "C" fn(UCHAR) -> ULONG>,
    pub GetTime: Option<unsafe extern "C" fn() -> *mut TIMEINFO>,
    pub GetRelativeTime: Option<unsafe extern "C" fn() -> ULONG>,
    pub InitializeBootDevices: Option<unsafe extern "C" fn() -> BOOLEAN>,
    pub HwDetect: Option<unsafe extern "C" fn(PCSTR) -> PCONFIGURATION_COMPONENT_DATA>,
    pub HwIdle: Option<unsafe extern "C" fn()>,
}
unsafe impl Sync for MACHVTBL {}

// ---- file-scope preamble (translated) ----
// `MACHVTBL MachVtbl;` — C global. #[no_mangle] so machpc.rs / meminit.rs
// extern statics resolve here. mem::zeroed is the C `BOOLEAN MachVtbl;` BSS
// initializer (all-Option-None == all-zero bit-pattern).
#[no_mangle]
pub static mut MachVtbl: MACHVTBL = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "C" fn ArcGetTime() -> *mut TIMEINFO {
    (MachVtbl.GetTime.unwrap())()
}

#[no_mangle]
pub unsafe extern "C" fn ArcGetRelativeTime() -> u32 {
    let TimeInfo: *mut TIMEINFO;
    let ret: u32;

    TimeInfo = ArcGetTime();
    ret = (((*TimeInfo).Hour as u32 * 24) + (*TimeInfo).Minute as u32) * 60 + (*TimeInfo).Second as u32;
    ret
}

#[no_mangle]
pub unsafe extern "C" fn MachHwDetect(
    Options: PCSTR,
) -> PCONFIGURATION_COMPONENT_DATA {
    (MachVtbl.HwDetect.unwrap())(Options)
}

#[no_mangle]
pub unsafe extern "C" fn MachPrepareForReactOS() {
    (MachVtbl.PrepareForReactOS.unwrap())();
}

#[no_mangle]
pub unsafe extern "C" fn MachGetExtendedBIOSData(
    ExtendedBIOSDataArea: PULONG,
    ExtendedBIOSDataSize: PULONG,
) {
    (MachVtbl.GetExtendedBIOSData.unwrap())(ExtendedBIOSDataArea, ExtendedBIOSDataSize);
}

#[no_mangle]
pub unsafe extern "C" fn MachVideoGetFontsFromFirmware(RomFontPointers: PULONG) {
    (MachVtbl.VideoGetFontsFromFirmware.unwrap())(RomFontPointers);
}

#[no_mangle]
pub unsafe extern "C" fn MachGetBootSectorLoadAddress(_DriveNumber: u8) -> u32 {
    0x7C00
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// MACHVTBL MachVtbl;
//
//
// /* ARC FUNCTIONS **************************************************************/
//
// TIMEINFO*
// ArcGetTime(VOID)
// {
//     return MachVtbl.GetTime();
// }
//
// ULONG
// ArcGetRelativeTime(VOID)
// {
//     TIMEINFO* TimeInfo;
//     ULONG ret;
//
//     TimeInfo = ArcGetTime();
//     ret = ((TimeInfo->Hour * 24) + TimeInfo->Minute) * 60 + TimeInfo->Second;
//     return ret;
// }
//
// PCONFIGURATION_COMPONENT_DATA
// MachHwDetect(_In_opt_ PCSTR Options)
// {
//     return MachVtbl.HwDetect(Options);
// }
//
// VOID MachPrepareForReactOS(VOID)
// {
//     MachVtbl.PrepareForReactOS();
// }
//
// VOID MachGetExtendedBIOSData(PULONG ExtendedBIOSDataArea, PULONG ExtendedBIOSDataSize)
// {
//     MachVtbl.GetExtendedBIOSData(ExtendedBIOSDataArea, ExtendedBIOSDataSize);
// }
//
// VOID MachVideoGetFontsFromFirmware(PULONG RomFontPointers)
// {
//     MachVtbl.VideoGetFontsFromFirmware(RomFontPointers);
// }
//
// ULONG
// MachGetBootSectorLoadAddress(IN UCHAR DriveNumber)
// {
//     return 0x7C00;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
