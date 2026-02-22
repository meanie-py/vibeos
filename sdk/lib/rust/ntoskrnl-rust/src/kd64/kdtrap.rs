use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{CONTEXT, KPROCESSOR_MODE, KiRestoreProcessorControlState, KiSaveProcessorControlState, PCONTEXT, PEXCEPTION_RECORD, PKD_SYMBOLS_INFO, PKEXCEPTION_FRAME, PKTRAP_FRAME, PSTRING};
use core::ffi::c_void;

extern "C" {
    fn KdpMoveMemory(Destination: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Length: usize);
    fn KdpGetParameterThree(ContextRecord: PCONTEXT) -> ULONG_PTR;
    fn KdpGetParameterFour(ContextRecord: PCONTEXT) -> ULONG_PTR;
    fn KdpPrompt( Prompt: PCHAR, PromptLength: USHORT, Response: PCHAR, MaximumResponseLength: USHORT, PreviousMode: KPROCESSOR_MODE, TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME, ) -> USHORT;
    fn KeGetContextPc(ContextRecord: PCONTEXT) -> ULONG_PTR;
    fn KeSetContextPc(ContextRecord: PCONTEXT, ProgramCounter: ULONG_PTR);
    fn KeSetContextReturnRegister(ContextRecord: PCONTEXT, ReturnValue: ULONG_PTR);
}

const STATUS_BREAKPOINT: NTSTATUS = 0x80000003u32 as i32;
const BREAKPOINT_BREAK: ULONG_PTR = 0;
const BREAKPOINT_PRINT: ULONG_PTR = 1;
const BREAKPOINT_PROMPT: ULONG_PTR = 2;
const BREAKPOINT_LOAD_SYMBOLS: ULONG_PTR = 3;
const BREAKPOINT_UNLOAD_SYMBOLS: ULONG_PTR = 4;
const BREAKPOINT_COMMAND_STRING: ULONG_PTR = 5;
const KD_BREAKPOINT_SIZE: ULONG_PTR = core::mem::size_of::<u8>() as ULONG_PTR;

#[no_mangle]
pub unsafe extern "system" fn KdpReport(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    ExceptionRecord: PEXCEPTION_RECORD,
    ContextRecord: PCONTEXT,
    PreviousMode: KPROCESSOR_MODE,
    SecondChanceException: u8
) -> u8 {
    let Enable: u8;
    let Handled: u8;
    let Prcb: PKPRCB;
    let ExceptionCode: NTSTATUS;

    ExceptionCode = (*ExceptionRecord).ExceptionCode;
    if (ExceptionCode == STATUS_BREAKPOINT) ||
        (ExceptionCode == STATUS_SINGLE_STEP) ||
        (ExceptionCode == STATUS_ASSERTION_FAILURE)
    {
    }
    else if (NtGlobalFlag & FLG_STOP_ON_EXCEPTION) != 0
    {
        if (SecondChanceException == 0u8) &&
            ((ExceptionCode == STATUS_PORT_DISCONNECTED) ||
             (NT_SUCCESS(ExceptionCode)))
        {
            return 0u8;
        }
    }
    else if SecondChanceException == 0u8
    {
        return 0u8;
    }

    Enable = KdEnterDebugger(TrapFrame, ExceptionFrame);

    Prcb = KeGetCurrentPrcb();
    KiSaveProcessorControlState(core::ptr::addr_of_mut!((*Prcb).ProcessorState));
    KdpMoveMemory(
        core::ptr::addr_of_mut!((*Prcb).ProcessorState.ContextFrame) as *mut core::ffi::c_void,
        ContextRecord as *const core::ffi::c_void,
        core::mem::size_of::<CONTEXT>()
    );

    Handled = KdpReportExceptionStateChange(
        ExceptionRecord,
        core::ptr::addr_of_mut!((*Prcb).ProcessorState.ContextFrame),
        SecondChanceException
    );

    KdpMoveMemory(
        ContextRecord as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*Prcb).ProcessorState.ContextFrame) as *const core::ffi::c_void,
        core::mem::size_of::<CONTEXT>()
    );
    KiRestoreProcessorControlState(core::ptr::addr_of_mut!((*Prcb).ProcessorState));

    KdExitDebugger(Enable);
    KdpControlCPressed = 0u8;
    return Handled;
}

#[no_mangle]
pub unsafe extern "system" fn KdpTrap(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    ExceptionRecord: PEXCEPTION_RECORD,
    ContextRecord: PCONTEXT,
    PreviousMode: KPROCESSOR_MODE,
    SecondChanceException: u8,
) -> u8 {
    let mut Unload: u8;
    let ProgramCounter: ULONG_PTR;
    let mut Handled: u8;
    let ReturnStatus: NTSTATUS;
    let ReturnLength: USHORT;

    if ((*ExceptionRecord).ExceptionCode == STATUS_BREAKPOINT) &&
        ((*ExceptionRecord).ExceptionInformation[0] != BREAKPOINT_BREAK)
    {
        ProgramCounter = KeGetContextPc(ContextRecord);

        Unload = 0;
        match (*ExceptionRecord).ExceptionInformation[0] {
            BREAKPOINT_PRINT => {
                ReturnStatus = KdpPrint(
                    KdpGetParameterThree(ContextRecord) as ULONG,
                    KdpGetParameterFour(ContextRecord) as ULONG,
                    (*ExceptionRecord).ExceptionInformation[1] as PCHAR,
                    (*ExceptionRecord).ExceptionInformation[2] as USHORT,
                    PreviousMode,
                    TrapFrame,
                    ExceptionFrame,
                    &mut Handled,
                );

                KeSetContextReturnRegister(ContextRecord, ReturnStatus as ULONG_PTR);
            }

            BREAKPOINT_PROMPT => {
                ReturnLength = KdpPrompt(
                    (*ExceptionRecord).ExceptionInformation[1] as PCHAR,
                    (*ExceptionRecord).ExceptionInformation[2] as USHORT,
                    KdpGetParameterThree(ContextRecord) as PCHAR,
                    KdpGetParameterFour(ContextRecord) as USHORT,
                    PreviousMode,
                    TrapFrame,
                    ExceptionFrame,
                );
                Handled = 1;

                KeSetContextReturnRegister(ContextRecord, ReturnLength as ULONG_PTR);
            }

            BREAKPOINT_UNLOAD_SYMBOLS => {
                Unload = 1;

                KdpSymbol(
                    (*ExceptionRecord).ExceptionInformation[1] as PSTRING,
                    (*ExceptionRecord).ExceptionInformation[2] as PKD_SYMBOLS_INFO,
                    Unload,
                    PreviousMode,
                    ContextRecord,
                    TrapFrame,
                    ExceptionFrame,
                );
                Handled = 1;
            }

            BREAKPOINT_LOAD_SYMBOLS => {
                KdpSymbol(
                    (*ExceptionRecord).ExceptionInformation[1] as PSTRING,
                    (*ExceptionRecord).ExceptionInformation[2] as PKD_SYMBOLS_INFO,
                    Unload,
                    PreviousMode,
                    ContextRecord,
                    TrapFrame,
                    ExceptionFrame,
                );
                Handled = 1;
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
                Handled = 1;
            }

            _ => {
                Handled = 0;
            }
        }

        if ProgramCounter == KeGetContextPc(ContextRecord) {
            KeSetContextPc(ContextRecord, ProgramCounter + KD_BREAKPOINT_SIZE);
        }
    }
    else
    {
        Handled = KdpReport(
            TrapFrame,
            ExceptionFrame,
            ExceptionRecord,
            ContextRecord,
            PreviousMode,
            SecondChanceException,
        );
    }

    Handled
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
    let ExceptionCommand: ULONG_PTR;

    ExceptionCommand = (*ExceptionRecord).ExceptionInformation[0];
    if ((*ExceptionRecord).ExceptionCode == STATUS_BREAKPOINT)
        && ((*ExceptionRecord).NumberParameters > 0)
        && ((ExceptionCommand == BREAKPOINT_LOAD_SYMBOLS)
            || (ExceptionCommand == BREAKPOINT_UNLOAD_SYMBOLS)
            || (ExceptionCommand == BREAKPOINT_COMMAND_STRING)
            || (ExceptionCommand == BREAKPOINT_PRINT))
    {
        KeSetContextPc(
            ContextRecord,
            KeGetContextPc(ContextRecord) + KD_BREAKPOINT_SIZE as ULONG_PTR,
        );
        return 1u8;
    } else if KdPitchDebugger != 0 {
        return 0u8;
    } else if (KdAutoEnableOnEvent != 0)
        && (KdPreviouslyEnabled != 0)
        && (KdDebuggerEnabled == 0)
        && (NT_SUCCESS(KdEnableDebugger()))
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
        return 0u8;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdIsThisAKdTrap(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
    PreviousMode: KPROCESSOR_MODE,
) -> u8 {
    if ((*ExceptionRecord).ExceptionCode == 0x80000003u32)
        && ((*ExceptionRecord).NumberParameters > 0)
        && ((*ExceptionRecord).ExceptionInformation[0] != 0)
    {
        1u8
    } else {
        0u8
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
