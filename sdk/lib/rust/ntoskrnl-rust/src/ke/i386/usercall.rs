use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::ALIGN_DOWN_BY;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;

use ntkrnl_sys::ddk::{
    _EPROCESS, _KTRAP_FRAME, _CONTEXT, _EXCEPTION_RECORD, _TEB, _KPCR, _KTSS,
    _KCALLOUT_FRAME, _FX_SAVE_AREA, _KTHREAD,
    PEPROCESS, PKTRAP_FRAME, PCONTEXT, PEXCEPTION_RECORD, PTEB, PKPCR, PKTSS,
    PKCALLOUT_FRAME, PFX_SAVE_AREA, PKTHREAD, PEXCEPTION_REGISTRATION_RECORD,
    PKEXCEPTION_FRAME, PKNORMAL_ROUTINE, PGDI_BATCHFLUSH_ROUTINE, PPEB,
    KPROCESSOR_MODE, HANDLE,
    KeTrapFrameToContext, KiCallUserMode, KiCallbackReturn,
    KiGetUserModeStackAddress, MmGrowKernelStack, PsGetCurrentProcessId,
};

use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut KeGdiFlushUserBatch: PGDI_BATCHFLUSH_ROUTINE;
    static mut KeUserApcDispatcher: PVOID;
    static mut KeUserCallbackDispatcher: PVOID;
    fn Ke386SanitizeSeg(Cs: ULONG, Mode: KPROCESSOR_MODE) -> ULONG;
    fn Ke386SanitizeFlags(Flags: ULONG, Mode: KPROCESSOR_MODE) -> ULONG;
    fn KiUserTrap(TrapFrame: PKTRAP_FRAME) -> BOOLEAN;
    fn KiDispatchException( ExceptionRecord: PEXCEPTION_RECORD, ExceptionFrame: PKEXCEPTION_FRAME, TrapFrame: PKTRAP_FRAME, PreviousMode: KPROCESSOR_MODE, FirstChance: BOOLEAN, );
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiServiceExit(TrapFrame: PKTRAP_FRAME, Reserved: ULONG);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiServiceExit(TrapFrame: PKTRAP_FRAME, Reserved: ULONG);
}

const EFLAGS_V86_MASK: ULONG = 0x00020000;
const EFLAGS_IOPL: ULONG = 0x00003000;
// ULONG, not USHORT: these feed Ke386SanitizeSeg, whose real prototype takes a
// ULONG selector and returns one (ke/i386/exp.rs had it right).
const KGDT_R3_CODE: ULONG = 0x18;
const KGDT_R3_DATA: ULONG = 0x20;
const KGDT_R3_TEB: ULONG = 0x38;
const CONTEXT_FULL: ULONG = 0x00010007;
const CONTEXT_DEBUG_REGISTERS: ULONG = 0x00010010;
const CONTEXT_ALIGNED_SIZE: usize = (core::mem::size_of::<_CONTEXT>() + 15) & !15;
const KERNEL_STACK_SIZE: ULONG = 12288;
const OriginalApcEnvironment: u8 = 0;

#[no_mangle]
pub unsafe extern "system" fn KiInitializeUserApc(
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
    NormalRoutine: PKNORMAL_ROUTINE,
    NormalContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let NormalContext = opaque_ptr(NormalContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    let mut Context: _CONTEXT = core::mem::zeroed();
    let mut SehExceptRecord: _EXCEPTION_RECORD = core::mem::zeroed();
    let Stack: ULONG_PTR;
    let AlignedEsp: ULONG_PTR;
    let ContextLength: ULONG;

    if (*TrapFrame).EFlags & EFLAGS_V86_MASK != 0 {
        return;
    }

    Context.ContextFlags = CONTEXT_FULL | CONTEXT_DEBUG_REGISTERS;
    KeTrapFrameToContext(TrapFrame, ExceptionFrame, &mut Context as PCONTEXT);

    let seh_status = 'seh: {
        if KiUserTrap(TrapFrame) == 0 {
            break 'seh STATUS_UNSUCCESSFUL;
        }

        AlignedEsp = (Context.Esp & !3) as ULONG_PTR;
        ContextLength = CONTEXT_ALIGNED_SIZE as ULONG + (4 * core::mem::size_of::<ULONG_PTR>() as ULONG);
        Stack = ((AlignedEsp - 8) & !3) - ContextLength as ULONG_PTR;

        if let Err(e) = probe_user_buffer_write(
            Stack as *const core::ffi::c_void,
            (AlignedEsp - Stack) as usize,
            1,
            1i8,
        ) {
            break 'seh e;
        }

        if let Err(e) = safe_copy_to_user(
            (Stack + (4 * core::mem::size_of::<ULONG_PTR>()) as ULONG_PTR) as *mut core::ffi::c_void,
            &Context as *const _CONTEXT as *const core::ffi::c_void,
            core::mem::size_of::<_CONTEXT>(),
            1,
            1i8,
        ) {
            break 'seh e;
        }

        (*TrapFrame).Eip = KeUserApcDispatcher as ULONG;
        (*TrapFrame).HardwareEsp = Stack as ULONG;

        (*TrapFrame).SegCs = Ke386SanitizeSeg(KGDT_R3_CODE, 1i8);
        (*TrapFrame).HardwareSegSs = Ke386SanitizeSeg(KGDT_R3_DATA, 1i8);
        (*TrapFrame).SegDs = Ke386SanitizeSeg(KGDT_R3_DATA, 1i8);
        (*TrapFrame).SegEs = Ke386SanitizeSeg(KGDT_R3_DATA, 1i8);
        (*TrapFrame).SegFs = Ke386SanitizeSeg(KGDT_R3_TEB, 1i8);
        (*TrapFrame).SegGs = 0;
        (*TrapFrame).ErrCode = 0;

        (*TrapFrame).EFlags = Ke386SanitizeFlags(Context.EFlags, 1i8);

        if (*(KeGetCurrentThread() as *mut _KTHREAD)).Iopl != 0 {
            (*TrapFrame).EFlags |= EFLAGS_IOPL;
        }

        if let Err(e) = write_user(
            (Stack + 0 * core::mem::size_of::<ULONG_PTR>()) as *mut ULONG_PTR,
            core::mem::transmute::<PKNORMAL_ROUTINE, ULONG_PTR>(NormalRoutine),
            1i8,
        ) {
            break 'seh e;
        }
        if let Err(e) = write_user(
            (Stack + 1 * core::mem::size_of::<ULONG_PTR>()) as *mut ULONG_PTR,
            NormalContext as ULONG_PTR,
            1i8,
        ) {
            break 'seh e;
        }
        if let Err(e) = write_user(
            (Stack + 2 * core::mem::size_of::<ULONG_PTR>()) as *mut ULONG_PTR,
            SystemArgument1 as ULONG_PTR,
            1i8,
        ) {
            break 'seh e;
        }
        if let Err(e) = write_user(
            (Stack + 3 * core::mem::size_of::<ULONG_PTR>()) as *mut ULONG_PTR,
            SystemArgument2 as ULONG_PTR,
            1i8,
        ) {
            break 'seh e;
        }

        STATUS_SUCCESS
    };

    if seh_status < 0 {
        SehExceptRecord.ExceptionAddress = (*TrapFrame).Eip as PVOID;
        KiDispatchException(
            &mut SehExceptRecord as PEXCEPTION_RECORD,
            ExceptionFrame,
            TrapFrame,
            1i8,
            1u8,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeUserModeCallback(
    RoutineIndex: ULONG,
    Argument: PVOID,
    ArgumentLength: ULONG,
    Result: *mut PVOID,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let Argument = opaque_ptr(Argument);
    let Result = opaque_ptr(Result);
    let ResultLength = opaque_ptr(ResultLength);
    let NewStack: ULONG_PTR;
    let mut OldStack: ULONG_PTR;
    let UserEsp: *mut ULONG;
    let CallbackStatus: NTSTATUS;
    let ExceptionList: PEXCEPTION_REGISTRATION_RECORD;
    let Teb: PTEB;
    let mut GdiBatchCount: ULONG = 0;

    {
        let Peb = (*(PsGetCurrentProcess() as *mut _EPROCESS)).Peb;
        if Peb.is_null() || (*Peb).KernelCallbackTable.is_null() {
            return STATUS_CALLBACK_POP_STACK;
        }
    }

    UserEsp = KiGetUserModeStackAddress();
    OldStack = *UserEsp as ULONG_PTR;

    // NB: the 'seh block must distinguish a probe/copy SEH failure (early return,
    // like C's _SEH2_YIELD) from the CALLBACK's own return status — a callback is
    // allowed to return a failure NTSTATUS, and that must still flow through the
    // `*UserEsp = OldStack` restore below. Conflating the two skipped the user-ESP
    // restore whenever a callback failed, returning the thread to user mode with
    // ESP inside the spent dispatch frame (winlogon death on the first failing
    // GETCHARSETINFO callback, field manual §9.39). Err(e) = SEH path, Ok(s) =
    // callback status.
    let seh_status: Result<NTSTATUS, NTSTATUS> = 'seh: {
        NewStack = (OldStack - ArgumentLength as ULONG_PTR) & !3;

        if let Err(e) = probe_user_buffer_write(
            (NewStack - 6 * core::mem::size_of::<ULONG_PTR>()) as *const core::ffi::c_void,
            (ArgumentLength as usize) + (6 * core::mem::size_of::<ULONG_PTR>()),
            core::mem::size_of::<i8>(),
            1i8,
        ) {
            break 'seh Err(e);
        }

        if let Err(e) = safe_copy_to_user(
            NewStack as *mut core::ffi::c_void,
            Argument as *const core::ffi::c_void,
            ArgumentLength as usize,
            1,
            1i8,
        ) {
            break 'seh Err(e);
        }

        let stack_base = NewStack - 24;
        if let Err(e) = write_user((stack_base) as *mut ULONG, 0u32, 1i8) {
            break 'seh Err(e);
        }
        if let Err(e) = write_user((stack_base + 4) as *mut ULONG, RoutineIndex, 1i8) {
            break 'seh Err(e);
        }
        /* Argument pointer: stack_base + 24 == NewStack (original, un-decremented).
         * C code: NewStack -= 24; then *(NewStack+8) = (NewStack+24).
         * Here stack_base = NewStack-24, so the argument area is at stack_base+24 = NewStack. */
        if let Err(e) = write_user((stack_base + 8) as *mut ULONG, NewStack as ULONG, 1i8) {
            break 'seh Err(e);
        }
        if let Err(e) = write_user((stack_base + 12) as *mut ULONG, ArgumentLength, 1i8) {
            break 'seh Err(e);
        }

        Teb = (*(KeGetCurrentThread() as *mut _KTHREAD)).Teb;
        ExceptionList = (*Teb).NtTib.ExceptionList;

        *UserEsp = stack_base as ULONG;
        let cb_status = KiCallUserMode(Result, ResultLength);
        if cb_status != STATUS_CALLBACK_POP_STACK {
            (*Teb).NtTib.ExceptionList = ExceptionList;
        } else {
            OldStack = *UserEsp as ULONG_PTR;
        }

        GdiBatchCount = (*Teb).GdiBatchCount;

        Ok(cb_status)
    };

    let CallbackStatus = match seh_status {
        // Probe/copy raised: bail like C's _SEH2_YIELD(return GetExceptionCode()) —
        // user mode was never entered, so there is nothing to restore.
        Err(e) => return e,
        // Callback status (may legitimately be a failure NTSTATUS) — fall through
        // to the user-ESP restore.
        Ok(s) => s,
    };

    if GdiBatchCount != 0 {
        *UserEsp = (*UserEsp).wrapping_sub(256);
        if let Some(flush) = KeGdiFlushUserBatch {
            flush();
        }
    }

    *UserEsp = OldStack as ULONG;
    CallbackStatus
}

#[no_mangle]
pub unsafe extern "fastcall" fn KiUserModeCallout(CalloutFrame: PKCALLOUT_FRAME) -> NTSTATUS {
    let CurrentThread: PKTHREAD;
    let TrapFrame: PKTRAP_FRAME;
    let CallbackTrapFrame: PKTRAP_FRAME;
    let FxSaveArea: PFX_SAVE_AREA;
    let OldFxSaveArea: PFX_SAVE_AREA;
    let Pcr: PKPCR;
    let Tss: PKTSS;
    let InitialStack: ULONG_PTR;
    let mut Status: NTSTATUS = 0;

    CurrentThread = KeGetCurrentThread() as PKTHREAD;

    #[cfg(debug_assertions)]
    {
        if KeGetCurrentIrql() != 0 {
            KeBugCheckEx(ntkrnl_sys::bugcodes::IRQL_GT_ZERO_AT_SYSTEM_SERVICE, 0, KeGetCurrentIrql() as ULONG_PTR, 0, 0);
        }

        if (*(CurrentThread as *mut _KTHREAD)).ApcStateIndex != OriginalApcEnvironment
            || (*(CurrentThread as *mut _KTHREAD)).__bindgen_anon_4.CombinedApcDisable > 0
        {
            KeBugCheckEx(
                ntkrnl_sys::bugcodes::APC_INDEX_MISMATCH,
                0,
                (*(CurrentThread as *mut _KTHREAD)).ApcStateIndex as ULONG_PTR,
                (*(CurrentThread as *mut _KTHREAD)).__bindgen_anon_4.CombinedApcDisable as ULONG_PTR,
                0,
            );
        }
    }

    InitialStack = ALIGN_DOWN_BY(CalloutFrame as ULONG_PTR, 16);

    if (InitialStack - KERNEL_STACK_SIZE as ULONG_PTR)
        < (*(CurrentThread as *mut _KTHREAD)).StackLimit as ULONG_PTR
    {
        Status = MmGrowKernelStack(InitialStack as PVOID);

        if Status < 0 {
            return Status;
        }
    }

    (*CalloutFrame).CallbackStack = (*(CurrentThread as *mut _KTHREAD)).CallbackStack as ULONG;
    (*CalloutFrame).InitialStack = (*(CurrentThread as *mut _KTHREAD)).InitialStack as ULONG;

    TrapFrame = (*(CurrentThread as *mut _KTHREAD)).TrapFrame;
    (*CalloutFrame).TrapFrame = TrapFrame as ULONG;

    (*(CurrentThread as *mut _KTHREAD)).CallbackStack = CalloutFrame as PVOID;

    OldFxSaveArea = ((*CalloutFrame).InitialStack as ULONG_PTR - core::mem::size_of::<_FX_SAVE_AREA>()) as PFX_SAVE_AREA;
    FxSaveArea = (InitialStack - core::mem::size_of::<_FX_SAVE_AREA>()) as PFX_SAVE_AREA;

    core::arch::asm!("cli");

    (*FxSaveArea).U.FnArea.ControlWord = (*OldFxSaveArea).U.FnArea.ControlWord;
    (*FxSaveArea).U.FnArea.StatusWord = (*OldFxSaveArea).U.FnArea.StatusWord;
    (*FxSaveArea).U.FnArea.TagWord = (*OldFxSaveArea).U.FnArea.TagWord;
    (*FxSaveArea).U.FnArea.DataSelector = (*OldFxSaveArea).U.FnArea.DataSelector;
    (*FxSaveArea).Cr0NpxState = (*OldFxSaveArea).Cr0NpxState;

    (*(CurrentThread as *mut _KTHREAD)).InitialStack = InitialStack as PVOID;

    CallbackTrapFrame = (FxSaveArea as ULONG_PTR - core::mem::size_of::<_KTRAP_FRAME>()) as PKTRAP_FRAME;

    *CallbackTrapFrame = *TrapFrame;

    Pcr = KeGetPcr() as PKPCR;

    (*CallbackTrapFrame).ExceptionList = (*Pcr).__bindgen_anon_1.NtTib.ExceptionList;

    Tss = (*Pcr).TSS;

    if (*CallbackTrapFrame).EFlags & EFLAGS_V86_MASK != 0 {
        (*Tss).Esp0 = (CallbackTrapFrame as ULONG_PTR + core::mem::size_of::<_KTRAP_FRAME>()) as ULONG;
    } else {
        (*Tss).Esp0 = core::ptr::addr_of_mut!((*CallbackTrapFrame).V86Es) as ULONG;
    }

    (*CallbackTrapFrame).Eip = KeUserCallbackDispatcher as ULONG;

    core::arch::asm!("sti");

    KiServiceExit(CallbackTrapFrame, 0);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtCallbackReturn(
    Result: PVOID,
    ResultLength: ULONG,
    CallbackStatus: NTSTATUS,
) -> NTSTATUS {
    let Result = opaque_ptr(Result);
    let CurrentThread: PKTHREAD;
    let CalloutFrame: PKCALLOUT_FRAME;
    let CallbackTrapFrame: PKTRAP_FRAME;
    let TrapFrame: PKTRAP_FRAME;
    let FxSaveArea: PFX_SAVE_AREA;
    let CbFxSaveArea: PFX_SAVE_AREA;
    let Size: ULONG;
    let Pcr: PKPCR;
    let Tss: PKTSS;

    CurrentThread = KeGetCurrentThread() as PKTHREAD;
    CalloutFrame = (*(CurrentThread as *mut _KTHREAD)).CallbackStack as PKCALLOUT_FRAME;
    if CalloutFrame.is_null() {
        return STATUS_NO_CALLBACK_ACTIVE;
    }

    CallbackTrapFrame = (*(CurrentThread as *mut _KTHREAD)).TrapFrame;

    Pcr = KeGetPcr() as PKPCR;
    (*Pcr).__bindgen_anon_1.NtTib.ExceptionList = (*CallbackTrapFrame).ExceptionList;

    *((*CalloutFrame).Result as *mut PVOID) = Result;
    *((*CalloutFrame).ResultLength as *mut ULONG) = ResultLength;

    core::arch::asm!("cli");

    CbFxSaveArea = ((*(CurrentThread as *mut _KTHREAD)).InitialStack as ULONG
        - core::mem::size_of::<_FX_SAVE_AREA>() as ULONG) as PFX_SAVE_AREA;
    FxSaveArea = ((*CalloutFrame).InitialStack as ULONG_PTR - core::mem::size_of::<_FX_SAVE_AREA>()) as PFX_SAVE_AREA;

    (*FxSaveArea).U.FnArea.ControlWord = (*CbFxSaveArea).U.FnArea.ControlWord;
    (*FxSaveArea).U.FnArea.StatusWord = (*CbFxSaveArea).U.FnArea.StatusWord;
    (*FxSaveArea).U.FnArea.TagWord = (*CbFxSaveArea).U.FnArea.TagWord;
    (*FxSaveArea).U.FnArea.DataSelector = (*CbFxSaveArea).U.FnArea.DataSelector;
    (*FxSaveArea).Cr0NpxState = (*CbFxSaveArea).Cr0NpxState;

    TrapFrame = (*CalloutFrame).TrapFrame as PKTRAP_FRAME;

    if CallbackStatus == STATUS_CALLBACK_POP_STACK {
        if (*CallbackTrapFrame).EFlags & EFLAGS_V86_MASK != 0 {
            Size = core::mem::size_of::<_KTRAP_FRAME>() as ULONG
                - core::mem::offset_of!(_KTRAP_FRAME, SegFs) as ULONG;
        } else {
            Size = core::mem::offset_of!(_KTRAP_FRAME, V86Es) as ULONG
                - core::mem::offset_of!(_KTRAP_FRAME, SegFs) as ULONG;
        }

        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*CallbackTrapFrame).SegFs) as *const u8,
            core::ptr::addr_of_mut!((*TrapFrame).SegFs) as *mut u8,
            Size as usize,
        );
    }

    (*TrapFrame).Dr7 = 0;

    if (*(CurrentThread as *mut _KTHREAD)).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.__bindgen_anon_1.DebugActive & 0xFF != 0 {
        (*TrapFrame).Dr0 = (*CallbackTrapFrame).Dr0;
        (*TrapFrame).Dr1 = (*CallbackTrapFrame).Dr1;
        (*TrapFrame).Dr2 = (*CallbackTrapFrame).Dr2;
        (*TrapFrame).Dr3 = (*CallbackTrapFrame).Dr3;
        (*TrapFrame).Dr6 = (*CallbackTrapFrame).Dr6;
        (*TrapFrame).Dr7 = (*CallbackTrapFrame).Dr7;
    }

    Tss = (*Pcr).TSS;

    if (*TrapFrame).EFlags & EFLAGS_V86_MASK != 0 {
        (*Tss).Esp0 = (TrapFrame as ULONG_PTR + core::mem::size_of::<_KTRAP_FRAME>()) as ULONG;
    } else {
        (*Tss).Esp0 = core::ptr::addr_of_mut!((*TrapFrame).V86Es) as ULONG;
    }

    (*(CurrentThread as *mut _KTHREAD)).InitialStack = (*CalloutFrame).InitialStack as PVOID;

    (*(CurrentThread as *mut _KTHREAD)).TrapFrame = TrapFrame;
    (*(CurrentThread as *mut _KTHREAD)).CallbackStack = (*CalloutFrame).CallbackStack as PVOID;

    core::arch::asm!("sti");

    KiCallbackReturn(core::ptr::addr_of_mut!((*CalloutFrame).Edi) as PVOID, CallbackStatus);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/i386/usercall.c
//  * PURPOSE:         User-mode Callout Mechanisms (APC and Win32K Callbacks)
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern PGDI_BATCHFLUSH_ROUTINE KeGdiFlushUserBatch;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*++
//  * @name KiInitializeUserApc
//  *
//  *     Prepares the Context for a User-Mode APC called through NTDLL.DLL
//  *
//  * @param Reserved
//  *        Pointer to the Exception Frame on non-i386 builds.
//  *
//  * @param TrapFrame
//  *        Pointer to the Trap Frame.
//  *
//  * @param NormalRoutine
//  *        Pointer to the NormalRoutine to call.
//  *
//  * @param NormalContext
//  *        Pointer to the context to send to the Normal Routine.
//  *
//  * @param SystemArgument[1-2]
//  *        Pointer to a set of two parameters that contain untyped data.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// KiInitializeUserApc(IN PKEXCEPTION_FRAME ExceptionFrame,
//                     IN PKTRAP_FRAME TrapFrame,
//                     IN PKNORMAL_ROUTINE NormalRoutine,
//                     IN PVOID NormalContext,
//                     IN PVOID SystemArgument1,
//                     IN PVOID SystemArgument2)
// {
//     CONTEXT Context = { 0 };
//     ULONG_PTR Stack, AlignedEsp;
//     ULONG ContextLength;
//     EXCEPTION_RECORD SehExceptRecord;
//
//     /* Don't deliver APCs in V86 mode */
//     if (TrapFrame->EFlags & EFLAGS_V86_MASK) return;
//
//     /* Save the full context */
//     Context.ContextFlags = CONTEXT_FULL | CONTEXT_DEBUG_REGISTERS;
//     KeTrapFrameToContext(TrapFrame, ExceptionFrame, &Context);
//
//     /* Protect with SEH */
//     _SEH2_TRY
//     {
//         /* Sanity check */
//         ASSERT(KiUserTrap(TrapFrame));
//
//         /* Get the aligned size */
//         AlignedEsp = Context.Esp & ~3;
//         ContextLength = CONTEXT_ALIGNED_SIZE + (4 * sizeof(ULONG_PTR));
//         Stack = ((AlignedEsp - 8) & ~3) - ContextLength;
//
//         /* Probe the stack */
//         ProbeForWrite((PVOID)Stack, AlignedEsp - Stack, 1);
//         ASSERT(!(Stack & 3));
//
//         /* Copy data into it */
//         RtlCopyMemory((PVOID)(Stack + (4 * sizeof(ULONG_PTR))),
//                       &Context,
//                       sizeof(CONTEXT));
//
//         /* Run at APC dispatcher */
//         TrapFrame->Eip = (ULONG)KeUserApcDispatcher;
//         TrapFrame->HardwareEsp = Stack;
//
//         /* Setup Ring 3 state */
//         TrapFrame->SegCs = Ke386SanitizeSeg(KGDT_R3_CODE, UserMode);
//         TrapFrame->HardwareSegSs = Ke386SanitizeSeg(KGDT_R3_DATA, UserMode);
//         TrapFrame->SegDs = Ke386SanitizeSeg(KGDT_R3_DATA, UserMode);
//         TrapFrame->SegEs = Ke386SanitizeSeg(KGDT_R3_DATA, UserMode);
//         TrapFrame->SegFs = Ke386SanitizeSeg(KGDT_R3_TEB, UserMode);
//         TrapFrame->SegGs = 0;
//         TrapFrame->ErrCode = 0;
//
//         /* Sanitize EFLAGS */
//         TrapFrame->EFlags = Ke386SanitizeFlags(Context.EFlags, UserMode);
//
//         /* Check if thread has IOPL and force it enabled if so */
//         if (KeGetCurrentThread()->Iopl) TrapFrame->EFlags |= EFLAGS_IOPL;
//
//         /* Setup the stack */
//         *(PULONG_PTR)(Stack + 0 * sizeof(ULONG_PTR)) = (ULONG_PTR)NormalRoutine;
//         *(PULONG_PTR)(Stack + 1 * sizeof(ULONG_PTR)) = (ULONG_PTR)NormalContext;
//         *(PULONG_PTR)(Stack + 2 * sizeof(ULONG_PTR)) = (ULONG_PTR)SystemArgument1;
//         *(PULONG_PTR)(Stack + 3 * sizeof(ULONG_PTR)) = (ULONG_PTR)SystemArgument2;
//     }
//     _SEH2_EXCEPT((RtlCopyMemory(&SehExceptRecord, _SEH2_GetExceptionInformation()->ExceptionRecord, sizeof(EXCEPTION_RECORD)), EXCEPTION_EXECUTE_HANDLER))
//     {
//         /* Dispatch the exception */
//         SehExceptRecord.ExceptionAddress = (PVOID)TrapFrame->Eip;
//         KiDispatchException(&SehExceptRecord,
//                             ExceptionFrame,
//                             TrapFrame,
//                             UserMode,
//                             TRUE);
//     }
//     _SEH2_END;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// KeUserModeCallback(IN ULONG RoutineIndex,
//                    IN PVOID Argument,
//                    IN ULONG ArgumentLength,
//                    OUT PVOID *Result,
//                    OUT PULONG ResultLength)
// {
//     ULONG_PTR NewStack, OldStack;
//     PULONG UserEsp;
//     NTSTATUS CallbackStatus;
//     PEXCEPTION_REGISTRATION_RECORD ExceptionList;
//     PTEB Teb;
//     ULONG GdiBatchCount = 0;
//     ASSERT(KeGetCurrentThread()->ApcState.KernelApcInProgress == FALSE);
//     ASSERT(KeGetPreviousMode() == UserMode);
//
// #ifdef _USE_DUMMY_PSEH
//     /* Guard: if PEB->KernelCallbackTable is not set up (user32 DllMain hasn't
//      * run yet), user-mode dispatch will crash with an access violation.
//      * Return STATUS_CALLBACK_POP_STACK so the caller knows the callback failed. */
//     {
//         PPEB Peb = PsGetCurrentProcess()->Peb;
//         if (!Peb || !Peb->KernelCallbackTable)
//         {
//             DbgPrint("[CALLBACK] Skipping RoutineIndex=%lu PID=%lu - KernelCallbackTable NULL\n",
//                      RoutineIndex, (ULONG)(ULONG_PTR)PsGetCurrentProcessId());
//             return STATUS_CALLBACK_POP_STACK;
//         }
//     }
// #endif
//
//     /* Get the current user-mode stack */
//     UserEsp = KiGetUserModeStackAddress();
//     OldStack = *UserEsp;
//
//     /* Enter a SEH Block */
//     _SEH2_TRY
//     {
//         /* Calculate and align the stack size */
//         NewStack = (OldStack - ArgumentLength) & ~3;
//
//         /* Make sure it's writable */
//         ProbeForWrite((PVOID)(NewStack - 6 * sizeof(ULONG_PTR)),
//                       ArgumentLength + 6 * sizeof(ULONG_PTR),
//                       sizeof(CHAR));
//
//         /* Copy the buffer into the stack */
//         RtlCopyMemory((PVOID)NewStack, Argument, ArgumentLength);
//
//         /* Write the arguments */
//         NewStack -= 24;
//         *(PULONG)NewStack = 0;
//         *(PULONG)(NewStack + 4) = RoutineIndex;
//         *(PULONG)(NewStack + 8) = (NewStack + 24);
//         *(PULONG)(NewStack + 12) = ArgumentLength;
//
//         /* Save the exception list */
//         Teb = KeGetCurrentThread()->Teb;
//         ExceptionList = Teb->NtTib.ExceptionList;
//
//         /* Jump to user mode */
//         *UserEsp = NewStack;
//         CallbackStatus = KiCallUserMode(Result, ResultLength);
//         if (CallbackStatus != STATUS_CALLBACK_POP_STACK)
//         {
//             /* Only restore the exception list if we didn't crash in ring 3 */
//             Teb->NtTib.ExceptionList = ExceptionList;
//         }
//         else
//         {
//             /* Otherwise, pop the stack */
//             OldStack = *UserEsp;
//         }
//
//         /* Read the GDI Batch count */
//         GdiBatchCount = Teb->GdiBatchCount;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get the SEH exception */
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Check if we have GDI Batch operations */
//     if (GdiBatchCount)
//     {
//           *UserEsp -= 256;
//           KeGdiFlushUserBatch();
//     }
//
//     /* Restore stack and return */
//     *UserEsp = OldStack;
//     return CallbackStatus;
// }
//
//
// /*
//  * Stack layout for KiUserModeCallout:
//  * ----------------------------------
//  * KCALLOUT_FRAME.ResultLength    <= 2nd Parameter to KiCallUserMode
//  * KCALLOUT_FRAME.Result          <= 1st Parameter to KiCallUserMode
//  * KCALLOUT_FRAME.ReturnAddress   <= Return address of KiCallUserMode
//  * KCALLOUT_FRAME.Ebp             \
//  * KCALLOUT_FRAME.Ebx              | = non-volatile registers, pushed
//  * KCALLOUT_FRAME.Esi              |   by KiCallUserMode
//  * KCALLOUT_FRAME.Edi             /
//  * KCALLOUT_FRAME.CallbackStack
//  * KCALLOUT_FRAME.TrapFrame
//  * KCALLOUT_FRAME.InitialStack    <= CalloutFrame points here
//  * ----------------------------------
//  * ~~ optional alignment ~~
//  * ----------------------------------
//  * FX_SAVE_AREA
//  * ----------------------------------
//  * KTRAP_FRAME
//  * ----------------------------------
//  * ~~ begin of stack frame for KiUserModeCallout ~~
//  *
//  */
//
// NTSTATUS
// FASTCALL
// KiUserModeCallout(PKCALLOUT_FRAME CalloutFrame)
// {
//     PKTHREAD CurrentThread;
//     PKTRAP_FRAME TrapFrame, CallbackTrapFrame;
//     PFX_SAVE_AREA FxSaveArea, OldFxSaveArea;
//     PKPCR Pcr;
//     PKTSS Tss;
//     ULONG_PTR InitialStack;
//     NTSTATUS Status;
//
//     /* Get the current thread */
//     CurrentThread = KeGetCurrentThread() as PKTHREAD;
//
// #if DBG
//     /* Check if we are at pasive level */
//     if (KeGetCurrentIrql() != PASSIVE_LEVEL)
//     {
//         /* We're not, bugcheck */
//         KeBugCheckEx(IRQL_GT_ZERO_AT_SYSTEM_SERVICE,
//                      0,
//                      KeGetCurrentIrql(),
//                      0,
//                      0);
//     }
//
//     /* Check if we are attached or APCs are disabled */
//     if ((CurrentThread->ApcStateIndex != OriginalApcEnvironment) ||
//         (CurrentThread->CombinedApcDisable > 0))
//     {
//         KeBugCheckEx(APC_INDEX_MISMATCH,
//                      0,
//                      CurrentThread->ApcStateIndex,
//                      CurrentThread->CombinedApcDisable,
//                      0);
//     }
// #endif
//
//     /* Align stack on a 16-byte boundary */
//     InitialStack = ALIGN_DOWN_BY(CalloutFrame, 16);
//
//     /* Check if we have enough space on the stack */
//     if ((InitialStack - KERNEL_STACK_SIZE) < CurrentThread->StackLimit)
//     {
//         /* We don't, we'll have to grow our stack */
//         Status = MmGrowKernelStack((PVOID)InitialStack);
//
//         /* Quit if we failed */
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_STACK_OVERFLOW)
//             {
//                 DPRINT1("Thread wants too much stack\n");
//             }
//
//             return Status;
//         }
//     }
//
//     /* Save the current callback stack and initial stack */
//     CalloutFrame->CallbackStack = (ULONG_PTR)CurrentThread->CallbackStack;
//     CalloutFrame->InitialStack = (ULONG_PTR)CurrentThread->InitialStack;
//
//     /* Get and save the trap frame */
//     TrapFrame = CurrentThread->TrapFrame;
//     CalloutFrame->TrapFrame = (ULONG_PTR)TrapFrame;
//
//     /* Set the new callback stack */
//     CurrentThread->CallbackStack = CalloutFrame;
//
//     /* Set destination and origin NPX Areas */
//     OldFxSaveArea = (PVOID)(CalloutFrame->InitialStack - sizeof(FX_SAVE_AREA));
//     FxSaveArea = (PVOID)(InitialStack - sizeof(FX_SAVE_AREA));
//
//     /* Disable interrupts so we can fill the NPX State */
//     _disable();
//
//     /* Now copy the NPX State */
//     FxSaveArea->U.FnArea.ControlWord = OldFxSaveArea->U.FnArea.ControlWord;
//     FxSaveArea->U.FnArea.StatusWord = OldFxSaveArea->U.FnArea.StatusWord;
//     FxSaveArea->U.FnArea.TagWord = OldFxSaveArea->U.FnArea.TagWord;
//     FxSaveArea->U.FnArea.DataSelector = OldFxSaveArea->U.FnArea.DataSelector;
//     FxSaveArea->Cr0NpxState = OldFxSaveArea->Cr0NpxState;
//
//     /* Set the stack address */
//     CurrentThread->InitialStack = (PVOID)InitialStack;
//
//     /* Locate the trap frame on the callback stack */
//     CallbackTrapFrame = (PVOID)((ULONG_PTR)FxSaveArea - sizeof(KTRAP_FRAME));
//
//     /* Copy the trap frame to the new location */
//     *CallbackTrapFrame = *TrapFrame;
//
//     /* Get PCR */
//     Pcr = KeGetPcr() as PKPCR;
//
//     /* Update the exception list */
//     CallbackTrapFrame->ExceptionList = Pcr->NtTib.ExceptionList;
//
//     /* Get TSS */
//     Tss = Pcr->TSS;
//
//     /* Check for V86 mode */
//     if (CallbackTrapFrame->EFlags & EFLAGS_V86_MASK)
//     {
//         /* Set new stack address in TSS (full trap frame) */
//         Tss->Esp0 = (ULONG_PTR)(CallbackTrapFrame + 1);
//     }
//     else
//     {
//         /* Set new stack address in TSS (non-V86 trap frame) */
//         Tss->Esp0 = (ULONG_PTR)&CallbackTrapFrame->V86Es;
//     }
//
//     /* Set user-mode dispatcher address as EIP */
//     CallbackTrapFrame->Eip = (ULONG_PTR)KeUserCallbackDispatcher;
//
//     /* Bring interrupts back */
//     _enable();
//
//     /* Exit to user-mode */
//     KiServiceExit(CallbackTrapFrame, 0);
// }
//
// /*++
//  * @name NtCallbackReturn
//  *
//  *     The NtCallbackReturn routine returns to kernel mode after a user-mode
//  *     callback was done through KeUserModeCallback. It uses the callback frame
//  *     which was setup in order to return the information, restores the stack,
//  *     and resumes execution where it was left off.
//  *
//  * @param Result
//  *        Pointer to a caller-allocated buffer where the return data
//  *               from the user-mode function is located.
//  *
//  * @param ResultLength
//  *        Size of the Output Buffer described above.
//  *
//  * @param CallbackStatus
//  *        Status code of the callback operation.
//  *
//  * @return Status code of the callback operation.
//  *
//  * @remark This call MUST be paired with KeUserModeCallback.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtCallbackReturn(
//     _In_ PVOID Result,
//     _In_ ULONG ResultLength,
//     _In_ NTSTATUS CallbackStatus)
// {
//     PKTHREAD CurrentThread;
//     PKCALLOUT_FRAME CalloutFrame;
//     PKTRAP_FRAME CallbackTrapFrame, TrapFrame;
//     PFX_SAVE_AREA FxSaveArea, CbFxSaveArea;
//     ULONG Size;
//     PKPCR Pcr;
//     PKTSS Tss;
//
//     /* Get the current thread and make sure we have a callback stack */
//     CurrentThread = KeGetCurrentThread() as PKTHREAD;
//     CalloutFrame = CurrentThread->CallbackStack;
//     if (CalloutFrame == NULL)
//     {
//         return STATUS_NO_CALLBACK_ACTIVE;
//     }
//
//     /* Get the trap frame */
//     CallbackTrapFrame = CurrentThread->TrapFrame;
//
//     /* Restore the exception list */
//     Pcr = KeGetPcr() as PKPCR;
//     Pcr->NtTib.ExceptionList = CallbackTrapFrame->ExceptionList;
//
//     /* Store the results in the callback stack */
//     *((PVOID*)CalloutFrame->Result) = Result;
//     *((ULONG*)CalloutFrame->ResultLength) = ResultLength;
//
//     /* Disable interrupts for NPX save and stack switch */
//     _disable();
//
//     /* Set desination and origin NPX Frames */
//     CbFxSaveArea = (PVOID)((ULONG)CurrentThread->InitialStack - sizeof(FX_SAVE_AREA));
//     FxSaveArea = (PVOID)(CalloutFrame->InitialStack - sizeof(FX_SAVE_AREA));
//
//     /* Now copy back NPX State */
//     FxSaveArea->U.FnArea.ControlWord = CbFxSaveArea->U.FnArea.ControlWord;
//     FxSaveArea->U.FnArea.StatusWord = CbFxSaveArea->U.FnArea.StatusWord;
//     FxSaveArea->U.FnArea.TagWord = CbFxSaveArea->U.FnArea.TagWord;
//     FxSaveArea->U.FnArea.DataSelector = CbFxSaveArea->U.FnArea.DataSelector;
//     FxSaveArea->Cr0NpxState = CbFxSaveArea->Cr0NpxState;
//
//     /* Get the previous trap frame */
//     TrapFrame = (PKTRAP_FRAME)CalloutFrame->TrapFrame;
//
//     /* Check if we failed in user mode */
//     if (CallbackStatus == STATUS_CALLBACK_POP_STACK)
//     {
//         /* Check if we came from v86 mode */
//         if (CallbackTrapFrame->EFlags & EFLAGS_V86_MASK)
//         {
//             Size = sizeof(KTRAP_FRAME) - FIELD_OFFSET(KTRAP_FRAME, SegFs);
//         }
//         else
//         {
//             Size = FIELD_OFFSET(KTRAP_FRAME, V86Es) - FIELD_OFFSET(KTRAP_FRAME, SegFs);
//         }
//
//         /* Copy back part of the trap frame */
//         RtlCopyMemory(&TrapFrame->SegFs, &CallbackTrapFrame->SegFs, Size);
//     }
//
//     /* Clear DR7 */
//     TrapFrame->Dr7 = 0;
//
//     /* Check if debugging was active */
//     if (CurrentThread->Header.DebugActive & 0xFF)
//     {
//         /* Copy debug registers data from it */
//         TrapFrame->Dr0 = CallbackTrapFrame->Dr0;
//         TrapFrame->Dr1 = CallbackTrapFrame->Dr1;
//         TrapFrame->Dr2 = CallbackTrapFrame->Dr2;
//         TrapFrame->Dr3 = CallbackTrapFrame->Dr3;
//         TrapFrame->Dr6 = CallbackTrapFrame->Dr6;
//         TrapFrame->Dr7 = CallbackTrapFrame->Dr7;
//     }
//
//     /* Get TSS */
//     Tss = Pcr->TSS;
//
//     /* Check for V86 mode */
//     if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//     {
//         /* Set new stack address in TSS (full trap frame) */
//         Tss->Esp0 = (ULONG_PTR)(TrapFrame + 1);
//     }
//     else
//     {
//         /* Set new stack address in TSS (non-V86 trap frame) */
//         Tss->Esp0 = (ULONG_PTR)&TrapFrame->V86Es;
//     }
//
//     /* Get the initial stack and restore it */
//     CurrentThread->InitialStack = (PVOID)CalloutFrame->InitialStack;
//
//     /* Restore the trap frame and the previous callback stack */
//     CurrentThread->TrapFrame = TrapFrame;
//     CurrentThread->CallbackStack = (PVOID)CalloutFrame->CallbackStack;
//
//     /* Bring interrupts back */
//     _enable();
//
//     /* Now switch back to the old stack */
//     KiCallbackReturn(&CalloutFrame->Edi, CallbackStatus);
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
