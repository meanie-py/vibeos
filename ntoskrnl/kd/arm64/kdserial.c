/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/kd/arm64/kdserial.c
 * PURPOSE:         PL011 UART Serial Port Functions for Kernel Debugger (ARM64)
 */

/* INCLUDES *******************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* PL011 register offsets */
#define PL011_DR    0x000   /* Data Register */
#define PL011_FR    0x018   /* Flag Register */
#define PL011_IBRD  0x024   /* Integer Baud Rate Divisor */
#define PL011_FBRD  0x028   /* Fractional Baud Rate Divisor */
#define PL011_LCRH  0x02C   /* Line Control Register */
#define PL011_CR    0x030   /* Control Register */

/* Flag Register bits */
#define PL011_FR_TXFF   0x20    /* Transmit FIFO full */
#define PL011_FR_RXFE   0x10    /* Receive FIFO empty */

/* QEMU virt PL011 base address — use kernel VA from global */
extern volatile PVOID Arm64UartKernelBase;

/* GLOBALS ********************************************************************/

/* Initialized lazily from Arm64UartKernelBase on first use */
static volatile ULONG *KdPl011Base;

static inline volatile ULONG *KdGetPl011Base(void)
{
    if (!KdPl011Base)
        KdPl011Base = (volatile ULONG *)Arm64UartKernelBase;
    return KdPl011Base;
}

/* FUNCTIONS ******************************************************************/

BOOLEAN
NTAPI
KdPortInitializeEx(
    IN PCPPORT PortInformation,
    IN ULONG ComPortNumber)
{
    /*
     * The PL011 UART is already initialized by UEFI firmware / FreeLDR.
     * Just record the base address so KdPortPutByteEx/KdPortGetByteEx
     * can use the CPPORT structure if needed.
     */
    if (PortInformation == NULL)
        return FALSE;

    KdPl011Base = (volatile ULONG *)Arm64UartKernelBase;
    PortInformation->Address = (PUCHAR)Arm64UartKernelBase;
    if (PortInformation->BaudRate == 0)
        PortInformation->BaudRate = 115200;

    return TRUE;
}

BOOLEAN
NTAPI
KdPortGetByteEx(
    IN PCPPORT PortInformation,
    OUT PUCHAR ByteReceived)
{
    volatile ULONG *base = KdGetPl011Base();
    if (!base)
        return FALSE;

    /* Check if receive FIFO is empty */
    if (base[PL011_FR / 4] & PL011_FR_RXFE)
        return FALSE;

    *ByteReceived = (UCHAR)(base[PL011_DR / 4] & 0xFF);
    return TRUE;
}

VOID
NTAPI
KdPortPutByteEx(
    IN PCPPORT PortInformation,
    IN UCHAR ByteToSend)
{
    volatile ULONG *base = KdGetPl011Base();
    if (!base)
        return;

    /* Wait until transmit FIFO is not full */
    while (base[PL011_FR / 4] & PL011_FR_TXFF)
        ;

    base[PL011_DR / 4] = ByteToSend;
}

/* EOF */
