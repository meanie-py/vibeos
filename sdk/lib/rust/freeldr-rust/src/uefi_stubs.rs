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

extern "C" {
    // Owned by uefildr (Rust uefildr.rs or uefildr.c).
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
}

// EFI_BOOT_SERVICES carries these slots as untyped pointers (frldr.rs).
type EfiStall = unsafe extern "C" fn(Microseconds: UINTN) -> EFI_STATUS;
type EfiCreateEvent = unsafe extern "C" fn(
    Type: UINT32,
    NotifyTpl: UINTN,
    NotifyFunction: *mut c_void,
    NotifyContext: *mut c_void,
    Event: *mut EFI_EVENT,
) -> EFI_STATUS;
type EfiSetTimer =
    unsafe extern "C" fn(Event: EFI_EVENT, Type: u32, TriggerTime: u64) -> EFI_STATUS;
type EfiWaitForEvent = unsafe extern "C" fn(
    NumberOfEvents: UINTN,
    Event: *mut EFI_EVENT,
    Index: *mut UINTN,
) -> EFI_STATUS;

const EVT_TIMER: UINT32 = 0x8000_0000;
const TPL_APPLICATION: UINTN = 4;
const TIMER_RELATIVE: u32 = 2; // EFI_TIMER_DELAY TimerRelative

// Upstream dc9ee344787 ("[FREELDR:UEFI] Fix broken boot menu countdown")
// moved StallExecutionProcessor / UefiHwIdle out of stubs.c into uefihw.c with
// real bodies; these were still the old empty stubs, so every UEFI-side stall
// returned at once and the idle loop spun flat out.
#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn StallExecutionProcessor(Microseconds: ULONG) {
    let stall = (*(*GlobalSystemTable).BootServices).Stall;
    if !stall.is_null() {
        let stall: EfiStall = core::mem::transmute(stall);
        stall(Microseconds as UINTN);
    }
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

static mut IdleTimerEvent: EFI_EVENT = core::ptr::null_mut();

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiHwIdle() {
    let boot_services = (*GlobalSystemTable).BootServices;
    let mut index: UINTN = 0;

    // Keep one timer event around and arm it each idle tick
    if IdleTimerEvent.is_null() {
        let create_event: EfiCreateEvent = core::mem::transmute((*boot_services).CreateEvent);
        let status = create_event(
            EVT_TIMER,
            TPL_APPLICATION,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut IdleTimerEvent,
        );
        if EFI_ERROR(status) != 0 {
            StallExecutionProcessor(10000); // 10 ms fallback
            return;
        }
    }

    // Set a 10ms (100,000 * 100ns) relative timer
    let set_timer: EfiSetTimer = core::mem::transmute((*boot_services).SetTimer);
    let mut status = set_timer(IdleTimerEvent, TIMER_RELATIVE, 100000);
    if EFI_ERROR(status) == 0 {
        let wait_for_event: EfiWaitForEvent = core::mem::transmute((*boot_services).WaitForEvent);
        status = wait_for_event(1, &raw mut IdleTimerEvent, &mut index);
    }
    if EFI_ERROR(status) != 0 {
        StallExecutionProcessor(10000); // 10 ms fallback
    }
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
