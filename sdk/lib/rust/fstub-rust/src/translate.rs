//! ntoskrnl/fstub/translate.c — Interrupt Translator Routines.
//!
//! Hand-translated from C. The C source uses `Source->u.Interrupt.Vector`
//! style union access. In bindgen, `_CM_PARTIAL_RESOURCE_DESCRIPTOR` and
//! `_IO_RESOURCE_DESCRIPTOR` have a NAMED inner field `u` (not an anonymous
//! union), so the access pattern in Rust is `(*x).u.Interrupt.Vector`,
//! same as the existing pnpmgr/pnpres.rs translation.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::ddk::{
    HalGetInterruptVector,
    INTERFACE_TYPE,
    IO_RESOURCE_DESCRIPTOR,
    KAFFINITY,
    KIRQL,
    PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    PDEVICE_OBJECT,
    PINTERFACE_DEREFERENCE,
    PINTERFACE_REFERENCE,
    PIO_RESOURCE_DESCRIPTOR,
    PTRANSLATE_RESOURCE_HANDLER,
    PTRANSLATE_RESOURCE_REQUIREMENTS_HANDLER,
    PTRANSLATOR_INTERFACE,
    PULONG,
    RESOURCE_TRANSLATION_DIRECTION,
    TRANSLATOR_INTERFACE,
    _RESOURCE_TRANSLATION_DIRECTION_TranslateChildToParent as TranslateChildToParent,
    _RESOURCE_TRANSLATION_DIRECTION_TranslateParentToChild as TranslateParentToChild,
    _INTERFACE_TYPE_Internal as Internal,
    _INTERFACE_TYPE_Isa as Isa,
    _INTERFACE_TYPE_MicroChannel as MicroChannel,
    _INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined,
};

extern "system" {
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
}

// CmResourceTypeInterrupt = 2 (cmtypes.h). HAL_IRQ_TRANSLATOR_VERSION = 0x0 (haltypes.h).
// STATUS_TRANSLATION_COMPLETE = 0x00000120 (ntstatus.h). Pool tag 'Fstb' little-endian = b"btsF".
const CM_RESOURCE_TYPE_INTERRUPT: u8 = 2;
const HAL_IRQ_TRANSLATOR_VERSION: u16 = 0x0;
const STATUS_TRANSLATION_COMPLETE: NTSTATUS = 0x00000120;

/// FstubTranslatorNull — does nothing. Stand-in for InterfaceReference/Dereference
/// callbacks in the translator interface.
#[no_mangle]
pub unsafe extern "system" fn FstubTranslatorNull(_Context: PVOID) {
    /* Do nothing */
}

/// FstubTranslateResource — translate a CM_PARTIAL_RESOURCE_DESCRIPTOR
/// between bus-relative (child) and system (parent) interrupt vectors.
#[no_mangle]
pub unsafe extern "system" fn FstubTranslateResource(
    Context: PVOID,
    Source: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Direction: RESOURCE_TRANSLATION_DIRECTION,
    AlternativesCount: ULONG,
    Alternatives: *mut IO_RESOURCE_DESCRIPTOR,
    _PhysicalDeviceObject: PDEVICE_OBJECT,
    Target: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    let mut Irql: KIRQL = 0;
    let mut Affinity: KAFFINITY = 0;
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;

    ntkrnl_sys::ASSERT!((*Source).Type == CM_RESOURCE_TYPE_INTERRUPT);

    /* Copy common information */
    (*Target).Type = (*Source).Type;
    (*Target).ShareDisposition = (*Source).ShareDisposition;
    (*Target).Flags = (*Source).Flags;

    if Direction == TranslateChildToParent {
        /* Get IRQL, affinity & system vector for the device vector */
        (*Target).u.Interrupt.Vector = HalGetInterruptVector(
            Context as usize as INTERFACE_TYPE,
            0,
            (*Source).u.Interrupt.Vector,
            (*Source).u.Interrupt.Vector,
            &mut Irql,
            &mut Affinity,
        );
        (*Target).u.Interrupt.Level = Irql as ULONG;
        (*Target).u.Interrupt.Affinity = Affinity;
        Status = STATUS_TRANSLATION_COMPLETE;
    } else if Direction == TranslateParentToChild {
        /* Browse all the resources */
        let mut k: ULONG = 0;
        while k < AlternativesCount {
            let Alternative: PIO_RESOURCE_DESCRIPTOR = Alternatives.add(k as usize);

            ntkrnl_sys::ASSERT!((*Alternative).Type == CM_RESOURCE_TYPE_INTERRUPT);

            /* Try to find the device vector by trial & error: try a vector
             * and translate it. */
            let mut MinimumVector: ULONG = (*Alternative).u.Interrupt.MinimumVector;
            while MinimumVector <= (*Alternative).u.Interrupt.MaximumVector {
                /* Translate the vector */
                let Vector = HalGetInterruptVector(
                    Context as usize as INTERFACE_TYPE,
                    0,
                    MinimumVector,
                    MinimumVector,
                    &mut Irql,
                    &mut Affinity,
                );

                /* If the translated vector matches the given translated vector */
                if Vector == (*Source).u.Interrupt.Vector {
                    /* We are done — send back the device vector */
                    (*Target).u.Interrupt.Affinity = !0 as KAFFINITY;
                    (*Target).u.Interrupt.Vector = MinimumVector;
                    (*Target).u.Interrupt.Level = MinimumVector;

                    return STATUS_SUCCESS;
                }

                MinimumVector = MinimumVector.wrapping_add(1);
            }

            k = k.wrapping_add(1);
        }
    }

    Status
}

/// FstubTranslateRequirement — translate an IO_RESOURCE_DESCRIPTOR (an
/// interrupt requirement) into one or more equivalent system-side requirements.
#[no_mangle]
pub unsafe extern "system" fn FstubTranslateRequirement(
    Context: PVOID,
    Source: PIO_RESOURCE_DESCRIPTOR,
    _PhysicalDeviceObject: PDEVICE_OBJECT,
    TargetCount: PULONG,
    Target: *mut PIO_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    let mut Irql: KIRQL = 0;
    let mut Affinity: KAFFINITY = 0;

    ntkrnl_sys::ASSERT!((*Source).Type == CM_RESOURCE_TYPE_INTERRUPT);

    /* Allocate output buffer. Pool tag 'Fstb' (little-endian byte order). */
    let allocation = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>(),
        u32::from_le_bytes(*b"Fstb"),
    ) as PIO_RESOURCE_DESCRIPTOR;
    *Target = allocation;
    if allocation.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero & set out count to 1 */
    core::ptr::write_bytes(allocation as *mut u8, 0, core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>());
    *TargetCount = 1;

    /* Translate minimum interrupt vector */
    (*allocation).u.Interrupt.MinimumVector = HalGetInterruptVector(
        Context as usize as INTERFACE_TYPE,
        0,
        (*Source).u.Interrupt.MinimumVector,
        (*Source).u.Interrupt.MinimumVector,
        &mut Irql,
        &mut Affinity,
    );

    /* Translate maximum interrupt vector */
    (*allocation).u.Interrupt.MaximumVector = HalGetInterruptVector(
        Context as usize as INTERFACE_TYPE,
        0,
        (*Source).u.Interrupt.MaximumVector,
        (*Source).u.Interrupt.MaximumVector,
        &mut Irql,
        &mut Affinity,
    );

    STATUS_TRANSLATION_COMPLETE
}

/// xHalGetInterruptTranslator — populate a TRANSLATOR_INTERFACE for an
/// (E)ISA bus bridging into a parent bus.
#[no_mangle]
pub unsafe extern "system" fn xHalGetInterruptTranslator(
    _ParentInterfaceType: INTERFACE_TYPE,
    _ParentBusNumber: ULONG,
    BridgeInterfaceType: INTERFACE_TYPE,
    Size: USHORT,
    Version: USHORT,
    Translator: PTRANSLATOR_INTERFACE,
    _BridgeBusNumber: PULONG,
) -> NTSTATUS {
    ntkrnl_sys::ASSERT!(Version == HAL_IRQ_TRANSLATOR_VERSION);
    ntkrnl_sys::ASSERT!(Size as usize >= core::mem::size_of::<TRANSLATOR_INTERFACE>());

    /* Only (E)ISA interfaces are supported */
    if BridgeInterfaceType == Internal || BridgeInterfaceType >= MicroChannel {
        return STATUS_NOT_IMPLEMENTED;
    }

    /* Fill in output struct */
    (*Translator).Size = core::mem::size_of::<TRANSLATOR_INTERFACE>() as USHORT;
    (*Translator).Version = HAL_IRQ_TRANSLATOR_VERSION;
    /* If the caller set the interface to undefined, default it to ISA. */
    let context_iface: INTERFACE_TYPE = if BridgeInterfaceType == InterfaceTypeUndefined {
        Isa
    } else {
        BridgeInterfaceType
    };
    (*Translator).Context = context_iface as usize as PVOID;
    /* The bindgen function pointer types use NTAPI calling convention
     * (stdcall on i386, default on amd64/arm64). Our `#[no_mangle] extern "system"`
     * functions emit the same ABI at the asm level, but Rust's type system
     * treats `extern "system"` and bindgen's `extern "C"`/`extern "stdcall"`
     * as distinct types — so we need `transmute` to convert. */
    (*Translator).InterfaceReference =
        core::mem::transmute::<usize, PINTERFACE_REFERENCE>(
            FstubTranslatorNull as usize,
        );
    (*Translator).InterfaceDereference =
        core::mem::transmute::<usize, PINTERFACE_DEREFERENCE>(
            FstubTranslatorNull as usize,
        );
    (*Translator).TranslateResources =
        core::mem::transmute::<usize, PTRANSLATE_RESOURCE_HANDLER>(
            FstubTranslateResource as usize,
        );
    (*Translator).TranslateResourceRequirements =
        core::mem::transmute::<usize, PTRANSLATE_RESOURCE_REQUIREMENTS_HANDLER>(
            FstubTranslateRequirement as usize,
        );

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/fstub/translate.c
// * PURPOSE:         Interrupt Translator Routines
// * PROGRAMMERS:     Pierre Schweitzer (pierre.schweitzer@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FstubTranslatorNull(PVOID Context)
// {
//     PAGED_CODE();
//
//     /* Do nothing */
//     return;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FstubTranslateResource(IN OUT PVOID Context OPTIONAL,
//                        IN PCM_PARTIAL_RESOURCE_DESCRIPTOR Source,
//                        IN RESOURCE_TRANSLATION_DIRECTION Direction,
//                        IN ULONG AlternativesCount OPTIONAL,
//                        IN IO_RESOURCE_DESCRIPTOR Alternatives[],
//                        IN PDEVICE_OBJECT PhysicalDeviceObject,
//                        OUT PCM_PARTIAL_RESOURCE_DESCRIPTOR Target)
// {
//     KIRQL Irql;
//     KAFFINITY Affinity;
//     ULONG MinimumVector, Vector, k;
//     PIO_RESOURCE_DESCRIPTOR Alternative;
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//     PAGED_CODE();
//
//     ASSERT(Source->Type == CmResourceTypeInterrupt);
//
//     /* Copy common information */
//     Target->Type = Source->Type;
//     Target->ShareDisposition = Source->ShareDisposition;
//     Target->Flags = Source->Flags;
//
//     if (Direction == TranslateChildToParent)
//     {
//         /* Get IRQL, affinity & system vector for the device vector */
//         Target->u.Interrupt.Vector = HalGetInterruptVector((INTERFACE_TYPE)Context, 0,
//                                                            Source->u.Interrupt.Vector,
//                                                            Source->u.Interrupt.Vector,
//                                                            &Irql, &Affinity);
//         Target->u.Interrupt.Level = Irql;
//         Target->u.Interrupt.Affinity = Affinity;
//         Status = STATUS_TRANSLATION_COMPLETE;
//     }
//     else if (Direction == TranslateParentToChild)
//     {
//         /* Browse all the resources */
//         for (k = 0; k < AlternativesCount; k++)
//         {
//             Alternative = &(Alternatives[k]);
//
//             ASSERT(Alternative->Type == CmResourceTypeInterrupt);
//
//             /* Try to find the device vector, proceeding by trial & error
//              * We try a vector, and translate it
//              */
//             MinimumVector = Alternative->u.Interrupt.MinimumVector;
//             while (MinimumVector <= Alternative->u.Interrupt.MaximumVector)
//             {
//                 /* Translate the vector */
//                 Vector = HalGetInterruptVector((INTERFACE_TYPE)Context, 0,
//                                                MinimumVector,
//                                                MinimumVector,
//                                                &Irql, &Affinity);
//
//                 /* If the translated vector is matching the given translated vector */
//                 if (Vector == Source->u.Interrupt.Vector)
//                 {
//                     /* We are done, send back device vector */
//                     Target->u.Interrupt.Affinity = -1;
//                     Target->u.Interrupt.Vector = MinimumVector;
//                     Target->u.Interrupt.Level = MinimumVector;
//
//                     return STATUS_SUCCESS;
//                 }
//
//                 MinimumVector++;
//             }
//         }
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FstubTranslateRequirement(IN OUT PVOID Context OPTIONAL,
//                           IN PIO_RESOURCE_DESCRIPTOR Source,
//                           IN PDEVICE_OBJECT PhysicalDeviceObject,
//                           OUT PULONG TargetCount,
//                           OUT PIO_RESOURCE_DESCRIPTOR *Target)
// {
//     KIRQL Irql;
//     KAFFINITY Affinity;
//     PAGED_CODE();
//
//     ASSERT(Source->Type == CmResourceTypeInterrupt);
//
//     /* Allocate output buffer */
//     *Target = ExAllocatePoolWithTag(PagedPool, sizeof(IO_RESOURCE_DESCRIPTOR), 'btsF');
//     if (!*Target)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Zero & set out count to 1 */
//     RtlZeroMemory(*Target, sizeof(IO_RESOURCE_DESCRIPTOR));
//     *TargetCount = 1;
//
//     /* Translate minimum interrupt vector */
//     (*Target)->u.Interrupt.MinimumVector = HalGetInterruptVector((INTERFACE_TYPE)Context, 0,
//                                                                  Source->u.Interrupt.MinimumVector,
//                                                                  Source->u.Interrupt.MinimumVector,
//                                                                  &Irql, &Affinity);
//
//     /* Translate maximum interrupt vector */
//     (*Target)->u.Interrupt.MaximumVector = HalGetInterruptVector((INTERFACE_TYPE)Context, 0,
//                                                                  Source->u.Interrupt.MaximumVector,
//                                                                  Source->u.Interrupt.MaximumVector,
//                                                                  &Irql, &Affinity);
//
//     return STATUS_TRANSLATION_COMPLETE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// xHalGetInterruptTranslator(IN INTERFACE_TYPE ParentInterfaceType,
//                            IN ULONG ParentBusNumber,
//                            IN INTERFACE_TYPE BridgeInterfaceType,
//                            IN USHORT Size,
//                            IN USHORT Version,
//                            OUT PTRANSLATOR_INTERFACE Translator,
//                            OUT PULONG BridgeBusNumber)
// {
//     PAGED_CODE();
//
//     ASSERT(Version == HAL_IRQ_TRANSLATOR_VERSION);
//     ASSERT(Size >= sizeof(TRANSLATOR_INTERFACE));
//
//     /* Only (E)ISA interfaces are supported */
//     if (BridgeInterfaceType == Internal || BridgeInterfaceType >= MicroChannel)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     /* Fill in output struct */
//     Translator->Size = sizeof(TRANSLATOR_INTERFACE);
//     Translator->Version = HAL_IRQ_TRANSLATOR_VERSION;
//     /* In case caller set interface to undefined, faulty it to ISA */
//     Translator->Context = UlongToPtr((BridgeInterfaceType == InterfaceTypeUndefined) ? Isa : BridgeInterfaceType);
//     Translator->InterfaceReference = FstubTranslatorNull;
//     Translator->InterfaceDereference = FstubTranslatorNull;
//     Translator->TranslateResources = FstubTranslateResource;
//     Translator->TranslateResourceRequirements = FstubTranslateRequirement;
//
//     return STATUS_SUCCESS;
// }
