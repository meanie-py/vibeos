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

    /* Ensure UART is enabled for TX and RX */
    {
        volatile ULONG *cr = (volatile ULONG *)((PUCHAR)g_UartBase + UARTCR);
        ULONG crVal = *cr;
        DbgPrint("[PL011] UARTCR = 0x%08lx (UARTEN=%d TXE=%d RXE=%d)\n",
                 crVal, (crVal & 1), (crVal >> 8) & 1, (crVal >> 9) & 1);
        /* If RXE is not set, enable it */
        if (!(crVal & (1 << 9)))
        {
            crVal |= (1 << 9) | (1 << 0); /* RXE + UARTEN */
            *cr = crVal;
            DbgPrint("[PL011] Enabled RXE: UARTCR = 0x%08lx\n", *cr);
        }
    }

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

    /* Pure non-blocking read.  ARM64 has no timer interrupts and no
     * preemption (KiIrqHandler is a no-op stub; see ntoskrnl/ke/arm64/trap.c),
     * so any in-kernel busy-wait here pegs the CPU and starves user-mode
     * threads (explorer, win32k paint) that need to run for the desktop to
     * render.  KeStallExecutionProcessor in particular blocks ALL other
     * threads for the stall duration with no yield path.  Just return what
     * the FIFO has right now (often zero); the user-mode caller polls via
     * Sleep(50) which puts the thread back on the timer queue. */
    while (BytesRead < MaxLength)
    {
        if (!Pl011RxReady())
        {
            break; /* Return what we have (0 means caller should sleep+retry) */
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
