#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_HWDETECT; // hwapm.c: DBG_DEFAULT_CHANNEL(HWDETECT)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_LIST, PCONFIGURATION_COMPONENT_DATA, PCSTR};
use core::ffi::c_void;

// MultiFunctionAdapter — bindgen prefixed (`_CONFIGURATION_TYPE_*`); alias
// here to the unprefixed name the original C uses.
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter;

extern "C" {
    fn Int386(interrupt: i32, in_regs: *mut REGS, out_regs: *mut REGS);
    fn FrLdrHeapAlloc(size: usize, tag: u32) -> *mut core::ffi::c_void;
    fn FldrCreateComponentKey(
        Parent: PCONFIGURATION_COMPONENT_DATA,
        Class: u32,
        Type: u32,
        Flags: ULONG,
        Key: ULONG,
        Affinity: ULONG,
        Identifier: PCSTR,
        Data: PCM_PARTIAL_RESOURCE_LIST,
        Size: ULONG,
        ComponentKey: *mut PCONFIGURATION_COMPONENT_DATA,
    );
}

unsafe fn FindApmBios() -> BOOLEAN {
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();

    // APM BIOS - Installation check. 'PM' is a C multi-char constant: in
    // big-endian char order, ('P' << 8) | 'M' == 0x504D — but C
    // multi-char literals are implementation-defined, and on Clang/GCC
    // x86 it evaluates to 0x504D. ReactOS depends on this matching the
    // BIOS-returned BX (which is the ASCII bytes 'P','M' in that order).
    RegsIn.w.ax = 0x5300;
    RegsIn.w.bx = 0x0000;
    Int386(0x15, &mut RegsIn, &mut RegsOut);
    if INT386_SUCCESS(RegsOut) != 0 && RegsOut.w.bx == 0x504D {
        TRACE!("Found APM BIOS\n");
        TRACE!("AH: %x\n", RegsOut.b.ah as u32);
        TRACE!("AL: %x\n", RegsOut.b.al as u32);
        TRACE!("BH: %x\n", RegsOut.b.bh as u32);
        TRACE!("BL: %x\n", RegsOut.b.bl as u32);
        TRACE!("CX: %x\n", RegsOut.w.cx as u32);

        return 1;
    }

    TRACE!("No APM BIOS found\n");

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn DetectApmBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG) {
    let mut BiosKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let Size: ULONG;

    if FindApmBios() == 0 {
        return;
    }

    Size = core::mem::offset_of!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG;
    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }
    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 0;
    (*PartialResourceList).Revision = 0;
    (*PartialResourceList).Count = 0;

    FldrCreateComponentKey(
        SystemKey,
        AdapterClass,
        MultiFunctionAdapter,
        0,
        0,
        0xFFFFFFFF,
        b"APM\0".as_ptr() as PCSTR,
        PartialResourceList,
        Size,
        &mut BiosKey,
    );

    *BusNumber += 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     APM BIOS detection routines
//  * COPYRIGHT:   Copyright 2004 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(HWDETECT);
//
// static BOOLEAN
// FindApmBios(VOID)
// {
//     REGS RegsIn, RegsOut;
//
//     /* APM BIOS - Installation check */
//     RegsIn.w.ax = 0x5300;
//     RegsIn.w.bx = 0x0000;
//     Int386(0x15, &RegsIn, &RegsOut);
//     if (INT386_SUCCESS(RegsOut) && RegsOut.w.bx == 'PM')
//     {
//         TRACE("Found APM BIOS\n");
//         TRACE("AH: %x\n", RegsOut.b.ah);
//         TRACE("AL: %x\n", RegsOut.b.al);
//         TRACE("BH: %x\n", RegsOut.b.bh);
//         TRACE("BL: %x\n", RegsOut.b.bl);
//         TRACE("CX: %x\n", RegsOut.w.cx);
//
//         return TRUE;
//     }
//
//     TRACE("No APM BIOS found\n");
//
//     return FALSE;
// }
//
// VOID
// DetectApmBios(PCONFIGURATION_COMPONENT_DATA SystemKey, ULONG *BusNumber)
// {
//     PCONFIGURATION_COMPONENT_DATA BiosKey;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     ULONG Size;
//
//     if (!FindApmBios())
//         return;
//
//     /* Set 'Configuration Data' value */
//     Size = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return;
//     }
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version = 0;
//     PartialResourceList->Revision = 0;
//     PartialResourceList->Count = 0;
//
//     /* FIXME: Add configuration data */
//
//     /* Create new bus key */
//     FldrCreateComponentKey(SystemKey,
//                            AdapterClass,
//                            MultiFunctionAdapter,
//                            0,
//                            0,
//                            0xFFFFFFFF,
//                            "APM",
//                            PartialResourceList,
//                            Size,
//                            &BiosKey);
//
//     /* Increment bus number */
//     (*BusNumber)++;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
