use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{GUID};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut IID_IUnknown: GUID = GUID {
    Data1: 0 as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub const NDEBUG: u32 = 0;

#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}

#[no_mangle]
pub unsafe extern "system" fn KoCreateInstance(
    ClassId: *const CLSID,
    UnkOuter: *mut IUnknown,
    ClsContext: ULONG,
    InterfaceId: *const IID,
    Interface: *mut PVOID,
) -> NTSTATUS {
    if ClsContext != CLSCTX_KERNEL_SERVER as ULONG {
        return STATUS_INVALID_PARAMETER_3;
    }

    if IsEqualGUIDAligned(InterfaceId, &raw const IID_IUnknown) {
        return STATUS_INVALID_PARAMETER_4;
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/allocators.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//                     Andrew Greenwood
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// const GUID IID_IUnknown = {0x00000000, 0x0000, 0x0000, {0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x46}};
//
// /* http://msdn2.microsoft.com/en-us/library/ms809781.aspx (DEAD_LINK) */
// COMDDKAPI NTSTATUS NTAPI
// KoCreateInstance(
//     IN  REFCLSID ClassId,
//     IN  IUnknown* UnkOuter OPTIONAL,
//     IN  ULONG ClsContext,
//     IN  REFIID InterfaceId,
//     OUT PVOID* Interface)
// {
//     /* If UnkOuter isn't NULL, it must be IUnknown - TODO: CHECK THIS PARAM */
//     /* TODO: Check IRQL? */
//
//     DPRINT("KoCreateInstance called\n");
//
//     if ( ClsContext != CLSCTX_KERNEL_SERVER )
//     {
//         DPRINT("KoCreateInstance: ClsContext must be CLSCTX_KERNEL_SERVER\n");
//         return STATUS_INVALID_PARAMETER_3;
//     }
//
//     if (IsEqualGUIDAligned(InterfaceId, &IID_IUnknown))
//     {
//         DPRINT("KoCreateInstance: InterfaceId cannot be IID_IUnknown\n");
//         return STATUS_INVALID_PARAMETER_4;
//     }
//
//
//     /*
//         Find the desired interface and create an instance.
//
//         But we also need to supply a
//         pointer which will be set to a list of available interfaces, to
//         IoGetDeviceInterfaces.
//
//         We can then create a file based on this information and thus talk
//         to the appropriate device.
//
//         Useful references:
//             https://www.freelists.org/post/wdmaudiodev/Need-audio-driver-deivce-interface-registration-help
//
//         TODO
//     */
//
//     DPRINT("** FAKING SUCCESS **\n");
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
