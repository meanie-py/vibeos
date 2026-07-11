use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    KeBugCheck,
    PXSTATE_CONFIGURATION,
    XSAVE_AREA,
    XSAVE_FORMAT,
    XSTATE_CONFIGURATION,
};

const XSTATE_PKRU: ULONG = 9;
const XSTATE_HDC: ULONG = 13;
const XSTATE_UINTR: ULONG = 14;
const XSTATE_LBR: ULONG = 15;
const XSTATE_MASK_PKRU: u64 = 1u64 << XSTATE_PKRU;
const XSTATE_MASK_HDC: u64 = 1u64 << XSTATE_HDC;
const XSTATE_MASK_UINTR: u64 = 1u64 << XSTATE_UINTR;
const XSTATE_MASK_LBR: u64 = 1u64 << XSTATE_LBR;

const XSTATE_MASK_SUPERVISOR: u64 = XSTATE_MASK_IPT
    | XSTATE_MASK_PASID
    | XSTATE_MASK_CET_U
    | XSTATE_MASK_CET_S
    | XSTATE_MASK_HDC
    | XSTATE_MASK_UINTR
    | XSTATE_MASK_LBR;

const CPUID_EXTENDED_STATE: i32 = 0x0D;
const CPUID_EXTENDED_STATE_MAIN_LEAF: i32 = 0x00;
const CPUID_EXTENDED_STATE_SUB_LEAF: i32 = 0x01;

const MSR_IA32_XSS: u32 = 0xDA0;

#[repr(C)]
struct CpuidExtendedStateMainLeafRegs {
    eax: CpuidEaxBits,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

#[repr(C)]
union CpuidEaxBits {
    uint32: u32,
    as_int32: [i32; 4],
}

#[repr(C)]
struct CpuidExtendedStateSubLeafRegs {
    eax: CpuidEaxSubBits,
    ebx: CpuidEbxSubBits,
    ecx: CpuidEcxSubBits,
    edx: u32,
}

#[repr(C)]
struct CpuidEaxSubBits {
    xsaveopt: u32,
    xsavec: u32,
    xgetbv: u32,
    xsaves: u32,
    xfd: u32,
    reserved: u32,
}

#[repr(C)]
struct CpuidEbxSubBits {
    xsave_area_size: u32,
}

#[repr(C)]
struct CpuidEcxSubBits {
    uint32: u32,
    bits: CpuidEcxSubBitFields,
}

#[repr(C)]
struct CpuidEcxSubBitFields {
    aligned: u32,
}

#[repr(C)]
struct CpuidExtendedStateSizeOffsetRegs {
    size: u32,
    offset: u32,
    ecx: CpuidComponentEcx,
    edx: u32,
}

#[repr(C)]
struct CpuidComponentEcx {
    bits: CpuidComponentBits,
}

#[repr(C)]
struct CpuidComponentBits {
    aligned: u32,
}

use crate::ke::krnlinit::KeFeatureBits;
use super::stubs::KeXStateLength;
extern "system" {
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
}

// KUSER_SHARED_DATA: bindgen layout (a shadow struct here put XState at +0x00,
// clobbering the tick/time fields — same landmine class as amd64/kiinit.rs).
use ntkrnl_sys::ddk::_KUSER_SHARED_DATA as KUSER_SHARED_DATA;

#[cfg(target_arch = "x86")]
extern "C" {
    fn __cpuidex(cpu_info: *mut i32, function_id: i32, subfunction_id: i32);
    fn _xsetbv(xcr: u32, value: u64);
    fn __writemsr(msr: u32, value: u64);
}

#[cfg(target_arch = "x86_64")]

// CODE_SEG("INIT")
unsafe fn KiGetXStateConfiguration(XStateConfig: PXSTATE_CONFIGURATION) {
    let mut SupportedUserMask: u64;
    let mut SupportedSupervisorMask: u64;
    let SupportedComponentMask: u64;
    let mut NextUserOffset: ULONG;
    let mut NextSupervisorOffset: ULONG;
    let NextOffset: ULONG;

    core::ptr::write_bytes(
        XStateConfig as *mut u8,
        0,
        core::mem::size_of::<XSTATE_CONFIGURATION>(),
    );

    let mut ExtStateMain: CpuidExtendedStateMainLeafRegs = core::mem::zeroed();
    __cpuidex(
        core::ptr::addr_of_mut!(ExtStateMain) as *mut i32,
        CPUID_EXTENDED_STATE,
        CPUID_EXTENDED_STATE_MAIN_LEAF,
    );

    SupportedUserMask = ((ExtStateMain.edx as u64) << 32) | (ExtStateMain.eax.uint32 as u64);

    SupportedUserMask &= !XSTATE_MASK_LARGE_FEATURES;

    SupportedUserMask &= XSTATE_MASK_ALLOWED;

    let mut ExtStateSub: CpuidExtendedStateSubLeafRegs = core::mem::zeroed();
    __cpuidex(
        core::ptr::addr_of_mut!(ExtStateSub) as *mut i32,
        CPUID_EXTENDED_STATE,
        CPUID_EXTENDED_STATE_SUB_LEAF,
    );

    (*XStateConfig).__bindgen_anon_1.__bindgen_anon_1.set_OptimizedSave((ExtStateSub.eax.xsaveopt & 1) as ULONG);
    (*XStateConfig).__bindgen_anon_1.__bindgen_anon_1.set_CompactionEnabled((ExtStateSub.eax.xsavec & 1) as ULONG);
    (*XStateConfig).__bindgen_anon_1.__bindgen_anon_1.set_ExtendedFeatureDisable((ExtStateSub.eax.xfd & 1) as ULONG);

    SupportedSupervisorMask = 0;
    if (ExtStateSub.eax.xsaves & 1) != 0 {
        SupportedSupervisorMask = ((ExtStateSub.edx as u64) << 32) | (ExtStateSub.ecx.uint32 as u64);
        SupportedSupervisorMask &= XSTATE_MASK_ALLOWED & XSTATE_MASK_SUPERVISOR;
    }

    SupportedComponentMask = SupportedUserMask | SupportedSupervisorMask;

    (*XStateConfig).Features[XSTATE_LEGACY_FLOATING_POINT as usize].Offset = 0;
    (*XStateConfig).Features[XSTATE_LEGACY_FLOATING_POINT as usize].Size =
        core::mem::offset_of!(XSAVE_FORMAT, XmmRegisters) as u32;
    (*XStateConfig).AllFeatures[XSTATE_LEGACY_FLOATING_POINT as usize] =
        core::mem::offset_of!(XSAVE_FORMAT, XmmRegisters) as u32;
    (*XStateConfig).Features[XSTATE_LEGACY_SSE as usize].Offset =
        core::mem::offset_of!(XSAVE_FORMAT, XmmRegisters) as u32;
    (*XStateConfig).Features[XSTATE_LEGACY_SSE as usize].Size =
        (core::mem::offset_of!(XSAVE_FORMAT, Reserved4)
            - core::mem::offset_of!(XSAVE_FORMAT, XmmRegisters)) as u32;
    (*XStateConfig).AllFeatures[XSTATE_LEGACY_SSE as usize] =
        (core::mem::offset_of!(XSAVE_FORMAT, Reserved4)
            - core::mem::offset_of!(XSAVE_FORMAT, XmmRegisters)) as u32;

    NextUserOffset = core::mem::size_of::<XSAVE_AREA>() as ULONG;
    NextSupervisorOffset = core::mem::size_of::<XSAVE_AREA>() as ULONG;

    for Component in 2..MAXIMUM_XSTATE_FEATURES as ULONG {
        let ComponentBit: u64 = 1u64 << Component;

        let mut ExtStateComponent: CpuidExtendedStateSizeOffsetRegs = core::mem::zeroed();
        __cpuidex(
            core::ptr::addr_of_mut!(ExtStateComponent) as *mut i32,
            CPUID_EXTENDED_STATE,
            Component as i32,
        );

        (*XStateConfig).AllFeatures[Component as usize] = ExtStateComponent.size;

        if ExtStateComponent.size == 0 {
            continue;
        }

        if (ComponentBit & SupportedComponentMask) == 0 {
            if (ComponentBit & XSTATE_MASK_SUPERVISOR) == 0 {
                (*XStateConfig).Features[Component as usize].Offset = ExtStateComponent.offset;
                (*XStateConfig).Features[Component as usize].Size = ExtStateComponent.size;
            }

            continue;
        }

        if (*XStateConfig).__bindgen_anon_1.__bindgen_anon_1.CompactionEnabled() != 0 {
            if (ExtStateComponent.ecx.bits.aligned & 1) != 0 {
                (*XStateConfig).AlignedFeatures |= ComponentBit;
                NextSupervisorOffset = (NextSupervisorOffset + 63) & !63;
                if (ComponentBit & SupportedUserMask) != 0 {
                    NextUserOffset = (NextUserOffset + 63) & !63;
                }
            }

            NextSupervisorOffset += ExtStateComponent.size;

            if (ComponentBit & SupportedUserMask) != 0 {
                (*XStateConfig).Features[Component as usize].Offset = NextUserOffset;
                (*XStateConfig).Features[Component as usize].Size = ExtStateComponent.size;
                NextUserOffset += ExtStateComponent.size;
            }
        } else {
            let NextOffset = ExtStateComponent.offset + ExtStateComponent.size;
            NextSupervisorOffset = if NextSupervisorOffset > NextOffset {
                NextSupervisorOffset
            } else {
                NextOffset
            };

            if (ComponentBit & SupportedUserMask) != 0 {
                (*XStateConfig).Features[Component as usize].Offset = ExtStateComponent.offset;
                (*XStateConfig).Features[Component as usize].Size = ExtStateComponent.size;
                NextUserOffset = if NextUserOffset > NextOffset {
                    NextUserOffset
                } else {
                    NextOffset
                };
            }
        }
    }

    (*XStateConfig).EnabledFeatures = SupportedUserMask;
    (*XStateConfig).EnabledVolatileFeatures = SupportedUserMask & !XSTATE_MASK_PERSISTENT;
    (*XStateConfig).EnabledSupervisorFeatures = SupportedSupervisorMask;
    (*XStateConfig).EnabledUserVisibleSupervisorFeatures =
        SupportedSupervisorMask & XSTATE_MASK_USER_VISIBLE_SUPERVISOR;

    (*XStateConfig).Size = NextUserOffset;
    (*XStateConfig).AllFeatureSize = NextSupervisorOffset;
}

// CODE_SEG("INIT")
unsafe fn ValidateXStateConfig(XState: PXSTATE_CONFIGURATION) {
    let GlobalXState = core::ptr::addr_of_mut!((*SharedUserData).XState);

    if ((*XState).EnabledFeatures != (*GlobalXState).EnabledFeatures)
        || ((*XState).EnabledSupervisorFeatures != (*GlobalXState).EnabledSupervisorFeatures)
        || ((*XState).Size != (*GlobalXState).Size)
        || ((*XState).AllFeatureSize != (*GlobalXState).AllFeatureSize)
    {
        KeBugCheck(MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED);
    }

    for i in 0..MAXIMUM_XSTATE_FEATURES as usize {
        if ((*XState).Features[i].Size != (*GlobalXState).Features[i].Size)
            || ((*XState).Features[i].Offset != (*GlobalXState).Features[i].Offset)
            || ((*XState).AllFeatures[i] != (*GlobalXState).AllFeatures[i])
        {
            KeBugCheck(MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED);
        }
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeXStateConfiguration(ProcessorNumber: ULONG) {
    if (KeFeatureBits & KF_XSTATE as u64) == 0 {
        return;
    }

    if ProcessorNumber == 0 {
        KiGetXStateConfiguration(core::ptr::addr_of_mut!((*SharedUserData).XState));

        if (*SharedUserData).XState.AllFeatureSize == 0 {
            KeFeatureBits &= !(KF_XSTATE as u64);
            return;
        }

        KeXStateLength = (*SharedUserData).XState.AllFeatureSize as usize;
    } else {
        let mut XState: XSTATE_CONFIGURATION = core::mem::zeroed();
        KiGetXStateConfiguration(&mut XState as PXSTATE_CONFIGURATION);
        ValidateXStateConfig(&mut XState as PXSTATE_CONFIGURATION);
    }

    _xsetbv(0, (*SharedUserData).XState.EnabledFeatures);

    let mut ExtStateMain: CpuidExtendedStateMainLeafRegs = core::mem::zeroed();
    __cpuidex(
        core::ptr::addr_of_mut!(ExtStateMain) as *mut i32,
        CPUID_EXTENDED_STATE,
        CPUID_EXTENDED_STATE_MAIN_LEAF,
    );

    if ExtStateMain.ebx > (*SharedUserData).XState.Size {
    }

    if (*SharedUserData).XState.EnabledSupervisorFeatures != 0 {
        __writemsr(
            MSR_IA32_XSS,
            (*SharedUserData).XState.EnabledSupervisorFeatures,
        );

        let mut ExtStateSubLeaf: CpuidExtendedStateSubLeafRegs = core::mem::zeroed();
        __cpuidex(
            core::ptr::addr_of_mut!(ExtStateSubLeaf) as *mut i32,
            CPUID_EXTENDED_STATE,
            CPUID_EXTENDED_STATE_SUB_LEAF,
        );

        if ExtStateSubLeaf.ebx.xsave_area_size > (*SharedUserData).XState.AllFeatureSize {
            (*SharedUserData).XState.AllFeatureSize = ExtStateSubLeaf.ebx.xsave_area_size;
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Extended processor state management
//  * COPYRIGHT:   Copyright 2025 Timo Kreuzer <timo.kreuzer@reactos.org>
//  */
//
// #include <ntoskrnl.h>
// #include <x86x64/Cpuid.h>
// #include <x86x64/Msr.h>
// #define NDEBUG
// #include <debug.h>
//
// // These are not officially documented
// #define XSTATE_PKRU                         9
// #define XSTATE_HDC                          13
// #define XSTATE_UINTR                        14
// #define XSTATE_LBR                          15
// #define XSTATE_MASK_PKRU                    (1LL << (XSTATE_PKRU))
// #define XSTATE_MASK_HDC                     (1LL << (XSTATE_HDC))
// #define XSTATE_MASK_UINTR                   (1LL << (XSTATE_UINTR))
// #define XSTATE_MASK_LBR                     (1LL << (XSTATE_LBR))
//
// #define XSTATE_MASK_SUPERVISOR \
//     (XSTATE_MASK_IPT | \
//      XSTATE_MASK_PASID | \
//      XSTATE_MASK_CET_U | \
//      XSTATE_MASK_CET_S | \
//      XSTATE_MASK_HDC | \
//      XSTATE_MASK_UINTR | \
//      XSTATE_MASK_LBR)
//
// /*!
//  *   \brief Determines the extended state configuration for the current processor
//  *
//  *   \param XStateConfig - Pointer to a XSTATE_CONFIGURATION structure that receives the configuration
//  *
//  *   \see https://windows-internals.com/cet-on-windows/#3-xstate-configuration
//  */
// CODE_SEG("INIT")
// static
// VOID
// KiGetXStateConfiguration(
//     _Out_ PXSTATE_CONFIGURATION XStateConfig)
// {
//     ULONG64 SupportedUserMask;
//     ULONG64 SupportedSupervisorMask;
//     ULONG64 SupportedComponentMask;
//     ULONG NextUserOffset, NextSupervisorOffset, NextOffset;
//
//     RtlZeroMemory(XStateConfig, sizeof(*XStateConfig));
//
//     /* Read CPUID_EXTENDED_STATE main leaf (0x0D, 0x00) */
//     CPUID_EXTENDED_STATE_MAIN_LEAF_REGS ExtStateMain;
//     __cpuidex(ExtStateMain.AsInt32,
//               CPUID_EXTENDED_STATE,
//               CPUID_EXTENDED_STATE_MAIN_LEAF);
//
//     /* Get the supported XCR0 bits */
//     SupportedUserMask = (ULONG64)ExtStateMain.Edx << 32 |
//                         (ULONG64)ExtStateMain.Eax.Uint32;
//
//     /* FIXME: Temporary workaround until we have dynamic kernel stack size */
//     SupportedUserMask &= ~XSTATE_MASK_LARGE_FEATURES;
//
//     /* Mask the allowed components */
//     SupportedUserMask &= XSTATE_MASK_ALLOWED;
//
//     /* Read CPUID_EXTENDED_STATE sub-leaf (0x0D, 0x01) */
//     CPUID_EXTENDED_STATE_SUB_LEAF_REGS ExtStateSub;
//     __cpuidex(ExtStateSub.AsInt32,
//               CPUID_EXTENDED_STATE,
//               CPUID_EXTENDED_STATE_SUB_LEAF);
//
//     /* Save control flags */
//     XStateConfig->OptimizedSave = ExtStateSub.Eax.Bits.XSAVEOPT;
//     XStateConfig->CompactionEnabled = ExtStateSub.Eax.Bits.XSAVEC;
//     XStateConfig->ExtendedFeatureDisable = ExtStateSub.Eax.Bits.Xfd;
//
//     /* Determine supported supervisor features */
//     SupportedSupervisorMask = 0;
//     if (ExtStateSub.Eax.Bits.XSAVES)
//     {
//         SupportedSupervisorMask = (ULONG64)ExtStateSub.Edx << 32 |
//                                   (ULONG64)ExtStateSub.Ecx.Uint32;
//         SupportedSupervisorMask &= XSTATE_MASK_ALLOWED & XSTATE_MASK_SUPERVISOR;
//     }
//
//     /* Calculate full mask */
//     SupportedComponentMask = SupportedUserMask | SupportedSupervisorMask;
//
//     /* Basic features (always enabled) */
//     XStateConfig->Features[XSTATE_LEGACY_FLOATING_POINT].Offset = 0;
//     XStateConfig->Features[XSTATE_LEGACY_FLOATING_POINT].Size = FIELD_OFFSET(XSAVE_FORMAT, XmmRegisters);
//     XStateConfig->AllFeatures[XSTATE_LEGACY_FLOATING_POINT] = FIELD_OFFSET(XSAVE_FORMAT, XmmRegisters);
//     XStateConfig->Features[XSTATE_LEGACY_SSE].Offset = FIELD_OFFSET(XSAVE_FORMAT, XmmRegisters);
//     XStateConfig->Features[XSTATE_LEGACY_SSE].Size = FIELD_SIZE(XSAVE_FORMAT, XmmRegisters);
//     XStateConfig->AllFeatures[XSTATE_LEGACY_SSE] = FIELD_SIZE(XSAVE_FORMAT, XmmRegisters);
//
//     /* Other components start after legacy state + header */
//     NextUserOffset = NextSupervisorOffset = sizeof(XSAVE_AREA);
//
//     /* Loop all components from 2 up */
//     for (ULONG Component = 2; Component < MAXIMUM_XSTATE_FEATURES; Component++)
//     {
//         ULONG64 ComponentBit = (1ULL << Component);
//
//         /* Query component features */
//         CPUID_EXTENDED_STATE_SIZE_OFFSET_REGS ExtStateComponent;
//         __cpuidex(ExtStateComponent.AsInt32,
//                   CPUID_EXTENDED_STATE,
//                   Component);
//
//         /* Save size for all features */
//         XStateConfig->AllFeatures[Component] = ExtStateComponent.Size;
//
//         /* If the offset is 0, this component isn't valid */
//         if (ExtStateComponent.Size == 0) continue;
//
//         /* Check for components that are not OS supported */
//         if ((ComponentBit & SupportedComponentMask) == 0)
//         {
//             /* This emulates weird (broken) Windows behavior */
//             if ((ComponentBit & XSTATE_MASK_SUPERVISOR) == 0)
//             {
//                 XStateConfig->Features[Component].Offset = ExtStateComponent.Offset;
//                 XStateConfig->Features[Component].Size = ExtStateComponent.Size;
//             }
//
//             /* Skip the rest */
//             continue;
//         }
//
//         /* Check if compaction is enabled */
//         if (XStateConfig->CompactionEnabled)
//         {
//             /* Align the offsets, if needed */
//             if (ExtStateComponent.Ecx.Bits.Aligned)
//             {
//                 XStateConfig->AlignedFeatures |= ComponentBit;
//                 NextSupervisorOffset = ALIGN_UP(NextSupervisorOffset, 64);
//                 if ((ComponentBit & SupportedUserMask) != 0)
//                 {
//                     NextUserOffset = ALIGN_UP(NextUserOffset, 64);
//                 }
//             }
//
//             /* Update the supervisor offset */
//             NextSupervisorOffset += ExtStateComponent.Size;
//
//             /* For user components save and update the offset and size */
//             if ((ComponentBit & SupportedUserMask) != 0)
//             {
//                 XStateConfig->Features[Component].Offset = NextUserOffset;
//                 XStateConfig->Features[Component].Size = ExtStateComponent.Size;
//                 NextUserOffset += ExtStateComponent.Size;
//             }
//         }
//         else
//         {
//             /* Not compacted, use the offset and size specified by the CPUID */
//             NextOffset = ExtStateComponent.Offset + ExtStateComponent.Size;
//             NextSupervisorOffset = max(NextSupervisorOffset, NextOffset);
//
//             /* For user components save and update the offset and size */
//             if ((ComponentBit & SupportedUserMask) != 0)
//             {
//                 XStateConfig->Features[Component].Offset = ExtStateComponent.Offset;
//                 XStateConfig->Features[Component].Size = ExtStateComponent.Size;
//                 NextUserOffset = max(NextUserOffset, NextOffset);
//             }
//         }
//     }
//
//     /* Save the features to be enabled */
//     XStateConfig->EnabledFeatures = SupportedUserMask;
//     XStateConfig->EnabledVolatileFeatures =
//         SupportedUserMask & ~XSTATE_MASK_PERSISTENT;
//     XStateConfig->EnabledSupervisorFeatures = SupportedSupervisorMask;
//     XStateConfig->EnabledUserVisibleSupervisorFeatures =
//         SupportedSupervisorMask & XSTATE_MASK_USER_VISIBLE_SUPERVISOR;
//
//     /* Save the calculated sizes */
//     XStateConfig->Size = NextUserOffset;
//     XStateConfig->AllFeatureSize = NextSupervisorOffset;
//     ASSERT(XStateConfig->AllFeatureSize >= XStateConfig->Size);
// }
//
// /*!
//  *   \brief Validates the provided extended state configuration against the global one
//  *
//  *   \param XStateConfig - Pointer to a XSTATE_CONFIGURATION structure containing the configuration
//  */
// CODE_SEG("INIT")
// static
// VOID
// ValidateXStateConfig(
//     _In_ PXSTATE_CONFIGURATION XState)
// {
//     PXSTATE_CONFIGURATION GlobalXState = &SharedUserData->XState;
//
//     if ((XState->EnabledFeatures != GlobalXState->EnabledFeatures) ||
//         (XState->EnabledSupervisorFeatures != GlobalXState->EnabledSupervisorFeatures) ||
//         (XState->Size != GlobalXState->Size) ||
//         (XState->AllFeatureSize != GlobalXState->AllFeatureSize))
//     {
//         /* Invalid features */
//         KeBugCheck(MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED);
//     }
//
//     for (ULONG i = 0; i < MAXIMUM_XSTATE_FEATURES; i++)
//     {
//         if ((XState->Features[i].Size != GlobalXState->Features[i].Size) ||
//             (XState->Features[i].Offset != GlobalXState->Features[i].Offset) ||
//             (XState->AllFeatures[i] != GlobalXState->AllFeatures[i]))
//         {
//             /* Invalid features */
//             KeBugCheck(MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED);
//         }
//     }
// }
//
// /*!
//  *   \brief Initializes the extended state configuration for the current processor
//  *
//  *   \param ProcessorNumber - Number of the current processor
//  */
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeXStateConfiguration(
//     _In_ ULONG ProcessorNumber)
// {
//     /* Check if XSAVE is supported */
//     if ((KeFeatureBits & KF_XSTATE) == 0)
//     {
//         /* XSAVE is not supported */
//         return;
//     }
//
//     if (ProcessorNumber == 0)
//     {
//         /* Processor 0: Retrieve the global configuration */
//         KiGetXStateConfiguration(&SharedUserData->XState);
//
//         if (SharedUserData->XState.AllFeatureSize == 0)
//         {
//             KeFeatureBits &= ~KF_XSTATE;
//             return;
//         }
//
//         KeXStateLength = SharedUserData->XState.AllFeatureSize;
//     }
//     else
//     {
//         /* Processor 1+: validate the configuration against the global one */
//         XSTATE_CONFIGURATION XState;
//         KiGetXStateConfiguration(&XState);
//         ValidateXStateConfig(&XState);
//     }
//
//     /* Enable the user mode components in XCR0 */
//     _xsetbv(0, SharedUserData->XState.EnabledFeatures);
//
//     /* Now that we have set everything up, query CPUID again to get the required
//        size based on components enabled in XCR0 */
//     CPUID_EXTENDED_STATE_MAIN_LEAF_REGS ExtStateMain;
//     __cpuidex(ExtStateMain.AsInt32,
//               CPUID_EXTENDED_STATE,
//               CPUID_EXTENDED_STATE_MAIN_LEAF);
//
//     /* CPUID 0xD, leaf 0, EBX should return the size required by all components
//        enabled in XCR0 and thus match our calculation. But VBox doesn't handle
//        this correctly and simply returns the full size of all *supported*
//        features, independent of XCR0. We check and warn. */
//     if (ExtStateMain.Ebx > SharedUserData->XState.Size)
//     {
//         DPRINT1("Processor %lu, CPUID 0xD, leaf 0, EBX returns 0x%x, but we calculated 0x%lx\n",
//                 ProcessorNumber,
//                 ExtStateMain.Ebx,
//                 SharedUserData->XState.Size);
//     }
//
//     /* Check if we have any supervisor components enabled */
//     if (SharedUserData->XState.EnabledSupervisorFeatures != 0)
//     {
//         /* Enable the supervisor components in IA32_XSS */
//         __writemsr(MSR_IA32_XSS, SharedUserData->XState.EnabledSupervisorFeatures);
//
//         /* Get the required size for features enabled in both XCR0 and IA32_XSS */
//         CPUID_EXTENDED_STATE_SUB_LEAF_REGS ExtStateSubLeaf;
//         __cpuidex(ExtStateSubLeaf.AsInt32,
//                   CPUID_EXTENDED_STATE,
//                   CPUID_EXTENDED_STATE_SUB_LEAF);
//
//         /* Check if all components fit into what we calculated. Same VBox issue
//            here as described above. */
//         if (ExtStateSubLeaf.Ebx.XSaveAreaSize > SharedUserData->XState.AllFeatureSize)
//         {
//             DPRINT1("Processor %lu, CPUID 0xD, leaf 1, EBX returns 0x%x, but we calculated 0x%lx\n",
//                     ProcessorNumber,
//                     ExtStateMain.Ebx,
//                     SharedUserData->XState.Size);
//
//             /* The problem is likely the VM, but to be safe, we adjust the size */
//             SharedUserData->XState.AllFeatureSize = ExtStateSubLeaf.Ebx.XSaveAreaSize;
//         }
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
