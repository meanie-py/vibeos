use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;

use ntkrnl_sys::ddk::{
    CSTRING, DBGKD_DEBUG_IO, KDSTATUS, KD_CONTEXT, PCCH, PCSTR, PDBGKD_DEBUG_IO,
    PSTRING, STRING,
};

const MAXUSHORT: usize = 0xFFFF;

static mut KdbgKdContext: KD_CONTEXT = unsafe { core::mem::zeroed() };

extern "system" {
    fn KdSendPacket( PacketType: ULONG, Header: PSTRING, Data: PSTRING, Context: *mut KD_CONTEXT, );
    fn KdReceivePacket( PacketType: ULONG, Header: PSTRING, Data: PSTRING, Length: *mut ULONG, Context: *mut KD_CONTEXT, ) -> KDSTATUS;
}
extern "C" {
    /// `strnlen` is a CRT function, cdecl.
    fn strnlen(s: PCSTR, maxlen: usize) -> usize;
}

const PACKET_TYPE_KD_DEBUG_IO: ULONG = 0x3;
const PACKET_MAX_SIZE: usize = 4000;
const DbgKdPrintStringApi: ULONG = 0x00003230;
const DbgKdGetStringApi: ULONG = 0x00003231;
const KdPacketNeedsResend: KDSTATUS = 2;
const KdPacketReceived: KDSTATUS = 0;

unsafe fn KdbPrintStringWorker(
    Output: *const CSTRING,
    ApiNumber: ULONG,
    DebugIo: PDBGKD_DEBUG_IO,
    Header: PSTRING,
    Data: PSTRING,
) {
    let Length: USHORT;

    let output_length = (*Output).Length as usize;
    let max_data_size = PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_DEBUG_IO>();
    Length = core::cmp::min(output_length, max_data_size) as USHORT;

    (*DebugIo).ApiNumber = ApiNumber;
    (*DebugIo).ProcessorLevel = 0;
    (*DebugIo).Processor = (*KeGetCurrentPrcb()).Number as USHORT;

    if ApiNumber == DbgKdPrintStringApi {
        (*DebugIo).u.PrintString.LengthOfString = Length as ULONG;
    } else {
        (*DebugIo).u.GetString.LengthOfPromptString = Length as ULONG;
    }

    (*Header).Length = core::mem::size_of::<DBGKD_DEBUG_IO>() as USHORT;
    (*Header).Buffer = DebugIo as PCHAR;

    (*Data).Length = Length;
    (*Data).Buffer = (*Output).Buffer as PCHAR;

    KdSendPacket(
        PACKET_TYPE_KD_DEBUG_IO,
        Header,
        Data,
        core::ptr::addr_of_mut!(KdbgKdContext),
    );
}

#[no_mangle]
pub unsafe extern "C" fn KdbPrintString(Output: *const CSTRING) {
    let mut DebugIo: DBGKD_DEBUG_IO = core::mem::zeroed();
    let mut Header: STRING = core::mem::zeroed();
    let mut Data: STRING = core::mem::zeroed();

    KdbPrintStringWorker(
        Output,
        DbgKdPrintStringApi,
        &mut DebugIo,
        &mut Header,
        &mut Data,
    );
}

unsafe fn KdbPromptStringWorker(
    PromptString: *const CSTRING,
    ResponseString: PSTRING,
) -> BOOLEAN {
    let mut DebugIo: DBGKD_DEBUG_IO = core::mem::zeroed();
    let mut Header: STRING = core::mem::zeroed();
    let mut Data: STRING = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let mut Status: KDSTATUS;

    DebugIo.u.GetString.LengthOfStringRead = (*ResponseString).MaximumLength as ULONG;
    KdbPrintStringWorker(
        PromptString,
        DbgKdGetStringApi,
        &mut DebugIo,
        &mut Header,
        &mut Data,
    );

    Header.MaximumLength = core::mem::size_of::<DBGKD_DEBUG_IO>() as USHORT;
    Data.MaximumLength = (*ResponseString).MaximumLength;
    Data.Buffer = (*ResponseString).Buffer;

    loop {
        Status = KdReceivePacket(
            PACKET_TYPE_KD_DEBUG_IO,
            &mut Header,
            &mut Data,
            &mut Length,
            core::ptr::addr_of_mut!(KdbgKdContext),
        );

        if Status == KdPacketNeedsResend {
            return 1u8;
        }

        if Status == KdPacketReceived {
            break;
        }
    }

    Length = core::cmp::min(Length, (*ResponseString).MaximumLength as ULONG);

    (*ResponseString).Length = Length as USHORT;

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbPromptString(
    PromptString: *const CSTRING,
    ResponseString: PSTRING,
) -> USHORT {
    (*ResponseString).Length = 0;
    while KdbPromptStringWorker(PromptString, ResponseString) != 0 {
    }
    (*ResponseString).Length
}

#[no_mangle]
pub unsafe extern "C" fn KdbPutsN(String: PCCH, Length: USHORT) {
    let mut Output: CSTRING = core::mem::zeroed();

    Output.Buffer = String;
    Output.Length = Length;
    Output.MaximumLength = Length;
    KdbPrintString(&Output);
}

#[no_mangle]
pub unsafe extern "C" fn KdbPuts(String: PCSTR) {
    let length = strnlen(String, (MAXUSHORT - core::mem::size_of::<u8>()) as usize);
    KdbPutsN(String as PCCH, length as USHORT);
}

// NOTE: KdbPrintf is a C-variadic function (__cdecl) that cannot be implemented
// in stable Rust. It must remain in C (kdb_print.c) or be provided by a C shim.
// Other Rust files declare it as: extern "C" { fn KdbPrintf(Format: PCHAR, ...); }
// which is valid in stable Rust for *calling* C variadic functions.
//
// Original Rust translation body (kept for reference):
//
// #[no_mangle]
// pub unsafe extern "cdecl" fn KdbPrintf(Format: PCSTR, mut args: ...) {
//     let mut Buffer: [i8; 1024] = [0; 1024];
//     let mut Length: isize;
//
//     Length = _vsnprintf(
//         Buffer.as_mut_ptr(),
//         1024,
//         Format,
//         args.as_va_list(),
//     ) as isize;
//
//     Length = core::cmp::min(
//         Length as usize,
//         (MAXUSHORT - core::mem::size_of::<u8>()) as usize,
//     ) as isize;
//
//     KdbPutsN(Buffer.as_ptr() as PCCH, Length as USHORT);
// }

#[no_mangle]
pub unsafe extern "C" fn KdbPrompt(
    Prompt: PCSTR,
    Buffer: PCHAR,
    Size: usize,
) -> usize {
    let mut PromptString: CSTRING = core::mem::zeroed();
    let mut ResponseBuffer: STRING = core::mem::zeroed();

    let prompt_len = strnlen(Prompt, (MAXUSHORT - core::mem::size_of::<u8>()) as usize);
    PromptString.Buffer = Prompt;
    PromptString.Length = prompt_len as USHORT;
    PromptString.MaximumLength = prompt_len as USHORT;

    ResponseBuffer.Buffer = Buffer;
    ResponseBuffer.Length = 0;
    ResponseBuffer.MaximumLength = core::cmp::min(Size, MAXUSHORT as usize) as USHORT;

    KdbPromptString(&PromptString, &mut ResponseBuffer) as usize
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS KDBG Kernel Debugger
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     IO interface for KDBG. Provides local KDBG versions
//  *              of KdpPrintString, KdpPromptString and KdpDprintf.
//  * COPYRIGHT:   Copyright 2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "kdb.h"
//
// /* FUNCTIONS *****************************************************************/
//
// static KD_CONTEXT KdbgKdContext;
//
// #undef KdSendPacket
// #define pKdSendPacket KdSendPacket
//
// #undef KdReceivePacket
// #define pKdReceivePacket KdReceivePacket
//
// static VOID
// KdbPrintStringWorker(
//     _In_ const CSTRING* Output,
//     _In_ ULONG ApiNumber,
//     _Inout_ PDBGKD_DEBUG_IO DebugIo,
//     _Inout_ PSTRING Header,
//     _Inout_ PSTRING Data)
// {
//     USHORT Length;
//     C_ASSERT(PACKET_MAX_SIZE >= sizeof(*DebugIo));
//
//     ASSERT((ApiNumber == DbgKdPrintStringApi) ||
//            (ApiNumber == DbgKdGetStringApi));
//
//     /* Make sure we don't exceed the KD Packet size */
//     Length = min(Output->Length, PACKET_MAX_SIZE - sizeof(*DebugIo));
//
//     /* Build the packet header */
//     DebugIo->ApiNumber = ApiNumber;
//     DebugIo->ProcessorLevel = 0; // (USHORT)KeProcessorLevel;
//     DebugIo->Processor = KeGetCurrentPrcb()->Number;
//
//     if (ApiNumber == DbgKdPrintStringApi)
//         DebugIo->u.PrintString.LengthOfString = Length;
//     else // if (ApiNumber == DbgKdGetStringApi)
//         DebugIo->u.GetString.LengthOfPromptString = Length;
//
//     Header->Length = sizeof(*DebugIo);
//     Header->Buffer = (PCHAR)DebugIo;
//
//     /* Build the data */
//     Data->Length = Length;
//     Data->Buffer = (PCHAR)Output->Buffer;
//
//     /* Send the packet */
//     /* IO packet: call KdTerm */
//     pKdSendPacket(PACKET_TYPE_KD_DEBUG_IO, Header, Data, &KdbgKdContext);
// }
//
// VOID
// KdbPrintString(
//     _In_ const CSTRING* Output)
// {
//     DBGKD_DEBUG_IO DebugIo;
//     STRING Header, Data;
//
//     KdbPrintStringWorker(Output, DbgKdPrintStringApi,
//                          &DebugIo, &Header, &Data);
// }
//
// static BOOLEAN
// KdbPromptStringWorker(
//     _In_ const CSTRING* PromptString,
//     _Inout_ PSTRING ResponseString)
// {
//     DBGKD_DEBUG_IO DebugIo;
//     STRING Header, Data;
//     ULONG Length;
//     KDSTATUS Status;
//
//     /* Print the prompt */
//     // DebugIo.u.GetString.LengthOfPromptString = Length;
//     DebugIo.u.GetString.LengthOfStringRead = ResponseString->MaximumLength;
//     KdbPrintStringWorker(PromptString, DbgKdGetStringApi,
//                          &DebugIo, &Header, &Data);
//
//     /* Set the maximum lengths for the receive */
//     Header.MaximumLength = sizeof(DebugIo);
//     Data.MaximumLength = ResponseString->MaximumLength;
//     /* Build the data */
//     Data.Buffer = ResponseString->Buffer;
//
//     /* Enter receive loop */
//     do
//     {
//         /* Get our reply */
//         /* IO packet: call KdTerm */
//         Status = pKdReceivePacket(PACKET_TYPE_KD_DEBUG_IO,
//                                   &Header,
//                                   &Data,
//                                   &Length,
//                                   &KdbgKdContext);
//
//         /* Return TRUE if we need to resend */
//         if (Status == KdPacketNeedsResend)
//             return TRUE;
//
//     /* Loop until we succeed */
//     } while (Status != KdPacketReceived);
//
//     /* Don't copy back a larger response than there is room for */
//     Length = min(Length, ResponseString->MaximumLength);
//
//     /* We've got the string back; return the length */
//     ResponseString->Length = (USHORT)Length;
//
//     /* Success; we don't need to resend */
//     return FALSE;
// }
//
// USHORT
// KdbPromptString(
//     _In_ const CSTRING* PromptString,
//     _Inout_ PSTRING ResponseString)
// {
//     /* Enter prompt loop: send the prompt and receive the response */
//     ResponseString->Length = 0;
//     while (KdbPromptStringWorker(PromptString, ResponseString))
//     {
//         /* Loop while we need to resend */
//     }
//     return ResponseString->Length;
// }
//
//
// VOID
// KdbPutsN(
//     _In_ PCCH String,
//     _In_ USHORT Length)
// {
//     CSTRING Output;
//
//     Output.Buffer = String;
//     Output.Length = Output.MaximumLength = Length;
//     KdbPrintString(&Output);
// }
//
// VOID
// KdbPuts(
//     _In_ PCSTR String)
// {
//     KdbPutsN(String, (USHORT)strnlen(String, MAXUSHORT - sizeof(ANSI_NULL)));
// }
//
// VOID
// __cdecl
// KdbPrintf(
//     _In_ PCSTR Format,
//     ...)
// {
//     va_list ap;
//     SIZE_T Length;
//     CHAR Buffer[1024];
//
//     /* Format the string */
//     va_start(ap, Format);
//     Length = _vsnprintf(Buffer,
//                         sizeof(Buffer),
//                         Format,
//                         ap);
//     Length = min(Length, MAXUSHORT - sizeof(ANSI_NULL));
//     va_end(ap);
//
//     /* Send it to the debugger directly */
//     KdbPutsN(Buffer, (USHORT)Length);
// }
//
// SIZE_T
// KdbPrompt(
//     _In_ PCSTR Prompt,
//     _Out_ PCHAR Buffer,
//     _In_ SIZE_T Size)
// {
//     CSTRING PromptString;
//     STRING ResponseBuffer;
//
//     PromptString.Buffer = Prompt;
//     PromptString.Length = PromptString.MaximumLength =
//         (USHORT)strnlen(Prompt, MAXUSHORT - sizeof(ANSI_NULL));
//
//     ResponseBuffer.Buffer = Buffer;
//     ResponseBuffer.Length = 0;
//     ResponseBuffer.MaximumLength = (USHORT)min(Size, MAXUSHORT);
//
//     return KdbPromptString(&PromptString, &ResponseBuffer);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
