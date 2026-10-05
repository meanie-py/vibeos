use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    CONTEXT, EXCEPTION_RECORD, KEXCEPTION_FRAME, KPROCESSOR_MODE,
    KeContextToTrapFrame, KeTestAlertThread, KiContinue as KiContinueExtern,
    KiExceptionExit, KiRaiseException as KiRaiseExceptionExtern,
    PCONTEXT, PEXCEPTION_RECORD, PKEXCEPTION_FRAME, PKTHREAD, PKTRAP_FRAME,
};
// KeGetPcr: FORCEINLINE on i386 (fs:[0x1c]), extern on amd64/arm64
// Need a shim for i386 — provide via ke_rust_shims.c
#[cfg(target_arch = "x86")]
extern "C" {
    fn KeGetPcr() -> *mut ntkrnl_sys::ddk::_KPCR;
}
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::KeGetPcr;

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn KiDispatchException(ExceptionRecord: PEXCEPTION_RECORD, ExceptionFrame: PKEXCEPTION_FRAME, TrapFrame: PKTRAP_FRAME, PreviousMode: KPROCESSOR_MODE, SearchFrames: BOOLEAN);
}
extern "C" {
    fn KiGetLinkedTrapFrame(TrapFrame: PKTRAP_FRAME) -> PKTRAP_FRAME;
}

const EXCEPTION_MAXIMUM_PARAMETERS: ULONG = 15;
const KI_EXCEPTION_INTERNAL: ULONG = 0x10000000;

/// C: void, but always called inside KiContinue's _SEH2_TRY, whose handler
/// stores the exception code as the return status. No SEH here, so the probe and
/// the capture copy report that code as `Err` instead of raising/faulting.
unsafe fn KiContinuePreviousModeUser(
    Context: PCONTEXT,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
) -> Result<(), NTSTATUS> {
    let mut LocalContext: CONTEXT = core::mem::zeroed();

    // C ProbeForRead checks unconditionally: UserMode (1).
    ntkrnl_sys::probe::safe_copy_from_user(
        &mut LocalContext as *mut CONTEXT as *mut core::ffi::c_void,
        Context as *const core::ffi::c_void,
        core::mem::size_of::<CONTEXT>(),
        core::mem::size_of::<ULONG>(),
        1,
    )?;

    KeContextToTrapFrame(
        &mut LocalContext,
        ExceptionFrame,
        TrapFrame,
        LocalContext.ContextFlags,
        1i8,
    );
    Ok(())
}

#[no_mangle]
pub unsafe extern "system" fn KiContinue(
    Context: PCONTEXT,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let mut OldIrql = APC_LEVEL;
    let PreviousMode = KeGetPreviousMode();

    if KeGetCurrentIrql() < APC_LEVEL {
        KeRaiseIrql(APC_LEVEL, &mut OldIrql);
    }

    if PreviousMode != 0i8 {
        if let Err(code) = KiContinuePreviousModeUser(Context, ExceptionFrame, TrapFrame) {
            Status = code;
        }
    } else {
        KeContextToTrapFrame(
            Context,
            ExceptionFrame,
            TrapFrame,
            (*Context).ContextFlags,
            0i8,
        );
    }

    if OldIrql < APC_LEVEL {
        KeLowerIrql(OldIrql);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KiRaiseException(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
    SearchFrames: BOOLEAN,
) -> NTSTATUS {
    let PreviousMode = KeGetPreviousMode();
    let mut LocalContext: CONTEXT = core::mem::zeroed();
    let mut LocalExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();
    let mut ParameterCount: ULONG = 0;
    let mut Size: usize = 0;

    let mut FinalContext = Context;
    let mut FinalExceptionRecord = ExceptionRecord;

    if PreviousMode != 0i8 {
        let probe_result: Result<(), NTSTATUS> = (|| {
            // C: one _SEH2_TRY whose handler returns the exception code (and the
            // too-many-parameters early return is STATUS_INVALID_PARAMETER). No SEH
            // here: Result probes + fault-tolerant reads/copies report the code.
            ntkrnl_sys::probe::probe_user_buffer(
                Context as PVOID,
                core::mem::size_of::<CONTEXT>(),
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            )?;

            ntkrnl_sys::probe::probe_user_buffer(
                ExceptionRecord as PVOID,
                core::mem::offset_of!(EXCEPTION_RECORD, NumberParameters) + core::mem::size_of::<ULONG>(),
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            )?;

            ParameterCount = ntkrnl_sys::probe::probe_read_generic(
                core::ptr::addr_of!((*ExceptionRecord).NumberParameters),
                1,
                PreviousMode,
            )?;
            if ParameterCount > EXCEPTION_MAXIMUM_PARAMETERS {
                return Err(STATUS_INVALID_PARAMETER);
            }

            Size = core::mem::size_of::<EXCEPTION_RECORD>() - ((EXCEPTION_MAXIMUM_PARAMETERS - ParameterCount) as usize * core::mem::size_of::<ULONG>());
            ntkrnl_sys::probe::probe_user_buffer(
                ExceptionRecord as PVOID,
                Size,
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            )?;

            ntkrnl_sys::probe::safe_copy_from_user(
                &mut LocalContext as *mut CONTEXT as *mut core::ffi::c_void,
                Context as *const core::ffi::c_void,
                core::mem::size_of::<CONTEXT>(),
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            )?;
            ntkrnl_sys::probe::safe_copy_from_user(
                &mut LocalExceptionRecord as *mut EXCEPTION_RECORD as *mut core::ffi::c_void,
                ExceptionRecord as *const core::ffi::c_void,
                Size,
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            )?;

            LocalExceptionRecord.NumberParameters = ParameterCount;

            Ok(())
        })();

        if let Err(code) = probe_result {
            return code;
        }

        FinalContext = &mut LocalContext;
        FinalExceptionRecord = &mut LocalExceptionRecord;
    }

    KeContextToTrapFrame(
        FinalContext,
        ExceptionFrame,
        TrapFrame,
        (*FinalContext).ContextFlags,
        PreviousMode,
    );

    (*FinalExceptionRecord).ExceptionCode &= !(KI_EXCEPTION_INTERNAL as i32);
    KiDispatchException(
        FinalExceptionRecord,
        ExceptionFrame,
        TrapFrame,
        PreviousMode,
        SearchFrames,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtRaiseException(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
    FirstChance: BOOLEAN,
) -> NTSTATUS {
    #[cfg(target_arch = "x86")]
    let ExceptionFrame: PKEXCEPTION_FRAME = core::ptr::null_mut();
    #[cfg(not(target_arch = "x86"))]
    let mut LocalExceptionFrame: KEXCEPTION_FRAME = core::mem::zeroed();
    #[cfg(not(target_arch = "x86"))]
    let ExceptionFrame: PKEXCEPTION_FRAME = &mut LocalExceptionFrame;

    let Thread = PsGetCurrentThread() as PKTHREAD;
    let TrapFrame = (*Thread).TrapFrame;
    (*Thread).TrapFrame = KiGetLinkedTrapFrame(TrapFrame);

    #[cfg(target_arch = "x86")]
    {
        (*KeGetPcr()).__bindgen_anon_1.NtTib.ExceptionList = (*TrapFrame).ExceptionList;
    }

    let Status = KiRaiseException(
        ExceptionRecord,
        Context,
        ExceptionFrame,
        TrapFrame,
        FirstChance,
    );
    if Status < 0 {
        return Status;
    }

    KiExceptionExit(TrapFrame, ExceptionFrame);
}

#[no_mangle]
pub unsafe extern "system" fn NtContinue(
    Context: PCONTEXT,
    TestAlert: BOOLEAN,
) -> NTSTATUS {
    #[cfg(target_arch = "x86")]
    let ExceptionFrame: PKEXCEPTION_FRAME = core::ptr::null_mut();
    #[cfg(not(target_arch = "x86"))]
    let mut LocalExceptionFrame: KEXCEPTION_FRAME = core::mem::zeroed();
    #[cfg(not(target_arch = "x86"))]
    let ExceptionFrame: PKEXCEPTION_FRAME = &mut LocalExceptionFrame;

    let Thread = PsGetCurrentThread() as PKTHREAD;
    let TrapFrame = (*Thread).TrapFrame;
    (*Thread).TrapFrame = KiGetLinkedTrapFrame(TrapFrame);

    let Status = KiContinue(Context, ExceptionFrame, TrapFrame);
    if Status < 0 {
        return Status;
    }

    if TestAlert != 0 {
        KeTestAlertThread(*kt_PreviousMode(Thread));
    }

    KiExceptionExit(TrapFrame, ExceptionFrame);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/ke/except.c
//  * PURPOSE:         Platform independent exception handling
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// KiContinuePreviousModeUser(
//     _In_ PCONTEXT Context,
//     _Out_ PKEXCEPTION_FRAME ExceptionFrame,
//     _Out_ PKTRAP_FRAME TrapFrame)
// {
//     CONTEXT LocalContext;
//
//     /* We'll have to make a copy and probe it */
//     ProbeForRead(Context, sizeof(CONTEXT), sizeof(ULONG));
//     RtlCopyMemory(&LocalContext, Context, sizeof(CONTEXT));
//     Context = &LocalContext;
//
//     /* Convert the context into Exception/Trap Frames */
//     KeContextToTrapFrame(&LocalContext,
//                          ExceptionFrame,
//                          TrapFrame,
//                          LocalContext.ContextFlags,
//                          UserMode);
// }
//
// NTSTATUS
// NTAPI
// KiContinue(IN PCONTEXT Context,
//            IN PKEXCEPTION_FRAME ExceptionFrame,
//            IN PKTRAP_FRAME TrapFrame)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     KIRQL OldIrql = APC_LEVEL;
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//
//     /* Raise to APC_LEVEL, only if needed */
//     if (KeGetCurrentIrql() < APC_LEVEL) KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Set up SEH to validate the context */
//     _SEH2_TRY
//     {
//         /* Check the previous mode */
//         if (PreviousMode != KernelMode)
//         {
//             /* Validate from user-mode */
//             KiContinuePreviousModeUser(Context,
//                                        ExceptionFrame,
//                                        TrapFrame);
//         }
//         else
//         {
//             /* Convert the context into Exception/Trap Frames */
//             KeContextToTrapFrame(Context,
//                                  ExceptionFrame,
//                                  TrapFrame,
//                                  Context->ContextFlags,
//                                  KernelMode);
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Save the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Lower the IRQL if needed */
//     if (OldIrql < APC_LEVEL) KeLowerIrql(OldIrql);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// KiRaiseException(
//     _In_ PEXCEPTION_RECORD ExceptionRecord,
//     _In_ PCONTEXT Context,
//     _Out_ PKEXCEPTION_FRAME ExceptionFrame,
//     _Out_ PKTRAP_FRAME TrapFrame,
//     _In_ BOOLEAN SearchFrames)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     CONTEXT LocalContext;
//     EXCEPTION_RECORD LocalExceptionRecord;
//     ULONG ParameterCount, Size;
//
//     /* Check if we need to probe */
//     if (PreviousMode != KernelMode)
//     {
//         /* Set up SEH */
//         _SEH2_TRY
//         {
//             /* Probe the context */
//             ProbeForRead(Context, sizeof(CONTEXT), sizeof(ULONG));
//
//             /* Probe the Exception Record */
//             ProbeForRead(ExceptionRecord,
//                          FIELD_OFFSET(EXCEPTION_RECORD, NumberParameters) +
//                          sizeof(ULONG),
//                          sizeof(ULONG));
//
//             /* Validate the maximum parameters */
//             if ((ParameterCount = ExceptionRecord->NumberParameters) >
//                 EXCEPTION_MAXIMUM_PARAMETERS)
//             {
//                 /* Too large */
//                 _SEH2_YIELD(return STATUS_INVALID_PARAMETER);
//             }
//
//             /* Probe the entire parameters now*/
//             Size = (sizeof(EXCEPTION_RECORD) -
//                     ((EXCEPTION_MAXIMUM_PARAMETERS - ParameterCount) * sizeof(ULONG)));
//             ProbeForRead(ExceptionRecord, Size, sizeof(ULONG));
//
//             /* Now make copies in the stack */
//             RtlCopyMemory(&LocalContext, Context, sizeof(CONTEXT));
//             RtlCopyMemory(&LocalExceptionRecord, ExceptionRecord, Size);
//             Context = &LocalContext;
//             ExceptionRecord = &LocalExceptionRecord;
//
//             /* Update the parameter count */
//             ExceptionRecord->NumberParameters = ParameterCount;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Don't fail silently */
//             DPRINT1("KiRaiseException: Failed to Probe\n");
//
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Convert the context record */
//     KeContextToTrapFrame(Context,
//                          ExceptionFrame,
//                          TrapFrame,
//                          Context->ContextFlags,
//                          PreviousMode);
//
//     /* Dispatch the exception */
//     ExceptionRecord->ExceptionCode &= ~KI_EXCEPTION_INTERNAL;
//     KiDispatchException(ExceptionRecord,
//                         ExceptionFrame,
//                         TrapFrame,
//                         PreviousMode,
//                         SearchFrames);
//
//     /* We are done */
//     return STATUS_SUCCESS;
// }
//
// /* SYSTEM CALLS ***************************************************************/
//
// NTSTATUS
// NTAPI
// NtRaiseException(
//     _In_ PEXCEPTION_RECORD ExceptionRecord,
//     _In_ PCONTEXT Context,
//     _In_ BOOLEAN FirstChance)
// {
//     NTSTATUS Status;
//     PKTHREAD Thread;
//     PKTRAP_FRAME TrapFrame;
// #ifdef _M_IX86
//     PKEXCEPTION_FRAME ExceptionFrame = NULL;
// #else
//     KEXCEPTION_FRAME LocalExceptionFrame;
//     PKEXCEPTION_FRAME ExceptionFrame = &LocalExceptionFrame;
// #endif
//
//     /* Get trap frame and link previous one */
//     Thread = KeGetCurrentThread();
//     TrapFrame = Thread->TrapFrame;
//     Thread->TrapFrame = KiGetLinkedTrapFrame(TrapFrame);
//
//     /* Set exception list */
// #ifdef _M_IX86
//     KeGetPcr()->NtTib.ExceptionList = TrapFrame->ExceptionList;
// #endif
//
//     /* Raise the exception */
//     Status = KiRaiseException(ExceptionRecord,
//                               Context,
//                               ExceptionFrame,
//                               TrapFrame,
//                               FirstChance);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("KiRaiseException failed. Status = 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* It was handled, so exit restoring all state */
//     KiExceptionExit(TrapFrame, ExceptionFrame);
// }
//
// NTSTATUS
// NTAPI
// NtContinue(
//     _In_ PCONTEXT Context,
//     _In_ BOOLEAN TestAlert)
// {
//     PKTHREAD Thread;
//     NTSTATUS Status;
//     PKTRAP_FRAME TrapFrame;
// #ifdef _M_IX86
//     PKEXCEPTION_FRAME ExceptionFrame = NULL;
// #else
//     KEXCEPTION_FRAME LocalExceptionFrame;
//     PKEXCEPTION_FRAME ExceptionFrame = &LocalExceptionFrame;
// #endif
//
//     /* Get trap frame and link previous one*/
//     Thread = KeGetCurrentThread();
//     TrapFrame = Thread->TrapFrame;
//     Thread->TrapFrame = KiGetLinkedTrapFrame(TrapFrame);
//
//     /* Continue from this point on */
//     Status = KiContinue(Context, ExceptionFrame, TrapFrame);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("KiContinue failed. Status = 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Check if alert was requested */
//     if (TestAlert)
//     {
//         KeTestAlertThread(Thread->PreviousMode);
//     }
//
//     /* Exit to new context */
//     KiExceptionExit(TrapFrame, ExceptionFrame);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
