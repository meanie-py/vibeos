use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;

use ntkrnl_sys::ddk::{
    MEMORY_ALLOCATION_DESCRIPTOR, PLOADER_PARAMETER_BLOCK, PMEMORY_ALLOCATION_DESCRIPTOR,
    _TYPE_OF_MEMORY_LoaderXIPRom as LoaderXIPRom,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn strstr(s1: PCHAR, s2: PCHAR) -> PCHAR;
}

#[no_mangle]
pub unsafe extern "system" fn XIPDispatch(
    DispatchCode: ULONG,
    Buffer: PVOID,
    BufferSize: ULONG,
) -> NTSTATUS {
    let Buffer = opaque_ptr(Buffer);
    STATUS_NOT_IMPLEMENTED
}

// CODE_SEG("INIT")
unsafe fn XIPpFindMemoryDescriptor(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> PMEMORY_ALLOCATION_DESCRIPTOR {
    let mut NextEntry: PLIST_ENTRY;
    let mut Descriptor: PMEMORY_ALLOCATION_DESCRIPTOR = core::ptr::null_mut();

    NextEntry = (*LoaderBlock).MemoryDescriptorListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead) {
        Descriptor = (NextEntry as usize - core::mem::offset_of!(MEMORY_ALLOCATION_DESCRIPTOR, ListEntry))
            as PMEMORY_ALLOCATION_DESCRIPTOR;
        if (*Descriptor).MemoryType == LoaderXIPRom {
            return Descriptor;
        }
        NextEntry = (*NextEntry).Flink;
    }

    core::ptr::null_mut()
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn XIPInit(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let CommandLine: PCHAR;
    let XipBoot: PCHAR;
    let XipRom: PCHAR;
    let XipMegs: PCHAR;
    let XipVerbose: PCHAR;
    let XipRam: PCHAR;
    let XipDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR;

    CommandLine = (*LoaderBlock).LoadOptions;
    if CommandLine.is_null() {
        return;
    }

    XipBoot = strstr(CommandLine, b"XIPBOOT\0".as_ptr() as PCHAR);
    XipRam = strstr(CommandLine, b"XIPRAM=\0".as_ptr() as PCHAR);
    XipRom = strstr(CommandLine, b"XIPROM=\0".as_ptr() as PCHAR);
    XipMegs = strstr(CommandLine, b"XIPMEGS=\0".as_ptr() as PCHAR);
    XipVerbose = strstr(CommandLine, b"XIPVERBOSE\0".as_ptr() as PCHAR);

    if !XipVerbose.is_null() {
        crate::dbg_print!(b"\n\nXIP: debug timestamp at line %d in %s:   <<<%s %s>>>\n\n\0",
            line!() as i32,
            file!().as_ptr(),
            b"__DATE__\0".as_ptr(),
            b"__TIME__\0".as_ptr(),
        );
    }

    XipDescriptor = XIPpFindMemoryDescriptor(LoaderBlock);
    if XipDescriptor.is_null() {
        return;
    }

    if (XipRom as ULONG_PTR) ^ (XipRam as ULONG_PTR) == 0 {
        return;
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/xipdisp.c
//  * PURPOSE:         eXecute In Place (XIP) Support.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// XIPDispatch(IN ULONG DispatchCode,
//             OUT PVOID Buffer,
//             IN ULONG BufferSize)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("INIT")
// PMEMORY_ALLOCATION_DESCRIPTOR
// NTAPI
// XIPpFindMemoryDescriptor(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR Descriptor = NULL;
//
//     /* Loop the memory descriptors */
//     for (NextEntry = LoaderBlock->MemoryDescriptorListHead.Flink;
//          NextEntry != &LoaderBlock->MemoryDescriptorListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get the current descriptor and check if it's the XIP ROM */
//         Descriptor = CONTAINING_RECORD(NextEntry,
//                                        MEMORY_ALLOCATION_DESCRIPTOR,
//                                        ListEntry);
//         if (Descriptor->MemoryType == LoaderXIPRom) return Descriptor;
//     }
//
//     /* Nothing found if we got here */
//     return NULL;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// XIPInit(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PCHAR CommandLine, XipBoot, XipRom, XipMegs, XipVerbose, XipRam;
//     PMEMORY_ALLOCATION_DESCRIPTOR XipDescriptor;
//
//     /* Get the command line */
//     CommandLine = LoaderBlock->LoadOptions;
//     if (!CommandLine) return;
//
//     /* Get XIP settings */
//     XipBoot = strstr(CommandLine, "XIPBOOT");
//     XipRam = strstr(CommandLine, "XIPRAM=");
//     XipRom = strstr(CommandLine, "XIPROM=");
//     XipMegs = strstr(CommandLine, "XIPMEGS=");
//     XipVerbose = strstr(CommandLine, "XIPVERBOSE");
//
//     /* Check if this is a verbose boot */
//     if (XipVerbose)
//     {
//         /* Print out our header */
//         DbgPrint("\n\nXIP: debug timestamp at line %d in %s:   <<<%s %s>>>\n\n",
//                  __LINE__,
//                  __FILE__,
//                  __DATE__,
//                  __TIME__);
//     }
//
//     /* Find the XIP memory descriptor */
//     XipDescriptor = XIPpFindMemoryDescriptor(LoaderBlock);
//     if (!XipDescriptor) return;
//
//     //
//     // Make sure this is really XIP, and not RAM Disk -- also validate XIP
//     // Basically, either this is a ROM boot or a RAM boot, but not both nor none
//     //
//     if (!((ULONG_PTR)XipRom ^ (ULONG_PTR)XipRam)) return;
//
//     /* FIXME: TODO */
//     DPRINT1("ReactOS does not yet support eXecute In Place boot technology\n");
//     DPRINT("%s MB requested (XIP = %s)\n", XipMegs, XipBoot);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
