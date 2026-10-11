use crate::ke::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;

use ntkrnl_sys::ps::PsGetCurrentProcess;

use ntkrnl_sys::ddk::{
    _KTRAP_FRAME, PKTRAP_FRAME, PKEXCEPTION_FRAME, PKNORMAL_ROUTINE, PCONTEXT,
    PTEB, PKTHREAD, PKCALLOUT_FRAME, PEXCEPTION_REGISTRATION_RECORD,
    PKIPCR, EXCEPTION_RECORD, _EPROCESS,
    PGDI_BATCHFLUSH_ROUTINE,
    KeTrapFrameToContext, KiDispatchException, MmAccessFault, MmGrowKernelStack,
    KiGetUserModeStackAddress, KeGetPcr,
    KiCallUserMode, KiCallbackReturn,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    static KeUserApcDispatcher: u64;
    static KeUserCallbackDispatcher: u64;
}

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn KiUserCallbackExit(TrapFrame: PKTRAP_FRAME) -> !;
    static mut KeGdiFlushUserBatch: PGDI_BATCHFLUSH_ROUTINE;
}

#[repr(C, align(16))]
struct UAPC_FRAME {
    Context: CONTEXT,
    MachineFrame: MACHINE_FRAME,
}

#[repr(C)]
struct MACHINE_FRAME {
    Rip: u64,
    SegCs: u16,
    Fill1: [u16; 3],
    EFlags: u32,
    Fill2: u32,
    Rsp: u64,
    SegSs: u16,
    Fill3: [u16; 3],
}

#[cfg(target_arch = "x86")]
#[repr(C)]
struct UCALLOUT_FRAME {
    Reserved: ULONG,
    ApiNumber: ULONG,
    Buffer: ULONG_PTR,
    Length: ULONG,
}

#[cfg(not(target_arch = "x86"))]
#[repr(C)]
struct UCALLOUT_FRAME {
    P1Home: u64,
    P2Home: u64,
    P3Home: u64,
    P4Home: u64,
    Buffer: PVOID,
    Length: ULONG,
    ApiNumber: ULONG,
    MachineFrame: MACHINE_FRAME,
}

type PUCALLOUT_FRAME = *mut UCALLOUT_FRAME;

const CONTEXT_FULL: ULONG = 0x0010_000B;
const CONTEXT_DEBUG_REGISTERS: ULONG = 0x0010_0010;
const MODE_MASK: u16 = 1;
const KGDT64_R3_CODE: u16 = 0x0030;
const KGDT64_R3_CMTEB: u16 = 0x0050;
const KGDT64_R3_DATA: u16 = 0x0028;
const RPL_MASK: u16 = 3;
const EFLAGS_USER_SANITIZE: u32 = 0x003F4DD7;
const EFLAGS_INTERRUPT_MASK: u32 = 0x200;
const KERNEL_STACK_SIZE: usize = 0xC000;

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
    let ApcFrame: *mut UAPC_FRAME;
    let Context: PCONTEXT;

    ntkrnl_sys::ASSERT!(((*TrapFrame).SegCs & MODE_MASK) != 0);

    ApcFrame = (((*TrapFrame).Rsp as usize - core::mem::size_of::<UAPC_FRAME>()) & !15) as *mut UAPC_FRAME;
    Context = core::ptr::addr_of_mut!((*ApcFrame).Context);

    let mut exception_record: EXCEPTION_RECORD = core::mem::zeroed();
    // The C wraps everything below in `_SEH2_TRY`, and its `_SEH2_EXCEPT` runs
    // `KiDispatchException`. The original translation kept the shape — a flag
    // plus the `if` — but the `__try` became a BARE BLOCK and the handler body
    // was dropped, so the flag was never assigned, the recovery path was dead
    // code, and under DUMMY_PSEH the `ProbeForWrite` raise escaped as an
    // unhandled kernel exception → BUGCHECK 0x1E (task #67).
    //
    // Fixed per dummy-pseh-verdict.md Tier 1: validate FIRST with the
    // Result-returning probe, then run exactly what the C `__except` ran.
    let mut exception_status: NTSTATUS = 0;

    'try_: {
        if let Err(e) = ntkrnl_sys::probe::probe_user_buffer_write(
            ApcFrame as *const core::ffi::c_void,
            core::mem::size_of::<UAPC_FRAME>(),
            16,
            1i8,
        ) {
            exception_status = e;
            break 'try_;
        }

        // probe_user_buffer_write touch-tests each page as ProbeForWrite does; this
        // pre-fault predates it and is now belt-and-braces. The C's `__try` also covered the
        // WRITES below — KeTrapFrameToContext
        // and the P*Home/MachineFrame stores all land in user memory, and under
        // DUMMY_PSEH a page fault there is just as unhandled as the probe raise
        // was. Pre-fault each page, same pattern (and same reasoning) as
        // KiDispatchExceptionToUser in ke/amd64/except.rs.
        {
            let start = ApcFrame as usize;
            let end = start + core::mem::size_of::<UAPC_FRAME>();
            let mut page = start & !0xFFF;
            while page < end {
                // FaultCode=2 (write, not present), Mode=UserMode(1)
                let status = MmAccessFault(2, page as PVOID, 1i8, core::ptr::null_mut());
                if status < 0 {
                    exception_status = status;
                    break 'try_;
                }
                page += 0x1000;
            }
        }

        (*Context).ContextFlags = CONTEXT_FULL | CONTEXT_DEBUG_REGISTERS;
        KeTrapFrameToContext(TrapFrame, ExceptionFrame, Context);

        (*Context).P1Home = NormalContext as u64;
        (*Context).P2Home = SystemArgument1 as u64;
        (*Context).P3Home = SystemArgument2 as u64;
        (*Context).P4Home = NormalRoutine.map(|f| f as u64).unwrap_or(0);

        (*ApcFrame).MachineFrame.Rip = (*TrapFrame).Rip;
        (*ApcFrame).MachineFrame.Rsp = (*TrapFrame).Rsp;
    }

    if exception_status != 0 {
        // Fail LOUD. The observed trigger is `TrapFrame->Rsp == 0`, which makes
        // ApcFrame `-sizeof(UAPC_FRAME)` — reporting Rsp keeps that root
        // question visible instead of letting the recovery hide it.
        // SegCs is the discriminator for WHY Rsp is bad: KGDT64_R3_CMCODE|RPL
        // (0x23) means a compat-mode WoW64 guest thread reached the 64-bit APC
        // path, i.e. a routing bug; KGDT64_R3_CODE|RPL (0x33) means a genuine
        // 64-bit thread whose user Rsp was never populated (task #71).
        let proc_ = PsGetCurrentProcess() as *mut _EPROCESS;
        crate::dbg_print!(
            b"[KI-APC] user APC frame REJECTED: image=%s Rsp=%p Rip=%p SegCs=0x%lx SegSs=0x%lx ApcFrame=%p size=0x%lx status=0x%lx\n\0",
            (*proc_).ImageFileName.as_ptr(),
            (*TrapFrame).Rsp as PVOID,
            (*TrapFrame).Rip as PVOID,
            (*TrapFrame).SegCs as u32,
            (*TrapFrame).SegSs as u32,
            ApcFrame as PVOID,
            core::mem::size_of::<UAPC_FRAME>() as u32,
            exception_status as u32,
        );
        exception_record.ExceptionCode = exception_status;
        exception_record.ExceptionFlags = 0;
        exception_record.NumberParameters = 0;
        exception_record.ExceptionAddress = (*TrapFrame).Rip as PVOID;
        KiDispatchException(
            &mut exception_record,
            ExceptionFrame,
            TrapFrame,
            1i8,
            1u8,
        );
    }

    (*TrapFrame).Rsp = Context as u64;
    (*TrapFrame).Rip = KeUserApcDispatcher;

    (*TrapFrame).SegCs = KGDT64_R3_CODE | RPL_MASK;
    (*TrapFrame).SegFs = KGDT64_R3_CMTEB | RPL_MASK;
    (*TrapFrame).SegGs = KGDT64_R3_DATA | RPL_MASK;
    (*TrapFrame).SegSs = KGDT64_R3_DATA | RPL_MASK;

    // SET EFlags, do not "sanitize" it. EFLAGS_USER_SANITIZE (0x003F4DD7) PRESERVES
    // bit 8 (TF) and bit 10 (DF) -- verified bit by bit -- so whatever those bits
    // held in the freshly seeded trap frame survived into the new user thread. TF
    // means an immediate single-step #DB on its first user instruction, which is
    // the crash upstream describes. Upstream-Commit: 6bc61687e6ed
    (*TrapFrame).EFlags = EFLAGS_INTERRUPT_MASK;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn KiUserModeCallout(CalloutFrame: PKCALLOUT_FRAME) -> NTSTATUS {
    KiUserModeCalloutImpl(CalloutFrame)
}

unsafe fn KiUserModeCalloutImpl(CalloutFrame: PKCALLOUT_FRAME) -> NTSTATUS {
    let CurrentThread: PKTHREAD;
    let TrapFrame: PKTRAP_FRAME;
    let mut CallbackTrapFrame: _KTRAP_FRAME;
    let Pcr: PKIPCR;
    let InitialStack: ULONG_PTR;
    let Status: NTSTATUS;

    CurrentThread = PsGetCurrentThread();

    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL);
    ntkrnl_sys::ASSERT!((*CurrentThread).ApcStateIndex == 0 && (*CurrentThread).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable == 0);

    InitialStack = (CalloutFrame as ULONG_PTR) & !15;

    if (InitialStack - KERNEL_STACK_SIZE) < (*CurrentThread).StackLimit as ULONG_PTR {
        Status = MmGrowKernelStack(InitialStack as PVOID);
        if Status < 0 {
            return Status;
        }
    }

    (*CalloutFrame).CallbackStack = (*CurrentThread).CallbackStack as u64;
    (*CalloutFrame).InitialStack = (*CurrentThread).InitialStack as u64;

    TrapFrame = (*CurrentThread).TrapFrame;
    (*CalloutFrame).TrapFrame = TrapFrame as u64;

    (*CurrentThread).CallbackStack = CalloutFrame as PVOID;

    core::arch::asm!("cli");

    (*CurrentThread).InitialStack = InitialStack as PVOID;

    CallbackTrapFrame = *TrapFrame;

    Pcr = KeGetPcr() as PKIPCR;

    (*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).Rsp0 = InitialStack as u64;
    (*Pcr).Prcb.RspBase = InitialStack as u64;
    CallbackTrapFrame.Rip = KeUserCallbackDispatcher;

    core::arch::asm!("sti");

    KiUserCallbackExit(&mut CallbackTrapFrame);
}

unsafe fn KiSetupUserCalloutFrame(
    UserCalloutFrame: PUCALLOUT_FRAME,
    TrapFrame: PKTRAP_FRAME,
    ApiNumber: ULONG,
    _Buffer: PVOID,
    BufferLength: ULONG,
) {
    #[cfg(target_arch = "x86")]
    {
        (*UserCalloutFrame).Reserved = 0;
        (*UserCalloutFrame).ApiNumber = ApiNumber;
        (*UserCalloutFrame).Buffer = (UserCalloutFrame as ULONG_PTR)
            .wrapping_add(core::mem::size_of::<UCALLOUT_FRAME>());
        (*UserCalloutFrame).Length = BufferLength;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*UserCalloutFrame).Buffer =
            UserCalloutFrame.add(1) as PVOID;
        (*UserCalloutFrame).Length = BufferLength;
        (*UserCalloutFrame).ApiNumber = ApiNumber;
        (*UserCalloutFrame).MachineFrame.Rip = (*TrapFrame).Rip;
        (*UserCalloutFrame).MachineFrame.Rsp = (*TrapFrame).Rsp;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeUserModeCallback(
    RoutineIndex: ULONG,
    Argument: PVOID,
    ArgumentLength: ULONG,
    Result: *mut PVOID,
    ResultLength: PULONG,
) -> NTSTATUS {
    let Argument = opaque_ptr(Argument);
    let Result = opaque_ptr(Result);
    let ResultLength = opaque_ptr(ResultLength);
    let mut OldStack: ULONG_PTR;
    let UserArguments: *mut u8;
    let CalloutFrame: PUCALLOUT_FRAME;
    let UserStackPointer: *mut u64;
    let mut CallbackStatus: NTSTATUS;
    #[cfg(target_arch = "x86")]
    let ExceptionList: PEXCEPTION_REGISTRATION_RECORD;
    let Teb: PTEB;
    let mut GdiBatchCount: ULONG = 0;

    ntkrnl_sys::ASSERT!((*PsGetCurrentThread()).__bindgen_anon_1.ApcState.KernelApcInProgress == 0);
    ntkrnl_sys::ASSERT!(KeGetPreviousMode() == 1i8);

    // DUMMY_PSEH: Check KernelCallbackTable before dispatching to user mode.
    // If user32.dll hasn't loaded yet, PEB->KernelCallbackTable is NULL and
    // KiUserCallbackDispatcher will ACCESS_VIOLATION trying to call through it.
    // This happens when win32k triggers a callback before user32 DllMain runs.
    {
        let teb = (*PsGetCurrentThread()).Teb;
        if !teb.is_null() {
            let peb = (*teb).ProcessEnvironmentBlock;
            if peb.is_null() || (*peb).KernelCallbackTable.is_null() {
                return 0xC0000001u32 as NTSTATUS; // STATUS_UNSUCCESSFUL
            }
        }
    }

    UserStackPointer = KiGetUserModeStackAddress();
    OldStack = *UserStackPointer as ULONG_PTR;

    let Status = {
        UserArguments = (((OldStack - ArgumentLength as ULONG_PTR) & !15) - 8) as *mut u8;
        CalloutFrame = UserArguments.sub(core::mem::size_of::<UCALLOUT_FRAME>()) as PUCALLOUT_FRAME;

        // C: inside the _SEH2_TRY whose handler returns the exception code. No SEH
        // here: Result probe + fault-tolerant copy onto the user stack. (The C
        // probes sizeof(PUCALLOUT_FRAME) — a pointer — the port the whole frame.)
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer_write(
            CalloutFrame as PVOID,
            core::mem::size_of::<UCALLOUT_FRAME>() + ArgumentLength as usize,
            core::mem::size_of::<PVOID>(),
            1,
        ) {
            return status;
        }

        if let Err(status) = ntkrnl_sys::probe::safe_copy_to_user(
            UserArguments as *mut core::ffi::c_void,
            Argument as *const core::ffi::c_void,
            ArgumentLength as usize,
            1,
            1,
        ) {
            return status;
        }

        KiSetupUserCalloutFrame(
            CalloutFrame,
            (*PsGetCurrentThread()).TrapFrame,
            RoutineIndex,
            UserArguments as PVOID,
            ArgumentLength,
        );

        Teb = (*PsGetCurrentThread()).Teb;
        #[cfg(target_arch = "x86")]
        {
            ExceptionList = (*Teb).NtTib.ExceptionList;
        }

        *UserStackPointer = CalloutFrame as u64;
        CallbackStatus = KiCallUserMode(Result, ResultLength);
        if CallbackStatus != STATUS_CALLBACK_POP_STACK {
            #[cfg(target_arch = "x86")]
            {
                (*Teb).NtTib.ExceptionList = ExceptionList;
            }
        } else {
            OldStack = *UserStackPointer as ULONG_PTR;
        }

        // A WoW64 guest's gdi32 bumps TEB32->GdiBatchCount, not this one. The
        // SYSCALL return path (traphandler.rs) was fixed for that; this is the
        // user-CALLBACK return path and had the same bug -- both arm64 paths
        // (trap.c + usercall.c) were fixed together and amd64's callback path
        // was missed. Found by sweeping every kernel read of a guest-written
        // TEB field rather than by another symptom.
        GdiBatchCount = match crate::ke::amd64::traphandler::wow64_teb32(Teb) {
            Some(teb32) => (*teb32).GdiBatchCount,
            None => (*Teb).GdiBatchCount,
        };
        STATUS_SUCCESS
    };

    if Status < 0 {
        return Status;
    }

    if GdiBatchCount != 0 {
        *UserStackPointer = (*UserStackPointer).wrapping_sub(256);
        // unwrap() will BSOD on NULL — matches the C usercall.c:322 path which also
        // doesn't NULL-check. Reaching here implies win32k registered the pointer
        // (GdiBatchCount > 0 is set by win32k batching).
        (KeGdiFlushUserBatch.unwrap())();
    }

    *UserStackPointer = OldStack as u64;
    CallbackStatus
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
    let Pcr: PKIPCR;

    CurrentThread = PsGetCurrentThread();
    CalloutFrame = (*CurrentThread).CallbackStack as PKCALLOUT_FRAME;
    if CalloutFrame.is_null() {
        return STATUS_NO_CALLBACK_ACTIVE;
    }

    *((*CalloutFrame).OutputBuffer as *mut PVOID) = Result;
    *((*CalloutFrame).OutputLength as *mut ULONG) = ResultLength;

    CallbackTrapFrame = (*CurrentThread).TrapFrame;

    core::arch::asm!("cli");

    Pcr = KeGetPcr() as PKIPCR;

    TrapFrame = (*CalloutFrame).TrapFrame as PKTRAP_FRAME;

    if CallbackStatus == STATUS_CALLBACK_POP_STACK {
        *TrapFrame = *CallbackTrapFrame;
    }

    (*TrapFrame).Dr7 = 0;

    if (*CurrentThread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.__bindgen_anon_1.DebugActive & 0xFF != 0 {
        (*TrapFrame).Dr0 = (*CallbackTrapFrame).Dr0;
        (*TrapFrame).Dr1 = (*CallbackTrapFrame).Dr1;
        (*TrapFrame).Dr2 = (*CallbackTrapFrame).Dr2;
        (*TrapFrame).Dr3 = (*CallbackTrapFrame).Dr3;
        (*TrapFrame).Dr6 = (*CallbackTrapFrame).Dr6;
        (*TrapFrame).Dr7 = (*CallbackTrapFrame).Dr7;
    }

    (*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).Rsp0 = (*CalloutFrame).InitialStack as u64;
    (*Pcr).Prcb.RspBase = (*CalloutFrame).InitialStack as u64;

    (*CurrentThread).InitialStack = (*CalloutFrame).InitialStack as PVOID;
    (*CurrentThread).TrapFrame = TrapFrame;
    (*CurrentThread).CallbackStack = (*CalloutFrame).CallbackStack as PVOID;

    core::arch::asm!("sti");

    KiCallbackReturn(CalloutFrame as PVOID, CallbackStatus);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL - 2.0 + (https ://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     AMD64 User-mode Callout Mechanisms (APC and Win32K Callbacks)
//  * COPYRIGHT:   Timo Kreuzer(timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /*!
//  *  \name KiInitializeUserApc
//  *
//  *  \brief
//  *      Prepares the current trap frame (which must have come from user mode)
//  *      with the ntdll.KiUserApcDispatcher entrypoint, copying a UAPC_FRAME
//  *      structure with the context from the old trap frame to the threads user
//  *      mode stack.
//  *
//  *  \param ExceptionFrame - Pointer to the Exception Frame
//  *
//  *  \param TrapFrame  Pointer to the Trap Frame.
//  *
//  *  \param NormalRoutine - Pointer to the NormalRoutine to call.
//  *
//  *  \param NormalContext - Pointer to the context to send to the Normal Routine.
//  *
//  *  \param SystemArgument[1-2]
//  *        Pointer to a set of two parameters that contain untyped data.
//  *
//  *  \remarks
//  *      This function is called from KiDeliverApc, when the trap frame came
//  *      from user mode. This happens before a systemcall or interrupt exits back
//  *      to usermode or when a thread is started from PspUserThreadstartup.
//  *      The trap exit code will then leave to KiUserApcDispatcher which in turn
//  *      calls the NormalRoutine, passing NormalContext, SystemArgument1 and
//  *      SystemArgument2 as parameters. When that function returns, it calls
//  *      NtContinue to return back to the kernel, where the old context that was
//  *      saved on the usermode stack is restored and execution is transferred
//  *      back to usermode, where the original trap originated from.
//  *
// *--*/
// VOID
// NTAPI
// KiInitializeUserApc(
//     _In_ PKEXCEPTION_FRAME ExceptionFrame,
//     _Inout_ PKTRAP_FRAME TrapFrame,
//     _In_ PKNORMAL_ROUTINE NormalRoutine,
//     _In_ PVOID NormalContext,
//     _In_ PVOID SystemArgument1,
//     _In_ PVOID SystemArgument2)
// {
//     PUAPC_FRAME ApcFrame;
//     PCONTEXT Context;
//     EXCEPTION_RECORD ExceptionRecord;
//
//     /* Sanity check, that the trap frame is from user mode */
//     ASSERT((TrapFrame->SegCs & MODE_MASK) != KernelMode);
//
//     /* Allocate a 16 byte aligned UAPC_FRAME structure on the user stack */
//     ApcFrame = (PUAPC_FRAME)ALIGN_DOWN_POINTER_BY(TrapFrame->Rsp - sizeof(*ApcFrame), 16);
//     Context = &ApcFrame->Context;
//
//     /* Protect with SEH */
//     _SEH2_TRY
//     {
//         /* Probe the user mode APC frame */
//         ProbeForWrite(ApcFrame, sizeof(*ApcFrame), 16);
//
//         /* Convert the current trap frame to a context */
//         Context->ContextFlags = CONTEXT_FULL | CONTEXT_DEBUG_REGISTERS;
//         KeTrapFrameToContext(TrapFrame, ExceptionFrame, Context);
//
//         /* Set parameters for KiUserApcDispatcher */
//         Context->P1Home = (ULONG64)NormalContext;
//         Context->P2Home = (ULONG64)SystemArgument1;
//         Context->P3Home = (ULONG64)SystemArgument2;
//         Context->P4Home = (ULONG64)NormalRoutine;
//
//         /* Set up the machine frame for unwinding */
//         ApcFrame->MachineFrame.Rip = TrapFrame->Rip;
//         ApcFrame->MachineFrame.Rsp = TrapFrame->Rsp;
//     }
//     _SEH2_EXCEPT(ExceptionRecord = *_SEH2_GetExceptionInformation()->ExceptionRecord, EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Dispatch the exception */
//         ExceptionRecord.ExceptionAddress = (PVOID)TrapFrame->Rip;
//         KiDispatchException(&ExceptionRecord,
//                             ExceptionFrame,
//                             TrapFrame,
//                             UserMode,
//                             TRUE);
//     }
//     _SEH2_END;
//
//     /* Set the stack pointer to the context record */
//     TrapFrame->Rsp = (ULONG64)Context;
//
//     /* We jump to KiUserApcDispatcher in ntdll */
//     TrapFrame->Rip = (ULONG64)KeUserApcDispatcher;
//
//     /* Setup Ring 3 segments */
//     TrapFrame->SegCs = KGDT64_R3_CODE | RPL_MASK;
//     TrapFrame->SegFs = KGDT64_R3_CMTEB | RPL_MASK;
//     TrapFrame->SegGs = KGDT64_R3_DATA | RPL_MASK;
//     TrapFrame->SegSs = KGDT64_R3_DATA | RPL_MASK;
//
//     /* Sanitize EFLAGS, enable interrupts */
//     TrapFrame->EFlags &= EFLAGS_USER_SANITIZE;
//     TrapFrame->EFlags |= EFLAGS_INTERRUPT_MASK;
// }
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
// NTSTATUS
// FASTCALL
// KiUserModeCallout(
//     _Out_ PKCALLOUT_FRAME CalloutFrame)
// {
//     PKTHREAD CurrentThread;
//     PKTRAP_FRAME TrapFrame;
//     KTRAP_FRAME CallbackTrapFrame;
//     PKIPCR Pcr;
//     ULONG_PTR InitialStack;
//     NTSTATUS Status;
//
//     /* Get the current thread */
//     CurrentThread = PsGetCurrentThread();
//
//     /* Check if we are at pasive level */
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//
//     /* Check if we are attached or APCs are disabled */
//     ASSERT((CurrentThread->ApcStateIndex == OriginalApcEnvironment) &&
//         (CurrentThread->CombinedApcDisable == 0));
//
//     /* Align stack on a 16-byte boundary */
//     InitialStack = (ULONG_PTR)ALIGN_DOWN_POINTER_BY(CalloutFrame, 16);
//
//     /* Check if we have enough space on the stack */
//     if ((InitialStack - KERNEL_STACK_SIZE) < CurrentThread->StackLimit)
//     {
//         /* We don't, we'll have to grow our stack */
//         Status = MmGrowKernelStack((PVOID)InitialStack);
//
//         /* Quit if we failed */
//         if (!NT_SUCCESS(Status)) return Status;
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
//     /* Disable interrupts so we can fill the NPX State */
//     _disable();
//
//     /* Set the stack address */
//     CurrentThread->InitialStack = (PVOID)InitialStack;
//
//     /* Copy the trap frame to the new location */
//     CallbackTrapFrame = *TrapFrame;
//
//     /* Get PCR */
//     Pcr = (PKIPCR)KeGetPcr();
//
//     /* Set user-mode dispatcher address as EIP */
//     Pcr->TssBase->Rsp0 = InitialStack;
//     Pcr->Prcb.RspBase = InitialStack;
//     CallbackTrapFrame.Rip = (ULONG_PTR)KeUserCallbackDispatcher;
//
//     /* Bring interrupts back */
//     _enable();
//
//     /* Exit to user-mode */
//     KiUserCallbackExit(&CallbackTrapFrame);
// }
//
// VOID
// KiSetupUserCalloutFrame(
//     _Out_ PUCALLOUT_FRAME UserCalloutFrame,
//     _In_ PKTRAP_FRAME TrapFrame,
//     _In_ ULONG ApiNumber,
//     _In_ PVOID Buffer,
//     _In_ ULONG BufferLength)
// {
// #ifdef _M_IX86
//     CalloutFrame->Reserved = 0;
//     CalloutFrame->ApiNumber = ApiNumber;
//     CalloutFrame->Buffer = (ULONG_PTR)NewStack;
//     CalloutFrame->Length = ArgumentLength;
// #elif defined(_M_AMD64)
//     UserCalloutFrame->Buffer = (PVOID)(UserCalloutFrame + 1);
//     UserCalloutFrame->Length = BufferLength;
//     UserCalloutFrame->ApiNumber = ApiNumber;
//     UserCalloutFrame->MachineFrame.Rip = TrapFrame->Rip;
//     UserCalloutFrame->MachineFrame.Rsp = TrapFrame->Rsp;
// #else
// #error "KiSetupUserCalloutFrame not implemented!"
// #endif
// }
//
// NTSTATUS
// NTAPI
// KeUserModeCallback(
//     IN ULONG RoutineIndex,
//     IN PVOID Argument,
//     IN ULONG ArgumentLength,
//     OUT PVOID *Result,
//     OUT PULONG ResultLength)
// {
//     ULONG_PTR OldStack;
//     PUCHAR UserArguments;
//     PUCALLOUT_FRAME CalloutFrame;
//     PULONG_PTR UserStackPointer;
//     NTSTATUS CallbackStatus;
// #ifdef _M_IX86
//     PEXCEPTION_REGISTRATION_RECORD ExceptionList;
// #endif // _M_IX86
//     PTEB Teb;
//     ULONG GdiBatchCount = 0;
//     ASSERT(PsGetCurrentThread()->ApcState.KernelApcInProgress == FALSE);
//     ASSERT(KeGetPreviousMode() == UserMode);
//
//     /* Get the current user-mode stack */
//     UserStackPointer = KiGetUserModeStackAddress();
//     OldStack = *UserStackPointer;
//
//     /* Enter a SEH Block */
//     _SEH2_TRY
//     {
//         /* Calculate and align the stack. This is unaligned by 8 bytes, since the following
//            UCALLOUT_FRAME compensates for that and on entry we already have a full stack
//            frame with home space for the next call, i.e. we are already inside the function
//            body and the stack needs to be 16 byte aligned. */
//         UserArguments = (PUCHAR)ALIGN_DOWN_POINTER_BY(OldStack - ArgumentLength, 16) - 8;
//
//         /* The callout frame is below the arguments */
//         CalloutFrame = ((PUCALLOUT_FRAME)UserArguments) - 1;
//
//         /* Make sure it's all writable */
//         ProbeForWrite(CalloutFrame,
//                       sizeof(PUCALLOUT_FRAME) + ArgumentLength,
//                       sizeof(PVOID));
//
//         /* Copy the buffer into the stack */
//         RtlCopyMemory(UserArguments, Argument, ArgumentLength);
//
//         /* Write the arguments */
//         KiSetupUserCalloutFrame(CalloutFrame,
//                                 PsGetCurrentThread()->TrapFrame,
//                                 RoutineIndex,
//                                 UserArguments,
//                                 ArgumentLength);
//
//         /* Save the exception list */
//         Teb = PsGetCurrentThread()->Teb;
// #ifdef _M_IX86
//         ExceptionList = Teb->NtTib.ExceptionList;
// #endif // _M_IX86
//
//         /* Jump to user mode */
//         *UserStackPointer = (ULONG_PTR)CalloutFrame;
//         CallbackStatus = KiCallUserMode(Result, ResultLength);
//         if (CallbackStatus != STATUS_CALLBACK_POP_STACK)
//         {
// #ifdef _M_IX86
//             /* Only restore the exception list if we didn't crash in ring 3 */
//             Teb->NtTib.ExceptionList = ExceptionList;
// #endif // _M_IX86
//         }
//         else
//         {
//             /* Otherwise, pop the stack */
//             OldStack = *UserStackPointer;
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
//         *UserStackPointer -= 256;
//         KeGdiFlushUserBatch();
//     }
//
//     /* Restore stack and return */
//     *UserStackPointer = OldStack;
//     return CallbackStatus;
// }
//
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
//     PKIPCR Pcr;
//
//     /* Get the current thread and make sure we have a callback stack */
//     CurrentThread = PsGetCurrentThread();
//     CalloutFrame = CurrentThread->CallbackStack;
//     if (CalloutFrame == NULL)
//     {
//         return STATUS_NO_CALLBACK_ACTIVE;
//     }
//
//     /* Store the results in the callback stack */
//     *((PVOID*)CalloutFrame->OutputBuffer) = Result;
//     *((ULONG*)CalloutFrame->OutputLength) = ResultLength;
//
//     /* Get the trap frame */
//     CallbackTrapFrame = CurrentThread->TrapFrame;
//
//     /* Disable interrupts for NPX save and stack switch */
//     _disable();
//
//     /* Restore the exception list */
//     Pcr = (PKIPCR)KeGetPcr();
//
//     /* Get the previous trap frame */
//     TrapFrame = (PKTRAP_FRAME)CalloutFrame->TrapFrame;
//
//     /* Check if we failed in user mode */
//     if (CallbackStatus == STATUS_CALLBACK_POP_STACK)
//     {
//         *TrapFrame = *CallbackTrapFrame;
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
//     /* Switch the stack back to the previous value */
//     Pcr->TssBase->Rsp0 = CalloutFrame->InitialStack;
//     Pcr->Prcb.RspBase = CalloutFrame->InitialStack;
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
//     KiCallbackReturn(CalloutFrame, CallbackStatus);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
