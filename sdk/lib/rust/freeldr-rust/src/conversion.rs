#[allow(unused_imports)] use crate::{TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WINDOWS; // conversion.c: DBG_DEFAULT_CHANNEL(WINDOWS)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCONFIGURATION_COMPONENT_DATA};
use core::ffi::c_void;

// PaToVa/VaToPa are FORCEINLINE in conversion.h (no external symbol in C),
// but several translated TUs (registry/wlregistry/winldr/setupldr)
// extern-call them as real cdecl `PVOID (*)(PVOID)` symbols. conversion.rs
// is the conversion.c replacement → it owns the out-of-line definitions
// (cdecl wrappers `PaToVa`/`VaToPa` further below). The generic ffi impl
// (per-arch KSEG0_BASE) stays in scope under aliases so the typed
// call-sites in this module keep the caller pointee type (no cast churn).
use crate::ffi::{PaToVa as PaToVaG, VaToPa as VaToPaG};

extern "C" {
}

#[no_mangle]
pub unsafe extern "C" fn PaToVa(Pa: *mut c_void) -> *mut c_void {
    PaToVaG(Pa as *const c_void)
}
#[no_mangle]
pub unsafe extern "C" fn VaToPa(Va: *mut c_void) -> *mut c_void {
    VaToPaG(Va as *const c_void)
}

#[no_mangle]
pub unsafe extern "C" fn List_PaToVa(ListHeadPa: PLIST_ENTRY) {

    let mut EntryPa: PLIST_ENTRY;
    let mut NextPa: PLIST_ENTRY;

    EntryPa = (*ListHeadPa).Flink;
    while EntryPa != ListHeadPa {
        NextPa = (*EntryPa).Flink;

        (*EntryPa).Flink = PaToVaG((*EntryPa).Flink);
        (*EntryPa).Blink = PaToVaG((*EntryPa).Blink);

        EntryPa = NextPa;
    }

    (*ListHeadPa).Flink = PaToVaG((*ListHeadPa).Flink);
    (*ListHeadPa).Blink = PaToVaG((*ListHeadPa).Blink);
}

#[no_mangle]
pub unsafe extern "C" fn ConvertConfigToVA(Start: PCONFIGURATION_COMPONENT_DATA) {

    let mut Child: PCONFIGURATION_COMPONENT_DATA;
    let mut Sibling: PCONFIGURATION_COMPONENT_DATA;

    TRACE!("ConvertConfigToVA(Start 0x%X)\n", (Start) as usize);
    Child = Start;

    while !Child.is_null() {
        if !(*Child).ConfigurationData.is_null() {
            (*Child).ConfigurationData = PaToVaG((*Child).ConfigurationData);
        }

        if !(*Child).Child.is_null() {
            (*Child).Child = PaToVaG((*Child).Child as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
        }

        if !(*Child).Parent.is_null() {
            (*Child).Parent = PaToVaG((*Child).Parent as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
        }

        if !(*Child).Sibling.is_null() {
            (*Child).Sibling = PaToVaG((*Child).Sibling as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
        }

        if !(*Child).ComponentEntry.Identifier.is_null() {
            (*Child).ComponentEntry.Identifier = PaToVaG((*Child).ComponentEntry.Identifier as *mut core::ffi::c_void) as *mut i8;
        }

        TRACE!("Device 0x%X class %d type %d id '%s', parent %p\n", (Child) as usize, ((*Child).ComponentEntry.Class) as usize, ((*Child).ComponentEntry.Type) as usize, (VaToPaG((*Child).ComponentEntry.Identifier as *mut core::ffi::c_void)) as usize, ((*Child).Parent) as usize);

        Sibling = VaToPaG((*Child).Sibling as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
        while !Sibling.is_null() {
            if !(*Sibling).ConfigurationData.is_null() {
                (*Sibling).ConfigurationData = PaToVaG((*Sibling).ConfigurationData);
            }

            if !(*Sibling).Child.is_null() {
                (*Sibling).Child = PaToVaG((*Sibling).Child as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
            }

            if !(*Sibling).Parent.is_null() {
                (*Sibling).Parent = PaToVaG((*Sibling).Parent as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
            }

            if !(*Sibling).Sibling.is_null() {
                (*Sibling).Sibling = PaToVaG((*Sibling).Sibling as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
            }

            if !(*Sibling).ComponentEntry.Identifier.is_null() {
                (*Sibling).ComponentEntry.Identifier = PaToVaG((*Sibling).ComponentEntry.Identifier as *mut core::ffi::c_void) as *mut i8;
            }

            TRACE!("Device 0x%X class %d type %d id '%s', parent %p\n", (Sibling) as usize, ((*Sibling).ComponentEntry.Class) as usize, ((*Sibling).ComponentEntry.Type) as usize, (VaToPaG((*Sibling).ComponentEntry.Identifier as *mut core::ffi::c_void)) as usize, ((*Sibling).Parent) as usize);

            if !VaToPaG((*Sibling).Child as *mut core::ffi::c_void).is_null() {
                ConvertConfigToVA(VaToPaG((*Sibling).Child as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA);
            }

            Sibling = VaToPaG((*Sibling).Sibling as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
        }

        Child = VaToPaG((*Child).Child as *mut core::ffi::c_void) as PCONFIGURATION_COMPONENT_DATA;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         EFI Windows Loader
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/windows/conversion.c
//  * PURPOSE:         Physical <-> Virtual addressing mode conversions
//  * PROGRAMMERS:     Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <freeldr.h>
//
// //#include <ndk/ldrtypes.h>
// #include <debug.h>
//
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// /* FUNCTIONS **************************************************************/
//
// VOID
// List_PaToVa(_In_ PLIST_ENTRY ListHeadPa)
// {
//     PLIST_ENTRY EntryPa, NextPa;
//
//     /* List must be properly initialized */
//     ASSERT(ListHeadPa->Flink != 0);
//     ASSERT(ListHeadPa->Blink != 0);
//
//     /* Loop the list in physical address space */
//     EntryPa = ListHeadPa->Flink;
//     while (EntryPa != ListHeadPa)
//     {
//         /* Save the physical address of the next entry */
//         NextPa = EntryPa->Flink;
//
//         /* Convert the addresses of this entry */
//         EntryPa->Flink = PaToVa(EntryPa->Flink);
//         EntryPa->Blink = PaToVa(EntryPa->Blink);
//
//         /* Go to the next entry */
//         EntryPa = NextPa;
//     }
//
//     /* Finally convert the list head */
//     ListHeadPa->Flink = PaToVa(ListHeadPa->Flink);
//     ListHeadPa->Blink = PaToVa(ListHeadPa->Blink);
// }
//
// // This function converts only Child->Child, and calls itself for each Sibling
// VOID
// ConvertConfigToVA(PCONFIGURATION_COMPONENT_DATA Start)
// {
//     PCONFIGURATION_COMPONENT_DATA Child;
//     PCONFIGURATION_COMPONENT_DATA Sibling;
//
//     TRACE("ConvertConfigToVA(Start 0x%X)\n", (Start) as usize);
//     Child = Start;
//
//     while (Child != NULL)
//     {
//         if (Child->ConfigurationData)
//             Child->ConfigurationData = PaToVa(Child->ConfigurationData);
//
//         if (Child->Child)
//             Child->Child = PaToVa(Child->Child);
//
//         if (Child->Parent)
//             Child->Parent = PaToVa(Child->Parent);
//
//         if (Child->Sibling)
//             Child->Sibling = PaToVa(Child->Sibling);
//
//         if (Child->ComponentEntry.Identifier)
//             Child->ComponentEntry.Identifier = PaToVa(Child->ComponentEntry.Identifier);
//
//         TRACE("Device 0x%X class %d type %d id '%s', parent %p\n", (Child) as usize, (//             Child->ComponentEntry.Class) as usize, (Child->ComponentEntry.Type) as usize, (VaToPa(Child->ComponentEntry.Identifier)) as usize, (Child->Parent) as usize);
//
//         // Go through siblings list
//         Sibling = VaToPa(Child->Sibling);
//         while (Sibling != NULL)
//         {
//             if (Sibling->ConfigurationData)
//                 Sibling->ConfigurationData = PaToVa(Sibling->ConfigurationData);
//
//             if (Sibling->Child)
//                 Sibling->Child = PaToVa(Sibling->Child);
//
//             if (Sibling->Parent)
//                 Sibling->Parent = PaToVa(Sibling->Parent);
//
//             if (Sibling->Sibling)
//                 Sibling->Sibling = PaToVa(Sibling->Sibling);
//
//             if (Sibling->ComponentEntry.Identifier)
//                 Sibling->ComponentEntry.Identifier = PaToVa(Sibling->ComponentEntry.Identifier);
//
//             TRACE("Device 0x%X class %d type %d id '%s', parent %p\n", (Sibling) as usize, (//                 Sibling->ComponentEntry.Class) as usize, (Sibling->ComponentEntry.Type) as usize, (VaToPa(Sibling->ComponentEntry.Identifier)) as usize, (Sibling->Parent) as usize);
//
//             // Recurse into the Child tree
//             if (VaToPa(Sibling->Child) != NULL)
//                 ConvertConfigToVA(VaToPa(Sibling->Child));
//
//             Sibling = VaToPa(Sibling->Sibling);
//         }
//
//         // Go to the next child
//         Child = VaToPa(Child->Child);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
