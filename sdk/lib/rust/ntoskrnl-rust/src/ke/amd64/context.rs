use crate::ke::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    PCONTEXT, PKEXCEPTION_FRAME, PKTRAP_FRAME, KPROCESSOR_MODE, DWORD64,
};

const MODE_MASK: u16 = 1;
const KERNEL_MODE: KPROCESSOR_MODE = 0;
const KGDT64_R0_CODE: u16 = 0x10;
const KGDT64_R0_DATA: u16 = 0x18;
const KGDT64_R3_CMCODE: u16 = 0x20;
const KGDT64_R3_DATA: u16 = 0x28;
const KGDT64_R3_CODE: u16 = 0x30;
const KGDT64_R3_CMTEB: u16 = 0x50;
const RPL_MASK: u16 = 3;

const CONTEXT_AMD64: u32 = 0x00100000;
const CONTEXT_INTEGER: u32 = 0x00000002;
const CONTEXT_FLOATING_POINT: u32 = 0x00000008;
// CONTEXT_AMD64 (0x100000) | flag bits -- winnt_old.h:1351-1363. Dropping the arch
// base leaves a mask that still matches well-formed same-arch ContextFlags by luck
// (the discriminating bits are the low ones), but accepts a context record carrying
// a DIFFERENT arch base, which is exactly what WoW64 hands the 64-bit side.
const CONTEXT_CONTROL: u32 = 0x00100001;
const CONTEXT_SEGMENTS: u32 = 0x00000004;
const CONTEXT_DEBUG_REGISTERS: u32 = 0x00100010;

const EFLAGS_USER_SANITIZE: u32 = 0x003F4DD7;
const EFLAGS_INTERRUPT_MASK: u32 = 0x00000200;

const MAX_SYSCALL_PARAM_SIZE: u64 = 0x28;

extern "system" {
    static MmHighestUserAddress: *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "system" fn KeContextToTrapFrame(
    Context: PCONTEXT,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
    mut ContextFlags: u32,
    PreviousMode: KPROCESSOR_MODE,
) {
    let mut OldIrql: KIRQL;

    ntkrnl_sys::ASSERT!((*Context).ContextFlags & CONTEXT_AMD64 != 0);
    ContextFlags &= !CONTEXT_AMD64;

    OldIrql = KeGetCurrentIrql();
    if OldIrql < APC_LEVEL {
        KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(OldIrql));
    }

    if ContextFlags & CONTEXT_INTEGER != 0 {
        (*TrapFrame).Rax = (*Context).Rax;
        (*TrapFrame).Rcx = (*Context).Rcx;
        (*TrapFrame).Rdx = (*Context).Rdx;
        (*TrapFrame).Rbp = (*Context).Rbp;
        (*TrapFrame).R8 = (*Context).R8;
        (*TrapFrame).R9 = (*Context).R9;
        (*TrapFrame).R10 = (*Context).R10;
        (*TrapFrame).R11 = (*Context).R11;
        if !ExceptionFrame.is_null() {
            (*ExceptionFrame).Rbx = (*Context).Rbx;
            (*ExceptionFrame).Rsi = (*Context).Rsi;
            (*ExceptionFrame).Rdi = (*Context).Rdi;
            (*ExceptionFrame).R12 = (*Context).R12;
            (*ExceptionFrame).R13 = (*Context).R13;
            (*ExceptionFrame).R14 = (*Context).R14;
            (*ExceptionFrame).R15 = (*Context).R15;
        }
    }

    if ContextFlags & CONTEXT_FLOATING_POINT != 0 {
        (*TrapFrame).MxCsr = (*Context).MxCsr;
        (*TrapFrame).Xmm0 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm0;
        (*TrapFrame).Xmm1 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm1;
        (*TrapFrame).Xmm2 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm2;
        (*TrapFrame).Xmm3 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm3;
        (*TrapFrame).Xmm4 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm4;
        (*TrapFrame).Xmm5 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm5;
        if !ExceptionFrame.is_null() {
            (*ExceptionFrame).Xmm6 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm6;
            (*ExceptionFrame).Xmm7 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm7;
            (*ExceptionFrame).Xmm8 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm8;
            (*ExceptionFrame).Xmm9 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm9;
            (*ExceptionFrame).Xmm10 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm10;
            (*ExceptionFrame).Xmm11 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm11;
            (*ExceptionFrame).Xmm12 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm12;
            (*ExceptionFrame).Xmm13 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm13;
            (*ExceptionFrame).Xmm14 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm14;
            (*ExceptionFrame).Xmm15 = (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm15;
        }
    }

    if ContextFlags & CONTEXT_CONTROL != 0 {
        (*TrapFrame).Rip = (*Context).Rip;
        (*TrapFrame).Rsp = (*Context).Rsp;
        (*TrapFrame).EFlags = (*Context).EFlags;

        if PreviousMode == KERNEL_MODE {
            (*TrapFrame).SegCs = KGDT64_R0_CODE;
            (*TrapFrame).SegSs = KGDT64_R0_DATA;

            (*TrapFrame).EFlags &= EFLAGS_USER_SANITIZE | EFLAGS_INTERRUPT_MASK;
        } else {
            (*TrapFrame).SegCs = (*Context).SegCs;
            if (*TrapFrame).SegCs != (KGDT64_R3_CODE | RPL_MASK) {
                (*TrapFrame).SegCs = KGDT64_R3_CMCODE | RPL_MASK;
            }

            (*TrapFrame).SegSs = (*Context).SegSs;

            (*TrapFrame).EFlags &= EFLAGS_USER_SANITIZE;
            (*TrapFrame).EFlags |= EFLAGS_INTERRUPT_MASK;
        }
    }

    if ContextFlags & CONTEXT_SEGMENTS != 0 {
        if PreviousMode == KERNEL_MODE {
            (*TrapFrame).SegDs = KGDT64_R3_DATA | RPL_MASK;
            (*TrapFrame).SegEs = KGDT64_R3_DATA | RPL_MASK;
            (*TrapFrame).SegFs = KGDT64_R3_CMTEB | RPL_MASK;
            (*TrapFrame).SegGs = KGDT64_R3_DATA | RPL_MASK;
        } else {
            (*TrapFrame).SegDs = (*Context).SegDs;
            (*TrapFrame).SegEs = (*Context).SegEs;
            (*TrapFrame).SegFs = (*Context).SegFs;
            (*TrapFrame).SegGs = (*Context).SegGs;
        }
    }

    if ContextFlags & CONTEXT_DEBUG_REGISTERS != 0 {
        (*TrapFrame).Dr0 = (*Context).Dr0;
        (*TrapFrame).Dr1 = (*Context).Dr1;
        (*TrapFrame).Dr2 = (*Context).Dr2;
        (*TrapFrame).Dr3 = (*Context).Dr3;
        (*TrapFrame).Dr6 = (*Context).Dr6;
        (*TrapFrame).Dr7 = (*Context).Dr7;

        if PreviousMode != KERNEL_MODE {
            if (*TrapFrame).Dr0 > MmHighestUserAddress as u64 {
                (*TrapFrame).Dr0 = 0;
            }
            if (*TrapFrame).Dr1 > MmHighestUserAddress as u64 {
                (*TrapFrame).Dr1 = 0;
            }
            if (*TrapFrame).Dr2 > MmHighestUserAddress as u64 {
                (*TrapFrame).Dr2 = 0;
            }
            if (*TrapFrame).Dr3 > MmHighestUserAddress as u64 {
                (*TrapFrame).Dr3 = 0;
            }
        }
    }

    if OldIrql < APC_LEVEL {
        KeLowerIrql(OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeTrapFrameToContext(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    Context: PCONTEXT,
) {
    let mut ContextFlags: u32;
    let mut OldIrql: KIRQL;

    OldIrql = KeGetCurrentIrql();
    if OldIrql < APC_LEVEL {
        KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(OldIrql));
    }

    ContextFlags = (*Context).ContextFlags;
    ntkrnl_sys::ASSERT!(ContextFlags & CONTEXT_AMD64 != 0);
    ContextFlags &= !CONTEXT_AMD64;

    if ContextFlags & CONTEXT_INTEGER != 0 {
        (*Context).Rax = (*TrapFrame).Rax;
        (*Context).Rcx = (*TrapFrame).Rcx;
        (*Context).Rdx = (*TrapFrame).Rdx;
        (*Context).Rbp = (*TrapFrame).Rbp;
        (*Context).R8 = (*TrapFrame).R8;
        (*Context).R9 = (*TrapFrame).R9;
        (*Context).R10 = (*TrapFrame).R10;
        (*Context).R11 = (*TrapFrame).R11;

        if !ExceptionFrame.is_null() {
            (*Context).Rbx = (*ExceptionFrame).Rbx;
            (*Context).Rsi = (*ExceptionFrame).Rsi;
            (*Context).Rdi = (*ExceptionFrame).Rdi;
            (*Context).R12 = (*ExceptionFrame).R12;
            (*Context).R13 = (*ExceptionFrame).R13;
            (*Context).R14 = (*ExceptionFrame).R14;
            (*Context).R15 = (*ExceptionFrame).R15;
        }
    }

    if ContextFlags & CONTEXT_FLOATING_POINT != 0 {
        (*Context).MxCsr = (*TrapFrame).MxCsr;
        (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm0 = (*TrapFrame).Xmm0;
        (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm1 = (*TrapFrame).Xmm1;
        (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm2 = (*TrapFrame).Xmm2;
        (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm3 = (*TrapFrame).Xmm3;
        (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm4 = (*TrapFrame).Xmm4;
        (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm5 = (*TrapFrame).Xmm5;
        if !ExceptionFrame.is_null() {
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm6 = (*ExceptionFrame).Xmm6;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm7 = (*ExceptionFrame).Xmm7;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm8 = (*ExceptionFrame).Xmm8;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm9 = (*ExceptionFrame).Xmm9;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm10 = (*ExceptionFrame).Xmm10;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm11 = (*ExceptionFrame).Xmm11;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm12 = (*ExceptionFrame).Xmm12;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm13 = (*ExceptionFrame).Xmm13;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm14 = (*ExceptionFrame).Xmm14;
            (*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm15 = (*ExceptionFrame).Xmm15;
        }
    }

    if ContextFlags & CONTEXT_CONTROL != 0 {
        if ((*TrapFrame).SegCs & MODE_MASK) as i8 == 0 {
            (*Context).SegCs = KGDT64_R0_CODE;
            (*Context).SegSs = KGDT64_R0_DATA;
        } else {
            (*Context).SegCs = (*TrapFrame).SegCs;
            (*Context).SegSs = (*TrapFrame).SegSs;
        }

        (*Context).Rip = (*TrapFrame).Rip;
        (*Context).Rsp = (*TrapFrame).Rsp;
        (*Context).EFlags = (*TrapFrame).EFlags;
    }

    if ContextFlags & CONTEXT_SEGMENTS != 0 {
        if ((*TrapFrame).SegCs & MODE_MASK) as i8 == 0 {
            (*Context).SegDs = KGDT64_R3_DATA | RPL_MASK;
            (*Context).SegEs = KGDT64_R3_DATA | RPL_MASK;
            (*Context).SegFs = KGDT64_R3_CMTEB | RPL_MASK;
            (*Context).SegGs = KGDT64_R3_DATA | RPL_MASK;
        } else {
            (*Context).SegDs = (*TrapFrame).SegDs;
            (*Context).SegEs = (*TrapFrame).SegEs;
            (*Context).SegFs = (*TrapFrame).SegFs;
            (*Context).SegGs = (*TrapFrame).SegGs;
        }
    }

    if ContextFlags & CONTEXT_DEBUG_REGISTERS != 0 {
        (*Context).Dr0 = (*TrapFrame).Dr0;
        (*Context).Dr1 = (*TrapFrame).Dr1;
        (*Context).Dr2 = (*TrapFrame).Dr2;
        (*Context).Dr3 = (*TrapFrame).Dr3;
        (*Context).Dr6 = (*TrapFrame).Dr6;
        (*Context).Dr7 = (*TrapFrame).Dr7;
    }

    if OldIrql < APC_LEVEL {
        KeLowerIrql(OldIrql);
    }
}

extern "system" {
    fn RtlGetUnwindContext(Context: PCONTEXT, TargetFrame: DWORD64);
    fn RtlSetUnwindContext(Context: PCONTEXT, TargetFrame: DWORD64);
}

#[no_mangle]
pub unsafe extern "C" fn KiGetTrapContextInternal(TrapFrame: PKTRAP_FRAME, Context: PCONTEXT) {
    let TargetFrame: u64;

    KeTrapFrameToContext(TrapFrame, core::ptr::null_mut(), Context);

    TargetFrame = TrapFrame as u64 - MAX_SYSCALL_PARAM_SIZE;

    RtlGetUnwindContext(Context, TargetFrame);
}

#[no_mangle]
pub unsafe extern "C" fn KiSetTrapContextInternal(
    TrapFrame: PKTRAP_FRAME,
    Context: PCONTEXT,
    RequestorMode: KPROCESSOR_MODE,
) {
    let TargetFrame: u64;

    KeContextToTrapFrame(
        Context,
        core::ptr::null_mut(),
        TrapFrame,
        (*Context).ContextFlags,
        RequestorMode,
    );

    TargetFrame = TrapFrame as u64 - MAX_SYSCALL_PARAM_SIZE;

    RtlSetUnwindContext(Context, TargetFrame);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * PURPOSE:         CONTEXT related functions
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// KeContextToTrapFrame(IN PCONTEXT Context,
//                      IN OUT PKEXCEPTION_FRAME ExceptionFrame,
//                      IN OUT PKTRAP_FRAME TrapFrame,
//                      IN ULONG ContextFlags,
//                      IN KPROCESSOR_MODE PreviousMode)
// {
//     KIRQL OldIrql;
//
//     /* Make sure we have an amd64 context, then remove the flag */
//     ASSERT(ContextFlags & CONTEXT_AMD64);
//     ContextFlags &= ~CONTEXT_AMD64;
//
//     /* Do this at APC_LEVEL */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < APC_LEVEL) KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Handle integer registers */
//     if (ContextFlags & CONTEXT_INTEGER)
//     {
//         TrapFrame->Rax = Context->Rax;
//         TrapFrame->Rcx = Context->Rcx;
//         TrapFrame->Rdx = Context->Rdx;
//         TrapFrame->Rbp = Context->Rbp;
//         TrapFrame->R8 = Context->R8;
//         TrapFrame->R9 = Context->R9;
//         TrapFrame->R10 = Context->R10;
//         TrapFrame->R11 = Context->R11;
//         if (ExceptionFrame)
//         {
//             ExceptionFrame->Rbx = Context->Rbx;
//             ExceptionFrame->Rsi = Context->Rsi;
//             ExceptionFrame->Rdi = Context->Rdi;
//             ExceptionFrame->R12 = Context->R12;
//             ExceptionFrame->R13 = Context->R13;
//             ExceptionFrame->R14 = Context->R14;
//             ExceptionFrame->R15 = Context->R15;
//         }
//     }
//
//     /* Handle floating point registers */
//     if (ContextFlags & CONTEXT_FLOATING_POINT)
//     {
//         TrapFrame->MxCsr = Context->MxCsr;
//         TrapFrame->Xmm0 = Context->Xmm0;
//         TrapFrame->Xmm1 = Context->Xmm1;
//         TrapFrame->Xmm2 = Context->Xmm2;
//         TrapFrame->Xmm3 = Context->Xmm3;
//         TrapFrame->Xmm4 = Context->Xmm4;
//         TrapFrame->Xmm5 = Context->Xmm5;
//         if (ExceptionFrame)
//         {
//             ExceptionFrame->Xmm6 = Context->Xmm6;
//             ExceptionFrame->Xmm7 = Context->Xmm7;
//             ExceptionFrame->Xmm8 = Context->Xmm8;
//             ExceptionFrame->Xmm9 = Context->Xmm9;
//             ExceptionFrame->Xmm10 = Context->Xmm10;
//             ExceptionFrame->Xmm11 = Context->Xmm11;
//             ExceptionFrame->Xmm12 = Context->Xmm12;
//             ExceptionFrame->Xmm13 = Context->Xmm13;
//             ExceptionFrame->Xmm14 = Context->Xmm14;
//             ExceptionFrame->Xmm15 = Context->Xmm15;
//         }
//     }
//
//     /* Handle control registers */
//     if (ContextFlags & CONTEXT_CONTROL)
//     {
//         /* RIP, RSP, EFLAGS */
//         TrapFrame->Rip = Context->Rip;
//         TrapFrame->Rsp = Context->Rsp;
//         TrapFrame->EFlags = Context->EFlags;
//
//         if ((Context->SegCs & MODE_MASK) == KernelMode)
//         {
//             /* Set valid selectors */
//             TrapFrame->SegCs = KGDT64_R0_CODE;
//             TrapFrame->SegSs = KGDT64_R0_DATA;
//
//             /* Set valid EFLAGS */
//             TrapFrame->EFlags &= (EFLAGS_USER_SANITIZE | EFLAGS_INTERRUPT_MASK);
//         }
//         else
//         {
//             /* Copy selectors */
//             TrapFrame->SegCs = Context->SegCs;
//             if (TrapFrame->SegCs != (KGDT64_R3_CODE | RPL_MASK))
//             {
//                 TrapFrame->SegCs = (KGDT64_R3_CMCODE | RPL_MASK);
//             }
//
//             TrapFrame->SegSs = Context->SegSs;
//
//             /* Set valid EFLAGS */
//             TrapFrame->EFlags &= EFLAGS_USER_SANITIZE;
//             TrapFrame->EFlags |= EFLAGS_INTERRUPT_MASK;
//         }
//     }
//
//     /* Handle segment selectors */
//     if (ContextFlags & CONTEXT_SEGMENTS)
//     {
//         /* Check if this was a Kernel Trap */
//         if ((Context->SegCs & MODE_MASK) == KernelMode)
//         {
//             /* Set valid selectors */
//             TrapFrame->SegDs = KGDT64_R3_DATA | RPL_MASK;
//             TrapFrame->SegEs = KGDT64_R3_DATA | RPL_MASK;
//             TrapFrame->SegFs = KGDT64_R3_CMTEB | RPL_MASK;
//             TrapFrame->SegGs = KGDT64_R3_DATA | RPL_MASK;
//         }
//         else
//         {
//             /* Copy selectors */
//             TrapFrame->SegDs = Context->SegDs;
//             TrapFrame->SegEs = Context->SegEs;
//             TrapFrame->SegFs = Context->SegFs;
//             TrapFrame->SegGs = Context->SegGs;
//         }
//     }
//
//     /* Handle debug registers */
//     if (ContextFlags & CONTEXT_DEBUG_REGISTERS)
//     {
//         /* Copy the debug registers */
//         TrapFrame->Dr0 = Context->Dr0;
//         TrapFrame->Dr1 = Context->Dr1;
//         TrapFrame->Dr2 = Context->Dr2;
//         TrapFrame->Dr3 = Context->Dr3;
//         TrapFrame->Dr6 = Context->Dr6;
//         TrapFrame->Dr7 = Context->Dr7;
//
//         if ((Context->SegCs & MODE_MASK) != KernelMode)
//         {
//             if (TrapFrame->Dr0 > (ULONG64)MmHighestUserAddress)
//                 TrapFrame->Dr0 = 0;
//             if (TrapFrame->Dr1 > (ULONG64)MmHighestUserAddress)
//                 TrapFrame->Dr1 = 0;
//             if (TrapFrame->Dr2 > (ULONG64)MmHighestUserAddress)
//                 TrapFrame->Dr2 = 0;
//             if (TrapFrame->Dr3 > (ULONG64)MmHighestUserAddress)
//                 TrapFrame->Dr3 = 0;
//         }
//     }
//
//     /* Restore IRQL */
//     if (OldIrql < APC_LEVEL) KeLowerIrql(OldIrql);
// }
//
// VOID
// NTAPI
// KeTrapFrameToContext(IN PKTRAP_FRAME TrapFrame,
//                      IN PKEXCEPTION_FRAME ExceptionFrame,
//                      IN OUT PCONTEXT Context)
// {
//     ULONG ContextFlags;
//     KIRQL OldIrql;
//
//     /* Do this at APC_LEVEL */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < APC_LEVEL) KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Make sure we have an amd64 context, then remove the flag */
//     ContextFlags = Context->ContextFlags;
//     ASSERT(ContextFlags & CONTEXT_AMD64);
//     ContextFlags &= ~CONTEXT_AMD64;
//
//     /* Handle integer registers */
//     if (ContextFlags & CONTEXT_INTEGER)
//     {
//         Context->Rax = TrapFrame->Rax;
//         Context->Rcx = TrapFrame->Rcx;
//         Context->Rdx = TrapFrame->Rdx;
//         Context->Rbp = TrapFrame->Rbp;
//         Context->R8 = TrapFrame->R8;
//         Context->R9 = TrapFrame->R9;
//         Context->R10 = TrapFrame->R10;
//         Context->R11 = TrapFrame->R11;
//
//         if (ExceptionFrame)
//         {
//             Context->Rbx = ExceptionFrame->Rbx;
//             Context->Rsi = ExceptionFrame->Rsi;
//             Context->Rdi = ExceptionFrame->Rdi;
//             Context->R12 = ExceptionFrame->R12;
//             Context->R13 = ExceptionFrame->R13;
//             Context->R14 = ExceptionFrame->R14;
//             Context->R15 = ExceptionFrame->R15;
//         }
//     }
//
//     /* Handle floating point registers */
//     if (ContextFlags & CONTEXT_FLOATING_POINT)
//     {
//         Context->MxCsr = TrapFrame->MxCsr;
//         Context->Xmm0 = TrapFrame->Xmm0;
//         Context->Xmm1 = TrapFrame->Xmm1;
//         Context->Xmm2 = TrapFrame->Xmm2;
//         Context->Xmm3 = TrapFrame->Xmm3;
//         Context->Xmm4 = TrapFrame->Xmm4;
//         Context->Xmm5 = TrapFrame->Xmm5;
//         if (ExceptionFrame)
//         {
//             Context->Xmm6 = ExceptionFrame->Xmm6;
//             Context->Xmm7 = ExceptionFrame->Xmm7;
//             Context->Xmm8 = ExceptionFrame->Xmm8;
//             Context->Xmm9 = ExceptionFrame->Xmm9;
//             Context->Xmm10 = ExceptionFrame->Xmm10;
//             Context->Xmm11 = ExceptionFrame->Xmm11;
//             Context->Xmm12 = ExceptionFrame->Xmm12;
//             Context->Xmm13 = ExceptionFrame->Xmm13;
//             Context->Xmm14 = ExceptionFrame->Xmm14;
//             Context->Xmm15 = ExceptionFrame->Xmm15;
//         }
//     }
//
//     /* Handle control registers */
//     if (ContextFlags & CONTEXT_CONTROL)
//     {
//         /* Check if this was a Kernel Trap */
//         if ((TrapFrame->SegCs & MODE_MASK) == KernelMode)
//         {
//             /* Set valid selectors */
//             Context->SegCs = KGDT64_R0_CODE;
//             Context->SegSs = KGDT64_R0_DATA;
//         }
//         else
//         {
//             /* Copy selectors */
//             Context->SegCs = TrapFrame->SegCs;
//             Context->SegSs = TrapFrame->SegSs;
//         }
//
//         /* Copy RIP, RSP, EFLAGS */
//         Context->Rip = TrapFrame->Rip;
//         Context->Rsp = TrapFrame->Rsp;
//         Context->EFlags = TrapFrame->EFlags;
//     }
//
//     /* Handle segment selectors */
//     if (ContextFlags & CONTEXT_SEGMENTS)
//     {
//         /* Check if this was a Kernel Trap */
//         if ((TrapFrame->SegCs & MODE_MASK) == KernelMode)
//         {
//             /* Set valid selectors */
//             Context->SegDs = KGDT64_R3_DATA | RPL_MASK;
//             Context->SegEs = KGDT64_R3_DATA | RPL_MASK;
//             Context->SegFs = KGDT64_R3_CMTEB | RPL_MASK;
//             Context->SegGs = KGDT64_R3_DATA | RPL_MASK;
//         }
//         else
//         {
//             /* Copy selectors */
//             Context->SegDs = TrapFrame->SegDs;
//             Context->SegEs = TrapFrame->SegEs;
//             Context->SegFs = TrapFrame->SegFs;
//             Context->SegGs = TrapFrame->SegGs;
//         }
//     }
//
//     /* Handle debug registers */
//     if (ContextFlags & CONTEXT_DEBUG_REGISTERS)
//     {
//         /* Copy the debug registers */
//         Context->Dr0 = TrapFrame->Dr0;
//         Context->Dr1 = TrapFrame->Dr1;
//         Context->Dr2 = TrapFrame->Dr2;
//         Context->Dr3 = TrapFrame->Dr3;
//         Context->Dr6 = TrapFrame->Dr6;
//         Context->Dr7 = TrapFrame->Dr7;
//     }
//
//     /* Restore IRQL */
//     if (OldIrql < APC_LEVEL) KeLowerIrql(OldIrql);
// }
//
// VOID
// RtlGetUnwindContext(
//     _Out_ PCONTEXT Context,
//     _In_ DWORD64 TargetFrame);
//
// VOID
// KiGetTrapContextInternal(
//     _In_ PKTRAP_FRAME TrapFrame,
//     _Out_ PCONTEXT Context)
// {
//     ULONG64 TargetFrame;
//
//     /* Get the volatile register context from the trap frame */
//     KeTrapFrameToContext(TrapFrame, NULL, Context);
//
//     /* The target frame is MAX_SYSCALL_PARAM_SIZE bytes before the trap frame */
//     TargetFrame = (ULONG64)TrapFrame - MAX_SYSCALL_PARAM_SIZE;
//
//     /* Get the nonvolatiles on the stack */
//     RtlGetUnwindContext(Context, TargetFrame);
// }
//
// VOID
// RtlSetUnwindContext(
//     _In_ PCONTEXT Context,
//     _In_ DWORD64 TargetFrame);
//
// VOID
// KiSetTrapContextInternal(
//     _Out_ PKTRAP_FRAME TrapFrame,
//     _In_ PCONTEXT Context,
//     _In_ KPROCESSOR_MODE RequestorMode)
// {
//     ULONG64 TargetFrame;
//
//     /* Save the volatile register context in the trap frame */
//     KeContextToTrapFrame(Context,
//                          NULL,
//                          TrapFrame,
//                          Context->ContextFlags,
//                          RequestorMode);
//
//     /* The target frame is MAX_SYSCALL_PARAM_SIZE bytes before the trap frame */
//     TargetFrame = (ULONG64)TrapFrame - MAX_SYSCALL_PARAM_SIZE ;
//
//     /* Set the nonvolatiles on the stack */
//     RtlSetUnwindContext(Context, TargetFrame);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
