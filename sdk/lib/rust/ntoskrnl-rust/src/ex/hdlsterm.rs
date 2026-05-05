use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    PHEADLESS_GLOBALS, PHEADLESS_LOADER_BLOCK, PLOADER_PARAMETER_BLOCK,
    PHEADLESS_CMD_ENABLE_TERMINAL, PHEADLESS_CMD_PUT_STRING,
    PHEADLESS_CMD_SET_COLOR, PHEADLESS_CMD_POSITION_CURSOR,
    PHEADLESS_RSP_GET_BYTE, PHEADLESS_RSP_QUERY_INFO,
    InbvPortInitialize, InbvPortEnableFifo, InbvPortTerminate,
    InbvPortPutByte, InbvPortPollOnly, InbvPortGetByte,
    HEADLESS_CMD, _HEADLESS_CMD_HeadlessCmdEnableTerminal,
    _HEADLESS_CMD_HeadlessCmdCheckForReboot, _HEADLESS_CMD_HeadlessCmdPutString,
    _HEADLESS_CMD_HeadlessCmdClearDisplay, _HEADLESS_CMD_HeadlessCmdClearToEndOfDisplay,
    _HEADLESS_CMD_HeadlessCmdClearToEndOfLine, _HEADLESS_CMD_HeadlessCmdDisplayAttributesOff,
    _HEADLESS_CMD_HeadlessCmdDisplayInverseVideo, _HEADLESS_CMD_HeadlessCmdSetColor,
    _HEADLESS_CMD_HeadlessCmdPositionCursor, _HEADLESS_CMD_HeadlessCmdTerminalPoll,
    _HEADLESS_CMD_HeadlessCmdGetByte, _HEADLESS_CMD_HeadlessCmdGetLine,
    _HEADLESS_CMD_HeadlessCmdStartBugCheck, _HEADLESS_CMD_HeadlessCmdDoBugCheckProcessing,
    _HEADLESS_CMD_HeadlessCmdQueryInformation, _HEADLESS_CMD_HeadlessCmdAddLogEntry,
    _HEADLESS_CMD_HeadlessCmdDisplayLog, _HEADLESS_CMD_HeadlessCmdSetBlueScreenData,
    _HEADLESS_CMD_HeadlessCmdSendBlueScreenData, _HEADLESS_CMD_HeadlessCmdQueryGUID,
    _HEADLESS_CMD_HeadlessCmdPutData,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// Constants not in bindgen
const HeadlessSerialPort: u32 = 1;
const SerialPortUndefined: u32 = 0;
const PCI_INVALID_VENDORID: u16 = 0xFFFF;

// Note: HeadlessSerialPort and SerialPortUndefined are defined above as local constants

const HeadlessCmdEnableTerminal: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdEnableTerminal;
const HeadlessCmdCheckForReboot: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdCheckForReboot;
const HeadlessCmdPutString: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdPutString;
const HeadlessCmdClearDisplay: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdClearDisplay;
const HeadlessCmdClearToEndOfDisplay: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdClearToEndOfDisplay;
const HeadlessCmdClearToEndOfLine: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdClearToEndOfLine;
const HeadlessCmdDisplayAttributesOff: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdDisplayAttributesOff;
const HeadlessCmdDisplayInverseVideo: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdDisplayInverseVideo;
const HeadlessCmdSetColor: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdSetColor;
const HeadlessCmdPositionCursor: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdPositionCursor;
const HeadlessCmdTerminalPoll: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdTerminalPoll;
const HeadlessCmdGetByte: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdGetByte;
const HeadlessCmdGetLine: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdGetLine;
const HeadlessCmdStartBugCheck: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdStartBugCheck;
const HeadlessCmdDoBugCheckProcessing: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdDoBugCheckProcessing;
const HeadlessCmdQueryInformation: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdQueryInformation;
const HeadlessCmdAddLogEntry: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdAddLogEntry;
const HeadlessCmdDisplayLog: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdDisplayLog;
const HeadlessCmdSetBlueScreenData: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdSetBlueScreenData;
const HeadlessCmdSendBlueScreenData: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdSendBlueScreenData;
const HeadlessCmdQueryGUID: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdQueryGUID;
const HeadlessCmdPutData: HEADLESS_CMD = _HEADLESS_CMD_HeadlessCmdPutData;

// HeadlessSerialPort and SerialPortUndefined defined above

const ANSI_NULL: u8 = 0;

#[no_mangle]
pub static mut HeadlessGlobals: PHEADLESS_GLOBALS = core::ptr::null_mut();

#[inline(always)]
unsafe fn HdlspAcquireGlobalLock() -> KIRQL {
    let mut OldIrql: KIRQL;

    if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.InBugCheck() == 0 {
        OldIrql = 0;
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*HeadlessGlobals).SpinLock) as PKSPIN_LOCK,
            &mut OldIrql as *mut KIRQL,
        );
    } else {
        OldIrql = 0xFF;
    }

    OldIrql
}

#[inline(always)]
unsafe fn HdlspReleaseGlobalLock(OldIrql: KIRQL) {
    if OldIrql != 0xFF {
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*HeadlessGlobals).SpinLock) as PKSPIN_LOCK,
            OldIrql,
        );
    }
}

unsafe fn HdlspSendStringAtBaud(mut String: *mut UCHAR) {
    while *String != ANSI_NULL {
        InbvPortPutByte((*HeadlessGlobals).TerminalPort, *String);
        String = String.add(1);
    }
}

unsafe fn HdlspPutData(Data: *mut UCHAR, DataSize: ULONG) {
    for i in 0..DataSize {
        InbvPortPutByte((*HeadlessGlobals).TerminalPort, *Data.add(i as usize));
    }
}

unsafe fn HdlspPutString(mut String: *mut UCHAR) {
    let mut Dest = (*HeadlessGlobals).TmpBuffer;
    let mut Char: UCHAR;

    while *String != ANSI_NULL {
        if Dest >= (*HeadlessGlobals).TmpBuffer.add(79) {
            *(*HeadlessGlobals).TmpBuffer.add(79) = ANSI_NULL;
            HdlspSendStringAtBaud((*HeadlessGlobals).TmpBuffer);
            Dest = (*HeadlessGlobals).TmpBuffer;
        } else {
            Char = *String;
            if (Char & 0x80) != 0 {
                match Char {
                    0xB0 | 0xB3 | 0xBA => {
                        Char = b'|';
                    }
                    0xB1 | 0xDC | 0xDD | 0xDE | 0xDF => {
                        Char = b'%';
                    }
                    0xB2 | 0xDB => {
                        Char = b'#';
                    }
                    0xA9 | 0xAA | 0xBB | 0xBC | 0xBF | 0xC0 | 0xC8 | 0xC9 | 0xD9 | 0xDA => {
                        Char = b'+';
                    }
                    0xC4 => {
                        Char = b'-';
                    }
                    0xCD => {
                        Char = b'=';
                    }
                    _ => {}
                }
            }

            if (Char & 0x80) != 0 {
                panic!("Unicode not implemented");
            } else {
                *Dest = Char;
                Dest = Dest.add(1);
            }

            String = String.add(1);
        }
    }

    *Dest = ANSI_NULL;
    HdlspSendStringAtBaud((*HeadlessGlobals).TmpBuffer);
}

unsafe fn HdlspEnableTerminal(Enable: BOOLEAN) -> NTSTATUS {
    if Enable != 0
        && (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalEnabled() == 0
        && !((*HeadlessGlobals).IsMMIODevice != 0 && (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.InBugCheck() != 0)
    {
        let te = InbvPortInitialize(
            (*HeadlessGlobals).TerminalBaudRate,
            (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalPortNumber(),
            (*HeadlessGlobals).TerminalPortAddress,
            core::ptr::addr_of_mut!((*HeadlessGlobals).TerminalPort),
            (*HeadlessGlobals).IsMMIODevice,
        );
        (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_TerminalEnabled(te as ULONG);
        if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalEnabled() == 0 {
            return STATUS_UNSUCCESSFUL;
        }

        HdlspSendStringAtBaud(b"\x1B[2J\0".as_ptr() as *mut UCHAR);
        HdlspSendStringAtBaud(b"\x1B[H\0".as_ptr() as *mut UCHAR);

        InbvPortEnableFifo((*HeadlessGlobals).TerminalPort, 1u8);
    } else if Enable == 0 {
        InbvPortTerminate((*HeadlessGlobals).TerminalPort);
        (*HeadlessGlobals).TerminalPort = 0;
        (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_TerminalEnabled(0);
    }

    STATUS_SUCCESS
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn HeadlessInit(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let HeadlessBlock: PHEADLESS_LOADER_BLOCK;

    HeadlessBlock = (*(*LoaderBlock).Extension).HeadlessLoaderBlock;
    if HeadlessBlock.is_null() {
        return;
    }

    if (*HeadlessBlock).PortNumber > 4 && (*HeadlessBlock).UsedBiosSettings != 0 {
        return;
    }

    HeadlessGlobals = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<ntkrnl_sys::ddk::_HEADLESS_GLOBALS>(),
        u32::from_be_bytes(*b"Hdls"),
    ) as PHEADLESS_GLOBALS;
    if HeadlessGlobals.is_null() {
        return;
    }

    RtlZeroMemory(
        HeadlessGlobals as PVOID,
        core::mem::size_of::<ntkrnl_sys::ddk::_HEADLESS_GLOBALS>(),
    );
    (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_TerminalPortNumber((*HeadlessBlock).PortNumber as ULONG);
    (*HeadlessGlobals).TerminalPortAddress = (*HeadlessBlock).PortAddress;
    (*HeadlessGlobals).TerminalBaudRate = (*HeadlessBlock).BaudRate;
    (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_TerminalParity((*HeadlessBlock).Parity as ULONG);
    (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_TerminalStopBits((*HeadlessBlock).StopBits as ULONG);
    (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_UsedBiosSettings((*HeadlessBlock).UsedBiosSettings as ULONG);
    (*HeadlessGlobals).IsMMIODevice = (*HeadlessBlock).IsMMIODevice;
    (*HeadlessGlobals).TerminalType = (*HeadlessBlock).TerminalType;
    (*HeadlessGlobals).SystemGUID = (*HeadlessBlock).SystemGUID;

    if (*HeadlessGlobals).IsMMIODevice != 0 {
        (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_IsNonLegacyDevice(1);
    }

    if (*HeadlessBlock).PciDeviceId != PCI_INVALID_VENDORID {
    }

    (*HeadlessGlobals).TmpBuffer = ExAllocatePoolWithTag(NonPagedPool, 80, u32::from_be_bytes(*b"Hdls")) as *mut UCHAR;
    if (*HeadlessGlobals).TmpBuffer.is_null() {
        return;
    }

    if (*HeadlessGlobals).TerminalBaudRate == 9600 {
    }

    HdlspEnableTerminal(1u8);
}

unsafe fn HdlspDispatch(
    Command: HEADLESS_CMD,
    InputBuffer: PVOID,
    InputBufferSize: SIZE_T,
    OutputBuffer: PVOID,
    OutputBufferSize: PSIZE_T,
) -> NTSTATUS {
    let OldIrql: KIRQL;
    let mut Status: NTSTATUS = STATUS_NOT_IMPLEMENTED;
    let HeadlessInfo: PHEADLESS_RSP_QUERY_INFO;
    let PutString: PHEADLESS_CMD_PUT_STRING;
    let EnableTerminal: PHEADLESS_CMD_ENABLE_TERMINAL;
    let SetColor: PHEADLESS_CMD_SET_COLOR;
    let CursorPos: PHEADLESS_CMD_POSITION_CURSOR;
    let GetByte: PHEADLESS_RSP_GET_BYTE;
    let mut DataBuffer: [UCHAR; 80] = [0; 80];

    if Command != HeadlessCmdAddLogEntry
        && Command != HeadlessCmdStartBugCheck
        && Command != HeadlessCmdSendBlueScreenData
        && Command != HeadlessCmdDoBugCheckProcessing
    {
        OldIrql = HdlspAcquireGlobalLock();

        if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.ProcessingCmd() != 0 {
            HdlspReleaseGlobalLock(OldIrql);
            return STATUS_UNSUCCESSFUL;
        }

        (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_ProcessingCmd(1);
        HdlspReleaseGlobalLock(OldIrql);
    }

    match Command {
        cmd if cmd == HeadlessCmdEnableTerminal => {
            if InputBuffer.is_null()
                || InputBufferSize
                    != core::mem::size_of::<ntkrnl_sys::ddk::_HEADLESS_CMD_ENABLE_TERMINAL>()
            {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                EnableTerminal = InputBuffer as PHEADLESS_CMD_ENABLE_TERMINAL;
                Status = HdlspEnableTerminal((*EnableTerminal).Enable);
            }
        }

        cmd if cmd == HeadlessCmdCheckForReboot => {}

        cmd if cmd == HeadlessCmdPutString => {
            if InputBuffer.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalEnabled() != 0 {
                    PutString = InputBuffer as PHEADLESS_CMD_PUT_STRING;
                    HdlspPutString((*PutString).String.as_ptr() as *mut UCHAR);
                }

                Status = STATUS_SUCCESS;
            }
        }

        cmd if cmd == HeadlessCmdClearDisplay
            || cmd == HeadlessCmdClearToEndOfDisplay
            || cmd == HeadlessCmdClearToEndOfLine
            || cmd == HeadlessCmdDisplayAttributesOff
            || cmd == HeadlessCmdDisplayInverseVideo
            || cmd == HeadlessCmdSetColor
            || cmd == HeadlessCmdPositionCursor =>
        {
            Status = STATUS_SUCCESS;

            if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalEnabled() != 0 {
                let mut CommandStr: *mut UCHAR = core::ptr::null_mut();

                if Command == HeadlessCmdClearDisplay {
                    CommandStr = b"\x1B[2J\0".as_ptr() as *mut UCHAR;
                } else if Command == HeadlessCmdClearToEndOfDisplay {
                    CommandStr = b"\x1B[0J\0".as_ptr() as *mut UCHAR;
                } else if Command == HeadlessCmdClearToEndOfLine {
                    CommandStr = b"\x1B[0K\0".as_ptr() as *mut UCHAR;
                } else if Command == HeadlessCmdDisplayAttributesOff {
                    CommandStr = b"\x1B[0m\0".as_ptr() as *mut UCHAR;
                } else if Command == HeadlessCmdDisplayInverseVideo {
                    CommandStr = b"\x1B[7m\0".as_ptr() as *mut UCHAR;
                } else if Command == HeadlessCmdSetColor {
                    if InputBuffer.is_null()
                        || InputBufferSize
                            != core::mem::size_of::<ntkrnl_sys::ddk::_HEADLESS_CMD_SET_COLOR>()
                    {
                        Status = STATUS_INVALID_PARAMETER;
                    } else {
                        SetColor = InputBuffer as PHEADLESS_CMD_SET_COLOR;
                        Status = crate::rtl_string_cb_printf_a!(
                            DataBuffer.as_mut_ptr(),
                            core::mem::size_of_val(&DataBuffer),
                            b"\x1B[%d;%dm",
                            (*SetColor).BkgdColor,
                            (*SetColor).TextColor,
                        );
                        if Status < 0 {
                        } else {
                            CommandStr = DataBuffer.as_mut_ptr();
                        }
                    }
                } else {
                    if InputBuffer.is_null()
                        || InputBufferSize
                            != core::mem::size_of::<ntkrnl_sys::ddk::_HEADLESS_CMD_POSITION_CURSOR>(
                            )
                    {
                        Status = STATUS_INVALID_PARAMETER;
                    } else {
                        CursorPos = InputBuffer as PHEADLESS_CMD_POSITION_CURSOR;
                        Status = crate::rtl_string_cb_printf_a!(
                            DataBuffer.as_mut_ptr(),
                            core::mem::size_of_val(&DataBuffer),
                            b"\x1B[%d;%dH",
                            (*CursorPos).CursorRow + 1,
                            (*CursorPos).CursorCol + 1,
                        );
                        if Status < 0 {
                        } else {
                            CommandStr = DataBuffer.as_mut_ptr();
                        }
                    }
                }

                if Status >= 0 {
                    HdlspSendStringAtBaud(CommandStr);
                }
            }
        }

        cmd if cmd == HeadlessCmdTerminalPoll => {}

        cmd if cmd == HeadlessCmdGetByte => {
            if OutputBuffer.is_null()
                || OutputBufferSize.is_null()
                || *OutputBufferSize
                    < core::mem::size_of::<ntkrnl_sys::ddk::_HEADLESS_RSP_GET_BYTE>()
            {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                GetByte = OutputBuffer as PHEADLESS_RSP_GET_BYTE;
                if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalEnabled() != 0 {
                    if InbvPortPollOnly((*HeadlessGlobals).TerminalPort) != 0 {
                        InbvPortGetByte(
                            (*HeadlessGlobals).TerminalPort,
                            core::ptr::addr_of_mut!((*GetByte).Value),
                        );
                    } else {
                        (*GetByte).Value = 0;
                    }
                } else {
                    (*GetByte).Value = 0;
                }

                Status = STATUS_SUCCESS;
            }
        }

        cmd if cmd == HeadlessCmdGetLine => {}

        cmd if cmd == HeadlessCmdStartBugCheck => {
            (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_InBugCheck(1);
            (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_ProcessingCmd(0);
            Status = STATUS_SUCCESS;
        }

        cmd if cmd == HeadlessCmdDoBugCheckProcessing => {}

        cmd if cmd == HeadlessCmdQueryInformation => {
            if OutputBuffer.is_null()
                || OutputBufferSize.is_null()
                || *OutputBufferSize
                    < core::mem::size_of::<ntkrnl_sys::ddk::_HEADLESS_RSP_QUERY_INFO>()
            {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                HeadlessInfo = OutputBuffer as PHEADLESS_RSP_QUERY_INFO;
                (*HeadlessInfo).PortType = HeadlessSerialPort;
                (*HeadlessInfo).__bindgen_anon_1.Serial.TerminalAttached = 1u8;
                (*HeadlessInfo).__bindgen_anon_1.Serial.UsedBiosSettings =
                    if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.UsedBiosSettings() != 0 {
                        1u8
                    } else {
                        0u8
                    };
                (*HeadlessInfo).__bindgen_anon_1.Serial.TerminalBaudRate =
                    (*HeadlessGlobals).TerminalBaudRate;
                (*HeadlessInfo).__bindgen_anon_1.Serial.TerminalType =
                    (*HeadlessGlobals).TerminalType;

                if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalPortNumber() >= 1
                    || (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.UsedBiosSettings() != 0
                {
                    (*HeadlessInfo).__bindgen_anon_1.Serial.TerminalPort =
                        (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalPortNumber();
                    (*HeadlessInfo).__bindgen_anon_1.Serial.TerminalPortBaseAddress =
                        (*HeadlessGlobals).TerminalPortAddress;
                } else {
                    (*HeadlessInfo).__bindgen_anon_1.Serial.TerminalPort = SerialPortUndefined;
                    (*HeadlessInfo).__bindgen_anon_1.Serial.TerminalPortBaseAddress = core::ptr::null_mut();
                }

                Status = STATUS_SUCCESS;
            }
        }

        cmd if cmd == HeadlessCmdAddLogEntry => {}
        cmd if cmd == HeadlessCmdDisplayLog => {}

        cmd if cmd == HeadlessCmdSetBlueScreenData => {
            if InputBuffer.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                Status = STATUS_SUCCESS;
            }
        }

        cmd if cmd == HeadlessCmdSendBlueScreenData => {}

        cmd if cmd == HeadlessCmdQueryGUID => {}

        cmd if cmd == HeadlessCmdPutData => {
            if InputBuffer.is_null() || InputBufferSize == 0 {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                if (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.TerminalEnabled() != 0 {
                    PutString = InputBuffer as PHEADLESS_CMD_PUT_STRING;
                    HdlspPutData((*PutString).String.as_ptr() as *mut UCHAR, InputBufferSize as ULONG);
                }

                Status = STATUS_SUCCESS;
            }
        }

        _ => {}
    }

    if Command != HeadlessCmdAddLogEntry
        && Command != HeadlessCmdStartBugCheck
        && Command != HeadlessCmdSendBlueScreenData
        && Command != HeadlessCmdDoBugCheckProcessing
    {
        (*HeadlessGlobals).__bindgen_anon_1.__bindgen_anon_1.set_ProcessingCmd(0);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn HeadlessDispatch(
    Command: HEADLESS_CMD,
    InputBuffer: PVOID,
    InputBufferSize: SIZE_T,
    OutputBuffer: PVOID,
    OutputBufferSize: PSIZE_T,
) -> NTSTATUS {
    let InputBuffer = opaque_ptr(InputBuffer);
    let OutputBuffer = opaque_ptr(OutputBuffer);
    let OutputBufferSize = opaque_ptr(OutputBufferSize);
    if HeadlessGlobals.is_null() {
        if Command == HeadlessCmdEnableTerminal {
            return STATUS_UNSUCCESSFUL;
        }

        if Command == HeadlessCmdQueryInformation
            || Command == HeadlessCmdGetByte
            || Command == HeadlessCmdGetLine
            || Command == HeadlessCmdCheckForReboot
            || Command == HeadlessCmdTerminalPoll
        {
            if OutputBuffer.is_null() || OutputBufferSize.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            RtlZeroMemory(OutputBuffer, *OutputBufferSize);
        }

        return STATUS_SUCCESS;
    }

    HdlspDispatch(Command, InputBuffer, InputBufferSize, OutputBuffer, OutputBufferSize)
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/ex/hdlsterm.c
//  * PURPOSE:         Headless Terminal Support
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// PHEADLESS_GLOBALS HeadlessGlobals;
//
// /* FUNCTIONS ******************************************************************/
//
// FORCEINLINE
// KIRQL
// HdlspAcquireGlobalLock(VOID)
// {
//     KIRQL OldIrql;
//
//     /* Don't acquire the lock if we are bugchecking */
//     if (!HeadlessGlobals->InBugCheck)
//     {
//         KeAcquireSpinLock(&HeadlessGlobals->SpinLock, &OldIrql);
//     }
//     else
//     {
//         OldIrql = 0xFF;
//     }
//
//     return OldIrql;
// }
//
// FORCEINLINE
// VOID
// HdlspReleaseGlobalLock(IN KIRQL OldIrql)
// {
//     /* Only release the lock if we aren't bugchecking */
//     if (OldIrql != 0xFF)
//     {
//         KeReleaseSpinLock(&HeadlessGlobals->SpinLock, OldIrql);
//     }
//     else
//     {
//         ASSERT(HeadlessGlobals->InBugCheck == TRUE);
//     }
// }
//
// VOID
// NTAPI
// HdlspSendStringAtBaud(IN PUCHAR String)
// {
//     /* Send every byte */
//     while (*String != ANSI_NULL)
//     {
//         InbvPortPutByte(HeadlessGlobals->TerminalPort, *String++);
//     }
// }
//
// VOID
// NTAPI
// HdlspPutData(IN PUCHAR Data,
//              IN ULONG DataSize)
// {
//     ULONG i;
//     for (i = 0; i < DataSize; i++)
//     {
//         InbvPortPutByte(HeadlessGlobals->TerminalPort, Data[i]);
//     }
// }
//
// VOID
// NTAPI
// HdlspPutString(IN PUCHAR String)
// {
//     PUCHAR Dest = HeadlessGlobals->TmpBuffer;
//     UCHAR Char = 0;
//
//     /* Scan each character */
//     while (*String != ANSI_NULL)
//     {
//         /* Check for rotate, send existing buffer and restart from where we are */
//         if (Dest >= &HeadlessGlobals->TmpBuffer[79])
//         {
//             HeadlessGlobals->TmpBuffer[79] = ANSI_NULL;
//             HdlspSendStringAtBaud(HeadlessGlobals->TmpBuffer);
//             Dest = HeadlessGlobals->TmpBuffer;
//         }
//         else
//         {
//             /* Get the current character and check for special graphical chars */
//             Char = *String;
//             if (Char & 0x80)
//             {
//                 switch (Char)
//                 {
//                     case 0xB0: case 0xB3: case 0xBA:
//                         Char = '|';
//                         break;
//                     case 0xB1: case 0xDC: case 0xDD: case 0xDE: case 0xDF:
//                         Char = '%';
//                         break;
//                     case 0xB2: case 0xDB:
//                         Char = '#';
//                         break;
//                     case 0xA9: case 0xAA: case 0xBB: case 0xBC: case 0xBF:
//                     case 0xC0: case 0xC8: case 0xC9: case 0xD9: case 0xDA:
//                         Char = '+';
//                         break;
//                     case 0xC4:
//                         Char = '-';
//                         break;
//                     case 0xCD:
//                         Char = '=';
//                         break;
//                 }
//             }
//
//             /* Anything else must be Unicode */
//             if (Char & 0x80)
//             {
//                 /* Can't do Unicode yet */
//                 UNIMPLEMENTED;
//             }
//             else
//             {
//                 /* Add the modified char to the temporary buffer */
//                 *Dest++ = Char;
//             }
//
//             /* Check the next char */
//             String++;
//         }
//     }
//
//     /* Finish and send */
//     *Dest = ANSI_NULL;
//     HdlspSendStringAtBaud(HeadlessGlobals->TmpBuffer);
// }
//
// NTSTATUS
// NTAPI
// HdlspEnableTerminal(IN BOOLEAN Enable)
// {
//     /* Enable if requested, as long as this isn't a PCI serial port crashing */
//     if ((Enable) &&
//         !(HeadlessGlobals->TerminalEnabled) &&
//         !((HeadlessGlobals->IsMMIODevice) && (HeadlessGlobals->InBugCheck)))
//     {
//         /* Initialize the COM port with cportlib */
//         HeadlessGlobals->TerminalEnabled = InbvPortInitialize(HeadlessGlobals->TerminalBaudRate,
//                                                               HeadlessGlobals->TerminalPortNumber,
//                                                               HeadlessGlobals->TerminalPortAddress,
//                                                               &HeadlessGlobals->TerminalPort,
//                                                               HeadlessGlobals->IsMMIODevice);
//         if (!HeadlessGlobals->TerminalEnabled)
//         {
//             DPRINT1("Failed to initialize port through cportlib\n");
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* Cleanup the screen and reset the cursor */
//         HdlspSendStringAtBaud((PUCHAR)"\x1B[2J");
//         HdlspSendStringAtBaud((PUCHAR)"\x1B[H");
//
//         /* Enable FIFO */
//         InbvPortEnableFifo(HeadlessGlobals->TerminalPort, TRUE);
//     }
//     else if (!Enable)
//     {
//         /* Specific case when headless is being disabled */
//         InbvPortTerminate(HeadlessGlobals->TerminalPort);
//         HeadlessGlobals->TerminalPort = 0;
//         HeadlessGlobals->TerminalEnabled = FALSE;
//     }
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// HeadlessInit(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PHEADLESS_LOADER_BLOCK HeadlessBlock;
//
//     /* Only initialize further if the loader found EMS enabled */
//     HeadlessBlock = LoaderBlock->Extension->HeadlessLoaderBlock;
//     if (!HeadlessBlock) return;
//
//     /* Ignore invalid EMS settings */
//     if ((HeadlessBlock->PortNumber > 4) && (HeadlessBlock->UsedBiosSettings)) return;
//
//     /* Allocate the global headless data */
//     HeadlessGlobals = ExAllocatePoolWithTag(NonPagedPool,
//                                             sizeof(*HeadlessGlobals),
//                                             'sldH');
//     if (!HeadlessGlobals) return;
//
//     /* Zero and copy loader data */
//     RtlZeroMemory(HeadlessGlobals, sizeof(*HeadlessGlobals));
//     HeadlessGlobals->TerminalPortNumber = HeadlessBlock->PortNumber;
//     HeadlessGlobals->TerminalPortAddress = HeadlessBlock->PortAddress;
//     HeadlessGlobals->TerminalBaudRate = HeadlessBlock->BaudRate;
//     HeadlessGlobals->TerminalParity = HeadlessBlock->Parity;
//     HeadlessGlobals->TerminalStopBits = HeadlessBlock->StopBits;
//     HeadlessGlobals->UsedBiosSettings = HeadlessBlock->UsedBiosSettings;
//     HeadlessGlobals->IsMMIODevice = HeadlessBlock->IsMMIODevice;
//     HeadlessGlobals->TerminalType = HeadlessBlock->TerminalType;
//     HeadlessGlobals->SystemGUID = HeadlessBlock->SystemGUID;
//     DPRINT1("EMS on Port %lu (0x%p) at %lu bps\n",
//              HeadlessGlobals->TerminalPortNumber,
//              HeadlessGlobals->TerminalPortAddress,
//              HeadlessGlobals->TerminalBaudRate);
//
//     /* These two are opposites of each other */
//     if (HeadlessGlobals->IsMMIODevice) HeadlessGlobals->IsNonLegacyDevice = TRUE;
//
//     /* Check for a PCI device, warn that this isn't supported */
//     if (HeadlessBlock->PciDeviceId != PCI_INVALID_VENDORID)
//     {
//         DPRINT1("PCI Serial Ports not supported\n");
//     }
//
//     /* Log entries are not yet supported */
//     DPRINT1("FIXME: No Headless logging support\n");
//
//     /* Allocate temporary buffer */
//     HeadlessGlobals->TmpBuffer = ExAllocatePoolWithTag(NonPagedPool, 80, 'sldH');
//     if (!HeadlessGlobals->TmpBuffer) return;
//
//     /* Windows seems to apply some special hacks for 9600 bps */
//     if (HeadlessGlobals->TerminalBaudRate == 9600)
//     {
//         DPRINT1("Please use other baud rate than 9600bps for now\n");
//     }
//
//     /* Enable the terminal */
//     HdlspEnableTerminal(TRUE);
// }
//
// NTSTATUS
// NTAPI
// HdlspDispatch(IN HEADLESS_CMD Command,
//               IN PVOID InputBuffer,
//               IN SIZE_T InputBufferSize,
//               OUT PVOID OutputBuffer,
//               OUT PSIZE_T OutputBufferSize)
// {
//     KIRQL OldIrql;
//     NTSTATUS Status = STATUS_NOT_IMPLEMENTED;
//     PHEADLESS_RSP_QUERY_INFO HeadlessInfo;
//     PHEADLESS_CMD_PUT_STRING PutString;
//     PHEADLESS_CMD_ENABLE_TERMINAL EnableTerminal;
//     PHEADLESS_CMD_SET_COLOR SetColor;
//     PHEADLESS_CMD_POSITION_CURSOR CursorPos;
//     PHEADLESS_RSP_GET_BYTE GetByte;
//     UCHAR DataBuffer[80];
//
//     ASSERT(HeadlessGlobals != NULL);
//     // ASSERT(HeadlessGlobals->PageLockHandle != NULL);
//
//     /* Ignore non-reentrant commands */
//     if ((Command != HeadlessCmdAddLogEntry) &&
//         (Command != HeadlessCmdStartBugCheck) &&
//         (Command != HeadlessCmdSendBlueScreenData) &&
//         (Command != HeadlessCmdDoBugCheckProcessing))
//     {
//         OldIrql = HdlspAcquireGlobalLock();
//
//         if (HeadlessGlobals->ProcessingCmd)
//         {
//             HdlspReleaseGlobalLock(OldIrql);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* Don't allow these commands next time */
//         HeadlessGlobals->ProcessingCmd = TRUE;
//         HdlspReleaseGlobalLock(OldIrql);
//     }
//
//     /* Handle each command */
//     switch (Command)
//     {
//         case HeadlessCmdEnableTerminal:
//         {
//             /* Make sure the caller passed valid data */
//             if (!(InputBuffer) ||
//                 (InputBufferSize != sizeof(*EnableTerminal)))
//             {
//                 DPRINT1("Invalid buffer\n");
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Go and enable it */
//             EnableTerminal = InputBuffer;
//             Status = HdlspEnableTerminal(EnableTerminal->Enable);
//             break;
//         }
//
//         case HeadlessCmdCheckForReboot:
//             break;
//
//         case HeadlessCmdPutString:
//         {
//             /* Validate the existence of an input buffer */
//             if (!InputBuffer)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Terminal should be on */
//             if (HeadlessGlobals->TerminalEnabled)
//             {
//                 /* Print each byte in the string making sure VT100 chars are used */
//                 PutString = InputBuffer;
//                 HdlspPutString(PutString->String);
//             }
//
//             /* Return success either way */
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case HeadlessCmdClearDisplay:
//         case HeadlessCmdClearToEndOfDisplay:
//         case HeadlessCmdClearToEndOfLine:
//         case HeadlessCmdDisplayAttributesOff:
//         case HeadlessCmdDisplayInverseVideo:
//         case HeadlessCmdSetColor:
//         case HeadlessCmdPositionCursor:
//         {
//             /* By default return success */
//             Status = STATUS_SUCCESS;
//
//             /* Send the VT100 commands only if the terminal is enabled */
//             if (HeadlessGlobals->TerminalEnabled)
//             {
//                 PUCHAR CommandStr = NULL;
//
//                 if (Command == HeadlessCmdClearDisplay)
//                     CommandStr = (PUCHAR)"\x1B[2J";
//                 else if (Command == HeadlessCmdClearToEndOfDisplay)
//                     CommandStr = (PUCHAR)"\x1B[0J";
//                 else if (Command == HeadlessCmdClearToEndOfLine)
//                     CommandStr = (PUCHAR)"\x1B[0K";
//                 else if (Command == HeadlessCmdDisplayAttributesOff)
//                     CommandStr = (PUCHAR)"\x1B[0m";
//                 else if (Command == HeadlessCmdDisplayInverseVideo)
//                     CommandStr = (PUCHAR)"\x1B[7m";
//                 else if (Command == HeadlessCmdSetColor)
//                 {
//                     /* Make sure the caller passed valid data */
//                     if (!InputBuffer ||
//                         (InputBufferSize != sizeof(*SetColor)))
//                     {
//                         DPRINT1("Invalid buffer\n");
//                         Status = STATUS_INVALID_PARAMETER;
//                         break;
//                     }
//
//                     SetColor = InputBuffer;
//                     Status = RtlStringCbPrintfA((PCHAR)DataBuffer, sizeof(DataBuffer),
//                                                 "\x1B[%d;%dm",
//                                                 SetColor->BkgdColor,
//                                                 SetColor->TextColor);
//                     if (!NT_SUCCESS(Status)) break;
//
//                     CommandStr = DataBuffer;
//                 }
//                 else // if (Command == HeadlessCmdPositionCursor)
//                 {
//                     /* Make sure the caller passed valid data */
//                     if (!InputBuffer ||
//                         (InputBufferSize != sizeof(*CursorPos)))
//                     {
//                         DPRINT1("Invalid buffer\n");
//                         Status = STATUS_INVALID_PARAMETER;
//                         break;
//                     }
//
//                     CursorPos = InputBuffer;
//                     /* Cursor position is 1-based */
//                     Status = RtlStringCbPrintfA((PCHAR)DataBuffer, sizeof(DataBuffer),
//                                                 "\x1B[%d;%dH",
//                                                 CursorPos->CursorRow + 1,
//                                                 CursorPos->CursorCol + 1);
//                     if (!NT_SUCCESS(Status)) break;
//
//                     CommandStr = DataBuffer;
//                 }
//
//                 /* Send the command */
//                 HdlspSendStringAtBaud(CommandStr);
//             }
//
//             break;
//         }
//
//         case HeadlessCmdTerminalPoll:
//             break;
//
//         case HeadlessCmdGetByte:
//         {
//             /* Make sure the caller passed valid data */
//             if (!(OutputBuffer) ||
//                 !(OutputBufferSize) ||
//                 (*OutputBufferSize < sizeof(*GetByte)))
//             {
//                 DPRINT1("Invalid buffer\n");
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Make sure the terminal is enabled */
//             GetByte = OutputBuffer;
//             if (HeadlessGlobals->TerminalEnabled)
//             {
//                 /* Poll if something is on the wire */
//                 if (InbvPortPollOnly(HeadlessGlobals->TerminalPort))
//                 {
//                     /* If so, read it */
//                     InbvPortGetByte(HeadlessGlobals->TerminalPort,
//                                     &GetByte->Value);
//                 }
//                 else
//                 {
//                     /* Nothing is there, return 0 */
//                     GetByte->Value = 0;
//                 }
//             }
//             else
//             {
//                 /* Otherwise return nothing */
//                 GetByte->Value = 0;
//             }
//
//             /* Return success either way */
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case HeadlessCmdGetLine:
//             break;
//
//         case HeadlessCmdStartBugCheck:
//         {
//             HeadlessGlobals->InBugCheck = TRUE;
//             HeadlessGlobals->ProcessingCmd = FALSE;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case HeadlessCmdDoBugCheckProcessing:
//             break;
//
//         case HeadlessCmdQueryInformation:
//         {
//             /* Make sure the caller passed valid data */
//             if (!(OutputBuffer) ||
//                 !(OutputBufferSize) ||
//                 (*OutputBufferSize < sizeof(*HeadlessInfo)))
//             {
//                 DPRINT1("Invalid buffer\n");
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* If we got here, headless is enabled -- we know this much */
//             HeadlessInfo = OutputBuffer;
//             HeadlessInfo->PortType = HeadlessSerialPort;
//             HeadlessInfo->Serial.TerminalAttached = TRUE;
//             HeadlessInfo->Serial.UsedBiosSettings = HeadlessGlobals->UsedBiosSettings != 0;
//             HeadlessInfo->Serial.TerminalBaudRate = HeadlessGlobals->TerminalBaudRate;
//             HeadlessInfo->Serial.TerminalType = HeadlessGlobals->TerminalType;
//
//             /* Now check on what port/baud it's enabled on */
//             if ((HeadlessGlobals->TerminalPortNumber >= 1) ||
//                 (HeadlessGlobals->UsedBiosSettings))
//             {
//                 /* Get the EMS information */
//                 HeadlessInfo->Serial.TerminalPort = HeadlessGlobals->
//                                                     TerminalPortNumber;
//                 HeadlessInfo->Serial.TerminalPortBaseAddress = HeadlessGlobals->
//                                                                TerminalPortAddress;
//             }
//             else
//             {
//                 /* We don't know for sure */
//                 HeadlessInfo->Serial.TerminalPort = SerialPortUndefined;
//                 HeadlessInfo->Serial.TerminalPortBaseAddress = 0;
//             }
//
//             /* All done */
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case HeadlessCmdAddLogEntry:
//             break;
//         case HeadlessCmdDisplayLog:
//             break;
//
//         case HeadlessCmdSetBlueScreenData:
//         {
//             /* Validate the existence of an input buffer */
//             if (!InputBuffer)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Lie so that we can get Hdl bringup a little bit further */
//             UNIMPLEMENTED;
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case HeadlessCmdSendBlueScreenData:
//             // TODO: Send XML description of bugcheck.
//             // InputBuffer points to the BugCheckCode.
//             break;
//
//         case HeadlessCmdQueryGUID:
//             break;
//
//         case HeadlessCmdPutData:
//         {
//             /* Validate the existence of an input buffer */
//             if (!(InputBuffer) || !(InputBufferSize))
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Terminal should be on */
//             if (HeadlessGlobals->TerminalEnabled)
//             {
//                 /* Print each byte in the string making sure VT100 chars are used */
//                 PutString = InputBuffer;
//                 HdlspPutData(PutString->String, InputBufferSize);
//             }
//
//             /* Return success either way */
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         default:
//             break;
//     }
//
//     /* Unset processing state */
//     if ((Command != HeadlessCmdAddLogEntry) &&
//         (Command != HeadlessCmdStartBugCheck) &&
//         (Command != HeadlessCmdSendBlueScreenData) &&
//         (Command != HeadlessCmdDoBugCheckProcessing))
//     {
//         ASSERT(HeadlessGlobals->ProcessingCmd == TRUE);
//         HeadlessGlobals->ProcessingCmd = FALSE;
//     }
//
//     /* All done */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// HeadlessDispatch(IN HEADLESS_CMD Command,
//                  IN PVOID InputBuffer,
//                  IN SIZE_T InputBufferSize,
//                  OUT PVOID OutputBuffer,
//                  OUT PSIZE_T OutputBufferSize)
// {
//     /* Check for stubs that will expect something even with headless off */
//     if (!HeadlessGlobals)
//     {
//         /* Don't allow the SAC to connect */
//         if (Command == HeadlessCmdEnableTerminal) return STATUS_UNSUCCESSFUL;
//
//         /* Send bogus reply */
//         if ((Command == HeadlessCmdQueryInformation) ||
//             (Command == HeadlessCmdGetByte) ||
//             (Command == HeadlessCmdGetLine) ||
//             (Command == HeadlessCmdCheckForReboot) ||
//             (Command == HeadlessCmdTerminalPoll))
//         {
//             if (!(OutputBuffer) || !(OutputBufferSize))
//             {
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             RtlZeroMemory(OutputBuffer, *OutputBufferSize);
//         }
//
//         return STATUS_SUCCESS;
//     }
//
//     /* Do the real work */
//     return HdlspDispatch(Command,
//                          InputBuffer,
//                          InputBufferSize,
//                          OutputBuffer,
//                          OutputBufferSize);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
