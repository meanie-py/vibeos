use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{CPPORT, PCPPORT, PLOADER_PARAMETER_BLOCK};
use core::ffi::c_void;

use crate::KDDBGPRINT;
use crate::kddll_h::*;

// cportlib `Cp*` are NTAPI = stdcall on i386 -> `extern "system"`. Provided by
// cport-rust (= cportlib). CpGetByte returns the CP_GET_* status code.
extern "system" {
    fn CpInitialize(Port: PCPPORT, Address: PVOID, BaudRate: ULONG) -> NTSTATUS;
    fn CpPutByte(Port: PCPPORT, Byte: u8);
    fn CpGetByte(Port: PCPPORT, Byte: PUCHAR, Wait: u8, Poll: u8) -> u16;
    fn CpDoesPortExist(Address: PVOID) -> BOOLEAN;
}

// CRT helpers are cdecl -> `extern "C"`.
extern "C" {
    fn _strupr(string: PCHAR) -> PCHAR;
    fn strstr(str1: PCHAR, str2: PCHAR) -> PCHAR;
    fn strlen(str: PCHAR) -> usize;
    fn strncmp(str1: PCHAR, str2: PCHAR, n: usize) -> i32;
    fn atol(str: PCHAR) -> i32;
}

// UlongToPtr is a trivial pointer-cast macro in C; reconstruct it locally.
#[inline]
unsafe fn UlongToPtr(Value: ULONG) -> PVOID {
    Value as usize as PVOID
}

// KdComPortInUse is a shared KD global not in the (non-_WINKD_) ntoskrnl.spec;
// define it locally. The kernel's definition wins under
// --allow-multiple-definition at final link.
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub static mut KdComPortInUse: PUCHAR = core::ptr::null_mut();

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

// aarch64: no port-I/O COM ports; the QEMU virt machine has a single PL011
// (owned by the console). No wire-KD channel yet — an empty table makes
// KdDebuggerInitialize0 fail cleanly, so the embedded transport never arms
// on arm64 (kd-rust falls back to the text path).
#[cfg(target_arch = "aarch64")]
pub static BaseArray: [ULONG; 1] = [0];

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

// KdpDbgPrint exists only under `#ifdef KDDEBUG` in C; gated likewise. Uses
// the C-variadic ABI, so it requires the c_variadic feature (only compiled
// when KDDEBUG is enabled).
#[cfg(feature = "KDDEBUG")]
extern "C" {
    fn _vsnprintf(
        buffer: *mut i8,
        count: usize,
        format: *const i8,
        args: ntkrnl_sys::ddk::va_list,
    ) -> i32;
}

#[cfg(feature = "KDDEBUG")]
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "C" fn KdpDbgPrint(Format: *const i8, mut args: ...) -> ULONG {
    let mut ap = args.as_va_list();
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

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdD0Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdD3Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdSave(_SleepTransition: BOOLEAN) -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdRestore(_SleepTransition: u8) -> i32 {
    STATUS_SUCCESS
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdDebuggerInitialize0(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {

    let mut ComPortNumber: ULONG = DEFAULT_DEBUG_PORT;
    let mut ComPortAddress: ULONG = 0;
    let mut ComPortBaudRate: ULONG = DEFAULT_DEBUG_BAUD_RATE;

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
            if *PortString == b':' as i8 {
                // COM:0x<port> - a UART at an I/O port no COM number names (an
                // ACPI SPCR console the loader found; kd-rust's KdpGetDebugMode
                // takes the same form). atol would read it as COM0, base 0.
                ComPortAddress = hex_port(PortString.add(1));
                if ComPortAddress == 0 {
                    return STATUS_INVALID_PARAMETER;
                }
            } else {
                Value = atol(PortString) as ULONG;
                if Value >= (core::mem::size_of_val(&BaseArray) / core::mem::size_of::<ULONG>()) as ULONG {
                    return STATUS_INVALID_PARAMETER;
                }

                ComPortNumber = Value;
            }
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
        let mut ComPort: ULONG = 4;
        while ComPort > 0 {
            if (ComPort != ComPortNumber) && CpDoesPortExist(UlongToPtr(BaseArray[ComPort as usize])) != 0 {
                break;
            }
            ComPort -= 1;
        }
        if ComPort != 0 {
            CpInitialize(core::ptr::addr_of_mut!(KdDebugComPort), UlongToPtr(BaseArray[ComPort as usize]), 19200);
        }
    }

    KDDBGPRINT!(b"KdDebuggerInitialize0\n\0".as_ptr() as PCHAR);

    if ComPortAddress != 0 {
        return KdpPortInitializeAt(ComPortAddress, ComPortBaudRate);
    }
    KdpPortInitialize(ComPortNumber, ComPortBaudRate)
}

/// `0x5000` / `0X5000` / `5000` (upper-cased by _strupr) as a hex port; 0 if empty.
unsafe fn hex_port(mut p: PCHAR) -> ULONG {
    if *p == b'0' as i8 && (*p.add(1) == b'X' as i8 || *p.add(1) == b'x' as i8) {
        p = p.add(2);
    }
    let mut v: ULONG = 0;
    loop {
        let c = *p as u8;
        let d = match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'F' => c - b'A' + 10,
            b'a'..=b'f' => c - b'a' + 10,
            _ => break,
        };
        v = v.wrapping_mul(16).wrapping_add(d as ULONG);
        p = p.add(1);
    }
    v
}

unsafe fn KdpPortInitializeAt(Address: ULONG, ComPortBaudRate: ULONG) -> NTSTATUS {
    let Status = CpInitialize(core::ptr::addr_of_mut!(KdComPort), UlongToPtr(Address), ComPortBaudRate);
    if !NT_SUCCESS(Status) {
        STATUS_INVALID_PARAMETER
    } else {
        KdComPortInUse = KdComPort.Address;
        STATUS_SUCCESS
    }
}

// KdpPortInitialize is defined in kdcom.c (NTAPI). The C body was preserved in
// the ORIGINAL C SOURCE block below; faithfully translated here.
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpPortInitialize(
    ComPortNumber: ULONG,
    ComPortBaudRate: ULONG,
) -> NTSTATUS {
    KDDBGPRINT!(b"KdpPortInitialize, Port = COM%ld\n\0".as_ptr() as PCHAR, ComPortNumber);

    let Status = CpInitialize(
        core::ptr::addr_of_mut!(KdComPort),
        UlongToPtr(BaseArray[ComPortNumber as usize]),
        ComPortBaudRate,
    );
    if !NT_SUCCESS(Status) {
        STATUS_INVALID_PARAMETER
    } else {
        KdComPortInUse = KdComPort.Address;
        STATUS_SUCCESS
    }
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdDebuggerInitialize1(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpSendByte(Byte: UCHAR) {
    CpPutByte(core::ptr::addr_of_mut!(KdComPort), Byte);
}
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpPollByte(OutByte: PUCHAR) -> KDP_STATUS {
    let Status: USHORT;

    Status = CpGetByte(core::ptr::addr_of_mut!(KdComPort), OutByte, 0u8, 0u8);
    match Status {
        CP_GET_SUCCESS => KDP_PACKET_RECEIVED,
        CP_GET_NODATA => KDP_PACKET_TIMEOUT,
        CP_GET_ERROR | _ => KDP_PACKET_RESEND,
    }
}
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpReceiveByte(OutByte: PUCHAR) -> KDP_STATUS {
    let Status: USHORT;

    Status = CpGetByte(core::ptr::addr_of_mut!(KdComPort), OutByte, 1u8, 0u8);
    match Status {
        CP_GET_SUCCESS => KDP_PACKET_RECEIVED,
        CP_GET_NODATA => KDP_PACKET_TIMEOUT,
        CP_GET_ERROR => KDP_PACKET_RESEND,
        _ => KDP_PACKET_RESEND,
    }
}
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpPollBreakIn() -> KDP_STATUS {
    let mut Byte: UCHAR = 0;
    let KdStatus: KDP_STATUS;

    KdStatus = KdpPollByte(&mut Byte as PUCHAR);
    if (KdStatus == KDP_PACKET_RECEIVED) && (Byte == BREAKIN_PACKET_BYTE) {
        return KDP_PACKET_RECEIVED;
    }
    return KDP_PACKET_TIMEOUT;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/kdcom.c
//  * PURPOSE:         COM port functions for the kernel debugger.
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include "kddll.h"
//
// #include <cportlib/cportlib.h>
// #include <arc/arc.h>
// #include <stdio.h>
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
//     KDDBGPRINT("KdpPortInitialize, Port = COM%ld\n", ComPortNumber);
//
//     Status = CpInitialize(&KdComPort,
//                           UlongToPtr(BaseArray[ComPortNumber]),
//                           ComPortBaudRate);
//     if (!NT_SUCCESS(Status))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//     else
//     {
//         KdComPortInUse = KdComPort.Address;
//         return STATUS_SUCCESS;
//     }
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
//     KDDBGPRINT("KdDebuggerInitialize0\n");
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
// KdpSendByte(IN UCHAR Byte)
// {
//     /* Send the byte */
//     CpPutByte(&KdComPort, Byte);
// }
//
// KDP_STATUS
// NTAPI
// KdpPollByte(OUT PUCHAR OutByte)
// {
//     USHORT Status;
//
//     /* Poll the byte */
//     Status = CpGetByte(&KdComPort, OutByte, FALSE, FALSE);
//     switch (Status)
//     {
//         case CP_GET_SUCCESS:
//             return KDP_PACKET_RECEIVED;
//
//         case CP_GET_NODATA:
//             return KDP_PACKET_TIMEOUT;
//
//         case CP_GET_ERROR:
//         default:
//             return KDP_PACKET_RESEND;
//     }
// }
//
// KDP_STATUS
// NTAPI
// KdpReceiveByte(OUT PUCHAR OutByte)
// {
//     USHORT Status;
//
//     /* Get the byte */
//     Status = CpGetByte(&KdComPort, OutByte, TRUE, FALSE);
//     switch (Status)
//     {
//         case CP_GET_SUCCESS:
//             return KDP_PACKET_RECEIVED;
//
//         case CP_GET_NODATA:
//             return KDP_PACKET_TIMEOUT;
//
//         case CP_GET_ERROR:
//         default:
//             return KDP_PACKET_RESEND;
//     }
// }
//
// KDP_STATUS
// NTAPI
// KdpPollBreakIn(VOID)
// {
//     KDP_STATUS KdStatus;
//     UCHAR Byte;
//
//     KdStatus = KdpPollByte(&Byte);
//     if ((KdStatus == KDP_PACKET_RECEIVED) && (Byte == BREAKIN_PACKET_BYTE))
//     {
//         return KDP_PACKET_RECEIVED;
//     }
//     return KDP_PACKET_TIMEOUT;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
