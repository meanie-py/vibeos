/*
 * PROJECT:     Freeldr ARM64
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     Arch specific debug
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 port
 */

#include <freeldr.h>
#include <debug.h>

/* PL011 UART base address (QEMU virt machine) */
#define PL011_UART_BASE 0x09000000
static volatile unsigned int * const UART0DR = (unsigned int *)PL011_UART_BASE;

BOOLEAN
Rs232PortInitialize(IN ULONG ComPort,
                    IN ULONG BaudRate)
{
    return TRUE;
}

VOID
Rs232PortPutByte(UCHAR ByteToSend)
{
    *UART0DR = ByteToSend;
}

DECLSPEC_NORETURN
VOID
FrLdrBugCheckWithMessage(
    ULONG BugCode,
    PCHAR File,
    ULONG Line,
    PSTR Format,
    ...)
{
    for (;;);
}

/* Stubs needed by the bootloader but not available from RTL */
VOID
DbgBreakPoint(VOID)
{
    __asm__ __volatile__("brk #0xF000");
}

KIRQL
KeGetCurrentIrql(VOID)
{
    return PASSIVE_LEVEL;
}

VOID
RtlFillMemoryUlong(
    PVOID Destination,
    SIZE_T Length,
    ULONG Pattern)
{
    PULONG d = (PULONG)Destination;
    SIZE_T Count = Length / sizeof(ULONG);
    while (Count--)
        *d++ = Pattern;
}
