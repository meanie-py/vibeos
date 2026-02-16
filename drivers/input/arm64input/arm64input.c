/*
 * PROJECT:     ReactOS ARM64 Input Stub Driver
 * LICENSE:     GPL-2.0-or-later
 * FILE:        drivers/input/arm64input/arm64input.c
 * PURPOSE:     Provides \Device\KeyboardClass0 and \Device\PointerClass0
 *              so the Win32k RIT (Raw Input Thread) can proceed past
 *              device discovery. IRP_MJ_READ pends forever; keyboard
 *              IOCTLs return reasonable defaults.
 */

#include <ntddk.h>
#include <ntddkbd.h>
#include <ntddmou.h>

#define TAG_INPUT 'pnIA'

/* Device extension — tracks pending read IRPs */
typedef struct _INPUT_DEVICE_EXT
{
    BOOLEAN IsKeyboard;  /* TRUE = keyboard, FALSE = mouse */
    LIST_ENTRY PendingIrps;
    KSPIN_LOCK IrpLock;
} INPUT_DEVICE_EXT, *PINPUT_DEVICE_EXT;

/* Cancel routine for pending read IRPs */
static VOID NTAPI
InputCancelIrp(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp)
{
    PINPUT_DEVICE_EXT DevExt = (PINPUT_DEVICE_EXT)DeviceObject->DeviceExtension;
    KIRQL OldIrql;

    IoReleaseCancelSpinLock(Irp->CancelIrql);

    KeAcquireSpinLock(&DevExt->IrpLock, &OldIrql);
    RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
    KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);

    Irp->IoStatus.Status = STATUS_CANCELLED;
    Irp->IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
}

/* IRP_MJ_CREATE — allow open */
static NTSTATUS NTAPI
InputCreate(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp)
{
    UNREFERENCED_PARAMETER(DeviceObject);
    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

/* IRP_MJ_CLOSE — allow close */
static NTSTATUS NTAPI
InputClose(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp)
{
    UNREFERENCED_PARAMETER(DeviceObject);
    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

/* IRP_MJ_CLEANUP — cancel all pending IRPs for this file object */
static NTSTATUS NTAPI
InputCleanup(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp)
{
    PINPUT_DEVICE_EXT DevExt = (PINPUT_DEVICE_EXT)DeviceObject->DeviceExtension;
    PFILE_OBJECT FileObject = IoGetCurrentIrpStackLocation(Irp)->FileObject;
    KIRQL OldIrql;
    LIST_ENTRY CancelList;
    PLIST_ENTRY Entry;
    PIRP PendingIrp;

    InitializeListHead(&CancelList);

    KeAcquireSpinLock(&DevExt->IrpLock, &OldIrql);
    Entry = DevExt->PendingIrps.Flink;
    while (Entry != &DevExt->PendingIrps)
    {
        PendingIrp = CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry);
        Entry = Entry->Flink;
        if (IoGetCurrentIrpStackLocation(PendingIrp)->FileObject == FileObject)
        {
            RemoveEntryList(&PendingIrp->Tail.Overlay.ListEntry);
            if (IoSetCancelRoutine(PendingIrp, NULL) != NULL)
            {
                InsertTailList(&CancelList, &PendingIrp->Tail.Overlay.ListEntry);
            }
        }
    }
    KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);

    while (!IsListEmpty(&CancelList))
    {
        Entry = RemoveHeadList(&CancelList);
        PendingIrp = CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry);
        PendingIrp->IoStatus.Status = STATUS_CANCELLED;
        PendingIrp->IoStatus.Information = 0;
        IoCompleteRequest(PendingIrp, IO_NO_INCREMENT);
    }

    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

/* IRP_MJ_READ — pend forever (RIT waits on FILE_OBJECT->Event) */
static NTSTATUS NTAPI
InputRead(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp)
{
    PINPUT_DEVICE_EXT DevExt = (PINPUT_DEVICE_EXT)DeviceObject->DeviceExtension;
    KIRQL OldIrql;

    IoMarkIrpPending(Irp);

    KeAcquireSpinLock(&DevExt->IrpLock, &OldIrql);
    IoSetCancelRoutine(Irp, InputCancelIrp);
    InsertTailList(&DevExt->PendingIrps, &Irp->Tail.Overlay.ListEntry);
    KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);

    return STATUS_PENDING;
}

/* IRP_MJ_DEVICE_CONTROL — keyboard IOCTLs */
static NTSTATUS NTAPI
InputDeviceControl(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp)
{
    PINPUT_DEVICE_EXT DevExt = (PINPUT_DEVICE_EXT)DeviceObject->DeviceExtension;
    PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation(Irp);
    NTSTATUS Status = STATUS_NOT_SUPPORTED;
    ULONG_PTR Information = 0;

    if (DevExt->IsKeyboard)
    {
        switch (IrpSp->Parameters.DeviceIoControl.IoControlCode)
        {
            case IOCTL_KEYBOARD_QUERY_ATTRIBUTES:
            {
                if (IrpSp->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(KEYBOARD_ATTRIBUTES))
                {
                    PKEYBOARD_ATTRIBUTES pAttr = (PKEYBOARD_ATTRIBUTES)Irp->AssociatedIrp.SystemBuffer;
                    RtlZeroMemory(pAttr, sizeof(KEYBOARD_ATTRIBUTES));
                    pAttr->KeyboardIdentifier.Type = 4;    /* AT-101 enhanced */
                    pAttr->KeyboardIdentifier.Subtype = 0;
                    pAttr->NumberOfFunctionKeys = 12;
                    pAttr->NumberOfIndicators = 3;
                    pAttr->NumberOfKeysTotal = 101;
                    pAttr->InputDataQueueLength = 100;
                    pAttr->KeyRepeatMinimum.Rate = 2;
                    pAttr->KeyRepeatMinimum.Delay = 250;
                    pAttr->KeyRepeatMaximum.Rate = 30;
                    pAttr->KeyRepeatMaximum.Delay = 1000;
                    Status = STATUS_SUCCESS;
                    Information = sizeof(KEYBOARD_ATTRIBUTES);
                }
                else
                {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }
                break;
            }

            case IOCTL_KEYBOARD_QUERY_INDICATORS:
            {
                if (IrpSp->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(KEYBOARD_INDICATOR_PARAMETERS))
                {
                    PKEYBOARD_INDICATOR_PARAMETERS pInd = (PKEYBOARD_INDICATOR_PARAMETERS)Irp->AssociatedIrp.SystemBuffer;
                    pInd->UnitId = 0;
                    pInd->LedFlags = 0;  /* All LEDs off */
                    Status = STATUS_SUCCESS;
                    Information = sizeof(KEYBOARD_INDICATOR_PARAMETERS);
                }
                else
                {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }
                break;
            }

            case IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION:
                /* No indicator translation — caller handles failure gracefully */
                Status = STATUS_NOT_SUPPORTED;
                break;

            case IOCTL_KEYBOARD_SET_INDICATORS:
                /* Accept but ignore LED set requests */
                Status = STATUS_SUCCESS;
                break;

            default:
                Status = STATUS_NOT_SUPPORTED;
                break;
        }
    }
    /* Mouse IOCTLs — none expected by RIT, return NOT_SUPPORTED */

    Irp->IoStatus.Status = Status;
    Irp->IoStatus.Information = Information;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return Status;
}

NTSTATUS NTAPI
DriverEntry(
    PDRIVER_OBJECT DriverObject,
    PUNICODE_STRING RegistryPath)
{
    NTSTATUS Status;
    PDEVICE_OBJECT KbdDevice = NULL, MouDevice = NULL;
    PINPUT_DEVICE_EXT DevExt;
    UNICODE_STRING DeviceName;

    UNREFERENCED_PARAMETER(RegistryPath);

    DbgPrint("[ARM64INPUT] DriverEntry\n");

    DriverObject->MajorFunction[IRP_MJ_CREATE]         = InputCreate;
    DriverObject->MajorFunction[IRP_MJ_CLOSE]          = InputClose;
    DriverObject->MajorFunction[IRP_MJ_CLEANUP]        = InputCleanup;
    DriverObject->MajorFunction[IRP_MJ_READ]           = InputRead;
    DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = InputDeviceControl;

    /* Create keyboard class device */
    RtlInitUnicodeString(&DeviceName, L"\\Device\\KeyboardClass0");
    Status = IoCreateDevice(DriverObject,
                            sizeof(INPUT_DEVICE_EXT),
                            &DeviceName,
                            FILE_DEVICE_KEYBOARD,
                            0,
                            FALSE,
                            &KbdDevice);
    if (!NT_SUCCESS(Status))
    {
        DbgPrint("[ARM64INPUT] Failed to create KeyboardClass0: 0x%08lx\n", Status);
        return Status;
    }

    DevExt = (PINPUT_DEVICE_EXT)KbdDevice->DeviceExtension;
    DevExt->IsKeyboard = TRUE;
    InitializeListHead(&DevExt->PendingIrps);
    KeInitializeSpinLock(&DevExt->IrpLock);
    KbdDevice->Flags |= DO_BUFFERED_IO;
    KbdDevice->Flags &= ~DO_DEVICE_INITIALIZING;

    DbgPrint("[ARM64INPUT] \\Device\\KeyboardClass0 created\n");

    /* Create mouse (pointer) class device */
    RtlInitUnicodeString(&DeviceName, L"\\Device\\PointerClass0");
    Status = IoCreateDevice(DriverObject,
                            sizeof(INPUT_DEVICE_EXT),
                            &DeviceName,
                            FILE_DEVICE_MOUSE,
                            0,
                            FALSE,
                            &MouDevice);
    if (!NT_SUCCESS(Status))
    {
        DbgPrint("[ARM64INPUT] Failed to create PointerClass0: 0x%08lx\n", Status);
        IoDeleteDevice(KbdDevice);
        return Status;
    }

    DevExt = (PINPUT_DEVICE_EXT)MouDevice->DeviceExtension;
    DevExt->IsKeyboard = FALSE;
    InitializeListHead(&DevExt->PendingIrps);
    KeInitializeSpinLock(&DevExt->IrpLock);
    MouDevice->Flags |= DO_BUFFERED_IO;
    MouDevice->Flags &= ~DO_DEVICE_INITIALIZING;

    DbgPrint("[ARM64INPUT] \\Device\\PointerClass0 created\n");

    return STATUS_SUCCESS;
}
