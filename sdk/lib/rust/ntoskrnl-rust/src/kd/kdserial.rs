use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CPPORT, PCSTR};

extern "system" {
    fn KdPortGetByteEx(PortInformation: *mut CPPORT, Byte: PUCHAR) -> BOOLEAN;
    fn KdPortPutByteEx(PortInformation: *mut CPPORT, ByteToSend: u8);
    static mut SerialPortInfo: CPPORT;
}

#[no_mangle]
pub unsafe extern "system" fn KdbpSendCommandSerial(
    Command: PCSTR)
{
    let mut cmd_ptr = Command;
    while *cmd_ptr != 0 {
        KdPortPutByteEx(core::ptr::addr_of_mut!(SerialPortInfo), *cmd_ptr as u8);
        cmd_ptr = cmd_ptr.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdbpTryGetCharSerial(Retry: ULONG) -> CHAR {
    let mut Result: CHAR = -1;

    if Retry == 0 {
        while KdPortGetByteEx(core::ptr::addr_of_mut!(SerialPortInfo), &mut Result as *mut CHAR as PUCHAR) == 0 {}
    } else {
        let mut retry_count = Retry;
        while KdPortGetByteEx(core::ptr::addr_of_mut!(SerialPortInfo), &mut Result as *mut CHAR as PUCHAR) == 0 && { let temp = retry_count; retry_count = retry_count.wrapping_sub(1); temp } > 0 {}
    }

    Result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS KDBG Kernel Debugger Terminal Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Serial driver
//  * COPYRIGHT:   Copyright 2004 Art Yerkes <ayerkes@speakeasy.net>
//  *              Copyright 2005 Gregor Anich <blight@blight.eu.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "kd.h"
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// KdbpSendCommandSerial(
//     _In_ PCSTR Command)
// {
//     while (*Command)
//         KdPortPutByteEx(&SerialPortInfo, *Command++);
// }
//
// CHAR
// KdbpTryGetCharSerial(
//     _In_ ULONG Retry)
// {
//     CHAR Result = -1;
//
//     if (Retry == 0)
//         while (!KdPortGetByteEx(&SerialPortInfo, (PUCHAR)&Result));
//     else
//         while (!KdPortGetByteEx(&SerialPortInfo, (PUCHAR)&Result) && Retry-- > 0);
//
//     return Result;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
