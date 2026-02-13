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

/* QEMU virt PL011 base address */
#define PL011_BASE  0x09000000ULL

/* GLOBALS ********************************************************************/

static volatile ULONG *KdPl011Base = (volatile ULONG *)PL011_BASE;

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

    PortInformation->Address = (PUCHAR)PL011_BASE;
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
    /* Check if receive FIFO is empty */
    if (KdPl011Base[PL011_FR / 4] & PL011_FR_RXFE)
        return FALSE;

    *ByteReceived = (UCHAR)(KdPl011Base[PL011_DR / 4] & 0xFF);
    return TRUE;
}

VOID
NTAPI
KdPortPutByteEx(
    IN PCPPORT PortInformation,
    IN UCHAR ByteToSend)
{
    /* Wait until transmit FIFO is not full */
    while (KdPl011Base[PL011_FR / 4] & PL011_FR_TXFF)
        ;

    KdPl011Base[PL011_DR / 4] = ByteToSend;
}

/* EOF */
