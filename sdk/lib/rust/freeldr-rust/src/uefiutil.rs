#[allow(unused_imports)] use crate::{ERR};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WARNING; // uefiutil.c: DBG_DEFAULT_CHANNEL(WARNING)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::TIMEINFO;
use core::ffi::c_void;

// FreeLDR ERR no-op (variadic to typecheck format args).
extern "C" {
}

// GlobalSystemTable is OWNED by uefildr.c (or uefildr.rs in future swap);
// uefiutil.c only `extern`-consumed it. Match that — do NOT define here,
// else we get a duplicate-symbol clash with uefildr.c at the UEFI cold link.
extern "C" {
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
}

// Cohabitation: `uefi` feature gates #[no_mangle] (UEFI archive is the
// only consumer; uefiutil.c is commented out in uefi.cmake under feature).
#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiGetTime() -> *mut TIMEINFO {
    static mut TimeInfo: TIMEINFO = TIMEINFO {
        Year: 0,
        Month: 0,
        Day: 0,
        Hour: 0,
        Minute: 0,
        Second: 0,
    };
    let Status: EFI_STATUS;
    let mut time: EFI_TIME = core::mem::zeroed();

    Status = ((*(*GlobalSystemTable).RuntimeServices).GetTime).unwrap()(&mut time, core::ptr::null_mut());
    if Status != EFI_SUCCESS {
        ERR!("UefiGetTime: cannot get time status %d\n");
    }

    // TIMEINFO uses u16 fields (Win32 SYSTEMTIME shape); EFI_TIME stores
    // calendar fields as u8 except Year (u16). Widen the u8 fields.
    TimeInfo.Year = time.Year;
    TimeInfo.Month = time.Month as u16;
    TimeInfo.Day = time.Day as u16;
    TimeInfo.Hour = time.Hour as u16;
    TimeInfo.Minute = time.Minute as u16;
    TimeInfo.Second = time.Second as u16;
    &mut TimeInfo as *mut TIMEINFO
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Utils source
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
// extern EFI_SYSTEM_TABLE *GlobalSystemTable;
//
// /* FUNCTIONS ******************************************************************/
//
// TIMEINFO*
// UefiGetTime(VOID)
// {
//     static TIMEINFO TimeInfo;
//     EFI_STATUS Status;
//     EFI_TIME time = {0};
//
//     Status = GlobalSystemTable->RuntimeServices->GetTime(&time, NULL);
//     if (Status != EFI_SUCCESS)
//         ERR("UefiGetTime: cannot get time status %d\n", Status);
//
//     TimeInfo.Year = time.Year;
//     TimeInfo.Month = time.Month;
//     TimeInfo.Day = time.Day;
//     TimeInfo.Hour = time.Hour;
//     TimeInfo.Minute = time.Minute;
//     TimeInfo.Second = time.Second;
//     return &TimeInfo;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
