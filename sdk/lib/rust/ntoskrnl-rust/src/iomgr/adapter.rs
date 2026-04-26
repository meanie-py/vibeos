use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;

use ntkrnl_sys::ddk::{
    PADAPTER_OBJECT,
    PDEVICE_OBJECT,
    PDRIVER_CONTROL,
    POBJECT_TYPE,
    PWAIT_CONTEXT_BLOCK,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn HalAllocateAdapterChannel( AdapterObject: PADAPTER_OBJECT, Wcb: PWAIT_CONTEXT_BLOCK, NumberOfMapRegisters: ULONG, ExecutionRoutine: PDRIVER_CONTROL, ) -> NTSTATUS;
}

#[no_mangle]
pub static mut IoAdapterObjectType: POBJECT_TYPE = core::ptr::null_mut();
#[no_mangle]
pub static mut IoDeviceHandlerObjectType: POBJECT_TYPE = core::ptr::null_mut();
#[no_mangle]
pub static mut IoDeviceHandlerObjectSize: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn IoAllocateAdapterChannel(
    AdapterObject: PADAPTER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    NumberOfMapRegisters: ULONG,
    ExecutionRoutine: PDRIVER_CONTROL,
    Context: PVOID,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Context = opaque_ptr(Context);
    let Wcb = core::ptr::addr_of_mut!((*DeviceObject).Queue.Wcb);

    (*Wcb).DeviceObject = DeviceObject as PVOID;
    (*Wcb).DeviceContext = Context;
    (*Wcb).CurrentIrp = (*DeviceObject).CurrentIrp as PVOID;

    HalAllocateAdapterChannel(
        AdapterObject,
        Wcb,
        NumberOfMapRegisters,
        ExecutionRoutine,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/adapter.c
//  * PURPOSE:         I/O Wrappers for HAL Adapter APIs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Filip Navara (navaraf@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Forward declaration for HAL function (may not be visible via DMA macro guards) */
// NTHALAPI
// NTSTATUS
// NTAPI
// HalAllocateAdapterChannel(
//     _In_ PADAPTER_OBJECT AdapterObject,
//     _In_ PWAIT_CONTEXT_BLOCK Wcb,
//     _In_ ULONG NumberOfMapRegisters,
//     _In_ PDRIVER_CONTROL ExecutionRoutine);
//
// /* DATA **********************************************************************/
//
// POBJECT_TYPE IoAdapterObjectType;
// POBJECT_TYPE IoDeviceHandlerObjectType;
// ULONG IoDeviceHandlerObjectSize;
//
// /* FUNCTIONS *****************************************************************/
//
// #undef IoAllocateAdapterChannel
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoAllocateAdapterChannel(IN PADAPTER_OBJECT AdapterObject,
//                          IN PDEVICE_OBJECT DeviceObject,
//                          IN ULONG NumberOfMapRegisters,
//                          IN PDRIVER_CONTROL ExecutionRoutine,
//                          IN PVOID Context)
// {
//     PWAIT_CONTEXT_BLOCK Wcb = &DeviceObject->Queue.Wcb;
//
//     /* Initialize the WCB */
//     Wcb->DeviceObject = DeviceObject;
//     Wcb->DeviceContext = Context;
//     Wcb->CurrentIrp = DeviceObject->CurrentIrp;
//
//     /* Call HAL */
//     return HalAllocateAdapterChannel(AdapterObject,
//                                      Wcb,
//                                      NumberOfMapRegisters,
//                                      ExecutionRoutine);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
