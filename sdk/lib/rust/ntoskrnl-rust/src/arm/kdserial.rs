use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::PCPPORT;

use crate::ke::arm64::stubs::{READ_REGISTER_ULONG, WRITE_REGISTER_ULONG};

// PL011 UART register addresses (from sdk/include/reactos/arm/peripherals/pl011.h).
const UART_BASE: ULONG_PTR = 0x101F1000;
const UART_PL01x_DR: ULONG_PTR = UART_BASE + 0x00;
const UART_PL01x_FR: ULONG_PTR = UART_BASE + 0x18;
const UART_PL011_IBRD: ULONG_PTR = UART_BASE + 0x24;
const UART_PL011_FBRD: ULONG_PTR = UART_BASE + 0x28;
const UART_PL011_LCRH: ULONG_PTR = UART_BASE + 0x2C;
const UART_PL011_CR: ULONG_PTR = UART_BASE + 0x30;

// LCR values
const UART_PL011_LCRH_WLEN_8: ULONG = 0x60;
const UART_PL011_LCRH_FEN: ULONG = 0x10;

// CR values
const UART_PL011_CR_UARTEN: ULONG = 0x01;
const UART_PL011_CR_TXE: ULONG = 0x100;
const UART_PL011_CR_RXE: ULONG = 0x200;

// FR values
const UART_PL01x_FR_TXFF: ULONG = 0x20;

// We need to build this in the configuration root and use KeFindConfigurationEntry
// to recover it later.
const HACK: ULONG = 24000000;

#[no_mangle]
pub unsafe extern "system" fn KdPortInitializeEx(
    PortInformation: PCPPORT,
    _ComPortNumber: ULONG,
) -> BOOLEAN {
    let Divider: ULONG;
    let Remainder: ULONG;
    let mut Fraction: ULONG;
    let Baudrate: ULONG = (*PortInformation).BaudRate;

    Divider = HACK / (16 * Baudrate);
    Remainder = HACK % (16 * Baudrate);

    Fraction = (8 * Remainder / Baudrate) >> 1;
    Fraction += (8 * Remainder / Baudrate) & 1;

    WRITE_REGISTER_ULONG(UART_PL011_CR as *mut ULONG, 0);

    WRITE_REGISTER_ULONG(UART_PL011_IBRD as *mut ULONG, Divider);
    WRITE_REGISTER_ULONG(UART_PL011_FBRD as *mut ULONG, Fraction);

    WRITE_REGISTER_ULONG(
        UART_PL011_LCRH as *mut ULONG,
        UART_PL011_LCRH_WLEN_8 | UART_PL011_LCRH_FEN,
    );

    WRITE_REGISTER_ULONG(
        UART_PL011_CR as *mut ULONG,
        UART_PL011_CR_UARTEN | UART_PL011_CR_TXE | UART_PL011_CR_RXE,
    );

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn KdPortGetByteEx(
    _PortInformation: PCPPORT,
    _ByteReceived: *mut u8,
) -> BOOLEAN {
    // UNIMPLEMENTED in C source: while (TRUE);
    loop {}
}

#[no_mangle]
pub unsafe extern "system" fn KdPortPutByteEx(_PortInformation: PCPPORT, ByteToSend: UCHAR) {
    while (READ_REGISTER_ULONG(UART_PL01x_FR as *mut ULONG) & UART_PL01x_FR_TXFF) != 0 {}

    WRITE_REGISTER_ULONG(UART_PL01x_DR as *mut ULONG, ByteToSend as ULONG);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/kd/arm/kdbg.c
//  * PURPOSE:         Serial Port Kernel Debugging Transport Library
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #include <arm/peripherals/pl011.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// CPPORT DefaultPort = {0, 0, 0};
//
// //
// // We need to build this in the configuration root and use KeFindConfigurationEntry
// // to recover it later.
// //
// #define HACK 24000000
//
// /* REACTOS FUNCTIONS **********************************************************/
//
// BOOLEAN
// NTAPI
// KdPortInitializeEx(IN PCPPORT PortInformation,
//                    IN ULONG ComPortNumber)
// {
//     ULONG Divider, Remainder, Fraction;
//     ULONG Baudrate = PortInformation->BaudRate;
//
//     //
//     // Calculate baudrate clock divider and remainder
//     //
//     Divider   = HACK / (16 * Baudrate);
//     Remainder = HACK % (16 * Baudrate);
//
//     //
//     // Calculate the fractional part
//     //
//     Fraction  = (8 * Remainder / Baudrate) >> 1;
//     Fraction += (8 * Remainder / Baudrate) & 1;
//
//     //
//     // Disable interrupts
//     //
//     WRITE_REGISTER_ULONG((PULONG)UART_PL011_CR, 0);
//
//     //
//     // Set the baud rate
//     //
//     WRITE_REGISTER_ULONG((PULONG)UART_PL011_IBRD, Divider);
//     WRITE_REGISTER_ULONG((PULONG)UART_PL011_FBRD, Fraction);
//
//     //
//     // Set 8 bits for data, 1 stop bit, no parity, FIFO enabled
//     //
//     WRITE_REGISTER_ULONG((PULONG)UART_PL011_LCRH,
//                          UART_PL011_LCRH_WLEN_8 | UART_PL011_LCRH_FEN);
//
//     //
//     // Clear and enable FIFO
//     //
//     WRITE_REGISTER_ULONG((PULONG)UART_PL011_CR,
//                          UART_PL011_CR_UARTEN |
//                          UART_PL011_CR_TXE |
//                          UART_PL011_CR_RXE);
//
//     //
//     // Done
//     //
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// KdPortGetByteEx(IN PCPPORT PortInformation,
//                 OUT PUCHAR ByteReceived)
// {
//     UNIMPLEMENTED;
//     while (TRUE);
//     return FALSE;
// }
//
// VOID
// NTAPI
// KdPortPutByteEx(IN PCPPORT PortInformation,
//                 IN UCHAR ByteToSend)
// {
//     //
//     // Wait for ready
//     //
//     while ((READ_REGISTER_ULONG((PULONG)UART_PL01x_FR) & UART_PL01x_FR_TXFF) != 0);
//
//     //
//     // Send the character
//     //
//     WRITE_REGISTER_ULONG((PULONG)UART_PL01x_DR, ByteToSend);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
