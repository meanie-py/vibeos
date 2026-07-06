use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    BUS_INTERFACE_STANDARD, DEVICE_DESCRIPTION, IO_STACK_LOCATION, _IO_STATUS_BLOCK,
    IoGetDeviceProperty, IopInitiatePnpIrp, PDEVICE_DESCRIPTION, PDEVICE_OBJECT,
    PDMA_ADAPTER, PINTERFACE, DEVICE_REGISTRY_PROPERTY_DevicePropertyLegacyBusType,
    _INTERFACE_TYPE, _INTERFACE_TYPE_PNPBus, _INTERFACE_TYPE_InterfaceTypeUndefined,
    _INTERFACE_TYPE_Internal, GUID, HAL_DISPATCH,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    static GUID_BUS_INTERFACE_STANDARD: GUID;
    static mut HalDispatchTable: HAL_DISPATCH;
}

const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
const PNPBus: _INTERFACE_TYPE = _INTERFACE_TYPE_PNPBus;
const InterfaceTypeUndefined: _INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined;
const Internal: _INTERFACE_TYPE = _INTERFACE_TYPE_Internal;

#[no_mangle]
pub unsafe extern "system" fn IoGetDmaAdapter(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    DeviceDescription: PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: *mut ULONG,
) -> PDMA_ADAPTER {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);
    let NumberOfMapRegisters = opaque_ptr(NumberOfMapRegisters);
    let mut ResultLength: ULONG = 0;
    let mut BusInterface: BUS_INTERFACE_STANDARD = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Stack: IO_STACK_LOCATION = core::mem::zeroed();
    let mut PrivateDeviceDescription: DEVICE_DESCRIPTION = core::mem::zeroed();
    let mut Adapter: PDMA_ADAPTER = core::ptr::null_mut();
    let mut DeviceDescriptionPtr = DeviceDescription;

    if !PhysicalDeviceObject.is_null() {
        if (*DeviceDescription).InterfaceType == PNPBus
            || (*DeviceDescription).InterfaceType == InterfaceTypeUndefined
        {
            core::ptr::copy_nonoverlapping(
                DeviceDescription as *const u8,
                core::ptr::addr_of_mut!(PrivateDeviceDescription) as *mut u8,
                core::mem::size_of::<DEVICE_DESCRIPTION>(),
            );

            let Status = IoGetDeviceProperty(
                PhysicalDeviceObject,
                DEVICE_REGISTRY_PROPERTY_DevicePropertyLegacyBusType,
                core::mem::size_of::<_INTERFACE_TYPE>() as ULONG,
                core::ptr::addr_of_mut!(PrivateDeviceDescription.InterfaceType) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(ResultLength),
            );

            if Status < 0 {
                PrivateDeviceDescription.InterfaceType = Internal;
            }

            DeviceDescriptionPtr = core::ptr::addr_of_mut!(PrivateDeviceDescription);
        }

        Stack.Parameters.QueryInterface.Size = core::mem::size_of::<BUS_INTERFACE_STANDARD>() as USHORT;
        Stack.Parameters.QueryInterface.Version = 1;
        Stack.Parameters.QueryInterface.Interface = core::ptr::addr_of_mut!(BusInterface) as PINTERFACE;
        Stack.Parameters.QueryInterface.InterfaceType = core::ptr::addr_of!(GUID_BUS_INTERFACE_STANDARD);

        let Status = IopInitiatePnpIrp(
            PhysicalDeviceObject,
            core::ptr::addr_of_mut!(IoStatusBlock),
            IRP_MN_QUERY_INTERFACE,
            core::ptr::addr_of_mut!(Stack),
        );

        if Status >= 0 {
            Adapter = (BusInterface.GetDmaAdapter.unwrap())(
                BusInterface.Context,
                DeviceDescriptionPtr,
                NumberOfMapRegisters,
            );

            (BusInterface.InterfaceDereference.unwrap())(BusInterface.Context);
            if !Adapter.is_null() {
                return Adapter;
            }
        }
    }

    /* Fall back to HAL */
    (HalDispatchTable.HalGetDmaAdapter.unwrap())(
        PhysicalDeviceObject as PVOID,
        DeviceDescriptionPtr,
        NumberOfMapRegisters,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * COPYRIGHT:       GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/pnpmgr/pnpdma.c
//  * PURPOSE:         PnP manager DMA routines
//  * PROGRAMMERS:     Filip Navara (xnavara@volny.cz)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
// #include <wdmguid.h>
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// PDMA_ADAPTER
// NTAPI
// IoGetDmaAdapter(IN PDEVICE_OBJECT PhysicalDeviceObject,
//                 IN PDEVICE_DESCRIPTION DeviceDescription,
//                 IN OUT PULONG NumberOfMapRegisters)
// {
//     NTSTATUS Status;
//     ULONG ResultLength;
//     BUS_INTERFACE_STANDARD BusInterface;
//     IO_STATUS_BLOCK IoStatusBlock;
//     IO_STACK_LOCATION Stack;
//     DEVICE_DESCRIPTION PrivateDeviceDescription;
//     PDMA_ADAPTER Adapter = NULL;
//
//     DPRINT("IoGetDmaAdapter called\n");
//
//
//     /* Try to create DMA adapter through bus driver */
//     if (PhysicalDeviceObject)
//     {
//         if (DeviceDescription->InterfaceType == PNPBus ||
//             DeviceDescription->InterfaceType == InterfaceTypeUndefined)
//         {
//             RtlCopyMemory(&PrivateDeviceDescription,
//                           DeviceDescription,
//                           sizeof(DEVICE_DESCRIPTION));
//
//             Status = IoGetDeviceProperty(PhysicalDeviceObject,
//                                          DEVICE_REGISTRY_PROPERTY_DevicePropertyLegacyBusType,
//                                          sizeof(INTERFACE_TYPE),
//                                          &PrivateDeviceDescription.InterfaceType,
//                                          &ResultLength);
//
//             if (!NT_SUCCESS(Status))
//                 PrivateDeviceDescription.InterfaceType = Internal;
//
//             DeviceDescription = &PrivateDeviceDescription;
//         }
//
//         Stack.Parameters.QueryInterface.Size = sizeof(BUS_INTERFACE_STANDARD);
//         Stack.Parameters.QueryInterface.Version = 1;
//         Stack.Parameters.QueryInterface.Interface = (PINTERFACE)&BusInterface;
//         Stack.Parameters.QueryInterface.InterfaceType =
//             &GUID_BUS_INTERFACE_STANDARD;
//
//         Status = IopInitiatePnpIrp(PhysicalDeviceObject,
//                                    &IoStatusBlock,
//                                    IRP_MN_QUERY_INTERFACE,
//                                    &Stack);
//
//         if (NT_SUCCESS(Status))
//         {
//             Adapter = BusInterface.GetDmaAdapter(BusInterface.Context,
//                                                  DeviceDescription,
//                                                  NumberOfMapRegisters);
//
//             BusInterface.InterfaceDereference(BusInterface.Context);
//             if (Adapter) return Adapter;
//         }
//     }
//
//     /* Fall back to HAL */
//     return HalGetDmaAdapter(PhysicalDeviceObject,
//                             DeviceDescription,
//                             NumberOfMapRegisters);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
