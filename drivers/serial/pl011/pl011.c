/*
 * PROJECT:     ReactOS PL011 UART Driver
 * FILE:        drivers/serial/pl011/pl011.c
 * PURPOSE:     Minimal PL011 UART driver for ARM64 QEMU virt
 * PROGRAMMERS: ReactOS ARM64 Port
 */

#include <ntddk.h>
#include <ntstrsafe.h>

/* PL011 register offsets */
#define UARTDR      0x000   /* Data Register */
#define UARTFR      0x018   /* Flag Register */
#define UARTCR      0x030   /* Control Register */

/* Flag register bits */
#define UARTFR_TXFF (1 << 5)  /* TX FIFO Full */
#define UARTFR_RXFE (1 << 4)  /* RX FIFO Empty */

/* Physical address of PL011 on QEMU virt */
#define PL011_PHYS_ADDR 0x09000000ULL
#define PL011_REG_SIZE  0x1000

static PVOID g_UartBase = NULL;

/* Forward declarations */
static NTSTATUS NTAPI Pl011Create(PDEVICE_OBJECT DeviceObject, PIRP Irp);
static NTSTATUS NTAPI Pl011Close(PDEVICE_OBJECT DeviceObject, PIRP Irp);
static NTSTATUS NTAPI Pl011Read(PDEVICE_OBJECT DeviceObject, PIRP Irp);
static NTSTATUS NTAPI Pl011Write(PDEVICE_OBJECT DeviceObject, PIRP Irp);

static VOID
Pl011WriteByte(UCHAR ch)
{
    volatile ULONG *dr = (volatile ULONG *)((PUCHAR)g_UartBase + UARTDR);
    volatile ULONG *fr = (volatile ULONG *)((PUCHAR)g_UartBase + UARTFR);

    /* Spin-wait until TX FIFO is not full */
    while (*fr & UARTFR_TXFF)
        ;
    *dr = (ULONG)ch;
}

static BOOLEAN
Pl011RxReady(VOID)
{
    volatile ULONG *fr = (volatile ULONG *)((PUCHAR)g_UartBase + UARTFR);
    return (*fr & UARTFR_RXFE) == 0;
}

static UCHAR
Pl011ReadByte(VOID)
{
    volatile ULONG *dr = (volatile ULONG *)((PUCHAR)g_UartBase + UARTDR);
    return (UCHAR)(*dr & 0xFF);
}

NTSTATUS
NTAPI
DriverEntry(
    PDRIVER_OBJECT DriverObject,
    PUNICODE_STRING RegistryPath)
{
    NTSTATUS Status;
    PDEVICE_OBJECT DeviceObject = NULL;
    UNICODE_STRING DeviceName, SymlinkName;
    PHYSICAL_ADDRESS PhysAddr;

    UNREFERENCED_PARAMETER(RegistryPath);

    DbgPrint("[PL011] DriverEntry\n");

    /* Map the PL011 registers */
    PhysAddr.QuadPart = PL011_PHYS_ADDR;
    g_UartBase = MmMapIoSpace(PhysAddr, PL011_REG_SIZE, MmNonCached);
    if (!g_UartBase)
    {
        DbgPrint("[PL011] MmMapIoSpace FAILED\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    DbgPrint("[PL011] UART mapped at %p\n", g_UartBase);

    /* Create device object */
    RtlInitUnicodeString(&DeviceName, L"\\Device\\ArmSerial");
    Status = IoCreateDevice(DriverObject, 0, &DeviceName,
                            FILE_DEVICE_SERIAL_PORT, 0, FALSE,
                            &DeviceObject);
    if (!NT_SUCCESS(Status))
    {
        DbgPrint("[PL011] IoCreateDevice failed: 0x%lx\n", Status);
        MmUnmapIoSpace(g_UartBase, PL011_REG_SIZE);
        g_UartBase = NULL;
        return Status;
    }

    DeviceObject->Flags |= DO_BUFFERED_IO;
    DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;

    /* Create symbolic link \DosDevices\COM1 */
    RtlInitUnicodeString(&SymlinkName, L"\\DosDevices\\COM1");
    Status = IoCreateSymbolicLink(&SymlinkName, &DeviceName);
    if (!NT_SUCCESS(Status))
    {
        DbgPrint("[PL011] IoCreateSymbolicLink failed: 0x%lx\n", Status);
        IoDeleteDevice(DeviceObject);
        MmUnmapIoSpace(g_UartBase, PL011_REG_SIZE);
        g_UartBase = NULL;
        return Status;
    }

    /* Set up dispatch routines */
    DriverObject->MajorFunction[IRP_MJ_CREATE] = Pl011Create;
    DriverObject->MajorFunction[IRP_MJ_CLOSE]  = Pl011Close;
    DriverObject->MajorFunction[IRP_MJ_READ]   = Pl011Read;
    DriverObject->MajorFunction[IRP_MJ_WRITE]  = Pl011Write;

    DbgPrint("[PL011] Driver loaded OK\n");
    return STATUS_SUCCESS;
}

static NTSTATUS NTAPI
Pl011Create(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
    UNREFERENCED_PARAMETER(DeviceObject);
    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

static NTSTATUS NTAPI
Pl011Close(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
    UNREFERENCED_PARAMETER(DeviceObject);
    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

static NTSTATUS NTAPI
Pl011Write(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
    PIO_STACK_LOCATION IrpSp;
    PUCHAR Buffer;
    ULONG Length, i;

    UNREFERENCED_PARAMETER(DeviceObject);

    IrpSp = IoGetCurrentIrpStackLocation(Irp);
    Buffer = (PUCHAR)Irp->AssociatedIrp.SystemBuffer;
    Length = IrpSp->Parameters.Write.Length;

    for (i = 0; i < Length; i++)
    {
        if (Buffer[i] == '\n')
            Pl011WriteByte('\r');
        Pl011WriteByte(Buffer[i]);
    }

    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = Length;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

static NTSTATUS NTAPI
Pl011Read(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
    PIO_STACK_LOCATION IrpSp;
    PUCHAR Buffer;
    ULONG MaxLength, BytesRead = 0;
    LARGE_INTEGER Delay;
    ULONG Retries;
    UCHAR ch;

    UNREFERENCED_PARAMETER(DeviceObject);

    IrpSp = IoGetCurrentIrpStackLocation(Irp);
    Buffer = (PUCHAR)Irp->AssociatedIrp.SystemBuffer;
    MaxLength = IrpSp->Parameters.Read.Length;

    if (MaxLength == 0)
    {
        Irp->IoStatus.Status = STATUS_SUCCESS;
        Irp->IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    /* Poll for input. Wait up to ~5 seconds if no data yet. */
    Delay.QuadPart = -(10000LL * 10); /* 10 ms */
    Retries = 500;

    while (BytesRead < MaxLength)
    {
        if (!Pl011RxReady())
        {
            if (BytesRead > 0)
                break; /* Return what we have */

            if (Retries-- == 0)
                break; /* Timeout with no data */

            KeDelayExecutionThread(KernelMode, FALSE, &Delay);
            continue;
        }

        ch = Pl011ReadByte();

        /* Handle backspace */
        if (ch == 0x08 || ch == 0x7F)
        {
            if (BytesRead > 0)
            {
                BytesRead--;
                /* Echo BS+Space+BS to erase character */
                Pl011WriteByte(0x08);
                Pl011WriteByte(' ');
                Pl011WriteByte(0x08);
            }
            continue;
        }

        /* Echo the character */
        if (ch == '\r')
        {
            Pl011WriteByte('\r');
            Pl011WriteByte('\n');
            Buffer[BytesRead++] = '\r';
            break; /* Line complete */
        }
        else
        {
            Pl011WriteByte(ch);
            Buffer[BytesRead++] = ch;
        }
    }

    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = BytesRead;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}
