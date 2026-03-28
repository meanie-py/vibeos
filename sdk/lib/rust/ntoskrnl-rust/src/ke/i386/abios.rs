use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KGDTENTRY, PCONTEXT, PDRIVER_OBJECT, PKGDTENTRY};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386FlatToGdtSelector(
    Base: ULONG,
    Length: USHORT,
    Selector: USHORT,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386ReleaseGdtSelectors(
    SelArray: *mut ULONG,
    NumOfSelectors: ULONG,
) -> NTSTATUS {
    let SelArray = opaque_ptr(SelArray);
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386AllocateGdtSelectors(
    SelArray: *mut ULONG,
    NumOfSelectors: ULONG,
) -> NTSTATUS {
    let SelArray = opaque_ptr(SelArray);
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386SetGdtSelector(
    Selector: ULONG,
    GdtValue: *mut KGDTENTRY,
) -> NTSTATUS {
    let GdtValue = opaque_ptr(GdtValue);
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386AbiosCall(
    CallId: USHORT,
    DriverObject: PDRIVER_OBJECT,
    RequestBlock: *mut ULONG,
    EntryPoint: USHORT,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let RequestBlock = opaque_ptr(RequestBlock);
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386GetLid(
    DeviceId: USHORT,
    RelativeLid: USHORT,
    SharedLid: BOOLEAN,
    DriverObject: PDRIVER_OBJECT,
    LogicalId: *mut USHORT,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let LogicalId = opaque_ptr(LogicalId);
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386ReleaseLid(
    LogicalId: USHORT,
    DriverObject: PDRIVER_OBJECT,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386Call16BitCStyleFunction(
    EntryOffset: ULONG,
    EntrySelector: ULONG,
    Parameters: *mut ULONG,
    Size: ULONG,
) -> USHORT {
    let Parameters = opaque_ptr(Parameters);
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeI386Call16BitFunction(Context: PCONTEXT) {
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/i386/abios.c
//  * PURPOSE:         Routines for ABIOS Support
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
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// KeI386FlatToGdtSelector(IN ULONG Base,
//                         IN USHORT Length,
//                         IN USHORT Selector)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// KeI386ReleaseGdtSelectors(OUT PULONG SelArray,
//                           IN ULONG NumOfSelectors)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// KeI386AllocateGdtSelectors(OUT PULONG SelArray,
//                            IN ULONG NumOfSelectors)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// KeI386SetGdtSelector(IN ULONG Selector,
//                      IN PKGDTENTRY GdtValue)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// KeI386AbiosCall(IN USHORT CallId,
//                 IN PDRIVER_OBJECT DriverObject,
//                 IN PULONG RequestBlock,
//                 IN USHORT EntryPoint)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// KeI386GetLid(IN USHORT DeviceId,
//              IN USHORT RelativeLid,
//              IN BOOLEAN SharedLid,
//              IN PDRIVER_OBJECT DriverObject,
//              OUT PUSHORT LogicalId)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// KeI386ReleaseLid(IN USHORT LogicalId,
//                  IN PDRIVER_OBJECT DriverObject)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// USHORT
// NTAPI
// KeI386Call16BitCStyleFunction(IN ULONG EntryOffset,
//                               IN ULONG EntrySelector,
//                               IN PULONG Parameters,
//                               IN ULONG Size)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// VOID
// NTAPI
// KeI386Call16BitFunction(IN OUT PCONTEXT Context)
// {
//     UNIMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
