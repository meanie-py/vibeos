use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{CPPORT, KDSTATUS, PLOADER_PARAMETER_BLOCK, va_list};
use core::ffi::c_void;

use crate::KDDBGPRINT;

// KdComPortInUse: the shared KD global the transport sets so the kernel knows
// which COM port is in use. In a real _WINKD_ kernel this is the kernel's
// global (kd/kdio); it is NOT in the (non-_WINKD_) ntoskrnl.spec, so for the
// standalone link spike it is defined here. --allow-multiple-definition lets
// the kernel's definition win if both are ever linked together.
#[no_mangle]
pub static mut KdComPortInUse: PUCHAR = core::ptr::null_mut();

// cportlib CpGetByte return codes (cportlib.h). Not in ntkrnl-sys.
pub const CP_GET_SUCCESS: USHORT = 0;
pub const CP_GET_NODATA: USHORT = 1;
pub const CP_GET_ERROR: USHORT = 2;

// winnt UlongToPtr macro: zero-extend a ULONG to a pointer.
#[inline]
unsafe fn UlongToPtr(Value: ULONG) -> PVOID {
    Value as usize as PVOID
}

// CRT (cdecl) — link from libcntpr / cnt-rust at final link.
extern "C" {
    fn _vsnprintf(buffer: *mut i8, count: usize, format: *const i8, argptr: va_list) -> i32;
    fn _strupr(s: PCHAR) -> PCHAR;
    fn atol(s: PCHAR) -> i32;
    fn strlen(s: PCHAR) -> usize;
    fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8;
}
// cportlib — NTAPI = stdcall on i386 (_CpPutByte@8 / _CpGetByte@16). Link
// from libcportlib / cport-rust at final link.
extern "system" {
    fn CpPutByte(port: *mut CPPORT, byte: u8);
    fn CpInitialize(Port: *mut CPPORT, Address: PVOID, BaudRate: ULONG) -> NTSTATUS;
    fn CpDoesPortExist(Address: PVOID) -> BOOLEAN;
    fn CpGetByte(Port: *mut CPPORT, Byte: PUCHAR, Wait: BOOLEAN, Poll: BOOLEAN) -> USHORT;
}

// ---- file-scope preamble (translated) ----
pub const DEFAULT_DEBUG_PORT: u32 = 2;

pub const DEFAULT_DEBUG_COM1_IRQ: u32 = 4;

pub const DEFAULT_DEBUG_COM2_IRQ: u32 = 3;

pub const DEFAULT_DEBUG_BAUD_RATE: u32 = 115200;

pub const DEFAULT_BAUD_RATE: u32 = 19200;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub static BaseArray: [ULONG; 5] = [0, 0x3F8, 0x2F8, 0x3E8, 0x2E8];

#[cfg(target_arch = "powerpc")]
pub static BaseArray: [ULONG; 2] = [0, 0x800003F8];

#[cfg(target_arch = "mips")]
pub static BaseArray: [ULONG; 3] = [0, 0x80006000, 0x80007000];

#[cfg(target_arch = "arm")]
pub static BaseArray: [ULONG; 2] = [0, 0xF1012000];

pub const MAX_COM_PORTS: usize = {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { 4 }
    #[cfg(target_arch = "powerpc")]
    { 1 }
    #[cfg(target_arch = "mips")]
    { 2 }
    #[cfg(target_arch = "arm")]
    { 1 }
};

pub static mut KdComPort: CPPORT = CPPORT {
    Address: core::ptr::null_mut(),
    BaudRate: 0,
    Flags: 0,
};

pub static mut KdComPortIrq: ULONG = 0;

#[cfg(feature = "KDDEBUG")]
pub static mut KdDebugComPort: CPPORT = CPPORT {
    Address: core::ptr::null_mut(),
    BaudRate: 0,
    Flags: 0,
};

// C: `KdpDbgPrint` is only defined `#ifdef KDDEBUG` (along with KdDebugComPort).
#[cfg(feature = "KDDEBUG")]
#[no_mangle]
pub unsafe extern "C" fn KdpDbgPrint(Format: *const i8, mut args: ...) -> ULONG {
    let mut ap: va_list = args.as_va_list();
    let mut Length: i32;
    let mut ptr: *mut i8;
    let mut Buffer: [i8; 512] = [0; 512];

    Length = _vsnprintf(Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer), Format, ap);

    if Length == -1 {
        Buffer[core::mem::size_of_val(&Buffer) - 1] = b'\n' as i8;
        Length = core::mem::size_of_val(&Buffer) as i32;
    }

    ptr = Buffer.as_mut_ptr();
    while Length > 0 {
        if *ptr == b'\n' as i8 {
            CpPutByte(core::ptr::addr_of_mut!(KdDebugComPort), b'\r');
        }
        CpPutByte(core::ptr::addr_of_mut!(KdDebugComPort), *ptr as u8);
        ptr = ptr.offset(1);
        Length -= 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn KdD0Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdD3Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdSave(_SleepTransition: BOOLEAN) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdRestore(_SleepTransition: u8) -> i32 {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpPortInitialize(
    ComPortNumber: ULONG,
    ComPortBaudRate: ULONG,
) -> NTSTATUS {
    let Status = CpInitialize(
        core::ptr::addr_of_mut!(KdComPort),
        UlongToPtr(BaseArray[ComPortNumber as usize]),
        ComPortBaudRate,
    );
    if !NT_SUCCESS(Status) {
        return STATUS_INVALID_PARAMETER;
    }

    KdComPortInUse = KdComPort.Address;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize0(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let mut ComPortNumber: ULONG = 2;
    let mut ComPortBaudRate: ULONG = 115200;

    let CommandLine: PCHAR;
    let mut PortString: PCHAR;
    let mut BaudString: PCHAR;
    let mut IrqString: PCHAR;
    let mut Value: ULONG;

    if !LoaderBlock.is_null() {
        CommandLine = (*LoaderBlock).LoadOptions;

        _strupr(CommandLine);

        PortString = strstr(CommandLine, b"DEBUGPORT\0".as_ptr() as PCHAR);
        BaudString = strstr(CommandLine, b"BAUDRATE\0".as_ptr() as PCHAR);
        IrqString = strstr(CommandLine, b"IRQ\0".as_ptr() as PCHAR);

        if !PortString.is_null() {
            PortString = PortString.add(strlen(b"DEBUGPORT\0".as_ptr() as PCHAR));

            while *PortString == b' ' as i8 {
                PortString = PortString.add(1);
            }
            /* Skip the equals sign, but never the terminator: a command line
             * ending in a bare "DEBUGPORT" would read past its end (upstream
             * 956d9f7973f) */
            if *PortString != 0 {
                PortString = PortString.add(1);
            }

            if strncmp(PortString, b"COM\0".as_ptr() as PCHAR, 3) != 0 {
                return STATUS_INVALID_PARAMETER;
            }

            PortString = PortString.add(3);
            Value = atol(PortString) as ULONG;
            if Value >= (core::mem::size_of_val(&BaseArray) / core::mem::size_of::<ULONG>()) as ULONG {
                return STATUS_INVALID_PARAMETER;
            }

            ComPortNumber = Value;
        }

        if !BaudString.is_null() {
            BaudString = BaudString.add(strlen(b"BAUDRATE\0".as_ptr() as PCHAR));

            while *BaudString == b' ' as i8 {
                BaudString = BaudString.add(1);
            }

            if *BaudString != 0 {
                Value = atol(BaudString.add(1)) as ULONG;
                if Value != 0 {
                    ComPortBaudRate = Value;
                }
            }
        }

        if !IrqString.is_null() {
            IrqString = IrqString.add(strlen(b"IRQ\0".as_ptr() as PCHAR));

            while *IrqString == b' ' as i8 {
                IrqString = IrqString.add(1);
            }

            if *IrqString != 0 {
                Value = atol(IrqString.add(1)) as ULONG;
                if Value != 0 {
                    KdComPortIrq = Value;
                }
            }
        }
    }

    #[cfg(feature = "KDDEBUG")]
    {
        let mut ComPort: ULONG = 5;
        while ComPort > 0 {
            if (ComPort != ComPortNumber) && (CpDoesPortExist(UlongToPtr(BaseArray[ComPort as usize])) != 0) {
                break;
            }
            ComPort -= 1;
        }
        if ComPort != 0 {
            CpInitialize(core::ptr::addr_of_mut!(KdDebugComPort), UlongToPtr(BaseArray[ComPort as usize]), 19200);
        }
    }

    KdpPortInitialize(ComPortNumber, ComPortBaudRate)
}

#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize1(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSendByte(Byte: UCHAR) {
    CpPutByte(core::ptr::addr_of_mut!(KdComPort), Byte);
}

#[no_mangle]
pub unsafe extern "system" fn KdpPollByte(OutByte: PUCHAR) -> KDSTATUS {
    if CpGetByte(core::ptr::addr_of_mut!(KdComPort), OutByte, 0u8, 0u8) == CP_GET_SUCCESS {
        KdPacketReceived
    } else {
        KdPacketTimedOut
    }
}
#[no_mangle]
pub unsafe extern "system" fn KdpReceiveByte(OutByte: PUCHAR) -> KDSTATUS {
    let mut CpStatus: USHORT;

    loop {
        CpStatus = CpGetByte(
            core::ptr::addr_of_mut!(KdComPort),
            OutByte,
            1u8,
            0u8
        );
        if CpStatus != CP_GET_NODATA {
            break;
        }
    }

    if CpStatus == CP_GET_SUCCESS {
        return KdPacketReceived;
    }

    return KdPacketTimedOut;
}

#[no_mangle]
pub unsafe extern "system" fn KdpPollBreakIn() -> KDSTATUS {
    let mut Byte: UCHAR = 0;

    let KdStatus = KdpPollByte(&mut Byte);
    if KdStatus == KdPacketReceived {
        if Byte == 0x03 {
            KDDBGPRINT!(b"BreakIn Polled.\n\0".as_ptr());
            return KdPacketReceived;
        } else if Byte == b'$' {
            KdpSendByte(b'-');
        }
    }
    KdPacketTimedOut
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kdgdb/kdcom.c
//  * PURPOSE:         COM port functions for the kernel debugger.
//  */
//
// #include "kdgdb.h"
//
// #include <cportlib/cportlib.h>
// #include <arc/arc.h>
// #include <stdlib.h>
// #include <ndk/halfuncs.h>
//
// /* Serial debug connection */
// #define DEFAULT_DEBUG_PORT      2 /* COM2 */
// #define DEFAULT_DEBUG_COM1_IRQ  4
// #define DEFAULT_DEBUG_COM2_IRQ  3
// #define DEFAULT_DEBUG_BAUD_RATE 115200
// #define DEFAULT_BAUD_RATE       19200
//
// #if defined(_M_IX86) || defined(_M_AMD64)
// const ULONG BaseArray[] = {0, 0x3F8, 0x2F8, 0x3E8, 0x2E8};
// #elif defined(_M_PPC)
// const ULONG BaseArray[] = {0, 0x800003F8};
// #elif defined(_M_MIPS)
// const ULONG BaseArray[] = {0, 0x80006000, 0x80007000};
// #elif defined(_M_ARM)
// const ULONG BaseArray[] = {0, 0xF1012000};
// #else
// #error Unknown architecture
// #endif
//
// #define MAX_COM_PORTS   (sizeof(BaseArray) / sizeof(BaseArray[0]) - 1)
//
// /* GLOBALS ********************************************************************/
//
// CPPORT KdComPort;
// ULONG  KdComPortIrq = 0; // Not used at the moment.
// #ifdef KDDEBUG
// CPPORT KdDebugComPort;
// #endif
//
// /* DEBUGGING ******************************************************************/
//
// #ifdef KDDEBUG
// ULONG KdpDbgPrint(const char *Format, ...)
// {
//     va_list ap;
//     int Length;
//     char* ptr;
//     CHAR Buffer[512];
//
//     va_start(ap, Format);
//     Length = _vsnprintf(Buffer, sizeof(Buffer), Format, ap);
//     va_end(ap);
//
//     /* Check if we went past the buffer */
//     if (Length == -1)
//     {
//         /* Terminate it if we went over-board */
//         Buffer[sizeof(Buffer) - 1] = '\n';
//
//         /* Put maximum */
//         Length = sizeof(Buffer);
//     }
//
//     ptr = Buffer;
//     while (Length--)
//     {
//         if (*ptr == '\n')
//             CpPutByte(&KdDebugComPort, '\r');
//
//         CpPutByte(&KdDebugComPort, *ptr++);
//     }
//
//     return 0;
// }
// #endif
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// KdD0Transition(VOID)
// {
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdD3Transition(VOID)
// {
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdSave(IN BOOLEAN SleepTransition)
// {
//     /* Nothing to do on COM ports */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdRestore(IN BOOLEAN SleepTransition)
// {
//     /* Nothing to do on COM ports */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdpPortInitialize(IN ULONG ComPortNumber,
//                   IN ULONG ComPortBaudRate)
// {
//     NTSTATUS Status;
//
//     Status = CpInitialize(&KdComPort,
//                           UlongToPtr(BaseArray[ComPortNumber]),
//                           ComPortBaudRate);
//     if (!NT_SUCCESS(Status))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     KdComPortInUse = KdComPort.Address;
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * \name KdDebuggerInitialize0
//  * \brief Phase 0 initialization.
//  * \param [opt] LoaderBlock Pointer to the Loader parameter block. Can be NULL.
//  * \return Status
//  */
// NTSTATUS
// NTAPI
// KdDebuggerInitialize0(IN PLOADER_PARAMETER_BLOCK LoaderBlock OPTIONAL)
// {
//     ULONG ComPortNumber   = DEFAULT_DEBUG_PORT;
//     ULONG ComPortBaudRate = DEFAULT_DEBUG_BAUD_RATE;
//
//     PCHAR CommandLine, PortString, BaudString, IrqString;
//     ULONG Value;
//
//     /* Check if we have a LoaderBlock */
//     if (LoaderBlock)
//     {
//         /* Get the Command Line */
//         CommandLine = LoaderBlock->LoadOptions;
//
//         /* Upcase it */
//         _strupr(CommandLine);
//
//         /* Get the port and baud rate */
//         PortString = strstr(CommandLine, "DEBUGPORT");
//         BaudString = strstr(CommandLine, "BAUDRATE");
//         IrqString  = strstr(CommandLine, "IRQ");
//
//         /* Check if we got the /DEBUGPORT parameter */
//         if (PortString)
//         {
//             /* Move past the actual string, to reach the port*/
//             PortString += strlen("DEBUGPORT");
//
//             /* Now get past any spaces and skip the equal sign */
//             while (*PortString == ' ') PortString++;
//             PortString++;
//
//             /* Do we have a serial port? */
//             if (strncmp(PortString, "COM", 3) != 0)
//             {
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Check for a valid Serial Port */
//             PortString += 3;
//             Value = atol(PortString);
//             if (Value >= sizeof(BaseArray) / sizeof(BaseArray[0]))
//             {
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Set the port to use */
//             ComPortNumber = Value;
//        }
//
//         /* Check if we got a baud rate */
//         if (BaudString)
//         {
//             /* Move past the actual string, to reach the rate */
//             BaudString += strlen("BAUDRATE");
//
//             /* Now get past any spaces */
//             while (*BaudString == ' ') BaudString++;
//
//             /* And make sure we have a rate */
//             if (*BaudString)
//             {
//                 /* Read and set it */
//                 Value = atol(BaudString + 1);
//                 if (Value) ComPortBaudRate = Value;
//             }
//         }
//
//         /* Check Serial Port Settings [IRQ] */
//         if (IrqString)
//         {
//             /* Move past the actual string, to reach the rate */
//             IrqString += strlen("IRQ");
//
//             /* Now get past any spaces */
//             while (*IrqString == ' ') IrqString++;
//
//             /* And make sure we have an IRQ */
//             if (*IrqString)
//             {
//                 /* Read and set it */
//                 Value = atol(IrqString + 1);
//                 if (Value) KdComPortIrq = Value;
//             }
//         }
//     }
//
// #ifdef KDDEBUG
//     /*
//      * Try to find a free COM port and use it as the KD debugging port.
//      * NOTE: Inspired by reactos/boot/freeldr/freeldr/comm/rs232.c, Rs232PortInitialize(...)
//      */
//     {
//     /*
//      * Start enumerating COM ports from the last one to the first one,
//      * and break when we find a valid port.
//      * If we reach the first element of the list, the invalid COM port,
//      * then it means that no valid port was found.
//      */
//     ULONG ComPort;
//     for (ComPort = MAX_COM_PORTS; ComPort > 0; ComPort--)
//     {
//         /* Check if the port exist; skip the KD port */
//         if ((ComPort != ComPortNumber) && CpDoesPortExist(UlongToPtr(BaseArray[ComPort])))
//             break;
//     }
//     if (ComPort != 0)
//         CpInitialize(&KdDebugComPort, UlongToPtr(BaseArray[ComPort]), DEFAULT_BAUD_RATE);
//     }
// #endif
//
//     /* Initialize the port */
//     return KdpPortInitialize(ComPortNumber, ComPortBaudRate);
// }
//
// /******************************************************************************
//  * \name KdDebuggerInitialize1
//  * \brief Phase 1 initialization.
//  * \param [opt] LoaderBlock Pointer to the Loader parameter block. Can be NULL.
//  * \return Status
//  */
// NTSTATUS
// NTAPI
// KdDebuggerInitialize1(IN PLOADER_PARAMETER_BLOCK LoaderBlock OPTIONAL)
// {
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// NTAPI
// KdpSendByte( UCHAR Byte)
// {
//     /* Send the byte */
//     CpPutByte(&KdComPort, Byte);
// }
//
// KDSTATUS
// NTAPI
// KdpPollByte(OUT PUCHAR OutByte)
// {
//     /* Poll the byte */
//     if (CpGetByte(&KdComPort, OutByte, FALSE, FALSE) == CP_GET_SUCCESS)
//     {
//         return KdPacketReceived;
//     }
//     else
//     {
//         return KdPacketTimedOut;
//     }
// }
//
// KDSTATUS
// NTAPI
// KdpReceiveByte( PUCHAR OutByte)
// {
//     USHORT CpStatus;
//
//     do
//     {
//         CpStatus = CpGetByte(&KdComPort, OutByte, TRUE, FALSE);
//     } while (CpStatus == CP_GET_NODATA);
//
//     /* Get the byte */
//     if (CpStatus == CP_GET_SUCCESS)
//     {
//         return KdPacketReceived;
//     }
//
//     return KdPacketTimedOut;
// }
//
// KDSTATUS
// NTAPI
// KdpPollBreakIn(VOID)
// {
//     KDSTATUS KdStatus;
//     UCHAR Byte;
//
//     KdStatus = KdpPollByte(&Byte);
//     if (KdStatus == KdPacketReceived)
//     {
//         if (Byte == 0x03)
//         {
//             KDDBGPRINT!("BreakIn Polled.\n");
//             return KdPacketReceived;
//         }
//         else if (Byte == '$')
//         {
//             /* GDB tried to send a new packet. N-ack it. */
//             KdpSendByte('-');
//         }
//     }
//     return KdPacketTimedOut;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
