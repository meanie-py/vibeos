use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PCONFIGURATION_COMPONENT_DATA,
    CONFIGURATION_CLASS,
    CONFIGURATION_TYPE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn KeFindConfigurationEntry(
    Child: PCONFIGURATION_COMPONENT_DATA,
    Class: CONFIGURATION_CLASS,
    Type: CONFIGURATION_TYPE,
    ComponentKey: PULONG,
) -> PCONFIGURATION_COMPONENT_DATA {
    let ComponentKey = opaque_ptr(ComponentKey);
    let mut NextLink: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();

    KeFindConfigurationNextEntry(
        Child,
        Class,
        Type,
        ComponentKey,
        core::ptr::addr_of_mut!(NextLink),
    )
}

#[no_mangle]
pub unsafe extern "system" fn KeFindConfigurationNextEntry(
    mut Child: PCONFIGURATION_COMPONENT_DATA,
    Class: CONFIGURATION_CLASS,
    Type: CONFIGURATION_TYPE,
    ComponentKey: PULONG,
    NextLink: *mut PCONFIGURATION_COMPONENT_DATA,
) -> PCONFIGURATION_COMPONENT_DATA {
    let ComponentKey = opaque_ptr(ComponentKey);
    let NextLink = opaque_ptr(NextLink);
    let mut Key: ULONG = 0;
    let mut Mask: ULONG = 0;

    if !ComponentKey.is_null() {
        Key = *ComponentKey;
        Mask = u32::MAX;
    }

    while !Child.is_null() {
        if !(*NextLink).is_null() {
            if Child == *NextLink {
                *NextLink = core::ptr::null_mut();
            }
        } else {
            if (*Child).ComponentEntry.Class == Class
                && (*Child).ComponentEntry.Type == Type
                && ((*Child).ComponentEntry.Key & Mask) == Key
            {
                return Child;
            }
        }

        let mut Sibling = (*Child).Sibling;
        while !Sibling.is_null() {
            if !(*NextLink).is_null() {
                if Sibling == *NextLink {
                    *NextLink = core::ptr::null_mut();
                }
            } else {
                if (*Sibling).ComponentEntry.Class == Class
                    && (*Sibling).ComponentEntry.Type == Type
                    && ((*Sibling).ComponentEntry.Key & Mask) == Key
                {
                    return Sibling;
                }
            }

            if !(*Sibling).Child.is_null() {
                let ReturnEntry = KeFindConfigurationNextEntry(
                    (*Sibling).Child,
                    Class,
                    Type,
                    ComponentKey,
                    NextLink,
                );
                if !ReturnEntry.is_null() {
                    return ReturnEntry;
                }
            }

            Sibling = (*Sibling).Sibling;
        }

        Child = (*Child).Child;
    }

    core::ptr::null_mut()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/config.c
//  * PURPOSE:         Configuration Tree Routines
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// PCONFIGURATION_COMPONENT_DATA
// NTAPI
// KeFindConfigurationEntry(IN PCONFIGURATION_COMPONENT_DATA Child,
//                          IN CONFIGURATION_CLASS Class,
//                          IN CONFIGURATION_TYPE Type,
//                          IN PULONG ComponentKey OPTIONAL)
// {
//     PCONFIGURATION_COMPONENT_DATA NextLink = NULL;
//
//     /* Start Search at Root */
//     return KeFindConfigurationNextEntry(Child,
//                                         Class,
//                                         Type,
//                                         ComponentKey,
//                                         &NextLink);
// }
//
// /*
//  * @implemented
//  */
// PCONFIGURATION_COMPONENT_DATA
// NTAPI
// KeFindConfigurationNextEntry(IN PCONFIGURATION_COMPONENT_DATA Child,
//                              IN CONFIGURATION_CLASS Class,
//                              IN CONFIGURATION_TYPE Type,
//                              IN PULONG ComponentKey OPTIONAL,
//                              IN PCONFIGURATION_COMPONENT_DATA *NextLink)
// {
//     ULONG Key = 0;
//     ULONG Mask = 0;
//     PCONFIGURATION_COMPONENT_DATA Sibling;
//     PCONFIGURATION_COMPONENT_DATA ReturnEntry;
//
//     /* If we did get a key, then use it instead */
//     if (ComponentKey)
//     {
//         Key = *ComponentKey;
//         Mask = -1;
//     }
//
//     /* Loop the Components until we find a a match */
//     while (Child)
//     {
//         /* Check if we are starting somewhere already */
//         if (*NextLink)
//         {
//             /* If we've found the place where we started, clear and continue */
//             if (Child == *NextLink) *NextLink = NULL;
//         }
//         else
//         {
//             /* Try to get a match */
//             if ((Child->ComponentEntry.Class) == Class &&
//                 (Child->ComponentEntry.Type) == Type &&
//                 (Child->ComponentEntry.Key & Mask) == Key)
//             {
//                 /* Match found */
//                 return Child;
//             }
//         }
//
//         /* Now we've also got to lookup the siblings */
//         Sibling = Child->Sibling;
//         while (Sibling)
//         {
//             /* Check if we are starting somewhere already */
//             if (*NextLink)
//             {
//                 /* If we've found the place where we started, clear and continue */
//                 if (Sibling == *NextLink) *NextLink = NULL;
//             }
//             else
//             {
//                 /* Try to get a match */
//                 if ((Sibling->ComponentEntry.Class == Class) &&
//                     (Sibling->ComponentEntry.Type == Type) &&
//                     (Sibling->ComponentEntry.Key & Mask) == Key)
//                 {
//                     /* Match found */
//                     return Sibling;
//                 }
//             }
//
//             /* We've got to check if the Sibling has a Child as well */
//             if (Sibling->Child)
//             {
//                 /* We're just going to call ourselves again */
//                 ReturnEntry = KeFindConfigurationNextEntry(Sibling->Child,
//                                                            Class,
//                                                            Type,
//                                                            ComponentKey,
//                                                            NextLink);
//                 if (ReturnEntry) return ReturnEntry;
//             }
//
//             /* Next Sibling */
//             Sibling = Sibling->Sibling;
//         }
//
//         /* Next Child */
//         Child = Child->Child;
//     }
//
//     /* If we got here, nothing was found */
//     return NULL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
