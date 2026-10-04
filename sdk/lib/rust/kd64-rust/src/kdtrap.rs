use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::globals::FLG_STOP_ON_EXCEPTION;
use ntkrnl_sys::ke::*;

use ntkrnl_sys::ddk::{
    _KPRCB, CONTEXT, KPROCESSOR_MODE, KiRestoreProcessorControlState, KiSaveProcessorControlState,
    PCONTEXT, PEXCEPTION_RECORD, PKD_SYMBOLS_INFO, PKEXCEPTION_FRAME, PKTRAP_FRAME, PSTRING,
};

use super::kdapi::KdpMoveMemory;

// Breakpoint sub-ID constants from kdtypes.h
const BREAKPOINT_BREAK: ULONG = 0;
const BREAKPOINT_PRINT: ULONG = 1;
const BREAKPOINT_PROMPT: ULONG = 2;
const BREAKPOINT_LOAD_SYMBOLS: ULONG = 3;
const BREAKPOINT_UNLOAD_SYMBOLS: ULONG = 4;
const BREAKPOINT_COMMAND_STRING: ULONG = 5;

#[cfg(target_arch = "x86")]
#[inline]
unsafe fn KdpGetParameterThree(context: PCONTEXT) -> u32 {
    (*context).Ebx
}

#[cfg(target_arch = "x86")]
#[inline]
unsafe fn KdpGetParameterFour(context: PCONTEXT) -> u32 {
    (*context).Edi
}

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn KdpGetParameterThree(context: PCONTEXT) -> u64 {
    (*context).R8
}

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn KdpGetParameterFour(context: PCONTEXT) -> u64 {
    (*context).R9
}

#[cfg(target_arch = "arm")]
#[inline]
unsafe fn KdpGetParameterThree(context: PCONTEXT) -> u32 {
    (*context).R3
}

#[cfg(target_arch = "arm")]
#[inline]
unsafe fn KdpGetParameterFour(context: PCONTEXT) -> u32 {
    (*context).R4
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn KdpGetParameterThree(context: PCONTEXT) -> u64 {
    (*context).__bindgen_anon_1.__bindgen_anon_1.X2
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn KdpGetParameterFour(context: PCONTEXT) -> u64 {
    (*context).__bindgen_anon_1.__bindgen_anon_1.X3
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn KdpReadPortUchar(port: u16) -> UCHAR {
    let value: UCHAR;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn KdpWritePortUchar(port: u16, value: UCHAR) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

extern "system" {
    static NtGlobalFlag: ULONG;
    static mut KdpControlCPressed: BOOLEAN;
    static KdPitchDebugger: BOOLEAN;
    static KdAutoEnableOnEvent: BOOLEAN;
    static KdPreviouslyEnabled: BOOLEAN;
    static KdDebuggerEnabled: BOOLEAN;
    fn KdEnterDebugger(TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME) -> BOOLEAN;
    fn KdExitDebugger(Enable: BOOLEAN);
    fn KdpReportExceptionStateChange( ExceptionRecord: PEXCEPTION_RECORD, ContextRecord: PCONTEXT, SecondChanceException: BOOLEAN, ) -> BOOLEAN;
    fn KdpPrint( ComponentId: ULONG, Level: ULONG, String: PCHAR, Length: USHORT, PreviousMode: KPROCESSOR_MODE, TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME, Handled: *mut BOOLEAN, ) -> NTSTATUS;
    fn KdpPrompt( PromptString: PCHAR, PromptLength: USHORT, ResponseString: PCHAR, ResponseLength: USHORT, PreviousMode: KPROCESSOR_MODE, TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME, ) -> USHORT;
    fn KdpSymbol( SymbolPath: PSTRING, SymbolInfo: PKD_SYMBOLS_INFO, Unload: BOOLEAN, PreviousMode: KPROCESSOR_MODE, ContextRecord: PCONTEXT, TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME, );
    fn KdpCommandString( CommandString: PSTRING, ResponseString: PSTRING, PreviousMode: KPROCESSOR_MODE, ContextRecord: PCONTEXT, TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME, );
    fn KdEnableDebugger() -> NTSTATUS;
}
extern "C" {
    fn KeGetContextPc(Context: PCONTEXT) -> ULONG_PTR;
    fn KeSetContextPc(Context: PCONTEXT, ProgramCounter: ULONG_PTR);
    fn KeSetContextReturnRegister(Context: PCONTEXT, ReturnValue: ULONG_PTR);
}
#[no_mangle]
pub unsafe extern "system" fn KdpReport(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    ExceptionRecord: PEXCEPTION_RECORD,
    ContextRecord: PCONTEXT,
    PreviousMode: KPROCESSOR_MODE,
    SecondChanceException: BOOLEAN,
) -> BOOLEAN {
    let mut enable: BOOLEAN;
    let handled: BOOLEAN;
    let prcb: *mut _KPRCB;
    let exception_code: NTSTATUS;

    exception_code = (*ExceptionRecord).ExceptionCode;

    if exception_code == STATUS_ASSERTION_FAILURE && SecondChanceException == 0 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            KeSetContextPc(ContextRecord, KeGetContextPc(ContextRecord).wrapping_add(2));
            return 1;
        }
        #[cfg(target_arch = "aarch64")]
        {
            KeSetContextPc(ContextRecord, KeGetContextPc(ContextRecord).wrapping_add(4));
            return 1;
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if exception_code == STATUS_BREAKPOINT && SecondChanceException == 0 {
        let ip = KeGetContextPc(ContextRecord) as *const UCHAR;
        if core::ptr::read(ip) == 0xCC
            && core::ptr::read(ip.add(1)) == 0xCD
            && core::ptr::read(ip.add(2)) == 0x2C
        {
            KeSetContextPc(ContextRecord, KeGetContextPc(ContextRecord).wrapping_add(1));
            return 1;
        }
    }

    if (exception_code == STATUS_BREAKPOINT)
        || (exception_code == STATUS_SINGLE_STEP)
        || (exception_code == STATUS_ASSERTION_FAILURE)
    {
    } else if (NtGlobalFlag & FLG_STOP_ON_EXCEPTION) != 0 {
        if (SecondChanceException == 0)
            && ((exception_code == STATUS_PORT_DISCONNECTED) || (exception_code >= 0))
        {
            return 0;
        }
    } else if SecondChanceException == 0 {
        return 0;
    }

    enable = KdEnterDebugger(TrapFrame, ExceptionFrame);

    prcb = KeGetCurrentPrcb() as *mut _KPRCB;
    KiSaveProcessorControlState(core::ptr::addr_of_mut!((*prcb).ProcessorState));
    KdpMoveMemory(
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame) as PVOID,
        ContextRecord as PVOID,
        core::mem::size_of::<CONTEXT>(),
    );

    handled = KdpReportExceptionStateChange(
        ExceptionRecord,
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame),
        SecondChanceException,
    );

    KdpMoveMemory(
        ContextRecord as PVOID,
        core::ptr::addr_of_mut!((*prcb).ProcessorState.ContextFrame) as PVOID,
        core::mem::size_of::<CONTEXT>(),
    );
    KiRestoreProcessorControlState(core::ptr::addr_of_mut!((*prcb).ProcessorState));

    KdExitDebugger(enable);
    KdpControlCPressed = 0;
    handled
}

#[no_mangle]
pub unsafe extern "system" fn KdpTrap(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    ExceptionRecord: PEXCEPTION_RECORD,
    ContextRecord: PCONTEXT,
    PreviousMode: KPROCESSOR_MODE,
    SecondChanceException: BOOLEAN,
) -> BOOLEAN {
    let mut unload: BOOLEAN;
    let mut program_counter: ULONG_PTR;
    let mut handled: BOOLEAN = 0;
    let mut return_status: NTSTATUS;
    let mut return_length: USHORT;

    if ((*ExceptionRecord).ExceptionCode == STATUS_BREAKPOINT)
        && ((*ExceptionRecord).ExceptionInformation[0] as ULONG != BREAKPOINT_BREAK)
    {
        program_counter = KeGetContextPc(ContextRecord);

        unload = 0;
        match (*ExceptionRecord).ExceptionInformation[0] as ULONG {
            BREAKPOINT_PRINT => {
                return_status = KdpPrint(
                    KdpGetParameterThree(ContextRecord) as ULONG,
                    KdpGetParameterFour(ContextRecord) as ULONG,
                    (*ExceptionRecord).ExceptionInformation[1] as PCHAR,
                    (*ExceptionRecord).ExceptionInformation[2] as USHORT,
                    PreviousMode,
                    TrapFrame,
                    ExceptionFrame,
                    &mut handled,
                );

                KeSetContextReturnRegister(ContextRecord, return_status as ULONG_PTR);
            }

            BREAKPOINT_PROMPT => {
                return_length = KdpPrompt(
                    (*ExceptionRecord).ExceptionInformation[1] as PCHAR,
                    (*ExceptionRecord).ExceptionInformation[2] as USHORT,
                    KdpGetParameterThree(ContextRecord) as PCHAR,
                    KdpGetParameterFour(ContextRecord) as USHORT,
                    PreviousMode,
                    TrapFrame,
                    ExceptionFrame,
                );
                handled = 1;

                KeSetContextReturnRegister(ContextRecord, return_length as ULONG_PTR);
            }

            BREAKPOINT_UNLOAD_SYMBOLS => {
                unload = 1;

                KdpSymbol(
                    (*ExceptionRecord).ExceptionInformation[1] as PSTRING,
                    (*ExceptionRecord).ExceptionInformation[2] as PKD_SYMBOLS_INFO,
                    unload,
                    PreviousMode,
                    ContextRecord,
                    TrapFrame,
                    ExceptionFrame,
                );
                handled = 1;
            }

            BREAKPOINT_LOAD_SYMBOLS => {
                KdpSymbol(
                    (*ExceptionRecord).ExceptionInformation[1] as PSTRING,
                    (*ExceptionRecord).ExceptionInformation[2] as PKD_SYMBOLS_INFO,
                    unload,
                    PreviousMode,
                    ContextRecord,
                    TrapFrame,
                    ExceptionFrame,
                );
                handled = 1;
            }

            BREAKPOINT_COMMAND_STRING => {
                KdpCommandString(
                    (*ExceptionRecord).ExceptionInformation[1] as PSTRING,
                    (*ExceptionRecord).ExceptionInformation[2] as PSTRING,
                    PreviousMode,
                    ContextRecord,
                    TrapFrame,
                    ExceptionFrame,
                );
                handled = 1;
            }

            _ => {
                handled = 0;
            }
        }

        if program_counter == KeGetContextPc(ContextRecord) {
            KeSetContextPc(ContextRecord, program_counter + KD_BREAKPOINT_SIZE as ULONG_PTR);
        }
    } else {
        handled = KdpReport(
            TrapFrame,
            ExceptionFrame,
            ExceptionRecord,
            ContextRecord,
            PreviousMode,
            SecondChanceException,
        );
    }

    handled
}

extern "C" {
    /// The UART KdpStub's text path writes DbgPrint to while real KD is off,
    /// shared with rtl's kernel-mode DebugPrint (rtl-diag owns it). COM1 unless
    /// kdinit.rs moved it for `DEBUGPORT=COM:0x<port>`: a PC without a legacy
    /// COM1 publishes its console UART in ACPI SPCR, and the loader hands it over.
    pub static mut RtlpDebugConsolePort: u16;
}

#[no_mangle]
pub unsafe extern "system" fn KdpStub(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    ExceptionRecord: PEXCEPTION_RECORD,
    ContextRecord: PCONTEXT,
    PreviousMode: KPROCESSOR_MODE,
    SecondChanceException: BOOLEAN,
) -> BOOLEAN {
    let exception_command: ULONG;

    exception_command = (*ExceptionRecord).ExceptionInformation[0] as ULONG;
    if ((*ExceptionRecord).ExceptionCode == STATUS_BREAKPOINT)
        && ((*ExceptionRecord).NumberParameters > 0)
        && ((exception_command == BREAKPOINT_LOAD_SYMBOLS)
            || (exception_command == BREAKPOINT_UNLOAD_SYMBOLS)
            || (exception_command == BREAKPOINT_COMMAND_STRING)
            || (exception_command == BREAKPOINT_PRINT))
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if exception_command == BREAKPOINT_PRINT {
            let buffer = (*ExceptionRecord).ExceptionInformation[1] as *const UCHAR;
            let length = (*ExceptionRecord).ExceptionInformation[2] as USHORT;
            let port = RtlpDebugConsolePort;
            let lcr = KdpReadPortUchar(port + 3);
            if (lcr & 0x80) != 0 {
                KdpWritePortUchar(port + 3, lcr & 0x7F);
            }
            let mut i: USHORT = 0;
            while i < length {
                KdpWritePortUchar(port, *buffer.add(i as usize));
                i += 1;
            }
        }

        // DbgPrint returns what this leaves in the return register, and nothing set it: the
        // caller got its own service code back (BREAKPOINT_PRINT == 1), so every user-mode
        // DPRINT1 read "failed" and printed a second "DbgPrint() failed!" line. KdpTrap sets
        // it; with no debugger attached a print succeeded, which is what Windows reports.
        if exception_command == BREAKPOINT_PRINT {
            KeSetContextReturnRegister(ContextRecord, STATUS_SUCCESS as ULONG_PTR);
        }
        KeSetContextPc(
            ContextRecord,
            KeGetContextPc(ContextRecord) + KD_BREAKPOINT_SIZE as ULONG_PTR,
        );
        return 1;
    } else if KdPitchDebugger != 0 {
        return 0;
    } else if (KdAutoEnableOnEvent != 0)
        && (KdPreviouslyEnabled != 0)
        && (KdDebuggerEnabled == 0)
        && (KdEnableDebugger() >= 0)
        && (KdDebuggerEnabled != 0)
    {
        return KdpTrap(
            TrapFrame,
            ExceptionFrame,
            ExceptionRecord,
            ContextRecord,
            PreviousMode,
            SecondChanceException,
        );
    } else {
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdIsThisAKdTrap(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
    PreviousMode: KPROCESSOR_MODE,
) -> BOOLEAN {
    if ((*ExceptionRecord).ExceptionCode == STATUS_BREAKPOINT)
        && ((*ExceptionRecord).NumberParameters > 0)
        && ((*ExceptionRecord).ExceptionInformation[0] as ULONG != BREAKPOINT_BREAK)
    {
        return 1;
    } else {
        return 0;
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/kd64/kdtrap.c
//  * PURPOSE:         KD64 Trap Handlers
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
// //
// // Retrieves the ComponentId and Level for BREAKPOINT_PRINT
// // and OutputString and OutputStringLength for BREAKPOINT_PROMPT.
// //
// #if defined(_X86_)
//
// //
// // EBX/EDI on x86
// //
// #define KdpGetParameterThree(Context)  ((Context)->Ebx)
// #define KdpGetParameterFour(Context)   ((Context)->Edi)
//
// #elif defined(_AMD64_)
//
// //
// // R8/R9 on AMD64
// //
// #define KdpGetParameterThree(Context)  ((Context)->R8)
// #define KdpGetParameterFour(Context)   ((Context)->R9)
//
// #elif defined(_ARM_)
//
// //
// // R3/R4 on ARM
// //
// #define KdpGetParameterThree(Context)  ((Context)->R3)
// #define KdpGetParameterFour(Context)   ((Context)->R4)
//
// #elif defined(_ARM64_)
//
// //
// // X2/X3 on ARM64
// //
// #define KdpGetParameterThree(Context)  ((Context)->X2)
// #define KdpGetParameterFour(Context)   ((Context)->X3)
//
// #else
// #error Unsupported Architecture
// #endif
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// NTAPI
// KdpReport(IN PKTRAP_FRAME TrapFrame,
//           IN PKEXCEPTION_FRAME ExceptionFrame,
//           IN PEXCEPTION_RECORD ExceptionRecord,
//           IN PCONTEXT ContextRecord,
//           IN KPROCESSOR_MODE PreviousMode,
//           IN BOOLEAN SecondChanceException)
// {
//     BOOLEAN Enable, Handled;
//     PKPRCB Prcb;
//     NTSTATUS ExceptionCode;
//
//     /*
//      * Determine whether to pass the exception to the debugger.
//      * First, check if this is a "debug exception", meaning breakpoint
//      * (including debug service), single step and assertion failure exceptions.
//      */
//     ExceptionCode = ExceptionRecord->ExceptionCode;
//     if ((ExceptionCode == STATUS_BREAKPOINT) ||
//         (ExceptionCode == STATUS_SINGLE_STEP) ||
//         (ExceptionCode == STATUS_ASSERTION_FAILURE))
//     {
//         /* This is a debug exception; we always pass them to the debugger */
//     }
//     else if (NtGlobalFlag & FLG_STOP_ON_EXCEPTION)
//     {
//         /*
//          * Not a debug exception, but the stop-on-exception flag is set,
//          * meaning the debugger requests that we pass it first chance
//          * exceptions. However, some exceptions are always passed to the
//          * exception handler first, namely exceptions with a code that isn't
//          * an error or warning code, and also exceptions with the special
//          * STATUS_PORT_DISCONNECTED code (an error code).
//          */
//         if ((SecondChanceException == FALSE) &&
//             ((ExceptionCode == STATUS_PORT_DISCONNECTED) ||
//              (NT_SUCCESS(ExceptionCode))))
//         {
//             /* Let the exception handler, if any, try to handle it */
//             return FALSE;
//         }
//     }
//     else if (SecondChanceException == FALSE)
//     {
//         /*
//          * This isn't a debug exception and the stop-on-exception flag isn't set,
//          * so don't bother handling it
//          */
//         return FALSE;
//     }
//
//     /* Enter the debugger */
//     Enable = KdEnterDebugger(TrapFrame, ExceptionFrame);
//
//     /*
//      * Get the KPRCB and save the CPU Control State manually instead of
//      * using KiSaveProcessorState, since we already have a valid CONTEXT.
//      */
//     Prcb = KeGetCurrentPrcb();
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
//     KdpMoveMemory(&Prcb->ProcessorState.ContextFrame,
//                   ContextRecord,
//                   sizeof(CONTEXT));
//
//     /* Report the new state */
//     Handled = KdpReportExceptionStateChange(ExceptionRecord,
//                                             &Prcb->ProcessorState.
//                                             ContextFrame,
//                                             SecondChanceException);
//
//     /* Now restore the processor state, manually again. */
//     KdpMoveMemory(ContextRecord,
//                   &Prcb->ProcessorState.ContextFrame,
//                   sizeof(CONTEXT));
//     KiRestoreProcessorControlState(&Prcb->ProcessorState);
//
//     /* Exit the debugger and clear the CTRL-C state */
//     KdExitDebugger(Enable);
//     KdpControlCPressed = FALSE;
//     return Handled;
// }
//
// BOOLEAN
// NTAPI
// KdpTrap(IN PKTRAP_FRAME TrapFrame,
//         IN PKEXCEPTION_FRAME ExceptionFrame,
//         IN PEXCEPTION_RECORD ExceptionRecord,
//         IN PCONTEXT ContextRecord,
//         IN KPROCESSOR_MODE PreviousMode,
//         IN BOOLEAN SecondChanceException)
// {
//     BOOLEAN Unload;
//     ULONG_PTR ProgramCounter;
//     BOOLEAN Handled;
//     NTSTATUS ReturnStatus;
//     USHORT ReturnLength;
//
//     /*
//      * Check if we got a STATUS_BREAKPOINT with a SubID for Print, Prompt or
//      * Load/Unload symbols. Make sure it isn't a software breakpoint as those
//      * are handled by KdpReport.
//      */
//     if ((ExceptionRecord->ExceptionCode == STATUS_BREAKPOINT) &&
//         (ExceptionRecord->ExceptionInformation[0] != BREAKPOINT_BREAK))
//     {
//         /* Save Program Counter */
//         ProgramCounter = KeGetContextPc(ContextRecord);
//
//         /* Check what kind of operation was requested from us */
//         Unload = FALSE;
//         switch (ExceptionRecord->ExceptionInformation[0])
//         {
//             /* DbgPrint */
//             case BREAKPOINT_PRINT:
//
//                 /* Call the worker routine */
//                 ReturnStatus = KdpPrint((ULONG)KdpGetParameterThree(ContextRecord),
//                                         (ULONG)KdpGetParameterFour(ContextRecord),
//                                         (PCHAR)ExceptionRecord->ExceptionInformation[1],
//                                         (USHORT)ExceptionRecord->ExceptionInformation[2],
//                                         PreviousMode,
//                                         TrapFrame,
//                                         ExceptionFrame,
//                                         &Handled);
//
//                 /* Update the return value for the caller */
//                 KeSetContextReturnRegister(ContextRecord, ReturnStatus);
//                 break;
//
//             /* DbgPrompt */
//             case BREAKPOINT_PROMPT:
//
//                 /* Call the worker routine */
//                 ReturnLength = KdpPrompt((PCHAR)ExceptionRecord->ExceptionInformation[1],
//                                          (USHORT)ExceptionRecord->ExceptionInformation[2],
//                                          (PCHAR)KdpGetParameterThree(ContextRecord),
//                                          (USHORT)KdpGetParameterFour(ContextRecord),
//                                          PreviousMode,
//                                          TrapFrame,
//                                          ExceptionFrame);
//                 Handled = TRUE;
//
//                 /* Update the return value for the caller */
//                 KeSetContextReturnRegister(ContextRecord, ReturnLength);
//                 break;
//
//             /* DbgUnLoadImageSymbols */
//             case BREAKPOINT_UNLOAD_SYMBOLS:
//
//                 /* Drop into the load case below, with the unload parameter */
//                 Unload = TRUE;
//
//             /* DbgLoadImageSymbols */
//             case BREAKPOINT_LOAD_SYMBOLS:
//
//                 /* Call the worker routine */
//                 KdpSymbol((PSTRING)ExceptionRecord->ExceptionInformation[1],
//                           (PKD_SYMBOLS_INFO)ExceptionRecord->ExceptionInformation[2],
//                           Unload,
//                           PreviousMode,
//                           ContextRecord,
//                           TrapFrame,
//                           ExceptionFrame);
//                 Handled = TRUE;
//                 break;
//
//             /* DbgCommandString */
//             case BREAKPOINT_COMMAND_STRING:
//
//                 /* Call the worker routine */
//                 KdpCommandString((PSTRING)ExceptionRecord->ExceptionInformation[1],
//                                  (PSTRING)ExceptionRecord->ExceptionInformation[2],
//                                  PreviousMode,
//                                  ContextRecord,
//                                  TrapFrame,
//                                  ExceptionFrame);
//                 Handled = TRUE;
//                 break;
//
//             /* Anything else, do nothing */
//             default:
//
//                 /* Invalid debug service! Don't handle this! */
//                 Handled = FALSE;
//                 break;
//         }
//
//         /*
//          * If the PC was not updated, we'll increment it ourselves so execution
//          * continues past the breakpoint.
//          */
//         if (ProgramCounter == KeGetContextPc(ContextRecord))
//         {
//             /* Update it */
//             KeSetContextPc(ContextRecord,
//                            ProgramCounter + KD_BREAKPOINT_SIZE);
//         }
//     }
//     else
//     {
//         /* Call the worker routine */
//         Handled = KdpReport(TrapFrame,
//                             ExceptionFrame,
//                             ExceptionRecord,
//                             ContextRecord,
//                             PreviousMode,
//                             SecondChanceException);
//     }
//
//     /* Return TRUE or FALSE to caller */
//     return Handled;
// }
//
// BOOLEAN
// NTAPI
// KdpStub(IN PKTRAP_FRAME TrapFrame,
//         IN PKEXCEPTION_FRAME ExceptionFrame,
//         IN PEXCEPTION_RECORD ExceptionRecord,
//         IN PCONTEXT ContextRecord,
//         IN KPROCESSOR_MODE PreviousMode,
//         IN BOOLEAN SecondChanceException)
// {
//     ULONG_PTR ExceptionCommand;
//
//     /* Check if this was a breakpoint due to DbgPrint or Load/UnloadSymbols */
//     ExceptionCommand = ExceptionRecord->ExceptionInformation[0];
//     if ((ExceptionRecord->ExceptionCode == STATUS_BREAKPOINT) &&
//         (ExceptionRecord->NumberParameters > 0) &&
//         ((ExceptionCommand == BREAKPOINT_LOAD_SYMBOLS) ||
//          (ExceptionCommand == BREAKPOINT_UNLOAD_SYMBOLS) ||
//          (ExceptionCommand == BREAKPOINT_COMMAND_STRING) ||
//          (ExceptionCommand == BREAKPOINT_PRINT)))
//     {
//         /* This we can handle: simply bump the Program Counter */
//         KeSetContextPc(ContextRecord,
//                        KeGetContextPc(ContextRecord) + KD_BREAKPOINT_SIZE);
//         return TRUE;
//     }
//     else if (KdPitchDebugger)
//     {
//         /* There's no debugger, fail. */
//         return FALSE;
//     }
//     else if ((KdAutoEnableOnEvent) &&
//              (KdPreviouslyEnabled) &&
//              !(KdDebuggerEnabled) &&
//              (NT_SUCCESS(KdEnableDebugger())) &&
//              (KdDebuggerEnabled))
//     {
//         /* Debugging was Auto-Enabled. We can now send this to KD. */
//         return KdpTrap(TrapFrame,
//                        ExceptionFrame,
//                        ExceptionRecord,
//                        ContextRecord,
//                        PreviousMode,
//                        SecondChanceException);
//     }
//     else
//     {
//         /* FIXME: All we can do in this case is trace this exception */
//         return FALSE;
//     }
// }
//
// BOOLEAN
// NTAPI
// KdIsThisAKdTrap(IN PEXCEPTION_RECORD ExceptionRecord,
//                 IN PCONTEXT Context,
//                 IN KPROCESSOR_MODE PreviousMode)
// {
//     /*
//      * Determine if this is a valid debug service call and make sure that
//      * it isn't a software breakpoint
//      */
//     if ((ExceptionRecord->ExceptionCode == STATUS_BREAKPOINT) &&
//         (ExceptionRecord->NumberParameters > 0) &&
//         (ExceptionRecord->ExceptionInformation[0] != BREAKPOINT_BREAK))
//     {
//         /* Then we have to handle it */
//         return TRUE;
//     }
//     else
//     {
//         /* We don't have to handle it */
//         return FALSE;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
