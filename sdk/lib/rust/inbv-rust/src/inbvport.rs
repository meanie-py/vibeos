use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;

use ntkrnl_sys::ddk::CPPORT;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

type PCPPORT = *mut CPPORT;

extern "system" {
    fn CpEnableFifo(Address: PUCHAR, Enable: BOOLEAN);
    fn CpGetByte(Port: PCPPORT, Byte: *mut UCHAR, Wait: BOOLEAN, Poll: BOOLEAN) -> ULONG;
    fn CpInitialize(Port: PCPPORT, Address: PUCHAR, BaudRate: ULONG);
    fn CpDoesPortExist(Address: PUCHAR) -> BOOLEAN;
    fn CpPutByte(Port: PCPPORT, Byte: UCHAR);
}

const CP_GET_SUCCESS: ULONG = 0;

#[no_mangle]
pub static mut Port: [CPPORT; 4] = [
    CPPORT { Address: core::ptr::null_mut(), BaudRate: 0, Flags: 1 },
    CPPORT { Address: core::ptr::null_mut(), BaudRate: 0, Flags: 1 },
    CPPORT { Address: core::ptr::null_mut(), BaudRate: 0, Flags: 1 },
    CPPORT { Address: core::ptr::null_mut(), BaudRate: 0, Flags: 1 },
];

#[no_mangle]
pub unsafe extern "system" fn InbvPortPollOnly(PortId: ULONG) -> BOOLEAN {
    let mut dummy: UCHAR = 0;
    
    if CpGetByte(
        core::ptr::addr_of_mut!(Port[PortId as usize]),
        &mut dummy,
        0u8,
        1u8,
    ) == CP_GET_SUCCESS
    {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvPortGetByte(PortId: ULONG, Byte: *mut UCHAR) -> BOOLEAN {
    let Byte = opaque_ptr(Byte);
    if CpGetByte(
        core::ptr::addr_of_mut!(Port[PortId as usize]),
        Byte,
        1u8,
        0u8,
    ) == CP_GET_SUCCESS
    {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvPortPutByte(PortId: ULONG, Byte: UCHAR) {
    CpPutByte(core::ptr::addr_of_mut!(Port[PortId as usize]), Byte);
}

#[no_mangle]
pub unsafe extern "system" fn InbvPortEnableFifo(PortId: ULONG, Enable: BOOLEAN) {
    CpEnableFifo(Port[PortId as usize].Address, Enable);
}

#[no_mangle]
pub unsafe extern "system" fn InbvPortTerminate(PortId: ULONG) {
    Port[PortId as usize].Address = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn InbvPortInitialize(
    BaudRate: ULONG,
    PortNumber: ULONG,
    PortAddress: *mut UCHAR,
    PortId: *mut ULONG,
    IsMMIODevice: BOOLEAN,
) -> BOOLEAN {
    let PortAddress = opaque_ptr(PortAddress);
    let PortId = opaque_ptr(PortId);
    ntkrnl_sys::ASSERT!(IsMMIODevice == 0u8);

    let mut baud_rate = BaudRate;
    let mut port_number = PortNumber;
    let mut port_address = PortAddress;

    {
        if baud_rate == 0 {
            baud_rate = 19200;
        }

        if port_number != 0 {
            if port_address.is_null() {
                port_address = match port_number {
                    1 => 0x3F8 as *mut UCHAR,
                    2 => 0x2F8 as *mut UCHAR,
                    3 => 0x3E8 as *mut UCHAR,
                    _ => {
                        port_number = 4;
                        0x2E8 as *mut UCHAR
                    }
                };
            }
        } else {
            port_address = 0x2F8 as *mut UCHAR;
            if CpDoesPortExist(port_address) != 0 {
                port_number = 2;
            } else {
                port_address = 0x3F8 as *mut UCHAR;
                if CpDoesPortExist(port_address) == 0 {
                    return 0u8;
                }
                port_number = 1;
            }
        }
    }

    if !Port[(port_number - 1) as usize].Address.is_null() {
        return 0u8;
    }

    CpInitialize(
        core::ptr::addr_of_mut!(Port[(port_number - 1) as usize]),
        port_address,
        baud_rate,
    );
    *PortId = port_number - 1;

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/inbv/inbvport.c
//  * PURPOSE:         Serial Port Boot Driver for Headless Terminal Support
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// CPPORT Port[4] =
// {
//     {NULL, 0, TRUE},
//     {NULL, 0, TRUE},
//     {NULL, 0, TRUE},
//     {NULL, 0, TRUE}
// };
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// NTAPI
// InbvPortPollOnly(IN ULONG PortId)
// {
//     UCHAR Dummy;
//
//     /* Poll a byte from the port */
//     return CpGetByte(&Port[PortId], &Dummy, FALSE, TRUE) == CP_GET_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// InbvPortGetByte(IN  ULONG  PortId,
//                 OUT PUCHAR Byte)
// {
//     /* Read a byte from the port */
//     return CpGetByte(&Port[PortId], Byte, TRUE, FALSE) == CP_GET_SUCCESS;
// }
//
// VOID
// NTAPI
// InbvPortPutByte(IN ULONG PortId,
//                 IN UCHAR Byte)
// {
//     /* Send the byte */
//     CpPutByte(&Port[PortId], Byte);
// }
//
// VOID
// NTAPI
// InbvPortEnableFifo(IN ULONG   PortId,
//                    IN BOOLEAN Enable)
// {
//     /* Set FIFO as requested */
//     CpEnableFifo(Port[PortId].Address, Enable);
// }
//
// VOID
// NTAPI
// InbvPortTerminate(IN ULONG PortId)
// {
//     /* The port is now available */
//     Port[PortId].Address = NULL;
// }
//
// BOOLEAN
// NTAPI
// InbvPortInitialize(IN  ULONG   BaudRate,
//                    IN  ULONG   PortNumber,
//                    IN  PUCHAR  PortAddress,
//                    OUT PULONG  PortId,
//                    IN  BOOLEAN IsMMIODevice)
// {
//     /* Not yet supported */
//     ASSERT(IsMMIODevice == FALSE);
//
// #if defined(SARCH_PC98)
//     /* Set default baud rate */
//     if (BaudRate == 0) BaudRate = 9600;
//
//     /* Check if port or address given */
//     if (PortNumber)
//     {
//         /* Pick correct address for port */
//         if (!PortAddress)
//         {
//             if (PortNumber == 1)
//             {
//                 PortAddress = (PUCHAR)0x30;
//             }
//             else
//             {
//                 PortAddress = (PUCHAR)0x238;
//                 PortNumber = 2;
//             }
//         }
//     }
//     else
//     {
//         /* Pick correct port for address */
//         PortAddress = (PUCHAR)0x30;
//         if (CpDoesPortExist(PortAddress))
//         {
//             PortNumber = 1;
//         }
//         else
//         {
//             PortAddress = (PUCHAR)0x238;
//             if (!CpDoesPortExist(PortAddress))
//                 return FALSE;
//
//             PortNumber = 2;
//         }
//     }
// #else
//     /* Set default baud rate */
//     if (BaudRate == 0) BaudRate = 19200;
//
//     /* Check if port or address given */
//     if (PortNumber)
//     {
//         /* Pick correct address for port */
//         if (!PortAddress)
//         {
//             switch (PortNumber)
//             {
//                 case 1:
//                     PortAddress = (PUCHAR)0x3F8;
//                     break;
//
//                 case 2:
//                     PortAddress = (PUCHAR)0x2F8;
//                     break;
//
//                 case 3:
//                     PortAddress = (PUCHAR)0x3E8;
//                     break;
//
//                 default:
//                     PortNumber = 4;
//                     PortAddress = (PUCHAR)0x2E8;
//             }
//         }
//     }
//     else
//     {
//         /* Pick correct port for address */
//         PortAddress = (PUCHAR)0x2F8;
//         if (CpDoesPortExist(PortAddress))
//         {
//             PortNumber = 2;
//         }
//         else
//         {
//             PortAddress = (PUCHAR)0x3F8;
//             if (!CpDoesPortExist(PortAddress)) return FALSE;
//             PortNumber = 1;
//         }
//     }
// #endif
//
//     /* Initialize the port unless it's already up, and then return it */
//     if (Port[PortNumber - 1].Address) return FALSE;
//
//     CpInitialize(&Port[PortNumber - 1], PortAddress, BaudRate);
//     *PortId = PortNumber - 1;
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
