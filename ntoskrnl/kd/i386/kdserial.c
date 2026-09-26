/*
 * COPYRIGHT:       See COPYING in the top level directory
 * PROJECT:         ReactOS kernel
 * FILE:            ntoskrnl/kd/i386/kdbg.c
 * PURPOSE:         Serial i/o functions for the kernel debugger.
 * PROGRAMMER:      Alex Ionescu
 *                  Hervé Poussineau
 */

/* INCLUDES *****************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

#define DEFAULT_BAUD_RATE   19200

#if defined(_M_IX86) || defined(_M_AMD64)
const ULONG BaseArray[] = {0, 0x3F8, 0x2F8, 0x3E8, 0x2E8};
#elif defined(_M_PPC)
const ULONG BaseArray[] = {0, 0x800003F8};
#elif defined(_M_MIPS)
const ULONG BaseArray[] = {0, 0x80006000, 0x80007000};
#elif defined(_M_ARM)
const ULONG BaseArray[] = {0, 0xF1012000};
#elif defined(_M_ARM64)
const ULONG BaseArray[] = {0, 0x09000000};  /* PL011 UART base on QEMU virt */
#else
#error Unknown architecture
#endif

#define MAX_COM_PORTS   (sizeof(BaseArray) / sizeof(BaseArray[0]) - 1)

/* STATIC VARIABLES ***********************************************************/

// static CPPORT DefaultPort = {0, 0, 0};

/* The COM port must only be initialized once! */
// static BOOLEAN PortInitialized = FALSE;

/* REACTOS FUNCTIONS **********************************************************/

BOOLEAN
NTAPI
KdPortInitializeEx(
    IN PCPPORT PortInformation,
    IN ULONG ComPortNumber)
{
    NTSTATUS Status;

#if 0 // Deactivated because never used in fact (was in KdPortInitialize but we use KdPortInitializeEx)
    /*
     * Find the port if needed
     */

    if (!PortInitialized)
    {
        DefaultPort.BaudRate = PortInformation->BaudRate;

        if (ComPortNumber == 0)
        {
            /*
             * Enumerate COM ports from the last to the first one, and stop
             * when we find a valid port. If we reach the first list element
             * (the undefined COM port), no valid port was found.
             */
            for (ComPortNumber = MAX_COM_PORTS; ComPortNumber > 0; ComPortNumber--)
            {
                if (CpDoesPortExist(UlongToPtr(BaseArray[ComPortNumber])))
                {
                    PortInformation->Address = DefaultPort.Address = BaseArray[ComPortNumber];
                    break;
                }
            }
            if (ComPortNumber == 0)
            {
                HalDisplayString("\r\nKernel Debugger: No serial port found\r\n\r\n");
                return FALSE;
            }
        }

        PortInitialized = TRUE;
    }
#endif

    /*
     * Initialize the port
     */
    Status = CpInitialize(PortInformation,
                          (ComPortNumber == 0 ? PortInformation->Address
                                              : UlongToPtr(BaseArray[ComPortNumber])),
                          (PortInformation->BaudRate == 0 ? DEFAULT_DEBUG_BAUD_RATE
                                                          : PortInformation->BaudRate));
    if (!NT_SUCCESS(Status))
    {
        HalDisplayString("\r\nKernel Debugger: Serial port not available\r\n\r\n");
        return FALSE;
    }
    else
    {
#ifndef NDEBUG
        int Length;
        CHAR Buffer[82];

        /* Print message to blue screen */
        Length = snprintf(Buffer, sizeof(Buffer),
                          "\r\nKernel Debugger: Using COM%lu (Port 0x%p) BaudRate %lu\r\n\r\n",
                          ComPortNumber,
                          PortInformation->Address,
                          PortInformation->BaudRate);
        if (Length == -1)
        {
            /* Terminate it if we went over-board */
            Buffer[sizeof(Buffer) - 1] = ANSI_NULL;
        }

        HalDisplayString(Buffer);
#endif /* NDEBUG */

#if 0
        /* Set global info */
        KdComPortInUse = DefaultPort.Address;
#endif
        return TRUE;
    }
}

BOOLEAN
NTAPI
KdPortGetByteEx(
    IN PCPPORT PortInformation,
    OUT PUCHAR ByteReceived)
{
    return (CpGetByte(PortInformation, ByteReceived, FALSE, FALSE) == CP_GET_SUCCESS);
}

VOID
NTAPI
KdPortPutByteEx(
    IN PCPPORT PortInformation,
    IN UCHAR ByteToSend)
{
    CpPutByte(PortInformation, ByteToSend);
}

/* EOF */
