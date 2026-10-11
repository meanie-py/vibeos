use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;

use ntkrnl_sys::ddk::{
    _KPRCB,
    CONTEXT,
    DBGKD_DEBUG_IO,
    KD_CONTEXT,
    KDSTATUS,
    KPROCESSOR_MODE,
    KeTestSpinLock,
    KeTryToAcquireSpinLockAtDpcLevel,
    KiReleaseSpinLock,
    KiRestoreProcessorControlState,
    KiSaveProcessorControlState,
    PCONTEXT,
    PCSTR,
    PKD_SYMBOLS_INFO,
    PKEXCEPTION_FRAME,
    PKTRAP_FRAME,
    PSTRING,
    STRING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut KdPrintCircularBuffer: PCHAR;
    static mut KdPrintBufferSize: ULONG;
    static mut KdPrintWritePointer: PCHAR;
    static mut KdPrintRolloverCount: ULONG;
    static mut KdpPrintSpinLock: KSPIN_LOCK;
    static mut KdpMessageBuffer: [i8; 4096];
    static mut KdpContext: KD_CONTEXT;
    static mut KeProcessorLevel: USHORT;
    static mut KdDebuggerNotPresent: BOOLEAN;
    fn KdSendPacket( PacketType: ULONG, MessageHeader: PSTRING, MessageData: PSTRING, Context: *mut KD_CONTEXT, );
    fn KdpPollBreakInWithPortLock() -> BOOLEAN;
    fn KdReceivePacket( PacketType: ULONG, MessageHeader: PSTRING, MessageData: PSTRING, DataLength: *mut ULONG, Context: *mut KD_CONTEXT, ) -> KDSTATUS;
    fn KdEnterDebugger(TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME) -> BOOLEAN;
    fn KdExitDebugger(Enable: BOOLEAN);
    fn KdpReportCommandStringStateChange( NameString: PSTRING, CommandString: PSTRING, ContextRecord: PCONTEXT, );
    fn KdpReportLoadSymbolsStateChange( DllPath: PSTRING, SymbolInfo: PKD_SYMBOLS_INFO, Unload: BOOLEAN, ContextRecord: PCONTEXT, );
    fn NtQueryDebugFilterState(ComponentId: ULONG, Level: ULONG) -> NTSTATUS;
}

extern "C" {
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

const KD_PRINT_MAX_BYTES: usize = 512;
const PACKET_MAX_SIZE: usize = 4000;
const PACKET_TYPE_KD_DEBUG_IO: ULONG = 3;
const DbgKdPrintStringApi: u32 = 0x00003230;
const DbgKdGetStringApi: u32 = 0x00003231;
const KdPacketNeedsResend: KDSTATUS = 2;
const KdPacketReceived: KDSTATUS = 0;
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005u32 as NTSTATUS;
const STATUS_DEVICE_NOT_CONNECTED: NTSTATUS = 0xC000009Du32 as NTSTATUS;
const STATUS_BREAKPOINT: NTSTATUS = 0x80000003u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn KdpMoveMemory(dst: PVOID, src: PVOID, len: usize) {
    let dst = opaque_ptr(dst);
    let src = opaque_ptr(src);
    let mut dst = dst as *mut u8;
    let mut src = src as *const u8;
    let mut len = len;
    while len > 0 {
        *dst = *src;
        dst = dst.add(1);
        src = src.add(1);
        len -= 1;
    }
}

fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

#[no_mangle]
pub unsafe extern "system" fn KdpAcquireLock(SpinLock: PKSPIN_LOCK) -> KIRQL {
    let mut old_irql: KIRQL = 0;

    loop {
        while KeTestSpinLock(SpinLock as *mut _) == 0 {}

        KeRaiseIrql(HIGH_LEVEL, &mut old_irql as *mut KIRQL);

        if KeTryToAcquireSpinLockAtDpcLevel(SpinLock as *mut _) != 0 {
            break;
        }

        KeLowerIrql(old_irql);
    }

    old_irql
}

#[no_mangle]
pub unsafe extern "system" fn KdpReleaseLock(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL) {
    KiReleaseSpinLock(SpinLock as *mut _);
    KeLowerIrql(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KdLogDbgPrint(String: PSTRING) {
    if (*String).Buffer.is_null() || (*String).Length == 0 {
        return;
    }

    if KdPrintCircularBuffer.is_null() {
        return;
    }

    let old_irql = KdpAcquireLock(&raw mut KdpPrintSpinLock as PKSPIN_LOCK);

    let mut length = min((*String).Length as usize, KdPrintBufferSize as usize);
    let remaining = (KdPrintCircularBuffer as usize + KdPrintBufferSize as usize - KdPrintWritePointer as usize) as usize;

    if length < remaining {
        KdpMoveMemory(KdPrintWritePointer as PVOID, (*String).Buffer as PVOID, length);
        KdPrintWritePointer = (KdPrintWritePointer as usize + length) as PCHAR;
    } else {
        KdpMoveMemory(KdPrintWritePointer as PVOID, (*String).Buffer as PVOID, remaining);
        length -= remaining;
        if length > 0 {
            KdpMoveMemory(
                KdPrintCircularBuffer as PVOID,
                ((*String).Buffer as usize + remaining) as PVOID,
                length,
            );
        }

        KdPrintWritePointer = (KdPrintCircularBuffer as usize + length) as PCHAR;

        KdPrintRolloverCount = KdPrintRolloverCount.wrapping_add(1);
        if KdPrintRolloverCount == 0 {
            KdPrintRolloverCount = KdPrintRolloverCount.wrapping_add(1);
        }
    }

    KdpReleaseLock(&raw mut KdpPrintSpinLock as PKSPIN_LOCK, old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn KdpPrintString(Output: PSTRING) -> BOOLEAN {
    let mut data: STRING = core::mem::zeroed();
    let mut header: STRING = core::mem::zeroed();
    let mut debug_io: DBGKD_DEBUG_IO = core::mem::zeroed();
    let mut length: USHORT;

    KdpMoveMemory(
        (*(&raw mut KdpMessageBuffer)).as_mut_ptr() as PVOID,
        (*Output).Buffer as PVOID,
        (*Output).Length as usize,
    );

    length = (*Output).Length;
    if (core::mem::size_of::<DBGKD_DEBUG_IO>() + length as usize) > PACKET_MAX_SIZE {
        length = (PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_DEBUG_IO>()) as USHORT;
    }

    debug_io.ApiNumber = DbgKdPrintStringApi;
    debug_io.ProcessorLevel = KeProcessorLevel;
    debug_io.Processor = (*(KeGetCurrentPrcb() as *mut _KPRCB)).Number as USHORT;
    debug_io.u.PrintString.LengthOfString = length as ULONG;
    header.Length = core::mem::size_of::<DBGKD_DEBUG_IO>() as USHORT;
    header.Buffer = &mut debug_io as *mut DBGKD_DEBUG_IO as PCHAR;

    data.Length = length;
    data.Buffer = (*(&raw mut KdpMessageBuffer)).as_mut_ptr();

    KdSendPacket(
        PACKET_TYPE_KD_DEBUG_IO,
        &mut header as PSTRING,
        &mut data as PSTRING,
        &raw mut KdpContext as *mut KD_CONTEXT,
    );

    KdpPollBreakInWithPortLock()
}

#[no_mangle]
pub unsafe extern "system" fn KdpPromptString(
    PromptString: PSTRING,
    ResponseString: PSTRING,
) -> BOOLEAN {
    let mut data: STRING = core::mem::zeroed();
    let mut header: STRING = core::mem::zeroed();
    let mut debug_io: DBGKD_DEBUG_IO = core::mem::zeroed();
    let mut length: ULONG;
    let mut status: KDSTATUS;

    KdpMoveMemory(
        (*(&raw mut KdpMessageBuffer)).as_mut_ptr() as PVOID,
        (*PromptString).Buffer as PVOID,
        (*PromptString).Length as usize,
    );

    length = (*PromptString).Length as ULONG;
    if (core::mem::size_of::<DBGKD_DEBUG_IO>() + length as usize) > PACKET_MAX_SIZE {
        length = (PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_DEBUG_IO>()) as ULONG;
    }

    debug_io.ApiNumber = DbgKdGetStringApi;
    debug_io.ProcessorLevel = KeProcessorLevel;
    debug_io.Processor = (*(KeGetCurrentPrcb() as *mut _KPRCB)).Number as USHORT;
    debug_io.u.GetString.LengthOfPromptString = length;
    debug_io.u.GetString.LengthOfStringRead = (*ResponseString).MaximumLength as ULONG;
    header.Length = core::mem::size_of::<DBGKD_DEBUG_IO>() as USHORT;
    header.Buffer = &mut debug_io as *mut DBGKD_DEBUG_IO as PCHAR;

    data.Length = length as USHORT;
    data.Buffer = (*(&raw mut KdpMessageBuffer)).as_mut_ptr();

    KdSendPacket(
        PACKET_TYPE_KD_DEBUG_IO,
        &mut header as PSTRING,
        &mut data as PSTRING,
        &raw mut KdpContext as *mut KD_CONTEXT,
    );

    header.MaximumLength = core::mem::size_of::<DBGKD_DEBUG_IO>() as USHORT;
    data.MaximumLength = (*(&raw const KdpMessageBuffer)).len() as USHORT;

    loop {
        status = KdReceivePacket(
            PACKET_TYPE_KD_DEBUG_IO,
            &mut header as PSTRING,
            &mut data as PSTRING,
            &mut length as *mut ULONG,
            &raw mut KdpContext as *mut KD_CONTEXT,
        );

        if status == KdPacketNeedsResend {
            return 1u8;
        }

        if status == KdPacketReceived {
            break;
        }
    }

    length = min(length as usize, (*ResponseString).MaximumLength as usize) as ULONG;

    KdpMoveMemory(
        (*ResponseString).Buffer as PVOID,
        (*(&raw mut KdpMessageBuffer)).as_mut_ptr() as PVOID,
        length as usize,
    );
    (*ResponseString).Length = length as USHORT;

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn KdpCommandString(
    NameString: PSTRING,
    CommandString: PSTRING,
    PreviousMode: KPROCESSOR_MODE,
    ContextRecord: PCONTEXT,
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
) {
    let prcb = KeGetCurrentPrcb() as *mut _KPRCB;

    if PreviousMode != 0 || KdDebuggerNotPresent != 0 {
        return;
    }

    let enable = KdEnterDebugger(TrapFrame, ExceptionFrame);

    KiSaveProcessorControlState(core::ptr::addr_of_mut!((*prcb).ProcessorState));
    KdpMoveMemory(
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame) as PVOID,
        ContextRecord as PVOID,
        core::mem::size_of::<CONTEXT>(),
    );

    KdpReportCommandStringStateChange(
        NameString,
        CommandString,
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame),
    );

    KdpMoveMemory(
        ContextRecord as PVOID,
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame) as PVOID,
        core::mem::size_of::<CONTEXT>(),
    );
    KiRestoreProcessorControlState(core::ptr::addr_of_mut!((*prcb).ProcessorState));

    KdExitDebugger(enable);
}

#[no_mangle]
pub unsafe extern "system" fn KdpSymbol(
    DllPath: PSTRING,
    SymbolInfo: PKD_SYMBOLS_INFO,
    Unload: BOOLEAN,
    PreviousMode: KPROCESSOR_MODE,
    ContextRecord: PCONTEXT,
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
) {
    let prcb = KeGetCurrentPrcb() as *mut _KPRCB;

    if PreviousMode != 0 || KdDebuggerNotPresent != 0 {
        return;
    }

    let enable = KdEnterDebugger(TrapFrame, ExceptionFrame);

    KiSaveProcessorControlState(core::ptr::addr_of_mut!((*prcb).ProcessorState));
    KdpMoveMemory(
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame) as PVOID,
        ContextRecord as PVOID,
        core::mem::size_of::<CONTEXT>(),
    );

    KdpReportLoadSymbolsStateChange(
        DllPath,
        SymbolInfo,
        Unload,
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame),
    );

    // A pre-ACK send can declare the debugger absent and return without any
    // remote state mutation.  In that case there is nothing to copy/restore;
    // restoring the snapshot is both unnecessary and unsafe on this
    // same-ring INT 2Dh path (the interrupted kernel stack is live beneath
    // the trap frame).  An attached debugger still gets the full restore.
    if KdDebuggerNotPresent == 0 {
        KdpMoveMemory(
            ContextRecord as PVOID,
            core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame) as PVOID,
            core::mem::size_of::<CONTEXT>(),
        );
        KiRestoreProcessorControlState(core::ptr::addr_of_mut!((*prcb).ProcessorState));
    }

    KdExitDebugger(enable);
}

#[no_mangle]
pub unsafe extern "system" fn KdpPrompt(
    PromptString: PCHAR,
    PromptLength: USHORT,
    ResponseString: PCHAR,
    MaximumResponseLength: USHORT,
    PreviousMode: KPROCESSOR_MODE,
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
) -> USHORT {
    let mut prompt_buffer: STRING = core::mem::zeroed();
    let mut response_buffer: STRING = core::mem::zeroed();
    let mut resend: BOOLEAN;
    let safe_response_string: PCHAR;
    let mut captured_prompt: [i8; KD_PRINT_MAX_BYTES] = [0; KD_PRINT_MAX_BYTES];
    let mut safe_response_buffer: [i8; KD_PRINT_MAX_BYTES] = [0; KD_PRINT_MAX_BYTES];

    let prompt_length = min(PromptLength as usize, captured_prompt.len()) as USHORT;
    let maximum_response_length = min(MaximumResponseLength as usize, safe_response_buffer.len()) as USHORT;

    if PreviousMode != 0 {
        // C: probe + capture the prompt and probe the response buffer under
        // _SEH2_TRY, whose handler bails out with `return 0`.
        if ntkrnl_sys::probe::safe_copy_from_user(
            captured_prompt.as_mut_ptr().cast(),
            PromptString as *const core::ffi::c_void,
            prompt_length as usize,
            1,
            1,
        )
        .is_err()
        {
            return 0;
        }
        let prompt_string_local = captured_prompt.as_mut_ptr();

        if ntkrnl_sys::probe::probe_user_buffer_write(
            ResponseString as *const core::ffi::c_void,
            maximum_response_length as usize,
            1,
            1,
        )
        .is_err()
        {
            return 0;
        }
        safe_response_string = safe_response_buffer.as_mut_ptr();

        prompt_buffer.Buffer = prompt_string_local;
        prompt_buffer.Length = prompt_length;
        prompt_buffer.MaximumLength = prompt_length;
        response_buffer.Buffer = safe_response_string;
        response_buffer.Length = 0;
        response_buffer.MaximumLength = maximum_response_length;

        KdLogDbgPrint(&mut prompt_buffer as PSTRING);

        let enable = KdEnterDebugger(TrapFrame, ExceptionFrame);

        loop {
            resend = KdpPromptString(&mut prompt_buffer as PSTRING, &mut response_buffer as PSTRING);
            if resend == 0 {
                break;
            }
        }

        KdExitDebugger(enable);

        // C: the copy-back is under _SEH2_TRY; "string became invalid", return 0.
        if ntkrnl_sys::probe::safe_copy_raw(
            ResponseString as PVOID,
            response_buffer.Buffer as *const core::ffi::c_void,
            response_buffer.Length as usize,
        )
        .is_err()
        {
            return 0;
        }

        response_buffer.Length
    } else {
        safe_response_string = ResponseString;

        prompt_buffer.Buffer = PromptString;
        prompt_buffer.Length = prompt_length;
        prompt_buffer.MaximumLength = prompt_length;
        response_buffer.Buffer = safe_response_string;
        response_buffer.Length = 0;
        response_buffer.MaximumLength = maximum_response_length;

        KdLogDbgPrint(&mut prompt_buffer as PSTRING);

        let enable = KdEnterDebugger(TrapFrame, ExceptionFrame);

        loop {
            resend = KdpPromptString(&mut prompt_buffer as PSTRING, &mut response_buffer as PSTRING);
            if resend == 0 {
                break;
            }
        }

        KdExitDebugger(enable);

        response_buffer.Length
    }
}

unsafe fn KdpPrintFromUser(
    ComponentId: ULONG,
    Level: ULONG,
    String: PCHAR,
    Length: USHORT,
    PreviousMode: KPROCESSOR_MODE,
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    Handled: PBOOLEAN,
) -> NTSTATUS {
    let mut captured_string: [i8; KD_PRINT_MAX_BYTES] = [0; KD_PRINT_MAX_BYTES];

    // C: probe + capture under _SEH2_TRY; a bad pointer returns STATUS_ACCESS_VIOLATION.
    if let Err(status) = ntkrnl_sys::probe::safe_copy_from_user(
        captured_string.as_mut_ptr().cast(),
        String as *const core::ffi::c_void,
        Length as usize,
        1,
        1,
    ) {
        return status;
    }
    let string_local = captured_string.as_mut_ptr();

    KdpPrint(
        ComponentId,
        Level,
        string_local,
        Length,
        0,
        TrapFrame,
        ExceptionFrame,
        Handled,
    )
}

#[no_mangle]
pub unsafe extern "system" fn KdpPrint(
    ComponentId: ULONG,
    Level: ULONG,
    String: PCHAR,
    Length: USHORT,
    PreviousMode: KPROCESSOR_MODE,
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    Handled: PBOOLEAN,
) -> NTSTATUS {
    let Handled = opaque_ptr(Handled);
    let mut status: NTSTATUS;
    let mut output_string: STRING = core::mem::zeroed();

    if NtQueryDebugFilterState(ComponentId, Level) == 0 {
        *Handled = 1u8;
        return STATUS_SUCCESS;
    }

    *Handled = 0u8;

    let length = min(Length as usize, KD_PRINT_MAX_BYTES) as USHORT;

    if PreviousMode != 0 {
        return KdpPrintFromUser(
            ComponentId,
            Level,
            String,
            length,
            PreviousMode,
            TrapFrame,
            ExceptionFrame,
            Handled,
        );
    }

    output_string.Buffer = String;
    output_string.Length = length;
    output_string.MaximumLength = length;

    KdLogDbgPrint(&mut output_string as PSTRING);

    if KdDebuggerNotPresent != 0 {
        *Handled = 1u8;
        return STATUS_DEVICE_NOT_CONNECTED;
    }

    let enable = KdEnterDebugger(TrapFrame, ExceptionFrame);

    if KdpPrintString(&mut output_string as PSTRING) != 0 {
        status = STATUS_BREAKPOINT;
    } else {
        status = STATUS_SUCCESS;
    }

    KdExitDebugger(enable);
    *Handled = 1u8;
    status
}

// NOTE: KdpDprintf is a C-variadic function that cannot be implemented in stable Rust.
// It must remain in C (kdprint.c) or be provided by a C shim file.
// The Rust kd64 files declare it as: extern "C" { fn KdpDprintf(Format: PCHAR, ...); }
// which is valid in stable Rust for calling C variadic functions.

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/kd64/kdprint.c
//  * PURPOSE:         KD64 Trap Handler Routines
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Stefan Ginsberg (stefan.ginsberg@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define KD_PRINT_MAX_BYTES 512
//
// /* FUNCTIONS *****************************************************************/
//
// KIRQL
// NTAPI
// KdpAcquireLock(
//     _In_ PKSPIN_LOCK SpinLock)
// {
//     KIRQL OldIrql;
//
//     /* Acquire the spinlock without waiting at raised IRQL */
//     while (TRUE)
//     {
//         /* Loop until the spinlock becomes available */
//         while (!KeTestSpinLock(SpinLock));
//
//         /* Spinlock is free, raise IRQL to high level */
//         KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//
//         /* Try to get the spinlock */
//         if (KeTryToAcquireSpinLockAtDpcLevel(SpinLock))
//             break;
//
//         /* Someone else got the spinlock, lower IRQL back */
//         KeLowerIrql(OldIrql);
//     }
//
//     return OldIrql;
// }
//
// VOID
// NTAPI
// KdpReleaseLock(
//     _In_ PKSPIN_LOCK SpinLock,
//     _In_ KIRQL OldIrql)
// {
//     /* Release the spinlock */
//     KiReleaseSpinLock(SpinLock);
//     // KeReleaseSpinLockFromDpcLevel(SpinLock);
//
//     /* Restore the old IRQL */
//     KeLowerIrql(OldIrql);
// }
//
// VOID
// NTAPI
// KdLogDbgPrint(
//     _In_ PSTRING String)
// {
//     SIZE_T Length, Remaining;
//     KIRQL OldIrql;
//
//     /* If the string is empty, bail out */
//     if (!String->Buffer || (String->Length == 0))
//         return;
//
//     /* If no log buffer available, bail out */
//     if (!KdPrintCircularBuffer /*|| (KdPrintBufferSize == 0)*/)
//         return;
//
//     /* Acquire the log spinlock without waiting at raised IRQL */
//     OldIrql = KdpAcquireLock(&KdpPrintSpinLock);
//
//     Length = min(String->Length, KdPrintBufferSize);
//     Remaining = KdPrintCircularBuffer + KdPrintBufferSize - KdPrintWritePointer;
//
//     if (Length < Remaining)
//     {
//         KdpMoveMemory(KdPrintWritePointer, String->Buffer, Length);
//         KdPrintWritePointer += Length;
//     }
//     else
//     {
//         KdpMoveMemory(KdPrintWritePointer, String->Buffer, Remaining);
//         Length -= Remaining;
//         if (Length > 0)
//             KdpMoveMemory(KdPrintCircularBuffer, String->Buffer + Remaining, Length);
//
//         KdPrintWritePointer = KdPrintCircularBuffer + Length;
//
//         /* Got a rollover, update count (handle wrapping, must always be >= 1) */
//         ++KdPrintRolloverCount;
//         if (KdPrintRolloverCount == 0)
//             ++KdPrintRolloverCount;
//     }
//
//     /* Release the spinlock */
//     KdpReleaseLock(&KdpPrintSpinLock, OldIrql);
// }
//
// BOOLEAN
// NTAPI
// KdpPrintString(
//     _In_ PSTRING Output)
// {
//     STRING Data, Header;
//     DBGKD_DEBUG_IO DebugIo;
//     USHORT Length;
//
//     /* Copy the string */
//     KdpMoveMemory(KdpMessageBuffer,
//                   Output->Buffer,
//                   Output->Length);
//
//     /* Make sure we don't exceed the KD Packet size */
//     Length = Output->Length;
//     if ((sizeof(DBGKD_DEBUG_IO) + Length) > PACKET_MAX_SIZE)
//     {
//         /* Normalize length */
//         Length = PACKET_MAX_SIZE - sizeof(DBGKD_DEBUG_IO);
//     }
//
//     /* Build the packet header */
//     DebugIo.ApiNumber = DbgKdPrintStringApi;
//     DebugIo.ProcessorLevel = (USHORT)KeProcessorLevel;
//     DebugIo.Processor = KeGetCurrentPrcb()->Number;
//     DebugIo.u.PrintString.LengthOfString = Length;
//     Header.Length = sizeof(DBGKD_DEBUG_IO);
//     Header.Buffer = (PCHAR)&DebugIo;
//
//     /* Build the data */
//     Data.Length = Length;
//     Data.Buffer = KdpMessageBuffer;
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_DEBUG_IO, &Header, &Data, &KdpContext);
//
//     /* Check if the user pressed CTRL+C */
//     return KdpPollBreakInWithPortLock();
// }
//
// BOOLEAN
// NTAPI
// KdpPromptString(
//     _In_ PSTRING PromptString,
//     _In_ PSTRING ResponseString)
// {
//     STRING Data, Header;
//     DBGKD_DEBUG_IO DebugIo;
//     ULONG Length;
//     KDSTATUS Status;
//
//     /* Copy the string to the message buffer */
//     KdpMoveMemory(KdpMessageBuffer,
//                   PromptString->Buffer,
//                   PromptString->Length);
//
//     /* Make sure we don't exceed the KD Packet size */
//     Length = PromptString->Length;
//     if ((sizeof(DBGKD_DEBUG_IO) + Length) > PACKET_MAX_SIZE)
//     {
//         /* Normalize length */
//         Length = PACKET_MAX_SIZE - sizeof(DBGKD_DEBUG_IO);
//     }
//
//     /* Build the packet header */
//     DebugIo.ApiNumber = DbgKdGetStringApi;
//     DebugIo.ProcessorLevel = (USHORT)KeProcessorLevel;
//     DebugIo.Processor = KeGetCurrentPrcb()->Number;
//     DebugIo.u.GetString.LengthOfPromptString = Length;
//     DebugIo.u.GetString.LengthOfStringRead = ResponseString->MaximumLength;
//     Header.Length = sizeof(DBGKD_DEBUG_IO);
//     Header.Buffer = (PCHAR)&DebugIo;
//
//     /* Build the data */
//     Data.Length = Length;
//     Data.Buffer = KdpMessageBuffer;
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_DEBUG_IO, &Header, &Data, &KdpContext);
//
//     /* Set the maximum lengths for the receive */
//     Header.MaximumLength = sizeof(DBGKD_DEBUG_IO);
//     Data.MaximumLength = sizeof(KdpMessageBuffer);
//
//     /* Enter receive loop */
//     do
//     {
//         /* Get our reply */
//         Status = KdReceivePacket(PACKET_TYPE_KD_DEBUG_IO,
//                                  &Header,
//                                  &Data,
//                                  &Length,
//                                  &KdpContext);
//
//         /* Return TRUE if we need to resend */
//         if (Status == KdPacketNeedsResend) return TRUE;
//
//     /* Loop until we succeed */
//     } while (Status != KdPacketReceived);
//
//     /* Don't copy back a larger response than there is room for */
//     Length = min(Length,
//                  ResponseString->MaximumLength);
//
//     /* Copy back the string and return the length */
//     KdpMoveMemory(ResponseString->Buffer,
//                   KdpMessageBuffer,
//                   Length);
//     ResponseString->Length = (USHORT)Length;
//
//     /* Success; we don't need to resend */
//     return FALSE;
// }
//
// VOID
// NTAPI
// KdpCommandString(IN PSTRING NameString,
//                  IN PSTRING CommandString,
//                  IN KPROCESSOR_MODE PreviousMode,
//                  IN PCONTEXT ContextRecord,
//                  IN PKTRAP_FRAME TrapFrame,
//                  IN PKEXCEPTION_FRAME ExceptionFrame)
// {
//     BOOLEAN Enable;
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     /* Check if we need to do anything */
//     if ((PreviousMode != KernelMode) || (KdDebuggerNotPresent)) return;
//
//     /* Enter the debugger */
//     Enable = KdEnterDebugger(TrapFrame, ExceptionFrame);
//
//     /* Save the CPU Control State and save the context */
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
//     KdpMoveMemory(&Prcb->ProcessorState.ContextFrame,
//                   ContextRecord,
//                   sizeof(CONTEXT));
//
//     /* Send the command string to the debugger */
//     KdpReportCommandStringStateChange(NameString,
//                                       CommandString,
//                                       &Prcb->ProcessorState.ContextFrame);
//
//     /* Restore the processor state */
//     KdpMoveMemory(ContextRecord,
//                   &Prcb->ProcessorState.ContextFrame,
//                   sizeof(CONTEXT));
//     KiRestoreProcessorControlState(&Prcb->ProcessorState);
//
//     /* Exit the debugger and return */
//     KdExitDebugger(Enable);
// }
//
// VOID
// NTAPI
// KdpSymbol(IN PSTRING DllPath,
//           IN PKD_SYMBOLS_INFO SymbolInfo,
//           IN BOOLEAN Unload,
//           IN KPROCESSOR_MODE PreviousMode,
//           IN PCONTEXT ContextRecord,
//           IN PKTRAP_FRAME TrapFrame,
//           IN PKEXCEPTION_FRAME ExceptionFrame)
// {
//     BOOLEAN Enable;
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     /* Check if we need to do anything */
//     if ((PreviousMode != KernelMode) || (KdDebuggerNotPresent)) return;
//
//     /* Enter the debugger */
//     Enable = KdEnterDebugger(TrapFrame, ExceptionFrame);
//
//     /* Save the CPU Control State and save the context */
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
//     KdpMoveMemory(&Prcb->ProcessorState.ContextFrame,
//                   ContextRecord,
//                   sizeof(CONTEXT));
//
//     /* Report the new state */
//     KdpReportLoadSymbolsStateChange(DllPath,
//                                     SymbolInfo,
//                                     Unload,
//                                     &Prcb->ProcessorState.ContextFrame);
//
//     /* Restore the processor state */
//     KdpMoveMemory(ContextRecord,
//                   &Prcb->ProcessorState.ContextFrame,
//                   sizeof(CONTEXT));
//     KiRestoreProcessorControlState(&Prcb->ProcessorState);
//
//     /* Exit the debugger and return */
//     KdExitDebugger(Enable);
// }
//
// USHORT
// NTAPI
// KdpPrompt(
//     _In_reads_bytes_(PromptLength) PCHAR PromptString,
//     _In_ USHORT PromptLength,
//     _Out_writes_bytes_(MaximumResponseLength) PCHAR ResponseString,
//     _In_ USHORT MaximumResponseLength,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _In_ PKTRAP_FRAME TrapFrame,
//     _In_ PKEXCEPTION_FRAME ExceptionFrame)
// {
//     STRING PromptBuffer, ResponseBuffer;
//     BOOLEAN Enable, Resend;
//     PCHAR SafeResponseString;
//     CHAR CapturedPrompt[KD_PRINT_MAX_BYTES];
//     CHAR SafeResponseBuffer[KD_PRINT_MAX_BYTES];
//
//     /* Normalize the lengths */
//     PromptLength = min(PromptLength,
//                        sizeof(CapturedPrompt));
//     MaximumResponseLength = min(MaximumResponseLength,
//                                 sizeof(SafeResponseBuffer));
//
//     /* Check if we need to verify the string */
//     if (PreviousMode != KernelMode)
//     {
//         /* Handle user-mode buffers safely */
//         _SEH2_TRY
//         {
//             /* Probe and capture the prompt */
//             ProbeForRead(PromptString, PromptLength, 1);
//             KdpMoveMemory(CapturedPrompt, PromptString, PromptLength);
//             PromptString = CapturedPrompt;
//
//             /* Probe and make room for the response */
//             ProbeForWrite(ResponseString, MaximumResponseLength, 1);
//             SafeResponseString = SafeResponseBuffer;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Bad string pointer, bail out */
//             _SEH2_YIELD(return 0);
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         SafeResponseString = ResponseString;
//     }
//
//     /* Setup the prompt and response buffers */
//     PromptBuffer.Buffer = PromptString;
//     PromptBuffer.Length = PromptBuffer.MaximumLength = PromptLength;
//     ResponseBuffer.Buffer = SafeResponseString;
//     ResponseBuffer.Length = 0;
//     ResponseBuffer.MaximumLength = MaximumResponseLength;
//
//     /* Log the print */
//     KdLogDbgPrint(&PromptBuffer);
//
//     /* Enter the debugger */
//     Enable = KdEnterDebugger(TrapFrame, ExceptionFrame);
//
//     /* Enter prompt loop */
//     do
//     {
//         /* Send the prompt and receive the response */
//         Resend = KdpPromptString(&PromptBuffer, &ResponseBuffer);
//
//     /* Loop while we need to resend */
//     } while (Resend);
//
//     /* Exit the debugger */
//     KdExitDebugger(Enable);
//
//     /* Copy back the response if required */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Safely copy back the response to user mode */
//             KdpMoveMemory(ResponseString,
//                           ResponseBuffer.Buffer,
//                           ResponseBuffer.Length);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* String became invalid after we exited, fail */
//             _SEH2_YIELD(return 0);
//         }
//         _SEH2_END;
//     }
//
//     /* Return the number of characters received */
//     return ResponseBuffer.Length;
// }
//
// static
// NTSTATUS
// NTAPI
// KdpPrintFromUser(
//     _In_ ULONG ComponentId,
//     _In_ ULONG Level,
//     _In_reads_bytes_(Length) PCHAR String,
//     _In_ USHORT Length,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _In_ PKTRAP_FRAME TrapFrame,
//     _In_ PKEXCEPTION_FRAME ExceptionFrame,
//     _Out_ PBOOLEAN Handled)
// {
//     CHAR CapturedString[KD_PRINT_MAX_BYTES];
//
//     ASSERT(PreviousMode == UserMode);
//     ASSERT(Length <= sizeof(CapturedString));
//
//     /* Capture user-mode buffers */
//     _SEH2_TRY
//     {
//         /* Probe and capture the string */
//         ProbeForRead(String, Length, 1);
//         KdpMoveMemory(CapturedString, String, Length);
//         String = CapturedString;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Bad string pointer, bail out */
//         _SEH2_YIELD(return STATUS_ACCESS_VIOLATION);
//     }
//     _SEH2_END;
//
//     /* Now go through the kernel-mode code path */
//     return KdpPrint(ComponentId,
//                     Level,
//                     String,
//                     Length,
//                     KernelMode,
//                     TrapFrame,
//                     ExceptionFrame,
//                     Handled);
// }
//
// NTSTATUS
// NTAPI
// KdpPrint(
//     _In_ ULONG ComponentId,
//     _In_ ULONG Level,
//     _In_reads_bytes_(Length) PCHAR String,
//     _In_ USHORT Length,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _In_ PKTRAP_FRAME TrapFrame,
//     _In_ PKEXCEPTION_FRAME ExceptionFrame,
//     _Out_ PBOOLEAN Handled)
// {
//     NTSTATUS Status;
//     BOOLEAN Enable;
//     STRING OutputString;
//
//     if (NtQueryDebugFilterState(ComponentId, Level) == (NTSTATUS)FALSE)
//     {
//         /* Mask validation failed */
//         *Handled = TRUE;
//         return STATUS_SUCCESS;
//     }
//
//     /* Assume failure */
//     *Handled = FALSE;
//
//     /* Normalize the length */
//     Length = min(Length, KD_PRINT_MAX_BYTES);
//
//     /* Check if we need to verify the string */
//     if (PreviousMode != KernelMode)
//     {
//         /* This case requires a 512 byte stack buffer.
//          * We don't want to use that much stack in the kernel case, but we
//          * can't use _alloca due to PSEH. So the buffer exists in this
//          * helper function instead.
//          */
//         return KdpPrintFromUser(ComponentId,
//                                 Level,
//                                 String,
//                                 Length,
//                                 PreviousMode,
//                                 TrapFrame,
//                                 ExceptionFrame,
//                                 Handled);
//     }
//
//     /* Setup the output string */
//     OutputString.Buffer = String;
//     OutputString.Length = OutputString.MaximumLength = Length;
//
//     /* Log the print */
//     KdLogDbgPrint(&OutputString);
//
//     /* Check for a debugger */
//     if (KdDebuggerNotPresent)
//     {
//         /* Fail */
//         *Handled = TRUE;
//         return STATUS_DEVICE_NOT_CONNECTED;
//     }
//
//     /* Enter the debugger */
//     Enable = KdEnterDebugger(TrapFrame, ExceptionFrame);
//
//     /* Print the string */
//     if (KdpPrintString(&OutputString))
//     {
//         /* User pressed CTRL-C, breakpoint on return */
//         Status = STATUS_BREAKPOINT;
//     }
//     else
//     {
//         /* String was printed */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Exit the debugger and return */
//     KdExitDebugger(Enable);
//     *Handled = TRUE;
//     return Status;
// }
//
// VOID
// __cdecl
// KdpDprintf(
//     _In_ PCSTR Format,
//     ...)
// {
//     STRING String;
//     USHORT Length;
//     va_list ap;
//     CHAR Buffer[512];
//
//     /* Format the string */
//     va_start(ap, Format);
//     Length = (USHORT)_vsnprintf(Buffer,
//                                 sizeof(Buffer),
//                                 Format,
//                                 ap);
//     va_end(ap);
//
//     /* Set it up */
//     String.Buffer = Buffer;
//     String.Length = String.MaximumLength = Length;
//
//     /* Send it to the debugger directly */
//     KdpPrintString(&String);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
