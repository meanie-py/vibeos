use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{EXCEPTION_RECORD64, KDSTATUS, PETHREAD, PSTRING, PsGetThreadId, PsGetThreadProcessId};
use core::ffi::c_void;

use crate::KDDBGPRINT;
use crate::kdgdb_h::{handle_to_gdb_pid, handle_to_gdb_tid};
use crate::kdpacket::CurrentStateChange;
// KdDebuggerNotPresent is an ntoskrnl DATA export (`@ extern` in
// ntoskrnl.spec). Cross-DLL data imports on i386/PE must go through the
// __imp_ indirection; ntkrnl_sys::globals declares it as a plain `extern
// static` (a direct ref that won't link from another module). raw-dylib
// makes rustc emit the import thunk — same pattern as the HAL bindgen.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut KdDebuggerNotPresent: BOOLEAN;
}
// Disambiguate: DbgKdExceptionStateChange lives in both globals and constants;
// DbgKdLoadSymbolsStateChange lives only in constants.
use ntkrnl_sys::constants::{DbgKdExceptionStateChange, DbgKdLoadSymbolsStateChange};

type VOID = core::ffi::c_void;

// NTSTATUS exception codes missing from ntkrnl-sys (winnt_old.h / ntstatus.h).
const STATUS_ARRAY_BOUNDS_EXCEEDED: NTSTATUS = 0xC000008Cu32 as NTSTATUS;
const STATUS_INTEGER_DIVIDE_BY_ZERO: NTSTATUS = 0xC0000094u32 as NTSTATUS;
const STATUS_INTEGER_OVERFLOW: NTSTATUS = 0xC0000095u32 as NTSTATUS;
const STATUS_STACK_OVERFLOW: NTSTATUS = 0xC00000FDu32 as NTSTATUS;

// kdgdb-internal byte I/O — NTAPI (stdcall on i386), defined in kdcom.rs.
extern "system" {
    fn KdpReceiveByte(byte: *mut u8) -> KDSTATUS;
    fn KdpSendByte(byte: u8);
}
// CRT formatting (cdecl) — links from libcntpr at final link.
extern "C" {
    fn sprintf(buffer: *mut CHAR, format: *const CHAR, ...) -> i32;
}

// ---- file-scope preamble (translated) ----
pub static hex_chars: [u8; 17] = *b"0123456789abcdef\0";
pub static mut currentChecksum: CHAR = 0;

unsafe fn exception_code_to_gdb(code: NTSTATUS, mut out: PCHAR) -> PCHAR {
    let SigVal: u8;

    match code {
        STATUS_INTEGER_DIVIDE_BY_ZERO => {
            SigVal = 8;
        }
        STATUS_SINGLE_STEP | STATUS_BREAKPOINT => {
            SigVal = 5;
        }
        STATUS_INTEGER_OVERFLOW | STATUS_ARRAY_BOUNDS_EXCEEDED => {
            SigVal = 16;
        }
        STATUS_ILLEGAL_INSTRUCTION => {
            SigVal = 4;
        }
        STATUS_STACK_OVERFLOW | STATUS_DATATYPE_MISALIGNMENT | STATUS_ACCESS_VIOLATION => {
            SigVal = 11;
        }
        _ => {
            SigVal = 7;
        }
    }
    
    *out = hex_chars[((SigVal >> 4) & 0xf) as usize] as CHAR;
    out = out.offset(1);
    *out = hex_chars[(SigVal & 0xf) as usize] as CHAR;
    out = out.offset(1);
    out
}
#[no_mangle]
pub unsafe extern "C" fn start_gdb_packet() {
    
    KdpSendByte(b'$');
    currentChecksum = 0;
}
#[no_mangle]
pub unsafe extern "C" fn send_gdb_partial_packet(Buffer: *const CHAR) {
    let mut ptr = Buffer;

    while *ptr != 0 {
        currentChecksum = currentChecksum.wrapping_add(*ptr as CHAR);
        KdpSendByte(*ptr as u8);
        ptr = ptr.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn finish_gdb_packet() -> KDSTATUS {
    let mut ack: u8 = 0;
    let Status: KDSTATUS;

    KdpSendByte(b'#');
    KdpSendByte(hex_chars[((currentChecksum >> 4) & 0xf) as usize]);
    KdpSendByte(hex_chars[(currentChecksum & 0xf) as usize]);

    Status = KdpReceiveByte(&mut ack);

    if Status != KdPacketReceived {
        KdDebuggerNotPresent = 1u8;
        return Status;
    }

    if ack != b'+' {
        return KdPacketNeedsResend;
    }

    return KdPacketReceived;
}
#[no_mangle]
pub unsafe extern "C" fn send_gdb_packet(Buffer: *const CHAR) -> KDSTATUS {
    start_gdb_packet();
    send_gdb_partial_packet(Buffer);
    return finish_gdb_packet();
}
#[no_mangle]
pub unsafe extern "C" fn send_gdb_partial_binary(
    Buffer: *const VOID,
    Length: usize
) -> ULONG {
    let mut ptr = Buffer as *const UCHAR;
    let mut Sent = Length as ULONG;
    let mut Length = Length;

    while Length > 0 {
        Length -= 1;
        let mut Byte = *ptr;
        ptr = ptr.offset(1);

        match Byte {
            0x7d | 0x23 | 0x24 | 0x2a => {
                currentChecksum = currentChecksum.wrapping_add(0x7d);
                KdpSendByte(0x7d);
                Byte ^= 0x20;
                Sent += 1;
                currentChecksum = currentChecksum.wrapping_add(Byte as CHAR);
                KdpSendByte(Byte);
            }
            _ => {
                currentChecksum = currentChecksum.wrapping_add(Byte as CHAR);
                KdpSendByte(Byte);
            }
        }
    }

    Sent
}
#[no_mangle]
pub unsafe extern "C" fn send_gdb_partial_memory(
    Buffer: *const VOID,
    Length: usize) {
    let mut ptr = Buffer as *const UCHAR;
    let mut gdb_out: [CHAR; 3] = [0; 3];
    let mut length = Length;

    gdb_out[2] = b'\0' as CHAR;

    while length > 0 {
        length -= 1;
        gdb_out[0] = hex_chars[((*ptr >> 4) & 0xf) as usize ] as CHAR;
        gdb_out[1] = hex_chars[(*ptr & 0xf) as usize ] as CHAR;
        ptr = ptr.offset(1);
        send_gdb_partial_packet(gdb_out.as_ptr());
    }
}
#[no_mangle]
pub unsafe extern "C" fn send_gdb_memory(
    Buffer: *const VOID,
    Length: usize) -> KDSTATUS {
    start_gdb_packet();
    send_gdb_partial_memory(Buffer, Length);
    return finish_gdb_packet();
}
#[no_mangle]
pub unsafe extern "C" fn gdb_send_debug_io(
    String: PSTRING,
    WithPrefix: BOOLEAN
) -> KDSTATUS {
    let mut gdb_out: [CHAR; 3] = [0; 3];
    let mut ptr = (*String).Buffer;
    let mut Length = (*String).Length;

    gdb_out[2] = 0;

    start_gdb_packet();

    if WithPrefix != 0 {
        send_gdb_partial_packet(b"O\0".as_ptr() as *const CHAR);
    }

    while Length > 0 {
        gdb_out[0] = hex_chars[((*ptr as u8 >> 4) & 0xf) as usize] as CHAR;
        gdb_out[1] = hex_chars[(*ptr as u8 & 0xf) as usize] as CHAR;
        send_gdb_partial_packet(gdb_out.as_ptr());
        ptr = ptr.offset(1);
        Length -= 1;
    }

    finish_gdb_packet()
}
#[no_mangle]
pub unsafe extern "C" fn gdb_send_exception() -> KDSTATUS {
    let mut gdb_out: [CHAR; 1024] = [0; 1024];
    let mut ptr: PCHAR = gdb_out.as_mut_ptr();
    let Thread: PETHREAD = CurrentStateChange.Thread as usize as PETHREAD;

    *ptr = b'T' as CHAR;
    ptr = ptr.offset(1);

    if CurrentStateChange.NewState == DbgKdExceptionStateChange {
        let ExceptionRecord: *mut EXCEPTION_RECORD64 = core::ptr::addr_of_mut!(CurrentStateChange.u.Exception.ExceptionRecord);
        ptr = exception_code_to_gdb((*ExceptionRecord).ExceptionCode, ptr);
    } else {
        sprintf(ptr, b"05\0".as_ptr() as *const CHAR);
        ptr = ptr.offset(2);
    }

    if CurrentStateChange.NewState == DbgKdLoadSymbolsStateChange {
        let len = sprintf(ptr, b"library:\0".as_ptr() as *const CHAR);
        ptr = ptr.offset(len as isize);
    }

    #[cfg(feature = "MONOPROCESS")]
    {
        let len = sprintf(ptr, b"thread:%lx;\0".as_ptr() as *const CHAR,
            handle_to_gdb_tid(PsGetThreadId(Thread)));
        ptr = ptr.offset(len as isize);
    }
    #[cfg(not(feature = "MONOPROCESS"))]
    {
        let len = sprintf(ptr, b"thread:p%lx.%lx;\0".as_ptr() as *const CHAR,
            handle_to_gdb_pid(PsGetThreadProcessId(Thread)),
            handle_to_gdb_tid(PsGetThreadId(Thread)));
        ptr = ptr.offset(len as isize);
    }

    let len = sprintf(ptr, b"core:%x;\0".as_ptr() as *const CHAR, CurrentStateChange.Processor as core::ffi::c_uint);
    ptr = ptr.offset(len as isize);
    
    return send_gdb_packet(gdb_out.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn send_gdb_ntstatus(Status: NTSTATUS) {
    let mut gdb_out: [CHAR; 4] = [0; 4];
    gdb_out[0] = b'E' as CHAR;
    exception_code_to_gdb(Status, gdb_out.as_mut_ptr().offset(1));
    gdb_out[3] = b'\0' as CHAR;
    send_gdb_packet(gdb_out.as_ptr());
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/gdb_send.c
//  * PURPOSE:         Base functions for the kernel debugger.
//  */
//
// #include "kdgdb.h"
//
// /* LOCALS *********************************************************************/
// const char hex_chars[] = "0123456789abcdef";
// static CHAR currentChecksum = 0;
//
// /* PRIVATE FUNCTIONS **********************************************************/
// static
// char*
// exception_code_to_gdb(NTSTATUS code, char* out)
// {
//     unsigned char SigVal;
//
//     switch (code)
//     {
//     case STATUS_INTEGER_DIVIDE_BY_ZERO:
//         SigVal = 8; /* divide by zero */
//         break;
//     case STATUS_SINGLE_STEP:
//     case STATUS_BREAKPOINT:
//         SigVal = 5; /* breakpoint */
//         break;
//     case STATUS_INTEGER_OVERFLOW:
//     case STATUS_ARRAY_BOUNDS_EXCEEDED:
//         SigVal = 16; /* bound instruction */
//         break;
//     case STATUS_ILLEGAL_INSTRUCTION:
//         SigVal = 4; /* Invalid opcode */
//         break;
//     case STATUS_STACK_OVERFLOW:
//     case STATUS_DATATYPE_MISALIGNMENT:
//     case STATUS_ACCESS_VIOLATION:
//         SigVal = 11; /* access violation */
//         break;
//     default:
//         SigVal = 7; /* "software generated" */
//     }
//     *out++ = hex_chars[(SigVal >> 4) & 0xf];
//     *out++ = hex_chars[SigVal & 0xf];
//     return out;
// }
//
// /* GLOBAL FUNCTIONS ***********************************************************/
// void
// start_gdb_packet(void)
// {
//     /* Start the start byte and begin checksum calculation */
//     KdpSendByte('$');
//     currentChecksum = 0;
// }
//
// void
// send_gdb_partial_packet( const CHAR* Buffer)
// {
//     const CHAR* ptr = Buffer;
//
//     /* Update check sum and send */
//     while (*ptr)
//     {
//         currentChecksum += *ptr;
//         KdpSendByte(*ptr++);
//     }
// }
//
//
// KDSTATUS
// finish_gdb_packet(void)
// {
//     UCHAR ack;
//     KDSTATUS Status;
//
//     /* Send finish byte and append checksum */
//     KdpSendByte('#');
//     KdpSendByte(hex_chars[(currentChecksum >> 4) & 0xf]);
//     KdpSendByte(hex_chars[currentChecksum & 0xf]);
//
//     /* Wait for acknowledgement */
//     Status = KdpReceiveByte(&ack);
//
//     if (Status != KdPacketReceived)
//     {
//         KD_DEBUGGER_NOT_PRESENT = TRUE;
//         return Status;
//     }
//
//     if (ack != '+')
//         return KdPacketNeedsResend;
//
//     return KdPacketReceived;
// }
//
// KDSTATUS
// send_gdb_packet( const CHAR* Buffer)
// {
//     start_gdb_packet();
//     send_gdb_partial_packet(Buffer);
//     return finish_gdb_packet();
// }
//
// ULONG
// send_gdb_partial_binary(
//      const VOID* Buffer,
//      size_t Length)
// {
//     const UCHAR* ptr = Buffer;
//     ULONG Sent = Length;
//
//     while(Length--)
//     {
//         UCHAR Byte = *ptr++;
//
//         switch (Byte)
//         {
//             case 0x7d:
//             case 0x23:
//             case 0x24:
//             case 0x2a:
//                 currentChecksum += 0x7d;
//                 KdpSendByte(0x7d);
//                 Byte ^= 0x20;
//                 Sent++;
//             /* Fall-through */
//             default:
//                 currentChecksum += Byte;
//                 KdpSendByte(Byte);
//         }
//     }
//
//     return Sent;
// }
//
// void
// send_gdb_partial_memory(
//      const VOID* Buffer,
//      size_t Length)
// {
//     const UCHAR* ptr = Buffer;
//     CHAR gdb_out[3];
//
//     gdb_out[2] = '\0';
//
//     while(Length--)
//     {
//         gdb_out[0] = hex_chars[(*ptr >> 4) & 0xf];
//         gdb_out[1] = hex_chars[*ptr++ & 0xf];
//         send_gdb_partial_packet(gdb_out);
//     }
// }
//
// KDSTATUS
// send_gdb_memory(
//      const VOID* Buffer,
//      size_t Length)
// {
//     start_gdb_packet();
//     send_gdb_partial_memory(Buffer, Length);
//     return finish_gdb_packet();
// }
//
// KDSTATUS
// gdb_send_debug_io(
//      PSTRING String,
//      BOOLEAN WithPrefix)
// {
//     CHAR gdb_out[3];
//     CHAR* ptr = String->Buffer;
//     USHORT Length = String->Length;
//
//     gdb_out[2] = '\0';
//
//     start_gdb_packet();
//
//     if (WithPrefix)
//     {
//         send_gdb_partial_packet("O");
//     }
//
//     /* Send the data */
//     while (Length--)
//     {
//         gdb_out[0] = hex_chars[(*ptr >> 4) & 0xf];
//         gdb_out[1] = hex_chars[*ptr++ & 0xf];
//         send_gdb_partial_packet(gdb_out);
//     }
//
//     return finish_gdb_packet();
// }
//
// KDSTATUS
// gdb_send_exception()
// {
//     char gdb_out[1024];
//     char* ptr = gdb_out;
//     PETHREAD Thread = (PETHREAD)(ULONG_PTR)CurrentStateChange.Thread;
//
//     /* Report to GDB */
//     *ptr++ = 'T';
//
//     if (CurrentStateChange.NewState == DbgKdExceptionStateChange)
//     {
//         EXCEPTION_RECORD64* ExceptionRecord = &CurrentStateChange.u.Exception.ExceptionRecord;
//         ptr = exception_code_to_gdb(ExceptionRecord->ExceptionCode, ptr);
//     }
//     else
//         ptr += sprintf(ptr, "05");
//
//     if (CurrentStateChange.NewState == DbgKdLoadSymbolsStateChange)
//         ptr += sprintf(ptr, "library:");
//
// #if MONOPROCESS
//     ptr += sprintf(ptr, "thread:%" PRIxPTR ";",
//         handle_to_gdb_tid(PsGetThreadId(Thread)));
// #else
//     ptr += sprintf(ptr, "thread:p%" PRIxPTR ".%" PRIxPTR ";",
//         handle_to_gdb_pid(PsGetThreadProcessId(Thread)),
//         handle_to_gdb_tid(PsGetThreadId(Thread)));
// #endif
//
//     ptr += sprintf(ptr, "core:%x;", CurrentStateChange.Processor);
//     return send_gdb_packet(gdb_out);
// }
//
// void
// send_gdb_ntstatus(
//      NTSTATUS Status)
// {
//     /* Just build a EXX packet and send it */
//     char gdb_out[4];
//     gdb_out[0] = 'E';
//     exception_code_to_gdb(Status, &gdb_out[1]);
//     gdb_out[3] = '\0';
//     send_gdb_packet(gdb_out);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
