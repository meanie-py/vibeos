#[allow(unused_imports)] use crate::{ERR};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WARNING; // uefisetup.c: DBG_DEFAULT_CHANNEL(WARNING)
use core::ffi::c_void;
use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::VIDEODISPLAYMODE;
use ntkrnl_sys::ddk::{PCONFIGURATION_COMPONENT_DATA, PCSTR, TIMEINFO};

// FreeLDR ERR no-op (variadic to typecheck format args; frldr.rs exports
// the non-variadic cdecl body — caller-cleans makes the call ABI-safe).
extern "C" {
}

// Full machine.h MACHVTBL (30 slots) — file-local shadow. arcemul.rs is
// the OWNER of MachVtbl (`#[no_mangle] static mut MachVtbl: MACHVTBL`);
// here we extern-consume + assign through it. Layout mirrors arcemul.rs /
// machpc.rs exactly so linker-resolved cross-rlib references read the
// same bytes.
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

extern "C" {
    // MachVtbl OWNED by arcemul.rs (#[no_mangle] BSS-zero), used here for
    // fill-in. Cohabitation-safe — both archives link arcemul.
    static mut MachVtbl: MACHVTBL;
}

// UEFI MACHVTBL slot functions. ConsPutChar/Kbhit/GetCh + Video*  +
// Disk* + Mem* + Hw* live across ueficon.rs/uefivid.rs/uefidisk.rs/
// uefimem.rs/uefihw.rs/uefi_stubs.rs. All cdecl extern "C".
extern "C" {
    fn UefiConsPutChar(Ch: i32);
    fn UefiConsKbHit() -> BOOLEAN;
    fn UefiConsGetCh() -> i32;
    fn UefiVideoClearScreen(Attr: UCHAR);
    fn UefiVideoSetDisplayMode(DisplayMode: PCSTR, Init: BOOLEAN) -> VIDEODISPLAYMODE;
    fn UefiVideoGetDisplaySize(Width: PULONG, Height: PULONG, Depth: PULONG);
    fn UefiVideoGetBufferSize() -> ULONG;
    fn UefiVideoGetFontsFromFirmware(RomFontPointers: PULONG);
    fn UefiVideoSetTextCursorPosition(X: UCHAR, Y: UCHAR);
    fn UefiVideoHideShowTextCursor(Show: BOOLEAN);
    fn UefiVideoPutChar(Ch: i32, Attr: UCHAR, X: ::core::ffi::c_uint, Y: ::core::ffi::c_uint);
    fn UefiVideoCopyOffScreenBufferToVRAM(Buffer: PVOID);
    fn UefiVideoIsPaletteFixed() -> BOOLEAN;
    fn UefiVideoSetPaletteColor(Color: UCHAR, Red: UCHAR, Green: UCHAR, Blue: UCHAR);
    fn UefiVideoGetPaletteColor(Color: UCHAR, Red: *mut UCHAR, Green: *mut UCHAR, Blue: *mut UCHAR);
    fn UefiVideoSync();
    fn UefiPcBeep();
    fn UefiPrepareForReactOS();
    fn UefiMemGetMemoryMap(MemoryMapSize: *mut ULONG) -> PFREELDR_MEMORY_DESCRIPTOR;
    fn UefiGetExtendedBIOSData(ExtendedBIOSDataArea: PULONG, ExtendedBIOSDataSize: PULONG);
    fn UefiGetFloppyCount() -> UCHAR;
    fn UefiDiskReadLogicalSectors(DriveNumber: UCHAR, SectorNumber: ULONGLONG, SectorCount: ULONG, Buffer: PVOID) -> BOOLEAN;
    fn UefiDiskGetDriveGeometry(DriveNumber: UCHAR, Geometry: PGEOMETRY) -> BOOLEAN;
    fn UefiDiskGetCacheableBlockCount(DriveNumber: UCHAR) -> ULONG;
    fn UefiGetTime() -> *mut TIMEINFO;
    fn UefiInitializeBootDevices() -> BOOLEAN;
    fn UefiHwDetect(Options: PCSTR) -> PCONFIGURATION_COMPONENT_DATA;
    fn UefiHwIdle();
    fn UefiInitializeVideo() -> EFI_STATUS;
}

// ---- file-scope preamble (translated) ----
// `extern EFI_SYSTEM_TABLE* GlobalSystemTable;` / `extern EFI_HANDLE
// GlobalImageHandle;` — both OWNED by uefildr.c (or future uefildr.rs).
// Extern-consume; defining here duplicates symbols at the UEFI cold link.
extern "C" {
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
    static mut GlobalImageHandle: EFI_HANDLE;
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn MachInit(_CmdLine: *const i8) {
    RtlZeroMemory(
        core::ptr::addr_of_mut!(MachVtbl) as *mut core::ffi::c_void,
        core::mem::size_of::<MACHVTBL>(),
    );

    MachVtbl.ConsPutChar = Some(UefiConsPutChar);
    MachVtbl.ConsKbHit = Some(UefiConsKbHit);
    MachVtbl.ConsGetCh = Some(UefiConsGetCh);
    MachVtbl.VideoClearScreen = Some(UefiVideoClearScreen);
    MachVtbl.VideoSetDisplayMode = Some(UefiVideoSetDisplayMode);
    MachVtbl.VideoGetDisplaySize = Some(UefiVideoGetDisplaySize);
    MachVtbl.VideoGetBufferSize = Some(UefiVideoGetBufferSize);
    MachVtbl.VideoGetFontsFromFirmware = Some(UefiVideoGetFontsFromFirmware);
    MachVtbl.VideoSetTextCursorPosition = Some(UefiVideoSetTextCursorPosition);
    MachVtbl.VideoHideShowTextCursor = Some(UefiVideoHideShowTextCursor);
    MachVtbl.VideoPutChar = Some(UefiVideoPutChar);
    MachVtbl.VideoCopyOffScreenBufferToVRAM = Some(UefiVideoCopyOffScreenBufferToVRAM);
    MachVtbl.VideoIsPaletteFixed = Some(UefiVideoIsPaletteFixed);
    MachVtbl.VideoSetPaletteColor = Some(UefiVideoSetPaletteColor);
    MachVtbl.VideoGetPaletteColor = Some(UefiVideoGetPaletteColor);
    MachVtbl.VideoSync = Some(UefiVideoSync);
    MachVtbl.Beep = Some(UefiPcBeep);
    MachVtbl.PrepareForReactOS = Some(UefiPrepareForReactOS);
    MachVtbl.GetMemoryMap = Some(UefiMemGetMemoryMap);
    MachVtbl.GetExtendedBIOSData = Some(UefiGetExtendedBIOSData);
    MachVtbl.GetFloppyCount = Some(UefiGetFloppyCount);
    MachVtbl.DiskReadLogicalSectors = Some(UefiDiskReadLogicalSectors);
    MachVtbl.DiskGetDriveGeometry = Some(UefiDiskGetDriveGeometry);
    MachVtbl.DiskGetCacheableBlockCount = Some(UefiDiskGetCacheableBlockCount);
    MachVtbl.GetTime = Some(UefiGetTime);
    MachVtbl.InitializeBootDevices = Some(UefiInitializeBootDevices);
    MachVtbl.HwDetect = Some(UefiHwDetect);
    MachVtbl.HwIdle = Some(UefiHwIdle);

    /* Setup GOP */
    if UefiInitializeVideo() != EFI_SUCCESS {
        ERR!("Failed to setup GOP\n");
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Machine Setup
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// #include <uefildr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WARNING);
//
// /* GLOBALS ********************************************************************/
//
// extern EFI_SYSTEM_TABLE* GlobalSystemTable;
// extern EFI_HANDLE GlobalImageHandle;
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// MachInit(const char *CmdLine)
// {
//     RtlZeroMemory(&MachVtbl, sizeof(MachVtbl));
//
//     MachVtbl.ConsPutChar = UefiConsPutChar;
//     MachVtbl.ConsKbHit = UefiConsKbHit;
//     MachVtbl.ConsGetCh = UefiConsGetCh;
//     MachVtbl.VideoClearScreen = UefiVideoClearScreen;
//     MachVtbl.VideoSetDisplayMode = UefiVideoSetDisplayMode;
//     MachVtbl.VideoGetDisplaySize = UefiVideoGetDisplaySize;
//     MachVtbl.VideoGetBufferSize = UefiVideoGetBufferSize;
//     MachVtbl.VideoGetFontsFromFirmware = UefiVideoGetFontsFromFirmware;
//     MachVtbl.VideoSetTextCursorPosition = UefiVideoSetTextCursorPosition;
//     MachVtbl.VideoHideShowTextCursor = UefiVideoHideShowTextCursor;
//     MachVtbl.VideoPutChar = UefiVideoPutChar;
//     MachVtbl.VideoCopyOffScreenBufferToVRAM = UefiVideoCopyOffScreenBufferToVRAM;
//     MachVtbl.VideoIsPaletteFixed = UefiVideoIsPaletteFixed;
//     MachVtbl.VideoSetPaletteColor = UefiVideoSetPaletteColor;
//     MachVtbl.VideoGetPaletteColor = UefiVideoGetPaletteColor;
//     MachVtbl.VideoSync = UefiVideoSync;
//     MachVtbl.Beep = UefiPcBeep;
//     MachVtbl.PrepareForReactOS = UefiPrepareForReactOS;
//     MachVtbl.GetMemoryMap = UefiMemGetMemoryMap;
//     MachVtbl.GetExtendedBIOSData = UefiGetExtendedBIOSData;
//     MachVtbl.GetFloppyCount = UefiGetFloppyCount;
//     MachVtbl.DiskReadLogicalSectors = UefiDiskReadLogicalSectors;
//     MachVtbl.DiskGetDriveGeometry = UefiDiskGetDriveGeometry;
//     MachVtbl.DiskGetCacheableBlockCount = UefiDiskGetCacheableBlockCount;
//     MachVtbl.GetTime = UefiGetTime;
//     MachVtbl.InitializeBootDevices = UefiInitializeBootDevices;
//     MachVtbl.HwDetect = UefiHwDetect;
//     MachVtbl.HwIdle = UefiHwIdle;
//
//     /* Setup GOP */
//     if (UefiInitializeVideo() != EFI_SUCCESS)
//     {
//         ERR("Failed to setup GOP\n");
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
