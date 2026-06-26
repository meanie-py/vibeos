/* Cohab C TU for the two usbport functions the Rust translation left as commented
 * stubs (lazy-stubbed during the API translation): USBPORT_AbortTransfers (from device.c)
 * and USBPORT_ClearStall (from urb.c). They are kept as faithful C here (verbatim from the
 * original .c) so the module links; the rest of usbport is Rust (usbport-rust). pnp.c is
 * likewise still C (whole file untranslated). These are the untranslated remainder to flag.
 * Never-delete / never-revert rules are not implicated: these bodies were never translated.
 */
#include "usbport.h"

#define NDEBUG
#include <debug.h>

#define NDEBUG_USBPORT_URB
#include "usbdebug.h"

/* Not declared in usbport.h (used file-locally in the original device.c); it is an
 * NTAPI export of usbport-rust, so the forward decl links to the Rust symbol. */
BOOLEAN
NTAPI
USBPORT_DeviceHasTransfers(IN PDEVICE_OBJECT FdoDevice,
                           IN PUSBPORT_DEVICE_HANDLE DeviceHandle);

VOID
NTAPI
USBPORT_AbortTransfers(IN PDEVICE_OBJECT FdoDevice,
                       IN PUSBPORT_DEVICE_HANDLE DeviceHandle)
{
    PLIST_ENTRY HandleList;
    PUSBPORT_PIPE_HANDLE PipeHandle;
    BOOLEAN Result;

    DPRINT("USBPORT_AbortAllTransfers: ... \n");

    HandleList = DeviceHandle->PipeHandleList.Flink;

    while (HandleList != &DeviceHandle->PipeHandleList)
    {
        PipeHandle = CONTAINING_RECORD(HandleList,
                                       USBPORT_PIPE_HANDLE,
                                       PipeLink);

        HandleList = HandleList->Flink;

        if (!(PipeHandle->Flags & DEVICE_HANDLE_FLAG_ROOTHUB))
        {
            PipeHandle->Endpoint->Flags |= ENDPOINT_FLAG_ABORTING;

            USBPORT_AbortEndpoint(FdoDevice, PipeHandle->Endpoint, NULL);
            USBPORT_FlushMapTransfers(FdoDevice);
        }
    }

    while (TRUE)
    {
        Result = USBPORT_DeviceHasTransfers(FdoDevice, DeviceHandle);

        if (!Result)
            break;

        USBPORT_Wait(FdoDevice, 100);
    }
}

NTSTATUS
NTAPI
USBPORT_ClearStall(IN PDEVICE_OBJECT FdoDevice,
                   IN PIRP Irp,
                   IN PURB Urb)
{
    PUSBPORT_DEVICE_HANDLE DeviceHandle;
    PUSBPORT_PIPE_HANDLE PipeHandle;
    USBD_STATUS USBDStatus;
    PUSBPORT_ENDPOINT Endpoint;
    NTSTATUS Status;
    USB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;

    DPRINT_URB("USBPORT_ClearStall: ... \n");

    PipeHandle = Urb->UrbPipeRequest.PipeHandle;
    DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;

    if (!USBPORT_ValidatePipeHandle(DeviceHandle, PipeHandle))
    {
        return USBPORT_USBDStatusToNtStatus(Urb,
                                            USBD_STATUS_INVALID_PIPE_HANDLE);
    }

    Endpoint = PipeHandle->Endpoint;

    RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));

    SetupPacket.bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
    SetupPacket.bRequest = USB_REQUEST_CLEAR_FEATURE;
    SetupPacket.wValue.W = 0;
    SetupPacket.wIndex.W = Endpoint->EndpointProperties.EndpointAddress;
    SetupPacket.wLength = 0;

    USBPORT_SendSetupPacket(DeviceHandle,
                            FdoDevice,
                            &SetupPacket,
                            NULL,
                            0,
                            NULL,
                            &USBDStatus);

    Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);

    return Status;
}
