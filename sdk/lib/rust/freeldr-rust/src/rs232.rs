use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CPPORT};
use core::ffi::c_void;

type PCPPORT = *mut CPPORT;
const CP_GET_SUCCESS: u16 = 0;

extern "system" {
    fn CpDoesPortExist(Address: PUCHAR) -> BOOLEAN;
    fn CpGetByte(P: PCPPORT, B: *mut u8, Wait: u8, Poll: u8) -> u16;
    fn CpPutByte(P: PCPPORT, B: u8);
    fn CpInitialize(PortInformation: PCPPORT, Address: PUCHAR, BaudRate: ULONG) -> NTSTATUS;
}
unsafe extern "C" {
    fn UlongToPtr(ul: ULONG) -> PUCHAR;
}

// rs232.c file-statics (rustconv dropped; i386 COM bases).
const DEFAULT_BAUD_RATE: ULONG = 19200;
static BaseArray: [ULONG; 5] = [0, 0x3F8, 0x2F8, 0x3E8, 0x2E8];
const MAX_COM_PORTS: ULONG = 4;
static mut Rs232ComPort: ULONG = 0;
static mut Rs232ComPortInfo: CPPORT = CPPORT {
    Address: core::ptr::null_mut(), BaudRate: 0, Flags: 0,
};

#[no_mangle]
pub unsafe extern "C" fn Rs232PortInitialize(
    mut ComPort: ULONG,
    mut BaudRate: ULONG,
) -> BOOLEAN {
    let Status: NTSTATUS;
    let mut Address: PUCHAR = core::ptr::null_mut();

    if (Rs232ComPort != 0) && (Rs232ComPort == ComPort) {
        return 1u8;
    }

    if BaudRate == 0 {
        BaudRate = DEFAULT_BAUD_RATE;
    }

    if ComPort == 0 {
        ComPort = MAX_COM_PORTS;
        while ComPort > 0 {
            if CpDoesPortExist(UlongToPtr(BaseArray[ComPort as usize])) != 0 {
                Address = UlongToPtr(BaseArray[ComPort as usize]);
                break;
            }
            ComPort -= 1;
        }
        if ComPort == 0 {
            return 0u8;
        }
    } else if ComPort <= MAX_COM_PORTS {
        if CpDoesPortExist(UlongToPtr(BaseArray[ComPort as usize])) != 0 {
            Address = UlongToPtr(BaseArray[ComPort as usize]);
        } else {
            return 0u8;
        }
    } else if ComPort <= 0xfff8 {
        // Not a COM number but an I/O-port base (DEBUGPORT=COM:0x5000, or the
        // UART an ACPI SPCR describes): no legacy board has one there to guess.
        if CpDoesPortExist(UlongToPtr(ComPort)) != 0 {
            Address = UlongToPtr(ComPort);
        } else {
            return 0u8;
        }
    } else {
        return 0u8;
    }

    Status = CpInitialize(
        core::ptr::addr_of_mut!(Rs232ComPortInfo),
        Address,
        BaudRate,
    );
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Rs232ComPort = ComPort;

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn Rs232PortGetByte(ByteReceived: PUCHAR) -> BOOLEAN {
    if Rs232ComPort == 0 {
        return 0u8;
    }
    (CpGetByte(
        core::ptr::addr_of_mut!(Rs232ComPortInfo),
        ByteReceived,
        1u8,
        0u8,
    ) == CP_GET_SUCCESS) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn Rs232PortPutByte(ByteToSend: u8) {
    if Rs232ComPort == 0 {
        return;
    }
    CpPutByte(core::ptr::addr_of_mut!(Rs232ComPortInfo), ByteToSend);
}

#[no_mangle]
pub unsafe extern "C" fn Rs232PortInUse(Base: PUCHAR) -> BOOLEAN {
    #[cfg(debug_assertions)]
    {
        if Rs232ComPort != 0 && Rs232ComPortInfo.Address == Base {
            1u8
        } else {
            0u8
        }
    }
    #[cfg(not(debug_assertions))]
    {
        0u8
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 2001  Brian Palmer  <brianp@sginet.com>
//  *  Copyright (C) 2001  Eric Kohl
//  *  Copyright (C) 2001  Emanuele Aliberti
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #if !defined(_M_ARM) && !defined(_M_ARM64)
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
// #include <cportlib/cportlib.h>
//
// #if DBG
//
// /* STATIC VARIABLES ***********************************************************/
//
// #define DEFAULT_BAUD_RATE   19200
//
// #if defined(_M_IX86) || defined(_M_AMD64)
// static const ULONG BaseArray[] = {0, 0x3F8, 0x2F8, 0x3E8, 0x2E8};
// #elif defined(_M_PPC)
// static const ULONG BaseArray[] = {0, 0x800003F8};
// #elif defined(_M_MIPS)
// static const ULONG BaseArray[] = {0, 0x80006000, 0x80007000};
// #elif defined(_M_ARM)
// static const ULONG BaseArray[] = {0, 0xF1012000};
// #else
// #error Unknown architecture
// #endif
//
// #define MAX_COM_PORTS   (sizeof(BaseArray) / sizeof(BaseArray[0]) - 1)
//
// /* The COM port must only be initialized once! */
// static ULONG Rs232ComPort = 0;
// static CPPORT Rs232ComPortInfo;
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN Rs232PortInitialize(IN ULONG ComPort,
//                             IN ULONG BaudRate)
// {
//     NTSTATUS Status;
//     PUCHAR Address;
//
//     /*
//      * Check whether it's the first time we initialize a COM port.
//      * If not, check whether the specified one was already initialized.
//      */
//     if ((Rs232ComPort != 0) && (Rs232ComPort == ComPort))
//         return TRUE;
//
//     if (BaudRate == 0)
//         BaudRate = DEFAULT_BAUD_RATE;
//
//     if (ComPort == 0)
//     {
//         /*
//          * Start enumerating COM ports from the last one to the first one,
//          * and break when we find a valid port.
//          * If we reach the first element of the list, the invalid COM port,
//          * then it means that no valid port was found.
//          */
//         for (ComPort = MAX_COM_PORTS; ComPort > 0; ComPort--)
//         {
//             if (CpDoesPortExist(UlongToPtr(BaseArray[ComPort])))
//             {
//                 Address = UlongToPtr(BaseArray[ComPort]);
//                 break;
//             }
//         }
//         if (ComPort == 0)
//             return FALSE;
//     }
//     else if (ComPort <= MAX_COM_PORTS)
//     {
//         if (CpDoesPortExist(UlongToPtr(BaseArray[ComPort])))
//             Address = UlongToPtr(BaseArray[ComPort]);
//         else
//             return FALSE;
//     }
//     else
//     {
//         return FALSE;
//     }
//
//     Status = CpInitialize(&Rs232ComPortInfo, Address, BaudRate);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     Rs232ComPort = ComPort;
//
//     return TRUE;
// }
//
// BOOLEAN Rs232PortGetByte(PUCHAR ByteReceived)
// {
//     if (Rs232ComPort == 0) return FALSE;
//     return (CpGetByte(&Rs232ComPortInfo, ByteReceived, TRUE, FALSE) == CP_GET_SUCCESS);
// }
//
// /*
// BOOLEAN Rs232PortPollByte(PUCHAR ByteReceived)
// {
//     if (Rs232ComPort == 0) return FALSE;
//     return (CpGetByte(&Rs232ComPortInfo, ByteReceived, FALSE, FALSE) == CP_GET_SUCCESS);
// }
// */
//
// VOID Rs232PortPutByte(UCHAR ByteToSend)
// {
//     if (Rs232ComPort == 0) return;
//     CpPutByte(&Rs232ComPortInfo, ByteToSend);
// }
//
// #endif /* DBG */
//
// BOOLEAN Rs232PortInUse(PUCHAR Base)
// {
// #if DBG
//     return ( ((Rs232ComPort != 0) && (Rs232ComPortInfo.Address == Base)) ? TRUE : FALSE );
// #else
//     return FALSE;
// #endif
// }
//
// #endif /* not _M_ARM and not _M_ARM64 */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
