use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{HalDisplayString, PCPPORT};

extern "system" {
    fn CpPutByte(PortInformation: PCPPORT, ByteToSend: u8);
}

// BaseArray was `static const ULONG[]` in i386/kdserial.c. Owned by this Rust
// file now since kdserial.c is excluded from the build. The i386 default array
// is the standard 4-port COM mapping (not the alternate _M_PPC / arm / arm64
// variants — those live in their own arch dirs).
#[no_mangle]
pub static BaseArray: [ULONG; 5] = [0, 0x3F8, 0x2F8, 0x3E8, 0x2E8];
extern "system" {
    fn CpDoesPortExist(Address: PVOID) -> BOOLEAN;
    fn CpInitialize(PortInformation: PCPPORT, Address: PVOID, BaudRate: ULONG) -> NTSTATUS;
    fn CpGetByte( PortInformation: PCPPORT, ByteReceived: PUCHAR, Wait: BOOLEAN, Poll: BOOLEAN, ) -> ULONG;
}

// UlongToPtr is a DDK macro in basetsd.h, not a real function.
#[inline(always)]
unsafe fn UlongToPtr(value: ULONG) -> PVOID {
    value as usize as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn KdPortInitializeEx(
    PortInformation: PCPPORT,
    ComPortNumber: ULONG,
) -> BOOLEAN {

    let Status: NTSTATUS = CpInitialize(
        PortInformation,
        if ComPortNumber == 0 {
            (*PortInformation).Address as PVOID
        } else {
            UlongToPtr(BaseArray[ComPortNumber as usize])
        },
        if (*PortInformation).BaudRate == 0 {
            19200
        } else {
            (*PortInformation).BaudRate
        },
    );

    if !NT_SUCCESS(Status) {
        HalDisplayString(b"\r\nKernel Debugger: Serial port not found!\r\n\r\n\0".as_ptr() as PCHAR);
        return 0u8;
    } else {
        #[cfg(debug_assertions)]
        {
            let mut Buffer: [CHAR; 82] = [0; 82];

            crate::rtl_string_cb_printf_a!(
                Buffer.as_mut_ptr(),
                core::mem::size_of_val(&Buffer),
                b"\r\nKernel Debugger: Serial port found: COM%ld (Port 0x%p) BaudRate %ld\r\n\r\n",
                ComPortNumber,
                (*PortInformation).Address,
                (*PortInformation).BaudRate,
            );

            HalDisplayString(Buffer.as_ptr());
        }

        return 1u8;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdPortGetByteEx(
    PortInformation: PCPPORT,
    ByteReceived: PUCHAR,
) -> BOOLEAN {
    
    const CP_GET_SUCCESS: ULONG = 0;
    
    (CpGetByte(PortInformation, ByteReceived, 0u8, 0u8) == CP_GET_SUCCESS) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn KdPortPutByteEx(
    PortInformation: PCPPORT,
    ByteToSend: u8,
) {
    CpPutByte(PortInformation, ByteToSend);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/kd/i386/kdbg.c
//  * PURPOSE:         Serial i/o functions for the kernel debugger.
//  * PROGRAMMER:      Alex Ionescu
//  *                  Herv� Poussineau
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #if defined(SARCH_PC98)
// #define DEFAULT_BAUD_RATE   9600
// #else
// #define DEFAULT_BAUD_RATE   19200
// #endif
//
// #if defined(_M_IX86) || defined(_M_AMD64)
// #if defined(SARCH_PC98)
// const ULONG BaseArray[] = {0, 0x30, 0x238};
// #else
// const ULONG BaseArray[] = {0, 0x3F8, 0x2F8, 0x3E8, 0x2E8};
// #endif
// #elif defined(_M_PPC)
// const ULONG BaseArray[] = {0, 0x800003F8};
// #elif defined(_M_MIPS)
// const ULONG BaseArray[] = {0, 0x80006000, 0x80007000};
// #elif defined(_M_ARM)
// const ULONG BaseArray[] = {0, 0xF1012000};
// #elif defined(_M_ARM64)
// const ULONG BaseArray[] = {0, 0x09000000};  /* PL011 UART base on QEMU virt */
// #else
// #error Unknown architecture
// #endif
//
// #define MAX_COM_PORTS   (sizeof(BaseArray) / sizeof(BaseArray[0]) - 1)
//
// /* STATIC VARIABLES ***********************************************************/
//
// // static CPPORT DefaultPort = {0, 0, 0};
//
// /* The COM port must only be initialized once! */
// // static BOOLEAN PortInitialized = FALSE;
//
// /* REACTOS FUNCTIONS **********************************************************/
//
// BOOLEAN
// NTAPI
// KdPortInitializeEx(
//     IN PCPPORT PortInformation,
//     IN ULONG ComPortNumber)
// {
//     NTSTATUS Status;
//
// #if 0 // Deactivated because never used in fact (was in KdPortInitialize but we use KdPortInitializeEx)
//     /*
//      * Find the port if needed
//      */
//
//     if (!PortInitialized)
//     {
//         DefaultPort.BaudRate = PortInformation->BaudRate;
//
//         if (ComPortNumber == 0)
//         {
//             /*
//              * Start enumerating COM ports from the last one to the first one,
//              * and break when we find a valid port.
//              * If we reach the first element of the list, the invalid COM port,
//              * then it means that no valid port was found.
//              */
//             for (ComPortNumber = MAX_COM_PORTS; ComPortNumber > 0; ComPortNumber--)
//             {
//                 if (CpDoesPortExist(UlongToPtr(BaseArray[ComPortNumber])))
//                 {
//                     PortInformation->Address = DefaultPort.Address = BaseArray[ComPortNumber];
//                     break;
//                 }
//             }
//             if (ComPortNumber == 0)
//             {
//                 HalDisplayString("\r\nKernel Debugger: No COM port found!\r\n\r\n");
//                 return FALSE;
//             }
//         }
//
//         PortInitialized = TRUE;
//     }
// #endif
//
//     /*
//      * Initialize the port
//      */
//     Status = CpInitialize(PortInformation,
//                           (ComPortNumber == 0 ? PortInformation->Address
//                                               : UlongToPtr(BaseArray[ComPortNumber])),
//                           (PortInformation->BaudRate == 0 ? DEFAULT_BAUD_RATE
//                                                           : PortInformation->BaudRate));
//     if (!NT_SUCCESS(Status))
//     {
//         HalDisplayString("\r\nKernel Debugger: Serial port not found!\r\n\r\n");
//         return FALSE;
//     }
//     else
//     {
// #ifndef NDEBUG
//         int Length;
//         CHAR Buffer[82];
//
//         /* Print message to blue screen */
//         Length = snprintf(Buffer, sizeof(Buffer),
//                           "\r\nKernel Debugger: Serial port found: COM%ld (Port 0x%p) BaudRate %ld\r\n\r\n",
//                           ComPortNumber,
//                           PortInformation->Address,
//                           PortInformation->BaudRate);
//         if (Length == -1)
//         {
//             /* Terminate it if we went over-board */
//             Buffer[sizeof(Buffer) - 1] = ANSI_NULL;
//         }
//
//         HalDisplayString(Buffer);
// #endif /* NDEBUG */
//
// #if 0
//         /* Set global info */
//         KdComPortInUse = DefaultPort.Address;
// #endif
//         return TRUE;
//     }
// }
//
// BOOLEAN
// NTAPI
// KdPortGetByteEx(
//     IN PCPPORT PortInformation,
//     OUT PUCHAR ByteReceived)
// {
//     return (CpGetByte(PortInformation, ByteReceived, FALSE, FALSE) == CP_GET_SUCCESS);
// }
//
// VOID
// NTAPI
// KdPortPutByteEx(
//     IN PCPPORT PortInformation,
//     IN UCHAR ByteToSend)
// {
//     CpPutByte(PortInformation, ByteToSend);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
