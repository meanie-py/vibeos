use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;

// READ_PORT_UCHAR, WRITE_PORT_UCHAR → crate::glue inline fns (FORCEINLINE in wdm.h)
#[no_mangle]
pub unsafe extern "C" fn SerialDetectUartType(BaseAddress: PUCHAR) -> UART_TYPE {
    let mut Lcr: UCHAR;
    let mut TestLcr: UCHAR;
    let mut OldScr: UCHAR;
    let mut Scr5A: UCHAR;
    let mut ScrA5: UCHAR;
    let mut FifoEnabled: BOOLEAN;
    let mut NewFifoStatus: UCHAR;

    Lcr = READ_PORT_UCHAR(SER_LCR(BaseAddress));
    WRITE_PORT_UCHAR(SER_LCR(BaseAddress), (Lcr as i32 ^ 0xFF) as UCHAR);
    TestLcr = (READ_PORT_UCHAR(SER_LCR(BaseAddress)) as i32 ^ 0xFF) as UCHAR;
    WRITE_PORT_UCHAR(SER_LCR(BaseAddress), Lcr);

    /* Accessing the LCR must work for a usable serial port */
    if TestLcr as i32 != Lcr as i32 {
        return UartUnknown;
    }

    /* Ensure that all following accesses are done as required */
    READ_PORT_UCHAR(SER_RBR(BaseAddress));
    READ_PORT_UCHAR(SER_IER(BaseAddress));
    READ_PORT_UCHAR(SER_IIR(BaseAddress));
    READ_PORT_UCHAR(SER_LCR(BaseAddress));
    READ_PORT_UCHAR(SER_MCR(BaseAddress));
    READ_PORT_UCHAR(SER_LSR(BaseAddress));
    READ_PORT_UCHAR(SER_MSR(BaseAddress));
    READ_PORT_UCHAR(SER_SCR(BaseAddress));

    /* Test scratch pad */
    OldScr = READ_PORT_UCHAR(SER_SCR(BaseAddress));
    WRITE_PORT_UCHAR(SER_SCR(BaseAddress), 0x5A as UCHAR);
    Scr5A = READ_PORT_UCHAR(SER_SCR(BaseAddress));
    WRITE_PORT_UCHAR(SER_SCR(BaseAddress), 0xA5 as UCHAR);
    ScrA5 = READ_PORT_UCHAR(SER_SCR(BaseAddress));
    WRITE_PORT_UCHAR(SER_SCR(BaseAddress), OldScr);

    /* When non-functional, we have a 8250 */
    if Scr5A as i32 != 0x5A || ScrA5 as i32 != 0xA5 {
        return Uart8250;
    }

    /* Test FIFO type */
    FifoEnabled = ((READ_PORT_UCHAR(SER_IIR(BaseAddress)) as i32 & 0x80) != 0) as BOOLEAN;
    WRITE_PORT_UCHAR(SER_FCR(BaseAddress), SR_FCR_ENABLE_FIFO as UCHAR);
    NewFifoStatus = (READ_PORT_UCHAR(SER_IIR(BaseAddress)) as i32 & 0xC0) as UCHAR;
    if FifoEnabled == 0 {
        WRITE_PORT_UCHAR(SER_FCR(BaseAddress), 0 as UCHAR);
    }
    match NewFifoStatus as i32 {
        0x00 => return Uart16450,
        0x40 | 0x80 => {
            /* Not sure about this but the documentation says that 0x40
             * indicates an unusable FIFO but my tests only worked
             * with 0x80 */
            return Uart16550;
        }
        _ => {}
    }

    /* FIFO is only functional for 16550A+ */
    return Uart16550A;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/bus/serial/legacy.c
//  * PURPOSE:         Legacy serial port enumeration
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  *                  Mark Junker (mjscod@gmx.de)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// UART_TYPE
// SerialDetectUartType(
// 	IN PUCHAR BaseAddress)
// {
// 	UCHAR Lcr, TestLcr;
// 	UCHAR OldScr, Scr5A, ScrA5;
// 	BOOLEAN FifoEnabled;
// 	UCHAR NewFifoStatus;
//
// 	Lcr = READ_PORT_UCHAR(SER_LCR(BaseAddress));
// 	WRITE_PORT_UCHAR(SER_LCR(BaseAddress), Lcr ^ 0xFF);
// 	TestLcr = READ_PORT_UCHAR(SER_LCR(BaseAddress)) ^ 0xFF;
// 	WRITE_PORT_UCHAR(SER_LCR(BaseAddress), Lcr);
//
// 	/* Accessing the LCR must work for a usable serial port */
// 	if (TestLcr != Lcr)
// 		return UartUnknown;
//
// 	/* Ensure that all following accesses are done as required */
// 	READ_PORT_UCHAR(SER_RBR(BaseAddress));
// 	READ_PORT_UCHAR(SER_IER(BaseAddress));
// 	READ_PORT_UCHAR(SER_IIR(BaseAddress));
// 	READ_PORT_UCHAR(SER_LCR(BaseAddress));
// 	READ_PORT_UCHAR(SER_MCR(BaseAddress));
// 	READ_PORT_UCHAR(SER_LSR(BaseAddress));
// 	READ_PORT_UCHAR(SER_MSR(BaseAddress));
// 	READ_PORT_UCHAR(SER_SCR(BaseAddress));
//
// 	/* Test scratch pad */
// 	OldScr = READ_PORT_UCHAR(SER_SCR(BaseAddress));
// 	WRITE_PORT_UCHAR(SER_SCR(BaseAddress), 0x5A);
// 	Scr5A = READ_PORT_UCHAR(SER_SCR(BaseAddress));
// 	WRITE_PORT_UCHAR(SER_SCR(BaseAddress), 0xA5);
// 	ScrA5 = READ_PORT_UCHAR(SER_SCR(BaseAddress));
// 	WRITE_PORT_UCHAR(SER_SCR(BaseAddress), OldScr);
//
// 	/* When non-functional, we have a 8250 */
// 	if (Scr5A != 0x5A || ScrA5 != 0xA5)
// 		return Uart8250;
//
// 	/* Test FIFO type */
// 	FifoEnabled = (READ_PORT_UCHAR(SER_IIR(BaseAddress)) & 0x80) != 0;
// 	WRITE_PORT_UCHAR(SER_FCR(BaseAddress), SR_FCR_ENABLE_FIFO);
// 	NewFifoStatus = READ_PORT_UCHAR(SER_IIR(BaseAddress)) & 0xC0;
// 	if (!FifoEnabled)
// 		WRITE_PORT_UCHAR(SER_FCR(BaseAddress), 0);
// 	switch (NewFifoStatus)
// 	{
// 		case 0x00:
// 			return Uart16450;
// 		case 0x40:
// 		case 0x80:
// 			/* Not sure about this but the documentation says that 0x40
// 			 * indicates an unusable FIFO but my tests only worked
// 			 * with 0x80 */
// 			return Uart16550;
// 	}
//
// 	/* FIFO is only functional for 16550A+ */
// 	return Uart16550A;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
