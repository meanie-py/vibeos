use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::PCSTR;
use core::ffi::c_void;

// stubs.c — UEFI-only no-op fills for MACHVTBL slots (Video font/sync/BIOS
// data/beep/idle) plus the BIOS drive-map shim that always returns 0 on
// UEFI. Cohabitation: only #[no_mangle] under feature="uefi"; the BIOS
// archive owns these via different .c (StallExecutionProcessor in pchw.c).

// stubs.c's #if !defined(_M_ARM) — translate as "not 32-bit ARM" (arm64
// is allowed). Rust target_arch="arm" is the 32-bit ARM gate.
#[cfg(not(target_arch = "arm"))]
#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn DriveMapGetBiosDriveNumber(_DeviceName: PCSTR) -> u8 {
    0
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn StallExecutionProcessor(_Microseconds: ULONG) {
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoGetFontsFromFirmware(_RomFontPointers: PULONG) {
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoSync() {
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiGetExtendedBIOSData(
    _ExtendedBIOSDataArea: PULONG,
    _ExtendedBIOSDataSize: PULONG,
) {
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiPcBeep() {
    /* Not possible on UEFI, for now */
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiHwIdle() {
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Function stubs
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// #include <uefildr.h>
//
// #if !defined(_M_ARM)
// /* TODO: Handle this with custom Disk / partition setup */
// /* For UEFI builds (including ARM64), always return 0 */
// UCHAR
// DriveMapGetBiosDriveNumber(PCSTR DeviceName)
// {
//     return 0;
// }
// #endif
//
// VOID
// StallExecutionProcessor(ULONG Microseconds)
// {
//
// }
//
// VOID
// UefiVideoGetFontsFromFirmware(PULONG RomFontPointers)
// {
//
// }
//
// VOID
// UefiVideoSync(VOID)
// {
//
// }
//
// VOID
// UefiGetExtendedBIOSData(PULONG ExtendedBIOSDataArea,
//                         PULONG ExtendedBIOSDataSize)
// {
//
// }
//
// VOID
// UefiPcBeep(VOID)
// {
//     /* Not possible on UEFI, for now */
// }
//
// VOID
// UefiHwIdle(VOID)
// {
//
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
