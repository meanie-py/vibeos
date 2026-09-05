use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    PIRP, PMDL, PPFN_NUMBER, _MDL,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn MmUnmapLockedPages(BaseAddress: PVOID, MemoryDescriptorList: PMDL);
}

const MDL_PARTIAL_HAS_BEEN_MAPPED: i16 = 0x0020;

#[inline]
unsafe fn IopAllocateMdlFromLookaside(lookaside_type: ULONG) -> PMDL {
    ntkrnl_sys::inline::ob::ObpAllocateObjectCreateInfoBuffer(lookaside_type) as PMDL
}

#[inline]
unsafe fn IopFreeMdlFromLookaside(buffer: PMDL, lookaside_type: ULONG) {
    ntkrnl_sys::inline::ob::ObpFreeCapturedAttributes(buffer as PVOID, lookaside_type);
}

#[inline]
unsafe fn MmInitializeMdl(mdl: PMDL, va: PVOID, length: ULONG) {
    ntkrnl_sys::inline::mm::MmInitializeMdl(mdl, va, length);
}

#[inline]
unsafe fn MmPrepareMdlForReuse(mdl: PMDL) {
    if ((*mdl).MdlFlags & MDL_PARTIAL_HAS_BEEN_MAPPED) != 0 {
        ntkrnl_sys::ASSERT!(((*mdl).MdlFlags & MDL_PARTIAL) != 0);
        MmUnmapLockedPages((*mdl).MappedSystemVa, mdl);
    } else if ((*mdl).MdlFlags & MDL_PARTIAL) == 0 {
        ntkrnl_sys::ASSERT!(((*mdl).MdlFlags & MDL_MAPPED_TO_SYSTEM_VA) == 0);
    }
}

// MdlFlags is CSHORT (i16) in bindgen, so define constants as i16
const MDL_ALLOCATED_FIXED_SIZE: i16 = 0x0008;
const MDL_ALLOCATED_MUST_SUCCEED: i16 = 0x4000;
const MDL_IO_PAGE_READ: i16 = 0x40;
const MDL_SOURCE_IS_NONPAGED_POOL: i16 = 0x0004;
const MDL_MAPPED_TO_SYSTEM_VA: i16 = 0x0001;
const MDL_IO_SPACE: i16 = 0x0800;
const MDL_PARTIAL: i16 = 0x0010;

const MAXUSHORT: ULONG = 0xFFFF;

#[inline]
unsafe fn BYTE_OFFSET(va: PVOID) -> ULONG {
    (va as ULONG_PTR & (PAGE_SIZE as ULONG_PTR - 1)) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IoAllocateMdl(
    VirtualAddress: PVOID,
    Length: ULONG,
    SecondaryBuffer: BOOLEAN,
    ChargeQuota: BOOLEAN,
    Irp: PIRP,
) -> PMDL {
    let VirtualAddress = opaque_ptr(VirtualAddress);
    let Irp = opaque_ptr(Irp);
    let mut Mdl: PMDL = core::ptr::null_mut();
    let mut Flags: ULONG = 0;
    let mut Size: ULONG;

    if Length & 0x80000000 != 0 {
        return core::ptr::null_mut();
    }

    Size = ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddress, Length as SIZE_T) as ULONG;
    if Size > 23 {
        Size *= core::mem::size_of::<PFN_NUMBER>() as ULONG;
        Size += core::mem::size_of::<_MDL>() as ULONG;
        if Size > MAXUSHORT {
            return core::ptr::null_mut();
        }
    } else {
        Size = (23 * core::mem::size_of::<PFN_NUMBER>() as ULONG) + core::mem::size_of::<_MDL>() as ULONG;
        Flags |= MDL_ALLOCATED_FIXED_SIZE as u32;

        Mdl = IopAllocateMdlFromLookaside(_PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList);
    }

    if Mdl.is_null() {
        Mdl = ExAllocatePoolWithTag(NonPagedPool, Size as SIZE_T, TAG_MDL) as PMDL;
        if Mdl.is_null() {
            return core::ptr::null_mut();
        }
    }

    MmInitializeMdl(Mdl, VirtualAddress, Length);
    (*Mdl).MdlFlags |= Flags as i16;

    if !Irp.is_null() {
        if SecondaryBuffer != 0 {
            let mut p = (*Irp).MdlAddress;
            while !(*p).Next.is_null() {
                p = (*p).Next;
            }
            (*p).Next = Mdl;
        } else {
            (*Irp).MdlAddress = Mdl;
        }
    }

    Mdl
}

#[no_mangle]
pub unsafe extern "system" fn IoBuildPartialMdl(
    SourceMdl: PMDL,
    TargetMdl: PMDL,
    VirtualAddress: PVOID,
    Length: ULONG,
) {
    let SourceMdl = opaque_ptr(SourceMdl);
    let TargetMdl = opaque_ptr(TargetMdl);
    let VirtualAddress = opaque_ptr(VirtualAddress);
    let TargetPages = (TargetMdl.add(1)) as PPFN_NUMBER;
    let SourcePages = (SourceMdl.add(1)) as PPFN_NUMBER;
    let mut Offset: ULONG;
    let FlagsMask: i16 = MDL_IO_PAGE_READ |
                            MDL_SOURCE_IS_NONPAGED_POOL |
                            MDL_MAPPED_TO_SYSTEM_VA |
                            MDL_IO_SPACE;

    Offset = (VirtualAddress as ULONG_PTR)
        .wrapping_sub((*SourceMdl).StartVa as ULONG_PTR)
        .wrapping_sub((*SourceMdl).ByteOffset as ULONG_PTR) as ULONG;

    let Length = if Length == 0 {
        (*SourceMdl).ByteCount - Offset
    } else {
        Length
    };

    (*TargetMdl).StartVa = PAGE_ROUND_DOWN(VirtualAddress as ULONG_PTR) as PVOID;
    (*TargetMdl).Process = (*SourceMdl).Process;
    (*TargetMdl).ByteCount = Length;
    (*TargetMdl).ByteOffset = BYTE_OFFSET(VirtualAddress);

    let Length = ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddress, Length as SIZE_T) as ULONG;

    (*TargetMdl).MdlFlags &= MDL_ALLOCATED_FIXED_SIZE | MDL_ALLOCATED_MUST_SUCCEED;
    (*TargetMdl).MdlFlags |= (*SourceMdl).MdlFlags & FlagsMask;
    (*TargetMdl).MdlFlags |= MDL_PARTIAL;

    (*TargetMdl).MappedSystemVa = ((*SourceMdl).MappedSystemVa as *mut u8).add(Offset as usize) as PVOID;

    Offset = (((*TargetMdl).StartVa as ULONG_PTR)
        .wrapping_sub((*SourceMdl).StartVa as ULONG_PTR) >> PAGE_SHIFT) as ULONG;
    let SourcePages = SourcePages.add(Offset as usize);
    core::ptr::copy_nonoverlapping(
        SourcePages as *const u8,
        TargetPages as *mut u8,
        Length as usize * core::mem::size_of::<PFN_NUMBER>(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn IoFreeMdl(Mdl: PMDL) {
    let Mdl = opaque_ptr(Mdl);
    MmPrepareMdlForReuse(Mdl);

    if (*Mdl).MdlFlags & MDL_ALLOCATED_FIXED_SIZE == 0 {
        ExFreePoolWithTag(Mdl as PVOID, TAG_MDL);
    } else {
        IopFreeMdlFromLookaside(Mdl, _PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/iomdl.c
//  * PURPOSE:         I/O Wrappers for MDL Allocation and Deallocation
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
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
// PMDL
// NTAPI
// IoAllocateMdl(IN PVOID VirtualAddress,
//               IN ULONG Length,
//               IN BOOLEAN SecondaryBuffer,
//               IN BOOLEAN ChargeQuota,
//               IN PIRP Irp)
// {
//     PMDL Mdl = NULL, p;
//     ULONG Flags = 0;
//     ULONG Size;
//
//     /* Make sure we got a valid length */
//     ASSERT(Length != 0);
//
//     /* Fail if allocation is over 2GB */
//     if (Length & 0x80000000) return NULL;
//
//     /* Calculate the number of pages for the allocation */
//     Size = ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddress, Length);
//     if (Size > 23)
//     {
//         /* This is bigger then our fixed-size MDLs. Calculate real size */
//         Size *= sizeof(PFN_NUMBER);
//         Size += sizeof(MDL);
//         if (Size > MAXUSHORT) return NULL;
//     }
//     else
//     {
//         /* Use an internal fixed MDL size */
//         Size = (23 * sizeof(PFN_NUMBER)) + sizeof(MDL);
//         Flags |= MDL_ALLOCATED_FIXED_SIZE;
//
//         /* Allocate one from the lookaside list */
//         Mdl = IopAllocateMdlFromLookaside(LookasideMdlList);
//     }
//
//     /* Check if we don't have an mdl yet */
//     if (!Mdl)
//     {
//         /* Allocate one from pool */
//         Mdl = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_MDL);
//         if (!Mdl) return NULL;
//     }
//
//     /* Initialize it */
//     MmInitializeMdl(Mdl, VirtualAddress, Length);
//     Mdl->MdlFlags |= Flags;
//
//     /* Check if an IRP was given too */
//     if (Irp)
//     {
//         /* Check if it came with a secondary buffer */
//         if (SecondaryBuffer)
//         {
//             /* Insert the MDL at the end */
//             p = Irp->MdlAddress;
//             while (p->Next) p = p->Next;
//             p->Next = Mdl;
//         }
//         else
//         {
//             /* Otherwise, insert it directly */
//             Irp->MdlAddress = Mdl;
//         }
//     }
//
//     /* Return the allocated mdl */
//     return Mdl;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoBuildPartialMdl(IN PMDL SourceMdl,
//                   IN PMDL TargetMdl,
//                   IN PVOID VirtualAddress,
//                   IN ULONG Length)
// {
//     PPFN_NUMBER TargetPages = (PPFN_NUMBER)(TargetMdl + 1);
//     PPFN_NUMBER SourcePages = (PPFN_NUMBER)(SourceMdl + 1);
//     ULONG Offset;
//     ULONG FlagsMask = (MDL_IO_PAGE_READ |
//                        MDL_SOURCE_IS_NONPAGED_POOL |
//                        MDL_MAPPED_TO_SYSTEM_VA |
//                        MDL_IO_SPACE);
//
//     /* Calculate the offset */
//     Offset = (ULONG)((ULONG_PTR)VirtualAddress -
//                      (ULONG_PTR)SourceMdl->StartVa) -
//                      SourceMdl->ByteOffset;
//
//     /* Check if we don't have a length and calculate it */
//     if (!Length) Length = SourceMdl->ByteCount - Offset;
//
//     /* Write the process, start VA and byte data */
//     TargetMdl->StartVa = (PVOID)PAGE_ROUND_DOWN(VirtualAddress);
//     TargetMdl->Process = SourceMdl->Process;
//     TargetMdl->ByteCount = Length;
//     TargetMdl->ByteOffset = BYTE_OFFSET(VirtualAddress);
//
//     /* Recalculate the length in pages */
//     Length = ADDRESS_AND_SIZE_TO_SPAN_PAGES(VirtualAddress, Length);
//
//     /* Set the MDL Flags */
//     TargetMdl->MdlFlags &= (MDL_ALLOCATED_FIXED_SIZE | MDL_ALLOCATED_MUST_SUCCEED);
//     TargetMdl->MdlFlags |= SourceMdl->MdlFlags & FlagsMask;
//     TargetMdl->MdlFlags |= MDL_PARTIAL;
//
//     /* Set the mapped VA */
//     TargetMdl->MappedSystemVa = (PCHAR)SourceMdl->MappedSystemVa + Offset;
//
//     /* Now do the copy */
//     Offset = (ULONG)(((ULONG_PTR)TargetMdl->StartVa -
//                       (ULONG_PTR)SourceMdl->StartVa) >> PAGE_SHIFT);
//     SourcePages += Offset;
//     RtlCopyMemory(TargetPages, SourcePages, Length * sizeof(PFN_NUMBER));
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoFreeMdl(PMDL Mdl)
// {
//     /* Tell Mm to reuse the MDL */
//     MmPrepareMdlForReuse(Mdl);
//
//     /* Check if this was a pool allocation */
//     if (!(Mdl->MdlFlags & MDL_ALLOCATED_FIXED_SIZE))
//     {
//         /* Free it from the pool */
//         ExFreePoolWithTag(Mdl, TAG_MDL);
//     }
//     else
//     {
//         /* Free it from the lookaside */
//         IopFreeMdlFromLookaside(Mdl, LookasideMdlList);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
