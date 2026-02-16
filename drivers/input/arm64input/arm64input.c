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

/* =========================================================================
 * Virtio MMIO transport definitions (virtio spec v1.2, MMIO transport)
 * ========================================================================= */

/* MMIO register offsets */
#define VIRTIO_MMIO_MAGIC           0x000  /* Must be 0x74726976 "virt" */
#define VIRTIO_MMIO_VERSION         0x004  /* 1=legacy, 2=modern */
#define VIRTIO_MMIO_DEVICE_ID       0x008  /* 0=empty, 18=input */
#define VIRTIO_MMIO_VENDOR_ID       0x00C
#define VIRTIO_MMIO_DEV_FEATURES    0x010
#define VIRTIO_MMIO_DEV_FEATURES_SEL 0x014
#define VIRTIO_MMIO_DRV_FEATURES    0x020
#define VIRTIO_MMIO_DRV_FEATURES_SEL 0x024
#define VIRTIO_MMIO_GUEST_PAGE_SIZE 0x028  /* Legacy v1 only */
#define VIRTIO_MMIO_QUEUE_SEL       0x030
#define VIRTIO_MMIO_QUEUE_NUM_MAX   0x034
#define VIRTIO_MMIO_QUEUE_NUM       0x038
#define VIRTIO_MMIO_QUEUE_PFN       0x040  /* Legacy v1 only */
#define VIRTIO_MMIO_QUEUE_READY     0x044
#define VIRTIO_MMIO_QUEUE_NOTIFY    0x050
#define VIRTIO_MMIO_INTERRUPT_STATUS 0x060
#define VIRTIO_MMIO_INTERRUPT_ACK   0x064
#define VIRTIO_MMIO_STATUS          0x070
#define VIRTIO_MMIO_QUEUE_DESC_LOW  0x080
#define VIRTIO_MMIO_QUEUE_DESC_HIGH 0x084
#define VIRTIO_MMIO_QUEUE_AVAIL_LOW 0x090
#define VIRTIO_MMIO_QUEUE_AVAIL_HIGH 0x094
#define VIRTIO_MMIO_QUEUE_USED_LOW  0x0A0
#define VIRTIO_MMIO_QUEUE_USED_HIGH 0x0A4
#define VIRTIO_MMIO_CONFIG          0x100

/* Device status bits */
#define VIRTIO_STATUS_ACK           0x01
#define VIRTIO_STATUS_DRIVER        0x02
#define VIRTIO_STATUS_DRIVER_OK     0x04
#define VIRTIO_STATUS_FEATURES_OK   0x08

/* Virtio input config select values */
#define VIRTIO_INPUT_CFG_UNSET      0x00
#define VIRTIO_INPUT_CFG_ID_NAME    0x01
#define VIRTIO_INPUT_CFG_EV_BITS    0x11
#define VIRTIO_INPUT_CFG_ABS_INFO   0x12

/* Linux evdev event types */
#define EV_SYN  0x00
#define EV_KEY  0x01
#define EV_REL  0x02
#define EV_ABS  0x03

/* Linux evdev absolute axes */
#define ABS_X   0x00
#define ABS_Y   0x01

/* Linux evdev relative axes */
#define REL_X       0x00
#define REL_Y       0x01
#define REL_WHEEL   0x08

/* Linux evdev button codes (mouse buttons start at 0x110) */
#define BTN_LEFT    0x110
#define BTN_RIGHT   0x111
#define BTN_MIDDLE  0x112

/* Linux evdev key codes of interest */
#define KEY_A       30

/* Virtqueue descriptor flags */
#define VIRTQ_DESC_F_WRITE  2

/* Input event struct from device (8 bytes) */
typedef struct _VIRTIO_INPUT_EVENT {
    USHORT Type;
    USHORT Code;
    ULONG  Value;
} VIRTIO_INPUT_EVENT;

/* Split virtqueue structures */
#pragma pack(push, 1)
typedef struct _VIRTQ_DESC {
    ULONGLONG Addr;
    ULONG     Len;
    USHORT    Flags;
    USHORT    Next;
} VIRTQ_DESC;

typedef struct _VIRTQ_AVAIL {
    USHORT Flags;
    USHORT Idx;
    USHORT Ring[];
} VIRTQ_AVAIL;

typedef struct _VIRTQ_USED_ELEM {
    ULONG Id;
    ULONG Len;
} VIRTQ_USED_ELEM;

typedef struct _VIRTQ_USED {
    USHORT Flags;
    USHORT Idx;
    VIRTQ_USED_ELEM Ring[];
} VIRTQ_USED;
#pragma pack(pop)

/* Per-device context */
#define VIRTIO_QUEUE_SIZE   64
#define MAX_VIRTIO_INPUTS   4

typedef struct _VIRTIO_INPUT_DEV {
    volatile UCHAR *Mmio;
    ULONG MmioVersion;
    BOOLEAN IsKeyboard;
    BOOLEAN IsTablet;
    ULONG AbsMaxX, AbsMaxY;

    /* Queue memory */
    PVOID QueueVa;
    PHYSICAL_ADDRESS QueuePa;
    ULONG QueueAllocSize;

    /* Queue component pointers (into QueueVa) */
    VIRTQ_DESC  *Desc;
    VIRTQ_AVAIL *Avail;
    VIRTQ_USED  *Used;
    ULONG QueueSize;

    /* Event buffers */
    VIRTIO_INPUT_EVENT *Events;
    PHYSICAL_ADDRESS EventsPa;

    USHORT LastUsedIdx;

    /* Accumulated mouse state between SYN_REPORT */
    LONG AccumX, AccumY;
    USHORT AccumButtons;
    BOOLEAN HasMove, HasButton;
    BOOLEAN AbsValid;
} VIRTIO_INPUT_DEV;

static VIRTIO_INPUT_DEV VirtioInputs[MAX_VIRTIO_INPUTS];
static ULONG VirtioInputCount = 0;
static HANDLE VirtioPollThreadHandle = NULL;
static BOOLEAN VirtioPollRunning = FALSE;

/* Forward declarations — these get set in DriverEntry */
static PDEVICE_OBJECT g_KbdDevice = NULL;
static PDEVICE_OBJECT g_MouDevice = NULL;

/* MMIO register helpers */
static __inline ULONG VirtioRead32(volatile UCHAR *Base, ULONG Offset) {
    return *(volatile ULONG *)(Base + Offset);
}
static __inline void VirtioWrite32(volatile UCHAR *Base, ULONG Offset, ULONG Val) {
    *(volatile ULONG *)(Base + Offset) = Val;
}

#define virtio_mb()  __asm__ volatile("dmb ish" ::: "memory")
#define virtio_wmb() __asm__ volatile("dmb ishst" ::: "memory")
#define virtio_rmb() __asm__ volatile("dmb ishld" ::: "memory")

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

/* =========================================================================
 * Evdev key code to AT Scan Code Set 1 mapping
 * For evdev codes 1-88, the value IS the AT scan code (identity mapping).
 * Extended keys (>88) need a lookup table.
 * ========================================================================= */

typedef struct _EVDEV_EXT_MAP {
    USHORT EvdevCode;
    USHORT ScanCode;
    USHORT Flags;       /* 0 or KEY_E0 */
} EVDEV_EXT_MAP;

static const EVDEV_EXT_MAP EvdevExtendedKeys[] = {
    {  96, 0x1C, KEY_E0 },  /* KEY_KPENTER */
    {  97, 0x1D, KEY_E0 },  /* KEY_RIGHTCTRL */
    { 100, 0x35, KEY_E0 },  /* KEY_KPSLASH */
    { 102, 0x47, KEY_E0 },  /* KEY_HOME */
    { 103, 0x48, KEY_E0 },  /* KEY_UP */
    { 104, 0x49, KEY_E0 },  /* KEY_PAGEUP */
    { 105, 0x4B, KEY_E0 },  /* KEY_LEFT */
    { 106, 0x4D, KEY_E0 },  /* KEY_RIGHT */
    { 107, 0x4F, KEY_E0 },  /* KEY_END */
    { 108, 0x50, KEY_E0 },  /* KEY_DOWN */
    { 109, 0x51, KEY_E0 },  /* KEY_PAGEDOWN */
    { 110, 0x52, KEY_E0 },  /* KEY_INSERT */
    { 111, 0x53, KEY_E0 },  /* KEY_DELETE */
    { 125, 0x5B, KEY_E0 },  /* KEY_LEFTMETA (Win) */
    { 126, 0x5C, KEY_E0 },  /* KEY_RIGHTMETA */
    { 127, 0x5D, KEY_E0 },  /* KEY_COMPOSE (Menu) */
};

#define EVDEV_EXT_COUNT (sizeof(EvdevExtendedKeys) / sizeof(EvdevExtendedKeys[0]))

static BOOLEAN
EvdevToScanCode(USHORT EvdevCode, USHORT *OutScanCode, USHORT *OutFlags)
{
    ULONG i;

    /* Direct mapping for codes 1-88 */
    if (EvdevCode >= 1 && EvdevCode <= 88)
    {
        *OutScanCode = EvdevCode;
        *OutFlags = 0;
        return TRUE;
    }

    /* Extended key lookup */
    for (i = 0; i < EVDEV_EXT_COUNT; i++)
    {
        if (EvdevExtendedKeys[i].EvdevCode == EvdevCode)
        {
            *OutScanCode = EvdevExtendedKeys[i].ScanCode;
            *OutFlags = EvdevExtendedKeys[i].Flags;
            return TRUE;
        }
    }

    return FALSE;
}

/* =========================================================================
 * Virtio queue setup
 * ========================================================================= */

static NTSTATUS
VirtioSetupQueue(VIRTIO_INPUT_DEV *Dev, ULONG QueueIdx)
{
    ULONG MaxSize, QueueSize, DescSize, AvailSize, UsedSize, EventsSize;
    ULONG AvailOff, UsedOff, EventsOff, TotalSize;
    PHYSICAL_ADDRESS HighAddr;
    ULONG i;

    /* Select queue */
    VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_SEL, QueueIdx);
    virtio_mb();

    MaxSize = VirtioRead32(Dev->Mmio, VIRTIO_MMIO_QUEUE_NUM_MAX);
    if (MaxSize == 0)
        return STATUS_DEVICE_NOT_READY;

    QueueSize = (MaxSize < VIRTIO_QUEUE_SIZE) ? MaxSize : VIRTIO_QUEUE_SIZE;
    Dev->QueueSize = QueueSize;

    /* Write queue size */
    VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_NUM, QueueSize);

    /* Calculate allocation sizes */
    DescSize = sizeof(VIRTQ_DESC) * QueueSize;
    AvailSize = sizeof(USHORT) * 2 + sizeof(USHORT) * QueueSize + sizeof(USHORT);
    UsedSize = sizeof(USHORT) * 2 + sizeof(VIRTQ_USED_ELEM) * QueueSize + sizeof(USHORT);
    EventsSize = sizeof(VIRTIO_INPUT_EVENT) * QueueSize;

    if (Dev->MmioVersion >= 2)
    {
        /* Modern: each component at independent page-aligned addresses */
        AvailOff = (DescSize + 0xFFF) & ~0xFFF;
        UsedOff = (AvailOff + AvailSize + 0xFFF) & ~0xFFF;
    }
    else
    {
        /* Legacy v1: spec-prescribed layout.
         * Avail ring immediately after descriptors (2-byte aligned).
         * Used ring at next page boundary after avail ring. */
        AvailOff = DescSize;
        UsedOff = (AvailOff + AvailSize + 0xFFF) & ~0xFFF;
    }
    EventsOff = (UsedOff + UsedSize + 0xFFF) & ~0xFFF;
    TotalSize = EventsOff + EventsSize;

    HighAddr.QuadPart = 0xFFFFFFFFLL;
    Dev->QueueVa = MmAllocateContiguousMemory(TotalSize, HighAddr);
    if (!Dev->QueueVa)
        return STATUS_INSUFFICIENT_RESOURCES;

    Dev->QueuePa = MmGetPhysicalAddress(Dev->QueueVa);
    Dev->QueueAllocSize = TotalSize;
    RtlZeroMemory(Dev->QueueVa, TotalSize);

    /* Set up pointers */
    Dev->Desc = (VIRTQ_DESC *)Dev->QueueVa;
    Dev->Avail = (VIRTQ_AVAIL *)((UCHAR *)Dev->QueueVa + AvailOff);
    Dev->Used = (VIRTQ_USED *)((UCHAR *)Dev->QueueVa + UsedOff);
    Dev->Events = (VIRTIO_INPUT_EVENT *)((UCHAR *)Dev->QueueVa + EventsOff);
    Dev->EventsPa.QuadPart = Dev->QueuePa.QuadPart + EventsOff;

    if (Dev->MmioVersion >= 2)
    {
        /* Modern: write physical addresses of each component */
        PHYSICAL_ADDRESS DescPa = Dev->QueuePa;
        PHYSICAL_ADDRESS AvailPa;
        PHYSICAL_ADDRESS UsedPa;
        AvailPa.QuadPart = Dev->QueuePa.QuadPart + AvailOff;
        UsedPa.QuadPart = Dev->QueuePa.QuadPart + UsedOff;

        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_DESC_LOW, (ULONG)DescPa.QuadPart);
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_DESC_HIGH, (ULONG)(DescPa.QuadPart >> 32));
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_AVAIL_LOW, (ULONG)AvailPa.QuadPart);
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_AVAIL_HIGH, (ULONG)(AvailPa.QuadPart >> 32));
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_USED_LOW, (ULONG)UsedPa.QuadPart);
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_USED_HIGH, (ULONG)(UsedPa.QuadPart >> 32));
        virtio_mb();
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_READY, 1);
    }
    else
    {
        /* Legacy v1: page-based addressing */
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_GUEST_PAGE_SIZE, 4096);
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_PFN,
                      (ULONG)(Dev->QueuePa.QuadPart / 4096));
    }

    /* Pre-fill all descriptors: each points to Events[i], device-writable */
    for (i = 0; i < QueueSize; i++)
    {
        Dev->Desc[i].Addr = Dev->EventsPa.QuadPart + i * sizeof(VIRTIO_INPUT_EVENT);
        Dev->Desc[i].Len = sizeof(VIRTIO_INPUT_EVENT);
        Dev->Desc[i].Flags = VIRTQ_DESC_F_WRITE;
        Dev->Desc[i].Next = 0;

        Dev->Avail->Ring[i] = (USHORT)i;
    }
    virtio_wmb();
    Dev->Avail->Idx = (USHORT)QueueSize;
    Dev->LastUsedIdx = 0;

    /* Kick the queue */
    virtio_mb();
    VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_NOTIFY, QueueIdx);

    return STATUS_SUCCESS;
}

/* =========================================================================
 * Virtio device discovery and initialization
 * ========================================================================= */

/* QEMU virt MMIO device base and stride */
#define VIRTIO_MMIO_BASE    0x0A000000ULL
#define VIRTIO_MMIO_STRIDE  0x200
#define VIRTIO_MMIO_SLOTS   32
#define VIRTIO_MMIO_SIZE    0x200

/* Virtio device ID for input */
#define VIRTIO_DEVICE_ID_INPUT  18

static VOID
VirtioQueryInputCaps(VIRTIO_INPUT_DEV *Dev)
{
    volatile UCHAR *cfg = Dev->Mmio + VIRTIO_MMIO_CONFIG;
    UCHAR bitmap[128];
    ULONG size, i;

    Dev->IsKeyboard = FALSE;
    Dev->IsTablet = FALSE;

    /* Query EV_KEY support: select=0x11, subsel=EV_KEY(1) */
    *(volatile UCHAR *)(cfg + 0) = VIRTIO_INPUT_CFG_EV_BITS; /* select */
    virtio_mb();
    *(volatile UCHAR *)(cfg + 1) = EV_KEY;                    /* subsel */
    virtio_mb();
    size = *(volatile UCHAR *)(cfg + 2);                       /* size */

    if (size > 0 && size <= sizeof(bitmap))
    {
        for (i = 0; i < size; i++)
            bitmap[i] = *(volatile UCHAR *)(cfg + 8 + i);

        /* Check for KEY_A (code 30 → byte 3, bit 6) */
        if (size > (KEY_A / 8) && (bitmap[KEY_A / 8] & (1 << (KEY_A % 8))))
            Dev->IsKeyboard = TRUE;
    }

    /* Query EV_ABS support: select=0x11, subsel=EV_ABS(3) */
    *(volatile UCHAR *)(cfg + 0) = VIRTIO_INPUT_CFG_EV_BITS;
    virtio_mb();
    *(volatile UCHAR *)(cfg + 1) = EV_ABS;
    virtio_mb();
    size = *(volatile UCHAR *)(cfg + 2);

    if (size > 0)
    {
        for (i = 0; i < size && i < sizeof(bitmap); i++)
            bitmap[i] = *(volatile UCHAR *)(cfg + 8 + i);

        /* Check for ABS_X (code 0 → byte 0, bit 0) */
        if (bitmap[0] & 1)
            Dev->IsTablet = TRUE;
    }

    /* If tablet, query ABS_X and ABS_Y axis ranges */
    if (Dev->IsTablet)
    {
        volatile ULONG *absinfo;

        /* ABS_X info: select=0x12, subsel=ABS_X(0) */
        *(volatile UCHAR *)(cfg + 0) = VIRTIO_INPUT_CFG_ABS_INFO;
        virtio_mb();
        *(volatile UCHAR *)(cfg + 1) = ABS_X;
        virtio_mb();
        size = *(volatile UCHAR *)(cfg + 2);
        if (size >= 12)
        {
            absinfo = (volatile ULONG *)(cfg + 8);
            /* absinfo layout: min(4), max(4), fuzz(4), flat(4) */
            Dev->AbsMaxX = absinfo[1]; /* max */
        }
        else
        {
            Dev->AbsMaxX = 32767;
        }

        /* ABS_Y info */
        *(volatile UCHAR *)(cfg + 0) = VIRTIO_INPUT_CFG_ABS_INFO;
        *(volatile UCHAR *)(cfg + 1) = ABS_Y;
        virtio_mb();
        size = *(volatile UCHAR *)(cfg + 2);
        if (size >= 12)
        {
            absinfo = (volatile ULONG *)(cfg + 8);
            Dev->AbsMaxY = absinfo[1];
        }
        else
        {
            Dev->AbsMaxY = 32767;
        }
    }

    /* Reset config select */
    *(volatile UCHAR *)(cfg + 0) = VIRTIO_INPUT_CFG_UNSET;
    *(volatile UCHAR *)(cfg + 1) = 0;
    virtio_mb();
}

static VOID
VirtioScanDevices(VOID)
{
    ULONG i;
    PHYSICAL_ADDRESS PhysAddr;
    volatile UCHAR *Mmio;
    ULONG Magic, Version, DevId;
    NTSTATUS Status;

    VirtioInputCount = 0;

    for (i = 0; i < VIRTIO_MMIO_SLOTS && VirtioInputCount < MAX_VIRTIO_INPUTS; i++)
    {
        PhysAddr.QuadPart = VIRTIO_MMIO_BASE + (ULONGLONG)i * VIRTIO_MMIO_STRIDE;
        Mmio = (volatile UCHAR *)MmMapIoSpace(PhysAddr, VIRTIO_MMIO_SIZE, MmNonCached);
        if (!Mmio)
            continue;

        Magic = VirtioRead32(Mmio, VIRTIO_MMIO_MAGIC);
        if (Magic != 0x74726976)
        {
            MmUnmapIoSpace((PVOID)Mmio, VIRTIO_MMIO_SIZE);
            continue;
        }

        Version = VirtioRead32(Mmio, VIRTIO_MMIO_VERSION);
        DevId = VirtioRead32(Mmio, VIRTIO_MMIO_DEVICE_ID);

        if (DevId != VIRTIO_DEVICE_ID_INPUT)
        {
            MmUnmapIoSpace((PVOID)Mmio, VIRTIO_MMIO_SIZE);
            continue;
        }

        DbgPrint("[VIRTIO-INPUT] Found input device at slot %lu (version %lu)\n", i, Version);

        /* Initialize device */
        VIRTIO_INPUT_DEV *Dev = &VirtioInputs[VirtioInputCount];
        RtlZeroMemory(Dev, sizeof(*Dev));
        Dev->Mmio = Mmio;
        Dev->MmioVersion = Version;

        /* Reset device */
        VirtioWrite32(Mmio, VIRTIO_MMIO_STATUS, 0);
        virtio_mb();

        /* ACK */
        VirtioWrite32(Mmio, VIRTIO_MMIO_STATUS, VIRTIO_STATUS_ACK);
        virtio_mb();

        /* DRIVER */
        VirtioWrite32(Mmio, VIRTIO_MMIO_STATUS,
                      VIRTIO_STATUS_ACK | VIRTIO_STATUS_DRIVER);
        virtio_mb();

        /* Negotiate features */
        VirtioWrite32(Mmio, VIRTIO_MMIO_DRV_FEATURES_SEL, 0);
        VirtioWrite32(Mmio, VIRTIO_MMIO_DRV_FEATURES, 0);
        if (Version >= 2)
        {
            /* Must acknowledge VIRTIO_F_VERSION_1 (bit 32 = word 1 bit 0) */
            VirtioWrite32(Mmio, VIRTIO_MMIO_DRV_FEATURES_SEL, 1);
            VirtioWrite32(Mmio, VIRTIO_MMIO_DRV_FEATURES, 1);
        }
        virtio_mb();

        /* FEATURES_OK (v2 only) */
        if (Version >= 2)
        {
            VirtioWrite32(Mmio, VIRTIO_MMIO_STATUS,
                          VIRTIO_STATUS_ACK | VIRTIO_STATUS_DRIVER |
                          VIRTIO_STATUS_FEATURES_OK);
            virtio_mb();

            ULONG StatusVal = VirtioRead32(Mmio, VIRTIO_MMIO_STATUS);
            if (!(StatusVal & VIRTIO_STATUS_FEATURES_OK))
            {
                DbgPrint("[VIRTIO-INPUT] FEATURES_OK not set, skipping slot %lu\n", i);
                MmUnmapIoSpace((PVOID)Mmio, VIRTIO_MMIO_SIZE);
                continue;
            }
        }

        /* Query device capabilities before setting up queues */
        VirtioQueryInputCaps(Dev);

        /* Setup eventq (queue 0) */
        Status = VirtioSetupQueue(Dev, 0);
        if (!NT_SUCCESS(Status))
        {
            DbgPrint("[VIRTIO-INPUT] Queue setup failed: 0x%08lx\n", Status);
            MmUnmapIoSpace((PVOID)Mmio, VIRTIO_MMIO_SIZE);
            continue;
        }

        /* DRIVER_OK */
        VirtioWrite32(Mmio, VIRTIO_MMIO_STATUS,
                      VIRTIO_STATUS_ACK | VIRTIO_STATUS_DRIVER |
                      (Version >= 2 ? VIRTIO_STATUS_FEATURES_OK : 0) |
                      VIRTIO_STATUS_DRIVER_OK);
        virtio_mb();

        DbgPrint("[VIRTIO-INPUT] Initialized %s%sat slot %lu (abs %lu x %lu)\n",
                Dev->IsKeyboard ? "keyboard " : "",
                Dev->IsTablet ? "tablet " : "",
                i, Dev->AbsMaxX, Dev->AbsMaxY);

        VirtioInputCount++;
    }

    DbgPrint("[VIRTIO-INPUT] Found %lu input device(s)\n", VirtioInputCount);
}

/* =========================================================================
 * Virtio event processing
 * ========================================================================= */

static VOID
VirtioProcessKeyEvent(USHORT EvdevCode, ULONG Value)
{
    PINPUT_DEVICE_EXT DevExt;
    USHORT ScanCode, Flags;

    if (!g_KbdDevice)
        return;

    if (!EvdevToScanCode(EvdevCode, &ScanCode, &Flags))
        return;

    DevExt = (PINPUT_DEVICE_EXT)g_KbdDevice->DeviceExtension;

    /* Value: 0=release, 1=press, 2=repeat */
    if (Value == 0)
        Flags |= KEY_BREAK;
    /* For press (1) and repeat (2), send KEY_MAKE (Flags unchanged) */

    RingPush(DevExt, ScanCode, Flags);
    TryCompleteIrp(DevExt);
}

static VOID
VirtioProcessMouseButton(USHORT EvdevCode, ULONG Value, VIRTIO_INPUT_DEV *Dev)
{
    switch (EvdevCode)
    {
    case BTN_LEFT:
        Dev->AccumButtons |= Value ?
            MOUSE_LEFT_BUTTON_DOWN : MOUSE_LEFT_BUTTON_UP;
        Dev->HasButton = TRUE;
        break;
    case BTN_RIGHT:
        Dev->AccumButtons |= Value ?
            MOUSE_RIGHT_BUTTON_DOWN : MOUSE_RIGHT_BUTTON_UP;
        Dev->HasButton = TRUE;
        break;
    case BTN_MIDDLE:
        Dev->AccumButtons |= Value ?
            MOUSE_MIDDLE_BUTTON_DOWN : MOUSE_MIDDLE_BUTTON_UP;
        Dev->HasButton = TRUE;
        break;
    }
}

static VOID
VirtioFlushMouseState(VIRTIO_INPUT_DEV *Dev)
{
    PINPUT_DEVICE_EXT DevExt;
    MOUSE_INPUT_DATA Data;

    if (!g_MouDevice)
        return;
    if (!Dev->HasMove && !Dev->HasButton)
        return;

    DevExt = (PINPUT_DEVICE_EXT)g_MouDevice->DeviceExtension;

    RtlZeroMemory(&Data, sizeof(Data));

    if (Dev->IsTablet && Dev->AbsValid)
    {
        /* Absolute positioning: scale to 0-65535 */
        Data.Flags = MOUSE_MOVE_ABSOLUTE;
        if (Dev->AbsMaxX > 0)
            Data.LastX = (LONG)((ULONGLONG)Dev->AccumX * 65535 / Dev->AbsMaxX);
        else
            Data.LastX = Dev->AccumX;
        if (Dev->AbsMaxY > 0)
            Data.LastY = (LONG)((ULONGLONG)Dev->AccumY * 65535 / Dev->AbsMaxY);
        else
            Data.LastY = Dev->AccumY;
    }
    else if (Dev->HasMove)
    {
        /* Relative movement */
        Data.Flags = 0;
        Data.LastX = Dev->AccumX;
        Data.LastY = Dev->AccumY;
    }

    Data.ButtonFlags = Dev->AccumButtons;

    MouRingPush(DevExt, &Data);
    MouTryCompleteIrp(DevExt);

    /* Reset accumulators — keep absolute position for next time */
    if (!Dev->IsTablet)
    {
        Dev->AccumX = 0;
        Dev->AccumY = 0;
    }
    Dev->AccumButtons = 0;
    Dev->HasMove = FALSE;
    Dev->HasButton = FALSE;
}

static BOOLEAN
VirtioPollDevice(VIRTIO_INPUT_DEV *Dev)
{
    BOOLEAN FoundEvents = FALSE;
    USHORT UsedIdx;
    ULONG DescId;
    VIRTIO_INPUT_EVENT *Evt;

    virtio_rmb();
    UsedIdx = Dev->Used->Idx;

    while (Dev->LastUsedIdx != UsedIdx)
    {
        FoundEvents = TRUE;
        DescId = Dev->Used->Ring[Dev->LastUsedIdx % Dev->QueueSize].Id;
        Evt = &Dev->Events[DescId];

        switch (Evt->Type)
        {
        case EV_KEY:
            if (Evt->Code < BTN_LEFT)
            {
                /* Keyboard key */
                VirtioProcessKeyEvent(Evt->Code, Evt->Value);
            }
            else
            {
                /* Mouse/tablet button */
                VirtioProcessMouseButton(Evt->Code, Evt->Value, Dev);
            }
            break;

        case EV_REL:
            switch (Evt->Code)
            {
            case REL_X:
                Dev->AccumX += (LONG)Evt->Value;
                Dev->HasMove = TRUE;
                break;
            case REL_Y:
                Dev->AccumY += (LONG)Evt->Value;
                Dev->HasMove = TRUE;
                break;
            case REL_WHEEL:
                /* TODO: scroll wheel support */
                break;
            }
            break;

        case EV_ABS:
            switch (Evt->Code)
            {
            case ABS_X:
                Dev->AccumX = (LONG)Evt->Value;
                Dev->HasMove = TRUE;
                Dev->AbsValid = TRUE;
                break;
            case ABS_Y:
                Dev->AccumY = (LONG)Evt->Value;
                Dev->HasMove = TRUE;
                Dev->AbsValid = TRUE;
                break;
            }
            break;

        case EV_SYN:
            VirtioFlushMouseState(Dev);
            break;
        }

        /* Re-queue the descriptor back to available ring */
        Dev->Desc[DescId].Len = sizeof(VIRTIO_INPUT_EVENT);
        Dev->Desc[DescId].Flags = VIRTQ_DESC_F_WRITE;
        virtio_wmb();
        Dev->Avail->Ring[Dev->Avail->Idx % Dev->QueueSize] = (USHORT)DescId;
        virtio_wmb();
        Dev->Avail->Idx++;

        Dev->LastUsedIdx++;
    }

    if (FoundEvents)
    {
        virtio_mb();
        VirtioWrite32(Dev->Mmio, VIRTIO_MMIO_QUEUE_NOTIFY, 0);
    }

    return FoundEvents;
}

/* =========================================================================
 * Virtio polling thread
 * ========================================================================= */

static VOID NTAPI
VirtioPollThread(PVOID Context)
{
    LARGE_INTEGER Delay;

    UNREFERENCED_PARAMETER(Context);

    Delay.QuadPart = -10000; /* 1ms relative */

    while (VirtioPollRunning)
    {
        BOOLEAN found = FALSE;
        ULONG i;

        for (i = 0; i < VirtioInputCount; i++)
        {
            if (VirtioPollDevice(&VirtioInputs[i]))
                found = TRUE;
        }

        /* Yield — KeDelayExecutionThread effectively yields on ARM64 */
        KeDelayExecutionThread(KernelMode, FALSE, &Delay);

        if (!found)
        {
            /* Extra yields when idle to reduce CPU waste */
            ULONG j;
            for (j = 0; j < 5; j++)
                KeDelayExecutionThread(KernelMode, FALSE, &Delay);
        }
    }

    PsTerminateSystemThread(STATUS_SUCCESS);
}

/* ========================================================================= */

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
    g_KbdDevice = KbdDevice;

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

    g_MouDevice = MouDevice;

    DbgPrint("[ARM64INPUT] \\Device\\PointerClass0 created\n");

    /* Scan for virtio-input MMIO devices */
    VirtioScanDevices();
    if (VirtioInputCount > 0)
    {
        VirtioPollRunning = TRUE;
        Status = PsCreateSystemThread(&VirtioPollThreadHandle,
                                      THREAD_ALL_ACCESS,
                                      NULL, NULL, NULL,
                                      VirtioPollThread, NULL);
        if (NT_SUCCESS(Status))
        {
            DbgPrint("[ARM64INPUT] Virtio poll thread started\n");
        }
        else
        {
            DbgPrint("[ARM64INPUT] Failed to create poll thread: 0x%08lx\n", Status);
            VirtioPollRunning = FALSE;
        }
    }

    return STATUS_SUCCESS;
}
