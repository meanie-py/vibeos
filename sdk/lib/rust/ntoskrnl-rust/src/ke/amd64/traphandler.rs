use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    PKTHREAD, PKTRAP_FRAME, PKEXCEPTION_FRAME, PKSERVICE_TABLE_DESCRIPTOR,
    PULONG64, PKPRCB, KIPCR, PGDI_BATCHFLUSH_ROUTINE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use crate::ke::procobj::KeServiceDescriptorTable;
extern "system" {
    fn KiRetireDpcListInDpcStack(Prcb: PKPRCB, DpcStack: PVOID);
    fn KiQuantumEnd();
    fn KiSwapContext(WaitIrql: KIRQL, OldThread: PKTHREAD);
    fn KiHandleNmi();
    static mut KeGdiFlushUserBatch: PGDI_BATCHFLUSH_ROUTINE;
    fn KiConvertToGuiThread() -> NTSTATUS;
    fn _enable();
    fn _disable();
    fn __addgsdword(offset: ULONG, value: ULONG);
    fn __debugbreak();
    static MmUserProbeAddress: ULONG64;
}
extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn KiSendEOI();
    #[cfg(not(feature = "smp"))]
    fn KxQueueReadyThread(Thread: PKTHREAD, Prcb: PKPRCB);
    fn KiAcquirePrcbLock(Prcb: PKPRCB);
    fn NtCurrentTeb() -> PVOID;
    fn __readmsr(msr: ULONG) -> ULONG64;
    fn __swapgs();
}
extern "C" {
    fn KiProcessorFreezeHandler(TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME) -> BOOLEAN;
}

const MSR_GS_BASE: ULONG = 0xC0000101;
const MODE_MASK: USHORT = 1;
const TABLE_OFFSET_BITS: ULONG = 12;
const TABLE_NUMBER_BITS: ULONG = 0x1;
const SERVICE_NUMBER_MASK: ULONG = 0xFFF;
const WIN32K_SERVICE_INDEX: ULONG = 1;
const MAX_SYSCALL_PARAMS: usize = 16;
const FIELD_OFFSET_KIPCR_PRCB_KESYSTEMCALLS: ULONG = core::mem::offset_of!(KIPCR, Prcb) as ULONG;

#[repr(u8)]
enum KTHREAD_STATE {
    Running = 2,
}

#[repr(u8)]
enum KWAIT_REASON {
    _KWAIT_REASON_WrDispatchInt = 31,
}


/// TEB32 of the current thread when this is a WoW64 process, else None.
///
/// The 32-bit TEB sits at TEB64 + 0x2000 in the combined WoW64 TEB region;
/// the allocator that lays the region out is `ntoskrnl-rust`'s
/// `ARM3/procsup.rs` (`WOW64_TEB32_OFFSET`) and win32k's projection
/// (`win32k-rust/src/wow64.rs`) uses the same constant.
#[inline]
pub(crate) unsafe fn wow64_teb32(
    teb: *const ntkrnl_sys::ddk::_TEB,
) -> Option<*const ntkrnl_sys::ddk::_TEB32> {
    const WOW64_TEB32_OFFSET: usize = 0x2000;
    let p = ntkrnl_sys::ps::PsGetCurrentProcess();
    if p.is_null() || (*(p as *const ntkrnl_sys::ddk::_EPROCESS)).Wow64Process.is_null() {
        return None;
    }
    Some((teb as *const u8).add(WOW64_TEB32_OFFSET) as *const ntkrnl_sys::ddk::_TEB32) // nolint: magic-offset (WoW64 combined-region layout, TEB32 at TEB64+0x2000 by design — same annotation as ARM3/procsup.rs:732, which allocates the region)
}

#[no_mangle]
pub unsafe extern "system" fn KiDpcInterruptHandler() {
    let Prcb = KeGetCurrentPrcb();
    let mut NewThread: PKTHREAD;
    let mut OldThread: PKTHREAD;
    let OldIrql: KIRQL;

    OldIrql = KfRaiseIrql(DISPATCH_LEVEL);

    KiSendEOI();

    // M3 SMP: if this CPU was idle in lazy-TLB state, service any deferred
    // shootdown BEFORE retiring DPCs (DPC routines touch remappable memory).
    #[cfg(feature = "smp")]
    {
        unsafe extern "C" { fn KiTbLazyIdleExit(); }
        KiTbLazyIdleExit();
    }

    if ((*Prcb).DpcData[0].DpcQueueDepth != 0) ||
       ((*Prcb).TimerRequest != 0) ||
       (!(*Prcb).DeferredReadyListHead.Next.is_null())
    {
        KiRetireDpcListInDpcStack(Prcb, (*Prcb).DpcStack);
    }

    _enable();

    if (*Prcb).QuantumEnd != 0 {
        (*Prcb).QuantumEnd = 0u8;
        KiQuantumEnd();
    } else if !(*Prcb).NextThread.is_null() {
        KiAcquirePrcbLock(Prcb);

        OldThread = (*Prcb).CurrentThread;
        NewThread = (*Prcb).NextThread;

        #[cfg(feature = "smp")]
        ntkrnl_sys::inline::ke::ki_check_promote_not_linked(NewThread, 0x02); // KiDpcInterruptHandler

        (*Prcb).NextThread = core::ptr::null_mut();
        (*Prcb).CurrentThread = NewThread;

        (*NewThread).__bindgen_anon_5.__bindgen_anon_6.State = KTHREAD_STATE::Running as u8;
        #[cfg(feature = "smp")]
        ntkrnl_sys::inline::ke::rq_inflight_unmark(NewThread);
        (*OldThread).WaitReason = KWAIT_REASON::_KWAIT_REASON_WrDispatchInt as u8;

        // NT MP (2k3 KiDispatchInterrupt): mark the outgoing thread
        // context-swap-busy BEFORE queueing it ready — without this, another
        // CPU that picks it sails straight through ctxswitch.S's SwapBusy
        // spin (flag never raised) and adopts a stack we're still running on.
        // KiSwapContextInternal clears it once the old context is fully saved.
        ntkrnl_sys::inline::ke::KiSetThreadSwapBusy(OldThread);

        KxQueueReadyThread(OldThread, Prcb);

        KiSwapContext(APC_LEVEL, OldThread);
    }

    _disable();
    KeLowerIrql(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KiNmiInterruptHandler(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
) {
    let mut ManualSwapGs = 0u8;

    if ((*TrapFrame).SegCs & MODE_MASK) == 0 {
        if (__readmsr(MSR_GS_BASE) as i64) >= 0 {
            __swapgs();
            ManualSwapGs = 1u8;
        }
    }

    if KiProcessorFreezeHandler(TrapFrame, ExceptionFrame) != 0 {
        if ManualSwapGs != 0 {
            __swapgs();
        }
        return;
    }

    KiHandleNmi();

    if ManualSwapGs != 0 {
        __swapgs();
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtSyscallFailure() -> NTSTATUS {
    (*(*PsGetCurrentThread()).TrapFrame).Rax as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn KiSystemCallHandler(TrapFrame: PKTRAP_FRAME) -> PVOID {
    let mut DescriptorTable: PKSERVICE_TABLE_DESCRIPTOR;
    let Thread: PKTHREAD;
    let mut KernelParams: PULONG64;
    let mut UserParams: PULONG64;
    let mut ServiceNumber: ULONG;
    let TableIndex: ULONG;
    let Count: ULONG;
    let mut UserRsp: ULONG64;

    __addgsdword(FIELD_OFFSET_KIPCR_PRCB_KESYSTEMCALLS, 1);

    Thread = PsGetCurrentThread();

    (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = 1i8;
    (*TrapFrame).PreviousMode = 1i8;

    (*TrapFrame).__bindgen_anon_4.ExceptionFrame = 0;

    UserRsp = (*TrapFrame).Rsp;

    _enable();

    // cswitchdep tap 4 (user→kernel): the syscall path is user-origin by
    // construction (kernel Zw goes via KiZwSystemService), and reaching here
    // means the kernel stack + trap frame are in place post-swapgs.
    #[cfg(feature = "cswitchdep")]
    crate::ke::cswitchdep::VibeosCswitchdepUserEntry(1, crate::code_addr!());

    if UserRsp > MmUserProbeAddress || UserRsp < 0x10000 {
        (*TrapFrame).Rax = STATUS_ACCESS_VIOLATION as u64;
        return NtSyscallFailure as PVOID;
    }

    UserParams = (UserRsp as PULONG64).add(1);
    KernelParams = (TrapFrame as PULONG64).sub(MAX_SYSCALL_PARAMS);

    ServiceNumber = (*TrapFrame).Rax as ULONG;
    let TableIndex = (ServiceNumber >> TABLE_OFFSET_BITS) & ((1 << TABLE_NUMBER_BITS) - 1);
    ServiceNumber &= SERVICE_NUMBER_MASK;

    if TableIndex == WIN32K_SERVICE_INDEX {
        // C: if (NtCurrentTeb()->GdiBatchCount) KeGdiFlushUserBatch()
        // (ke/amd64/traphandler.c:224). Typed field read — the previous
        // hand-rolled `*teb_ptr.add(0x77 / 4)` read TEB+0x74; the real amd64
        // TEB.GdiBatchCount is at 0x1740 (peb_teb.h), so the flush NEVER
        // fired and the gdi32 TEB batch overflowed GdiTebBatch.Buffer
        // (0x300..0x7d8) into the GDI TEB fields and CLIENTINFO at 0x800+
        // (field manual §9.41: explorer's Ctrl+Esc death + blank menu labels).
        //
        // A WoW64 guest's gdi32 batches into its OWN TEB -- the TEB32 -- and
        // increments TEB32->GdiBatchCount (win32ss/gdi/gdi32/include/gdi32p.h).
        // Reading TEB64's counter here therefore NEVER sees a guest's batch, so
        // the flush never ran for one, so neither Offset nor GdiBatchCount was
        // ever reset. gdi32 calls NtGdiFlush() when it thinks the buffer is
        // full and then writes at Buffer+Offset WITHOUT re-checking, trusting
        // the kernel to have zeroed Offset -- so Offset climbed without bound
        // and walked straight out of GdiTebBatch.Buffer into the TEB32 fields
        // above it. MEASURED: Offset reached 0x508 from the buffer base, which
        // is exactly CLIENTINFO.pDeskInfo, and user32's ValidateHandleNoErr
        // then dereferenced a batch record byte as a pointer.
        //
        // This is field manual §9.41 ("the GDI TEB batch was never flushed")
        // a second time: same bug, same symptoms, reached through the wrong
        // TEB rather than the wrong offset -- which is why scan_magic_offsets,
        // the detector added for §9.41, cannot see it.
        //
        let GdiBatchCount: ULONG = {
            let teb = NtCurrentTeb() as *const ntkrnl_sys::ddk::_TEB;
            if teb.is_null() {
                0
            } else if let Some(teb32) = crate::ke::amd64::traphandler::wow64_teb32(teb) {
                (*teb32).GdiBatchCount
            } else {
                (*teb).GdiBatchCount
            }
        };

        if GdiBatchCount != 0 {
            // GdiBatchCount > 0 implies win32k has registered the flush routine.
            if let Some(flush) = KeGdiFlushUserBatch {
                flush();
            }
        }
    }

    // C: DescriptorTable = (PKSERVICE_TABLE_DESCRIPTOR)Thread->ServiceTable + TableIndex;
    // Pointer arithmetic: advances by TableIndex * sizeof(KSERVICE_TABLE_DESCRIPTOR)
    DescriptorTable = ((*Thread).ServiceTable as PKSERVICE_TABLE_DESCRIPTOR).add(TableIndex as usize);

    if ServiceNumber >= (*DescriptorTable).Limit {
        if (TableIndex == WIN32K_SERVICE_INDEX) &&
           ((*Thread).ServiceTable == core::ptr::addr_of_mut!(KeServiceDescriptorTable) as PVOID)
        {
            return KiConvertToGuiThread as PVOID;
        }
        (*TrapFrame).Rax = STATUS_INVALID_SYSTEM_SERVICE as u64;
        return NtSyscallFailure as PVOID;
    }

    let count_bytes = *(*DescriptorTable).Number.add(ServiceNumber as usize);
    let Count = count_bytes / 8;

    if Count > 4 {
        let EndAddr = (UserParams.add((Count - 1) as usize) as ULONG64)
            .wrapping_add(core::mem::size_of::<ULONG64>() as ULONG64)
            .wrapping_sub(1);
        if EndAddr > MmUserProbeAddress {
            (*TrapFrame).Rax = STATUS_ACCESS_VIOLATION as u64;
            return NtSyscallFailure as PVOID;
        }
    }

    // C uses Duff's device-style fall-through: case 16 copies params 15..4,
    // case 5 copies only param 4, etc. Rust match doesn't fall through,
    // so use if-chain instead. First 4 params are in registers (rcx,rdx,r8,r9).
    if Count >= 16 { *KernelParams.add(15) = *UserParams.add(15); }
    if Count >= 15 { *KernelParams.add(14) = *UserParams.add(14); }
    if Count >= 14 { *KernelParams.add(13) = *UserParams.add(13); }
    if Count >= 13 { *KernelParams.add(12) = *UserParams.add(12); }
    if Count >= 12 { *KernelParams.add(11) = *UserParams.add(11); }
    if Count >= 11 { *KernelParams.add(10) = *UserParams.add(10); }
    if Count >= 10 { *KernelParams.add(9) = *UserParams.add(9); }
    if Count >= 9  { *KernelParams.add(8) = *UserParams.add(8); }
    if Count >= 8  { *KernelParams.add(7) = *UserParams.add(7); }
    if Count >= 7  { *KernelParams.add(6) = *UserParams.add(6); }
    if Count >= 6  { *KernelParams.add(5) = *UserParams.add(5); }
    if Count >= 5  { *KernelParams.add(4) = *UserParams.add(4); }

    (*(*DescriptorTable).Base.add(ServiceNumber as usize)) as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn KiSystemService(
    Thread: PKTHREAD,
    TrapFrame: PKTRAP_FRAME,
    Instruction: ULONG,
) {
    let Thread = opaque_ptr(Thread);
    __debugbreak();
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * PURPOSE:         x64 trap handlers
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
// VOID
// KiRetireDpcListInDpcStack(
//     PKPRCB Prcb,
//     PVOID DpcStack);
//
// NTSTATUS
// KiConvertToGuiThread(
//     VOID);
//
// _Requires_lock_not_held_(Prcb->PrcbLock)
// VOID
// NTAPI
// KiDpcInterruptHandler(VOID)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD NewThread, OldThread;
//     KIRQL OldIrql;
//
//     /* Raise to DISPATCH_LEVEL */
//     OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
//
//     /* Send an EOI */
//     KiSendEOI();
//
//     /* Check for pending timers, pending DPCs, or pending ready threads */
//     if ((Prcb->DpcData[0].DpcQueueDepth) ||
//         (Prcb->TimerRequest) ||
//         (Prcb->DeferredReadyListHead.Next))
//     {
//         /* Retire DPCs while under the DPC stack */
//         KiRetireDpcListInDpcStack(Prcb, Prcb->DpcStack);
//     }
//
//     /* Enable interrupts */
//     _enable();
//
//     /* Check for quantum end */
//     if (Prcb->QuantumEnd)
//     {
//         /* Handle quantum end */
//         Prcb->QuantumEnd = FALSE;
//         KiQuantumEnd();
//     }
//     else if (Prcb->NextThread)
//     {
//         /* Acquire the PRCB lock */
//         KiAcquirePrcbLock(Prcb);
//
//         /* Capture current thread data */
//         OldThread = Prcb->CurrentThread;
//         NewThread = Prcb->NextThread;
//
//         /* Set new thread data */
//         Prcb->NextThread = NULL;
//         Prcb->CurrentThread = NewThread;
//
//         /* The thread is now running */
//         NewThread->State = Running;
//         OldThread->WaitReason = WrDispatchInt;
//
//         /* Make the old thread ready */
//         KxQueueReadyThread(OldThread, Prcb);
//
//         /* Swap to the new thread */
//         KiSwapContext(APC_LEVEL, OldThread);
//     }
//
//     /* Disable interrupts and go back to old irql */
//     _disable();
//     KeLowerIrql(OldIrql);
// }
//
// VOID
// KiNmiInterruptHandler(
//     _In_ PKTRAP_FRAME TrapFrame,
//     _In_ PKEXCEPTION_FRAME ExceptionFrame)
// {
//     BOOLEAN ManualSwapGs = FALSE;
//
//     /* Check if the NMI came from kernel mode */
//     if ((TrapFrame->SegCs & MODE_MASK) == 0)
//     {
//         /* Check if GS base is already kernel mode. This is needed, because
//            we might be interrupted during an interrupt/exception from user-mode
//            before the swapgs instruction. */
//         if ((LONG64)__readmsr(MSR_GS_BASE) >= 0)
//         {
//             /* Swap GS to kernel */
//             __swapgs();
//             ManualSwapGs = TRUE;
//         }
//     }
//
//     /* Check if this is a freeze */
//     if (KiProcessorFreezeHandler(TrapFrame, ExceptionFrame))
//     {
//         /* NMI was handled */
//         goto Exit;
//     }
//
//     /* Handle the NMI */
//     KiHandleNmi();
//
// Exit:
//     /* Check if we need to swap GS back */
//     if (ManualSwapGs)
//     {
//         /* Swap GS back to user */
//         __swapgs();
//     }
// }
//
// #define MAX_SYSCALL_PARAMS 16
//
// NTSTATUS
// NtSyscallFailure(void)
// {
//     /* This is the failure function */
//     return (NTSTATUS)KeGetCurrentThread()->TrapFrame->Rax;
// }
//
// PVOID
// KiSystemCallHandler(
//     _In_ PKTRAP_FRAME TrapFrame)
// {
//     PKSERVICE_TABLE_DESCRIPTOR DescriptorTable;
//     PKTHREAD Thread;
//     PULONG64 KernelParams, UserParams;
//     ULONG ServiceNumber, TableIndex, Count;
//     ULONG64 UserRsp;
//
//     /* TrapFrame is passed from KiSystemCallEntry64 assembly.
//        The old _AddressOfReturnAddress() calculation is broken with clang
//        because clang's non-standard frame pointer layout makes
//        __builtin_frame_address(0) return a wrong address. */
//
//     /* Increase system call count */
//     __addgsdword(FIELD_OFFSET(KIPCR, Prcb.KeSystemCalls), 1);
//
//     /* Get the current thread */
//     Thread = PsGetCurrentThread();
//
//     /* Set previous mode */
//     Thread->PreviousMode = TrapFrame->PreviousMode = UserMode;
//
//     /* We don't have an exception frame yet */
//     TrapFrame->ExceptionFrame = 0;
//
//     /* Get the user Stack pointer */
//     UserRsp = TrapFrame->Rsp;
//
//     /* Enable interrupts */
//     _enable();
//
//     /* If the usermode rsp was not a usermode address, prepare an exception */
//     if (UserRsp > MmUserProbeAddress
// #ifdef _USE_DUMMY_PSEH
//         /* DUMMY_PSEH: also reject NULL/low RSP since SEH won't catch faults */
//         || UserRsp < 0x10000
// #endif
//         )
//     {
// #ifdef _USE_DUMMY_PSEH
//         /* DUMMY_PSEH: _SEH2_EXCEPT won't catch the probe page fault, fail now */
//         TrapFrame->Rax = STATUS_ACCESS_VIOLATION;
//         return (PVOID)NtSyscallFailure;
// #else
//         UserRsp = MmUserProbeAddress;
// #endif
//     }
//
//     /* Get the address of the usermode and kernelmode parameters */
//     UserParams = (PULONG64)UserRsp + 1;
//     KernelParams = (PULONG64)TrapFrame - MAX_SYSCALL_PARAMS;
//
//     /* Get the system call number from the trap frame and decode it */
//     ServiceNumber = (ULONG)TrapFrame->Rax;
//     TableIndex = (ServiceNumber >> TABLE_OFFSET_BITS) & ((1 << TABLE_NUMBER_BITS) - 1);
//     ServiceNumber &= SERVICE_NUMBER_MASK;
//
//     /* Check for win32k system calls */
//     if (TableIndex == WIN32K_SERVICE_INDEX)
//     {
//         ULONG GdiBatchCount;
//
//         /* Read the GDI batch count from the TEB */
//         _SEH2_TRY
//         {
//             GdiBatchCount = NtCurrentTeb()->GdiBatchCount;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             GdiBatchCount = 0;
//         }
//         _SEH2_END;
//
//         /* Flush batch, if there are entries */
//         if (GdiBatchCount != 0)
//         {
//             KeGdiFlushUserBatch();
//         }
//     }
//
//     /* Get descriptor table */
//     DescriptorTable = &((PKSERVICE_TABLE_DESCRIPTOR)Thread->ServiceTable)[TableIndex];
//
//     /* Validate the system call number */
//     if (ServiceNumber >= DescriptorTable->Limit)
//     {
//         /* Check if this is a GUI call and this is not a GUI thread yet */
//         if ((TableIndex == WIN32K_SERVICE_INDEX) &&
//             (Thread->ServiceTable == KeServiceDescriptorTable))
//         {
//             /* Convert this thread to a GUI thread.
//                It is invalid to change the stack in the middle of a C function,
//                therefore we return KiConvertToGuiThread to the system call entry
//                point, which then calls the asm function KiConvertToGuiThread,
//                which allocates a new stack, switches to it, calls
//                PsConvertToGuiThread and resumes in the middle of
//                KiSystemCallEntry64 to restart the system call handling.
//                If converting fails, the system call returns a failure code. */
//             return (PVOID)KiConvertToGuiThread;
//         }
//
//         /* Fail the call */
//         TrapFrame->Rax = STATUS_INVALID_SYSTEM_SERVICE;
//         return (PVOID)NtSyscallFailure;
//     }
//
//     /* Get stack bytes and calculate argument count */
//     Count = DescriptorTable->Number[ServiceNumber] / 8;
//
// #ifdef _USE_DUMMY_PSEH
//     /* DUMMY_PSEH: validate user stack range before copying parameters.
//        Only check address range — do NOT use MmIsAddressValid, as demand-paged
//        user stack pages may not be resident yet and will be faulted in normally. */
//     if (Count > 4)
//     {
//         ULONG64 EndAddr = (ULONG64)&UserParams[Count - 1] + sizeof(ULONG64) - 1;
//         if (EndAddr > MmUserProbeAddress)
//         {
//             TrapFrame->Rax = STATUS_ACCESS_VIOLATION;
//             return (PVOID)NtSyscallFailure;
//         }
//     }
// #endif
//
//     _SEH2_TRY
//     {
//         switch (Count)
//         {
//             case 16: KernelParams[15] = UserParams[15];
//             case 15: KernelParams[14] = UserParams[14];
//             case 14: KernelParams[13] = UserParams[13];
//             case 13: KernelParams[12] = UserParams[12];
//             case 12: KernelParams[11] = UserParams[11];
//             case 11: KernelParams[10] = UserParams[10];
//             case 10: KernelParams[9] = UserParams[9];
//             case 9: KernelParams[8] = UserParams[8];
//             case 8: KernelParams[7] = UserParams[7];
//             case 7: KernelParams[6] = UserParams[6];
//             case 6: KernelParams[5] = UserParams[5];
//             case 5: KernelParams[4] = UserParams[4];
//             case 4:
//             case 3:
//             case 2:
//             case 1:
//             case 0:
//                 break;
//
//             default:
//                 ASSERT(FALSE);
//                 break;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         TrapFrame->Rax = _SEH2_GetExceptionCode();
//         return (PVOID)NtSyscallFailure;
//     }
//     _SEH2_END;
//
//     return (PVOID)DescriptorTable->Base[ServiceNumber];
// }
//
//
// // FIXME: we need to
// VOID
// KiSystemService(IN PKTHREAD Thread,
//                 IN PKTRAP_FRAME TrapFrame,
//                 IN ULONG Instruction)
// {
//     UNIMPLEMENTED;
//     __debugbreak();
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
