use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;
use crate::cb;

#[no_mangle]
pub unsafe extern "system" fn AllocFunction(
    ItemSize: ULONG,
) -> PVOID {
    let Item = ExAllocatePoolWithTag(NonPagedPool, ItemSize as _, USBAUDIO_TAG as _);
    if !Item.is_null() {
        RtlZeroMemory(Item, ItemSize as _);
    }
    Item
}

#[no_mangle]
pub unsafe extern "system" fn FreeFunction(Item: PVOID) {
    ExFreePoolWithTag(Item, USBAUDIO_TAG as ULONG);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:     ReactOS Universal Audio Class Driver
// * LICENSE:     GPL - See COPYING in the top level directory
// * FILE:        drivers/usb/usbaudio/pool.c
// * PURPOSE:     USB Audio device driver.
// * PROGRAMMERS:
// *              Johannes Anderwald (johannes.anderwald@reactos.org)
// */
// #include "usbaudio.h"
//
// PVOID
// NTAPI
// AllocFunction(
//     IN ULONG ItemSize)
// {
//     PVOID Item = ExAllocatePoolWithTag(NonPagedPool, ItemSize as _, USBAUDIO_TAG as _);
//     if (Item)
//     {
//         // zero item
//         RtlZeroMemory(Item, ItemSize as _);
//     }
//
//     // done
//     return Item;
// }
//
// VOID
// NTAPI
// FreeFunction(
//     IN PVOID Item)
// {
//     /* free item */
//     ExFreePoolWithTag(Item, USBAUDIO_TAG);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
