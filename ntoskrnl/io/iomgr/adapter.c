/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL - See COPYING in the top level directory
 * FILE:            ntoskrnl/io/iomgr/adapter.c
 * PURPOSE:         I/O Wrappers for HAL Adapter APIs
 * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
 *                  Filip Navara (navaraf@reactos.org)
 */

/* INCLUDES *****************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* Forward declaration for HAL function (may not be visible via DMA macro guards) */
NTHALAPI
NTSTATUS
NTAPI
HalAllocateAdapterChannel(
    _In_ PADAPTER_OBJECT AdapterObject,
    _In_ PWAIT_CONTEXT_BLOCK Wcb,
    _In_ ULONG NumberOfMapRegisters,
    _In_ PDRIVER_CONTROL ExecutionRoutine);

/* DATA **********************************************************************/

POBJECT_TYPE IoAdapterObjectType;
POBJECT_TYPE IoDeviceHandlerObjectType;
ULONG IoDeviceHandlerObjectSize;

/* FUNCTIONS *****************************************************************/

#undef IoAllocateAdapterChannel
/*
 * @implemented
 */
NTSTATUS
NTAPI
IoAllocateAdapterChannel(IN PADAPTER_OBJECT AdapterObject,
                         IN PDEVICE_OBJECT DeviceObject,
                         IN ULONG NumberOfMapRegisters,
                         IN PDRIVER_CONTROL ExecutionRoutine,
                         IN PVOID Context)
{
    PWAIT_CONTEXT_BLOCK Wcb = &DeviceObject->Queue.Wcb;

    /* Initialize the WCB */
    Wcb->DeviceObject = DeviceObject;
    Wcb->DeviceContext = Context;
    Wcb->CurrentIrp = DeviceObject->CurrentIrp;

    /* Call HAL */
    return HalAllocateAdapterChannel(AdapterObject,
                                     Wcb,
                                     NumberOfMapRegisters,
                                     ExecutionRoutine);
}

/* EOF */
