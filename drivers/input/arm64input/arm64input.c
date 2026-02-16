/*
 * PROJECT:     ReactOS ARM64 Input Driver
 * LICENSE:     GPL-2.0-or-later
 * FILE:        drivers/input/arm64input/arm64input.c
 * PURPOSE:     Provides \Device\KeyboardClass0 and \Device\PointerClass0
 *              for the Win32k RIT (Raw Input Thread). Keyboard events are
 *              injected via IOCTL_ARM64INPUT_INJECT_BYTE from user-mode
 *              (e.g., the serial console frontend).
 */

#include <ntddk.h>
#include <ntddkbd.h>
#include <ntddmou.h>

#define TAG_INPUT 'pnIA'

/*
 * Custom IOCTL: inject a single ASCII byte as keyboard input.
 * Input buffer: 1 byte (the ASCII character).
 * The driver converts it to KEYBOARD_INPUT_DATA (make + break).
 */
#define IOCTL_ARM64INPUT_INJECT_BYTE \
    CTL_CODE(FILE_DEVICE_KEYBOARD, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)

/*
 * Custom IOCTL: inject a mouse event as MOUSE_INPUT_DATA.
 * Input buffer: sizeof(MOUSE_INPUT_DATA).
 */
#define IOCTL_ARM64INPUT_INJECT_MOUSE \
    CTL_CODE(FILE_DEVICE_MOUSE, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)

/* Ring buffer sizes */
#define KBD_RING_SIZE 64
#define MOU_RING_SIZE 32

/* Device extension — tracks pending read IRPs and keyboard ring buffer */
typedef struct _INPUT_DEVICE_EXT
{
    BOOLEAN IsKeyboard;
    LIST_ENTRY PendingIrps;
    KSPIN_LOCK IrpLock;

    /* Keyboard-only fields (used when IsKeyboard == TRUE) */
    KEYBOARD_INPUT_DATA Ring[KBD_RING_SIZE];
    ULONG RingHead;                 /* Next event to consume */
    ULONG RingTail;                 /* Next slot to produce into */

    /* Mouse-only fields (used when IsKeyboard == FALSE) */
    MOUSE_INPUT_DATA MouRing[MOU_RING_SIZE];
    ULONG MouRingHead;
    ULONG MouRingTail;

    PDEVICE_OBJECT SelfDevice;
} INPUT_DEVICE_EXT, *PINPUT_DEVICE_EXT;

/*
 * Minimal ASCII-to-Scan Code Set 1 table.
 * Index = ASCII character, Value = AT scan code (0 = unsupported).
 */
static const UCHAR AsciiToScanCode[128] = {
    /* 0x00-0x0F: control chars */
    0,    0x01, 0,    0x2E, 0,    0,    0,    0,     /* NUL, ESC, _, Ctrl-C */
    0x0E, 0x0F, 0x1C, 0,    0,    0x1C, 0,    0,     /* BS, TAB, LF, CR */
    /* 0x10-0x1F */
    0,    0,    0,    0,    0,    0,    0,    0,
    0,    0,    0,    0x01, 0,    0,    0,    0,     /* ESC at 0x1B */
    /* 0x20-0x2F: space, punctuation */
    0x39, 0x02, 0x28, 0x04, 0x05, 0x06, 0x08, 0x28, /* space ! " # $ % & ' */
    0x0A, 0x0B, 0x09, 0x0D, 0x33, 0x0C, 0x34, 0x35, /* ( ) * + , - . / */
    /* 0x30-0x39: digits */
    0x0B, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, /* 0-7 */
    0x09, 0x0A,                                       /* 8-9 */
    /* 0x3A-0x3F */
    0x27, 0x27, 0x33, 0x0D, 0x34, 0x35,              /* : ; < = > ? */
    /* 0x40-0x5A: @ and uppercase A-Z */
    0x03,                                              /* @ */
    0x1E, 0x30, 0x2E, 0x20, 0x12, 0x21, 0x22, 0x23,  /* A-H */
    0x17, 0x24, 0x25, 0x26, 0x32, 0x31, 0x18, 0x19,  /* I-P */
    0x10, 0x13, 0x1F, 0x14, 0x16, 0x2F, 0x11, 0x2D,  /* Q-X */
    0x15, 0x2C,                                        /* Y-Z */
    /* 0x5B-0x60 */
    0x1A, 0x2B, 0x1B, 0x07, 0x0C, 0x29,              /* [ \ ] ^ _ ` */
    /* 0x61-0x7A: lowercase a-z (same scan codes as uppercase) */
    0x1E, 0x30, 0x2E, 0x20, 0x12, 0x21, 0x22, 0x23,  /* a-h */
    0x17, 0x24, 0x25, 0x26, 0x32, 0x31, 0x18, 0x19,  /* i-p */
    0x10, 0x13, 0x1F, 0x14, 0x16, 0x2F, 0x11, 0x2D,  /* q-x */
    0x15, 0x2C,                                        /* y-z */
    /* 0x7B-0x7F */
    0x1A, 0x2B, 0x1B, 0x29, 0x0E                     /* { | } ~ DEL */
};

/* Check if ring buffer has space */
static BOOLEAN
RingHasSpace(PINPUT_DEVICE_EXT DevExt)
{
    return ((DevExt->RingTail + 1) % KBD_RING_SIZE) != DevExt->RingHead;
}

/* Check if ring buffer has data */
static BOOLEAN
RingHasData(PINPUT_DEVICE_EXT DevExt)
{
    return DevExt->RingHead != DevExt->RingTail;
}

/* Push a keyboard event to the ring buffer */
static VOID
RingPush(PINPUT_DEVICE_EXT DevExt, USHORT MakeCode, USHORT Flags)
{
    if (!RingHasSpace(DevExt))
        return;
    DevExt->Ring[DevExt->RingTail].UnitId = 0;
    DevExt->Ring[DevExt->RingTail].MakeCode = MakeCode;
    DevExt->Ring[DevExt->RingTail].Flags = Flags;
    DevExt->Ring[DevExt->RingTail].Reserved = 0;
    DevExt->Ring[DevExt->RingTail].ExtraInformation = 0;
    DevExt->RingTail = (DevExt->RingTail + 1) % KBD_RING_SIZE;
}

/* Pop a keyboard event from the ring buffer */
static KEYBOARD_INPUT_DATA
RingPop(PINPUT_DEVICE_EXT DevExt)
{
    KEYBOARD_INPUT_DATA data = DevExt->Ring[DevExt->RingHead];
    DevExt->RingHead = (DevExt->RingHead + 1) % KBD_RING_SIZE;
    return data;
}

/* Mouse ring buffer helpers */
static BOOLEAN
MouRingHasSpace(PINPUT_DEVICE_EXT DevExt)
{
    return ((DevExt->MouRingTail + 1) % MOU_RING_SIZE) != DevExt->MouRingHead;
}

static BOOLEAN
MouRingHasData(PINPUT_DEVICE_EXT DevExt)
{
    return DevExt->MouRingHead != DevExt->MouRingTail;
}

static VOID
MouRingPush(PINPUT_DEVICE_EXT DevExt, PMOUSE_INPUT_DATA Data)
{
    if (!MouRingHasSpace(DevExt))
        return;
    DevExt->MouRing[DevExt->MouRingTail] = *Data;
    DevExt->MouRingTail = (DevExt->MouRingTail + 1) % MOU_RING_SIZE;
}

static MOUSE_INPUT_DATA
MouRingPop(PINPUT_DEVICE_EXT DevExt)
{
    MOUSE_INPUT_DATA data = DevExt->MouRing[DevExt->MouRingHead];
    DevExt->MouRingHead = (DevExt->MouRingHead + 1) % MOU_RING_SIZE;
    return data;
}

/* Try to complete a pending IRP with buffered mouse data */
static VOID
MouTryCompleteIrp(PINPUT_DEVICE_EXT DevExt)
{
    KIRQL OldIrql;
    PIRP Irp;
    PLIST_ENTRY Entry;
    MOUSE_INPUT_DATA Data;

    while (MouRingHasData(DevExt))
    {
        KeAcquireSpinLock(&DevExt->IrpLock, &OldIrql);
        if (IsListEmpty(&DevExt->PendingIrps))
        {
            KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);
            break;
        }
        Entry = RemoveHeadList(&DevExt->PendingIrps);
        Irp = CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry);
        if (IoSetCancelRoutine(Irp, NULL) == NULL)
        {
            KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);
            continue;
        }
        KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);

        Data = MouRingPop(DevExt);

        RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer,
                       &Data,
                       sizeof(MOUSE_INPUT_DATA));
        Irp->IoStatus.Status = STATUS_SUCCESS;
        Irp->IoStatus.Information = sizeof(MOUSE_INPUT_DATA);
        IoCompleteRequest(Irp, IO_MOUSE_INCREMENT);
    }
}

/* Try to complete a pending IRP with buffered keyboard data */
static VOID
TryCompleteIrp(PINPUT_DEVICE_EXT DevExt)
{
    KIRQL OldIrql;
    PIRP Irp;
    PLIST_ENTRY Entry;
    KEYBOARD_INPUT_DATA Data;

    while (RingHasData(DevExt))
    {
        KeAcquireSpinLock(&DevExt->IrpLock, &OldIrql);
        if (IsListEmpty(&DevExt->PendingIrps))
        {
            KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);
            break;
        }
        Entry = RemoveHeadList(&DevExt->PendingIrps);
        Irp = CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry);
        if (IoSetCancelRoutine(Irp, NULL) == NULL)
        {
            /* IRP is being cancelled, skip it */
            KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);
            continue;
        }
        KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);

        Data = RingPop(DevExt);

        /* Copy keyboard data to IRP output buffer */
        RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer,
                       &Data,
                       sizeof(KEYBOARD_INPUT_DATA));
        Irp->IoStatus.Status = STATUS_SUCCESS;
        Irp->IoStatus.Information = sizeof(KEYBOARD_INPUT_DATA);
        IoCompleteRequest(Irp, IO_KEYBOARD_INCREMENT);
    }
}

/*
 * Inject one ASCII byte as keyboard input: convert to scan code,
 * push make+break events, and try to complete pending RIT IRPs.
 */
static VOID
InjectByte(PINPUT_DEVICE_EXT DevExt, UCHAR Byte)
{
    USHORT ScanCode;
    BOOLEAN NeedShift;

    if (Byte >= 128)
        return;

    ScanCode = AsciiToScanCode[Byte];
    if (ScanCode == 0)
        return;

    /* Check if shift is needed (uppercase or shifted punctuation) */
    NeedShift = (Byte >= 'A' && Byte <= 'Z') ||
                Byte == '!' || Byte == '@' || Byte == '#' ||
                Byte == '$' || Byte == '%' || Byte == '^' ||
                Byte == '&' || Byte == '*' || Byte == '(' ||
                Byte == ')' || Byte == '_' || Byte == '+' ||
                Byte == '{' || Byte == '}' || Byte == '|' ||
                Byte == ':' || Byte == '"' || Byte == '<' ||
                Byte == '>' || Byte == '?' || Byte == '~';

    /* Generate key events: [Shift down,] key down, key up [, Shift up] */
    if (NeedShift)
        RingPush(DevExt, 0x2A, 0);  /* Left Shift down */

    RingPush(DevExt, ScanCode, 0);          /* Key down */
    RingPush(DevExt, ScanCode, KEY_BREAK);  /* Key up */

    if (NeedShift)
        RingPush(DevExt, 0x2A, KEY_BREAK);  /* Left Shift up */

    TryCompleteIrp(DevExt);
}

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

/* IRP_MJ_READ — complete immediately if data available, else pend */
static NTSTATUS NTAPI
InputRead(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp)
{
    PINPUT_DEVICE_EXT DevExt = (PINPUT_DEVICE_EXT)DeviceObject->DeviceExtension;
    KIRQL OldIrql;

    /* If data is buffered, complete immediately */
    if (DevExt->IsKeyboard && RingHasData(DevExt))
    {
        KEYBOARD_INPUT_DATA Data = RingPop(DevExt);
        RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer,
                       &Data,
                       sizeof(KEYBOARD_INPUT_DATA));
        Irp->IoStatus.Status = STATUS_SUCCESS;
        Irp->IoStatus.Information = sizeof(KEYBOARD_INPUT_DATA);
        IoCompleteRequest(Irp, IO_KEYBOARD_INCREMENT);
        return STATUS_SUCCESS;
    }

    if (!DevExt->IsKeyboard && MouRingHasData(DevExt))
    {
        MOUSE_INPUT_DATA Data = MouRingPop(DevExt);
        RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer,
                       &Data,
                       sizeof(MOUSE_INPUT_DATA));
        Irp->IoStatus.Status = STATUS_SUCCESS;
        Irp->IoStatus.Information = sizeof(MOUSE_INPUT_DATA);
        IoCompleteRequest(Irp, IO_MOUSE_INCREMENT);
        return STATUS_SUCCESS;
    }

    /* No data — pend the IRP */
    IoMarkIrpPending(Irp);

    KeAcquireSpinLock(&DevExt->IrpLock, &OldIrql);
    IoSetCancelRoutine(Irp, InputCancelIrp);
    InsertTailList(&DevExt->PendingIrps, &Irp->Tail.Overlay.ListEntry);
    KeReleaseSpinLock(&DevExt->IrpLock, OldIrql);

    return STATUS_PENDING;
}

/* IRP_MJ_DEVICE_CONTROL — keyboard IOCTLs + byte injection */
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
            case IOCTL_ARM64INPUT_INJECT_BYTE:
            {
                if (IrpSp->Parameters.DeviceIoControl.InputBufferLength >= 1)
                {
                    PUCHAR pByte = (PUCHAR)Irp->AssociatedIrp.SystemBuffer;
                    InjectByte(DevExt, *pByte);
                    Status = STATUS_SUCCESS;
                }
                else
                {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }
                break;
            }

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
                    pInd->LedFlags = 0;
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
                Status = STATUS_NOT_SUPPORTED;
                break;

            case IOCTL_KEYBOARD_SET_INDICATORS:
                Status = STATUS_SUCCESS;
                break;

            default:
                Status = STATUS_NOT_SUPPORTED;
                break;
        }
    }
    else
    {
        /* Mouse device IOCTLs */
        switch (IrpSp->Parameters.DeviceIoControl.IoControlCode)
        {
            case IOCTL_ARM64INPUT_INJECT_MOUSE:
            {
                if (IrpSp->Parameters.DeviceIoControl.InputBufferLength >= sizeof(MOUSE_INPUT_DATA))
                {
                    PMOUSE_INPUT_DATA pMou = (PMOUSE_INPUT_DATA)Irp->AssociatedIrp.SystemBuffer;
                    MouRingPush(DevExt, pMou);
                    MouTryCompleteIrp(DevExt);
                    Status = STATUS_SUCCESS;
                }
                else
                {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }
                break;
            }

            case IOCTL_MOUSE_QUERY_ATTRIBUTES:
            {
                if (IrpSp->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(MOUSE_ATTRIBUTES))
                {
                    PMOUSE_ATTRIBUTES pAttr = (PMOUSE_ATTRIBUTES)Irp->AssociatedIrp.SystemBuffer;
                    RtlZeroMemory(pAttr, sizeof(MOUSE_ATTRIBUTES));
                    pAttr->MouseIdentifier = MOUSE_SERIAL_HARDWARE;
                    pAttr->NumberOfButtons = 3;
                    pAttr->SampleRate = 60;
                    pAttr->InputDataQueueLength = MOU_RING_SIZE;
                    Status = STATUS_SUCCESS;
                    Information = sizeof(MOUSE_ATTRIBUTES);
                }
                else
                {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }
                break;
            }

            default:
                Status = STATUS_NOT_SUPPORTED;
                break;
        }
    }

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
    UNICODE_STRING DeviceName, SymlinkName;

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

    /* Create DOS device symlink so user-mode can open \\.\KeyboardClass0 */
    RtlInitUnicodeString(&SymlinkName, L"\\DosDevices\\KeyboardClass0");
    IoCreateSymbolicLink(&SymlinkName, &DeviceName);

    DevExt = (PINPUT_DEVICE_EXT)KbdDevice->DeviceExtension;
    DevExt->IsKeyboard = TRUE;
    InitializeListHead(&DevExt->PendingIrps);
    KeInitializeSpinLock(&DevExt->IrpLock);
    DevExt->RingHead = 0;
    DevExt->RingTail = 0;
    DevExt->SelfDevice = KbdDevice;

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
    DevExt->RingHead = 0;
    DevExt->MouRingHead = 0;
    DevExt->MouRingTail = 0;
    DevExt->SelfDevice = MouDevice;
    MouDevice->Flags |= DO_BUFFERED_IO;
    MouDevice->Flags &= ~DO_DEVICE_INITIALIZING;

    /* Create DOS device symlink so user-mode can open \\.\PointerClass0 */
    RtlInitUnicodeString(&SymlinkName, L"\\DosDevices\\PointerClass0");
    RtlInitUnicodeString(&DeviceName, L"\\Device\\PointerClass0");
    IoCreateSymbolicLink(&SymlinkName, &DeviceName);

    DbgPrint("[ARM64INPUT] \\Device\\PointerClass0 created\n");

    return STATUS_SUCCESS;
}
