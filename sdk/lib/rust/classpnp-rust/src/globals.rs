//! globals.rs — file-scope C globals (extern in classp.h, defined across the .c files).
//! Zero/null-init is correct: the C code initialises these at DriverEntry.
#![allow(non_upper_case_globals)]
use ntkrnl_sys::driver_ddk::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{BOOLEAN, KGUARDED_MUTEX, LIST_ENTRY, PVOID, ULONG};

#[no_mangle] pub static mut AllFdosList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut IdlePowerFDOList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut IdlePowerFDOListMutex: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut ScreenStateNotificationHandle: PVOID = core::ptr::null_mut();
#[no_mangle] pub static mut PowerSettingNotificationHandle: PVOID = core::ptr::null_mut();
#[no_mangle] pub static mut ClasspScreenOff: BOOLEAN = 0;
#[no_mangle] pub static mut ClassMaxInterleavePerCriticalIo: ULONG = 4;
#[no_mangle] pub static mut DiskIdleTimeoutInMS: ULONG = 0xFFFFFFFF;
