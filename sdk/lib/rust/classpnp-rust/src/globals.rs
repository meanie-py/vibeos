//! globals.rs — file-scope C globals (extern in classp.h, defined across the .c files).
//! These are THE definitions every `crate::types` extern binds to (no_mangle). Keep them
//! single-authority: duplicated non-no_mangle twins in class.rs/data.rs caused a
//! split-brain where ClassInitialize lazily initialized its LOCAL IdlePowerFDOList while
//! power.rs walked THIS zeroed one, and a zeroed AllFdosList AV'd InsertTailList in
//! ClassPnpStartDevice on the first disk start (USB-disk bring-up, 2026-07-06).
#![allow(non_upper_case_globals)]
use ntkrnl_sys::driver_ddk::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{BOOLEAN, KGUARDED_MUTEX, LIST_ENTRY, PVOID, ULONG};

// C data.c: `LIST_ENTRY AllFdosList = {&AllFdosList, &AllFdosList};` — self-referential
// empty list head, NOT zero.
#[no_mangle] pub static mut AllFdosList: LIST_ENTRY = LIST_ENTRY {
    Flink: &raw const AllFdosList as *mut _,
    Blink: &raw const AllFdosList as *mut _,
};
// C class.c: `LIST_ENTRY IdlePowerFDOList = {0};` — zero-init IS correct here; the C
// (and Rust ClassInitialize, class.rs) lazily InitializeListHead()s it on first use.
#[no_mangle] pub static mut IdlePowerFDOList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut IdlePowerFDOListMutex: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut ScreenStateNotificationHandle: PVOID = core::ptr::null_mut();
#[no_mangle] pub static mut PowerSettingNotificationHandle: PVOID = core::ptr::null_mut();
#[no_mangle] pub static mut ClasspScreenOff: BOOLEAN = 0;
#[no_mangle] pub static mut ClassMaxInterleavePerCriticalIo: ULONG = 4;
#[no_mangle] pub static mut DiskIdleTimeoutInMS: ULONG = 0xFFFFFFFF;
