use crate::ke::ke_defs::*;
// KeGdiFlushUserBatch: ps-rust global read across the crate boundary as a link symbol
// (#[no_mangle] in ps-rust). crate-split Phase 1b.
extern "C" {
    static mut KeGdiFlushUserBatch: ntkrnl_sys::ddk::PGDI_BATCHFLUSH_ROUTINE;
}
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    MmAccessFault, MmIsAddressValid, MmMapIoSpace,
    _EXCEPTION_RECORD as EXCEPTION_RECORD,
    _KTRAP_FRAME,
    _KTHREAD,
    _KSERVICE_TABLE_DESCRIPTOR,
    KPROCESSOR_MODE,
    PKTHREAD,
    PKTRAP_FRAME,
    PKSERVICE_TABLE_DESCRIPTOR,
    PTEB,
    SCHAR,
    KiDispatchException,
    NtYieldExecution,
};

// NOTE: origin/master's KiCaptureCalleeSaved (a bl-preserving asm helper that
// snapshots x19-x28 from inside the Rust handler) was dropped here in favour of
// the fex-arm EXFRAME_SAVE/EXFRAME_RESTORE pair in trap.S — see the handler
// signature below. EXFRAME_SAVE captures in pure asm before ANY Rust prolog can
// spill a callee-saved reg, and EXFRAME_RESTORE writes back the values
// KiDispatchException modifies (Rd>=19 fixup, NtContinue resume) so they reach
// KiTrapReturn — a discarded local (the origin/master approach) could not.
extern "C" {
    static mut Arm64UartKernelBase: PVOID;
    static mut MmUserProbeAddress: ULONG_PTR;
    /// GICv2 interrupt dispatch — implemented in the HAL (hal-rust
    /// arm64/clock.rs). Acknowledges the GIC, services the periodic
    /// virtual-timer clock tick (→ KeUpdateSystemTime), re-arms + EOIs.
    /// No-op until HalInitializeClockArm64 runs in Phase 1.
    fn HalClockInterruptArm64(TrapFrame: PKTRAP_FRAME);
}

extern "system" {
    /// HAL interrupt-context IRQL pair (hal-rust arm64/halstubs.rs): raise /
    /// restore WITHOUT the KfLowerIrql software-interrupt drain — safe in IRQ
    /// exception context where APC delivery would be fatal.
    fn HalpEnterInterruptIrql(NewIrql: KIRQL) -> KIRQL;
    fn HalpExitInterruptIrql(RestoreIrql: KIRQL);
}
use crate::ke::procobj::KeServiceDescriptorTable;
use crate::ke::dpc::KiRetireDeviceDpcs;

unsafe fn TrapPutChar(c: u8) {
    // FIFO-aware: a physical PL011 drops bytes written faster than it drains,
    // and this is the path that reports faults - a shredded fault dump is worse
    // than useless because it reads like a crash mid-message.
    crate::arm64_dbg::uart_putc(Arm64UartKernelBase as *mut u32, c);
}

unsafe fn TrapPuts(s: &[u8]) {
    for &c in s {
        TrapPutChar(c);
    }
}

unsafe fn TrapPutHex4(nibble: ULONG) {
    let hex_chars = b"0123456789ABCDEF";
    TrapPutChar(hex_chars[(nibble & 0xF) as usize]);
}

unsafe fn TrapPutHex32(val: ULONG) {
    TrapPuts(b"0x");
    for i in (0..=28).rev().step_by(4) {
        TrapPutHex4(val >> i);
    }
}

unsafe fn TrapPutHex64(val: u64) {
    TrapPuts(b"0x");
    for i in (0..=60).rev().step_by(4) {
        TrapPutHex4((val >> i) as ULONG);
    }
}

/// [TRAP-PROBE] Called from KiTrapReturn (trap.S, gated on ARM64_TRAP_DEBUG)
/// just before the `eret`, with x0 = the KTRAP_FRAME being restored. Catches a
/// bad EL0-return state at the point of use: an illegal SPSR (IL/M4/garbage
/// bits) or a non-usermode ELR. This is the KiTrapReturn 0xC000001D illegal-
/// exec-state hunt (task #9). Dumps the frame so the corrupting writer can be
/// correlated by frame address.
#[no_mangle]
pub unsafe extern "C" fn KiTrapReturnProbe(tf: *const _KTRAP_FRAME) {
    if tf.is_null() {
        return;
    }
    let spsr = (*tf).Spsr;
    let pc = (*tf).Pc;
    let el = spsr & 0xF;
    // A KERNEL return (M[3:0] != 0) to a USER-range PC. This used to `return`
    // here, looking only at EL0t frames — so the one frame shape that cannot
    // possibly be right was the one shape the probe declined to inspect. The
    // kernel never executes user memory and AArch64 will not let it (an
    // EL0-writable page is implicitly PXN), so this erets straight into a fault
    // that repeats forever.
    if el != 0 {
        if pc < 0x0000_8000_0000_0000 {
            TrapPuts(b"[TRPROBE] EL1ret to USER pc tf=");
            TrapPutHex64(tf as u64);
            TrapPuts(b" spsr="); TrapPutHex32(spsr);
            TrapPuts(b" pc=");   TrapPutHex64(pc);
            TrapPuts(b" sp=");   TrapPutHex64((*tf).Sp);
            TrapPuts(b" lr=");   TrapPutHex64((*tf).Lr);
            TrapPuts(b"\r\n");
        }
        return;
    }
    // Illegal for an EL0t AArch64 return: M[4] (bit4), IL (bit20), or any of
    // bits[19:10] set (a valid user PSTATE is ~NZCV|DAIF|SS|feature-bits, never
    // a pointer pattern — the captured corruption was 0x0013DC80).
    let spsr_bad = (spsr & 0x0010_0010) != 0 || (spsr & 0x000F_FC00) != 0;
    // ELR (eret target) must be a real user-code VA: non-tiny and canonical
    // low half. pc==0 / pc<0x10000 (null or near-null entry) is a bug too.
    // `<=`, not `<`: the boundary value 0x10000 is exactly the PC the arm64
    // boot wedge kept showing, and a strict less-than passed it as good.
    let pc_bad = pc <= 0x1_0000 || (pc >> 48) != 0;
    if spsr_bad || pc_bad {
        TrapPuts(b"[TRPROBE] EL0ret BAD tf=");
        TrapPutHex64(tf as u64);
        TrapPuts(b" spsr=");
        TrapPutHex32(spsr);
        TrapPuts(b" pc=");
        TrapPutHex64(pc);
        TrapPuts(b" sp=");
        TrapPutHex64((*tf).Sp);
        TrapPuts(b" lr=");
        TrapPutHex64((*tf).Lr);
        TrapPuts(b" x0=");
        TrapPutHex64((*tf).__bindgen_anon_3.X[0]);
        TrapPuts(b"\r\n");
    }
}

unsafe fn KiGetExceptionClassName(EC: ULONG) -> &'static [u8] {
    match EC {
        0x00 => b"Unknown",
        0x01 => b"WFx trap",
        0x0E => b"Illegal Execution",
        0x15 => b"SVC (AArch64)",
        0x18 => b"MSR/MRS trap",
        0x20 => b"Instruction Abort (lower EL)",
        0x21 => b"Instruction Abort (same EL)",
        0x22 => b"PC Alignment",
        0x24 => b"Data Abort (lower EL)",
        0x25 => b"Data Abort (same EL)",
        0x26 => b"SP Alignment",
        0x2C => b"FP Exception",
        0x3C => b"BRK (AArch64)",
        _ => b"Other",
    }
}

pub(super) unsafe fn KiDumpTrapFrame(Tf: PKTRAP_FRAME) {
    let EC = ((*Tf).Esr >> 26) & 0x3F;

    TrapPuts(b"\r\n*** ARM64 EXCEPTION ***\r\n");
    TrapPuts(b"Type: ");
    TrapPuts(KiGetExceptionClassName(EC));
    TrapPuts(b" (EC=");
    TrapPutHex32(EC);
    TrapPuts(b")\r\n");

    TrapPuts(b"PC:   "); TrapPutHex64((*Tf).Pc);   TrapPuts(b"\r\n");
    TrapPuts(b"LR:   "); TrapPutHex64((*Tf).Lr);   TrapPuts(b"\r\n");
    TrapPuts(b"SP:   "); TrapPutHex64((*Tf).Sp);   TrapPuts(b"\r\n");
    TrapPuts(b"FP:   "); TrapPutHex64((*Tf).Fp);   TrapPuts(b"\r\n");
    TrapPuts(b"ESR:  "); TrapPutHex32((*Tf).Esr);  TrapPuts(b"\r\n");
    TrapPuts(b"FAR:  "); TrapPutHex64((*Tf).__bindgen_anon_2.__bindgen_anon_1.FaultAddress); TrapPuts(b"\r\n");
    TrapPuts(b"SPSR: "); TrapPutHex32((*Tf).Spsr); TrapPuts(b"\r\n");

    TrapPuts(b"X0:   "); TrapPutHex64((*Tf).__bindgen_anon_3.X[0]);  TrapPuts(b"  X1:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[1]);  TrapPuts(b"\r\n");
    TrapPuts(b"X2:   "); TrapPutHex64((*Tf).__bindgen_anon_3.X[2]);  TrapPuts(b"  X3:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[3]);  TrapPuts(b"\r\n");
    TrapPuts(b"X4:   "); TrapPutHex64((*Tf).__bindgen_anon_3.X[4]);  TrapPuts(b"  X5:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[5]);  TrapPuts(b"\r\n");
    TrapPuts(b"X6:   "); TrapPutHex64((*Tf).__bindgen_anon_3.X[6]);  TrapPuts(b"  X7:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[7]);  TrapPuts(b"\r\n");
    TrapPuts(b"X8:   "); TrapPutHex64((*Tf).__bindgen_anon_3.X[8]);  TrapPuts(b"  X9:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[9]);  TrapPuts(b"\r\n");
    TrapPuts(b"X10:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[10]); TrapPuts(b"  X11: "); TrapPutHex64((*Tf).__bindgen_anon_3.X[11]); TrapPuts(b"\r\n");
    TrapPuts(b"X12:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[12]); TrapPuts(b"  X13: "); TrapPutHex64((*Tf).__bindgen_anon_3.X[13]); TrapPuts(b"\r\n");
    TrapPuts(b"X14:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[14]); TrapPuts(b"  X15: "); TrapPutHex64((*Tf).__bindgen_anon_3.X[15]); TrapPuts(b"\r\n");
    TrapPuts(b"X16:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[16]); TrapPuts(b"  X17: "); TrapPutHex64((*Tf).__bindgen_anon_3.X[17]); TrapPuts(b"\r\n");
    TrapPuts(b"X18:  "); TrapPutHex64((*Tf).__bindgen_anon_3.X[18]); TrapPuts(b"\r\n");
}

type PSYSCALL_ROUTINE = unsafe extern "system" fn(
    ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
    ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
    ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
    ULONG_PTR, ULONG_PTR, ULONG_PTR) -> ULONG_PTR;

extern "system" {
    fn PsConvertToGuiThread() -> NTSTATUS;
}

unsafe fn KiSvcDispatch(TrapFrame: PKTRAP_FRAME) {
    let ServiceNumber = (*TrapFrame).__bindgen_anon_3.X[8] as ULONG;
    let TableIndex = ((ServiceNumber >> 12) & 1) as usize;
    let ServiceIndex = (ServiceNumber & 0xFFF) as usize;
    let mut Table: PKSERVICE_TABLE_DESCRIPTOR;
    let Routine: PSYSCALL_ROUTINE;
    let SvcResult: ULONG_PTR;
    let GuiConvertStatus: NTSTATUS;
    let OldMode: KPROCESSOR_MODE;
    let Thread: PKTHREAD;
    let OldTrapFrame: PKTRAP_FRAME;
    let mut Args: [ULONG_PTR; 15] = [0; 15];
    let ArgCount: ULONG;

    Thread = KeGetCurrentThread() as PKTHREAD;
    Table = (*Thread).ServiceTable as PKSERVICE_TABLE_DESCRIPTOR;
    if Table.is_null() {
        Table = core::ptr::addr_of_mut!(KeServiceDescriptorTable) as PKSERVICE_TABLE_DESCRIPTOR;
    }
    Table = Table.add(TableIndex);

    if ServiceIndex >= (*Table).Limit as usize {
        if TableIndex == 1 &&
           (*Thread).ServiceTable as PVOID == core::ptr::addr_of_mut!(KeServiceDescriptorTable) as PVOID
        {
            GuiConvertStatus = PsConvertToGuiThread();
            if GuiConvertStatus >= 0 {
                Table = (*Thread).ServiceTable as PKSERVICE_TABLE_DESCRIPTOR;
                Table = Table.add(TableIndex);
                if ServiceIndex < (*Table).Limit as usize {
                    // goto DispatchCall
                } else {
                    (*TrapFrame).__bindgen_anon_3.X[0] = STATUS_INVALID_SYSTEM_SERVICE as u64;
                    return;
                }
            } else {
                TrapPuts(b"[SVC] GUI conversion failed: ");
                TrapPutHex32(GuiConvertStatus as ULONG);
                TrapPuts(b"\r\n");
                (*TrapFrame).__bindgen_anon_3.X[0] = STATUS_INVALID_SYSTEM_SERVICE as u64;
                return;
            }
        } else {
            (*TrapFrame).__bindgen_anon_3.X[0] = STATUS_INVALID_SYSTEM_SERVICE as u64;
            return;
        }
    }

    // DispatchCall:
    if TableIndex == 1 && ((*TrapFrame).Spsr & 0xF) == 0 {
        let Teb = (*Thread).Teb;
        if !Teb.is_null() && MmIsAddressValid(core::ptr::addr_of!((*Teb).GdiBatchCount) as PVOID) != 0 &&
           (*Teb).GdiBatchCount != 0
        {
            if let Some(GdiFlush) = KeGdiFlushUserBatch {
                GdiFlush();
            }
        }
    }

    Routine = core::ptr::read(((*Table).Base as *const PSYSCALL_ROUTINE).add(ServiceIndex));
    if Routine as PVOID == core::ptr::null_mut() {
        (*TrapFrame).__bindgen_anon_3.X[0] = STATUS_INVALID_SYSTEM_SERVICE as u64;
        return;
    }

    for i in 0..8.min(15) {
        Args[i] = (*TrapFrame).__bindgen_anon_3.X[i] as ULONG_PTR;
    }

    ArgCount = (*(*Table).Number.add(ServiceIndex)) as ULONG / core::mem::size_of::<ULONG_PTR>() as ULONG;
    if ArgCount > 8 {
        let CallerStack = (*TrapFrame).Sp as *const ULONG_PTR;
        let mut StackArgs = ArgCount - 8;
        if StackArgs > 7 {
            StackArgs = 7;
        }
        for i in 0..StackArgs as usize {
            Args[8 + i] = *CallerStack.add(i);
        }
    }

    OldMode = (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode;
    if ((*TrapFrame).Spsr & 0xF) == 0 {
        (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = 1i8; // UserMode
    } else {
        (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = 0i8; // KernelMode
    }

    OldTrapFrame = (*Thread).TrapFrame;
    (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.TrapFrame = OldTrapFrame as u64;
    (*Thread).TrapFrame = TrapFrame;

    SvcResult = Routine(
        Args[0], Args[1], Args[2], Args[3],
        Args[4], Args[5], Args[6], Args[7],
        Args[8], Args[9], Args[10], Args[11],
        Args[12], Args[13], Args[14]
    );

    (*Thread).TrapFrame = OldTrapFrame;
    (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = OldMode;
    (*TrapFrame).__bindgen_anon_3.X[0] = SvcResult as u64;

    if ((*TrapFrame).Spsr & 0xF) == 0 {
        let Prcb = KeGetCurrentPrcb();
        if (*Prcb).ReadySummary != 0 {
            let mut HighBit: ULONG = 0;
            #[cfg(target_arch = "aarch64")]
            {
                HighBit = 31 - ((*Prcb).ReadySummary.leading_zeros() as ULONG);
            }
            if (HighBit as SCHAR) > (*Thread).Priority {
                NtYieldExecution();
            }
        }
    }
}

extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn KfLowerIrql(NewIrql: KIRQL);
}

/// Deliver a pending user-mode APC on the return-to-EL0 path.
///
/// The arm64 SVC exit (`KiSvcDispatch` → `KiTrapReturn` → `eret`) had NO
/// user-APC delivery step — unlike amd64's `KiSystemServiceExit` (trap.S:911),
/// which checks `Thread->ApcState.UserApcPending` and calls `KiInitiateUserApc`
/// before returning to user mode (and i386's equivalent in its trap epilog).
/// Without it a queued user APC is never delivered: a `UserMode`
/// `KeWaitForSingleObject` keeps returning STATUS_USER_APC because
/// `UserApcPending` is never cleared, and the thread never runs the APC
/// routine. (Root cause of the kmtest usermode-callback worker wedge —
/// `IOCTL_KMTEST_USERMODE_AWAIT_REQ` returned 0xC0 persistently — which was
/// the last blocker on a full arm64 conformance baseline.)
///
/// Mirrors amd64 `KiInitiateUserApc`: raise to APC_LEVEL, run
/// `KiDeliverApc(UserMode, ExceptionFrame, TrapFrame)` — which dequeues the
/// user APC, runs its KernelRoutine, clears `UserApcPending`, and (via
/// `KiInitializeUserApc`) rewrites the trap frame (Pc → KeUserApcDispatcher,
/// Sp → user stack holding the saved CONTEXT) so the trailing `eret` lands in
/// the dispatcher — then lower back to the entry IRQL.
///
/// We deliberately do NOT toggle PSTATE.DAIF (unlike amd64's sti/cli around the
/// call): `KiTrapReturn` owns the ELR/SPSR per-exception banking window, and the
/// APC's user stack is the thread's own resident stack (KiInitializeUserApc
/// pre-faults it anyway). Since `UNMASK_IRQ_FROM_SPSR` (trap.S) the SVC body
/// runs with IRQs ON, which is the amd64 state at this point too.
/// IRQL here is a GIC-PMR / PCR concern, not a DAIF one — `KfRaiseIrql` is the
/// correct, DAIF-neutral gate. This is SVC (synchronous, thread-body) context,
/// the canonical safe point for user-APC delivery — NOT the IRQ-exception
/// context the `KiIrqHandler` DAIF warning is about.
#[inline]
unsafe fn KiDeliverUserApcArm64(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: ntkrnl_sys::ddk::PKEXCEPTION_FRAME,
) {
    // Only a return to EL0 (user) with a pending user APC needs this.
    if ((*TrapFrame).Spsr & 0xF) != 0 {
        return;
    }
    let Thread = KeGetCurrentThread() as PKTHREAD;
    if kt_ApcState(Thread).UserApcPending == 0 {
        return;
    }

    let OldIrql = KfRaiseIrql(APC_LEVEL);
    ntkrnl_sys::ddk::KiDeliverApc(1i8 /* UserMode */, ExceptionFrame, TrapFrame);
    KfLowerIrql(OldIrql);
}

/// How many times in a row the pager may claim it resolved the SAME fault
/// before we stop believing it. A real re-fault after a genuine fix-up is one
/// or two (copy-on-write, a racing trim); dozens with the address, the faulting
/// PC and the address space all unchanged is not a retry, it is a loop.
const KI_FAULT_RETRY_REPORT: u32 = 64;

/// Where "retrying" stops being a description and becomes a proof. Measured,
/// not guessed: a clean arm64 boot ALREADY drives the same (VA, PC, address
/// space) past 64 consecutive pager-reported successes and then completes, so
/// 64 is a fine place to start printing and a catastrophic place to start
/// killing — a first cut at 64 fatal murdered a services.exe thread on every
/// boot with an access violation that had never happened before.
///
/// A genuine unbreakable loop reaches this in a few seconds of emulated time,
/// because each turn is one exception round trip and nothing else runs; the
/// boot budget is 180s, so a true hang still crashes long inside it. Anything
/// that legitimately re-faults keeps working.
const KI_FAULT_RETRY_FATAL: u32 = 1_000_000;

/// Per-CPU last-fault fingerprint: (fault VA, faulting PC, TTBR0, count).
/// TTBR0 rather than the thread pointer because the question this answers is
/// "same translation, same instruction" — the address space is what decides
/// which page table the walker consults. Indexed by MPIDR Aff0; a diagnostic,
/// so a torn read on a foreign CPU's slot can at worst delay the report.
static mut KiFaultRetryState: [(u64, u64, u64, u32); 8] = [(0, 0, 0, 0); 8];

#[inline(always)]
unsafe fn ki_cpu_slot() -> usize {
    let mpidr: u64;
    core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nomem, nostack));
    (mpidr & 0x7) as usize
}

/// Forget the fingerprint. Called whenever a fault does NOT end in the pager
/// reporting success, because that breaks any loop in progress.
unsafe fn ki_fault_retry_forget() {
    KiFaultRetryState[ki_cpu_slot()] = (0, 0, 0, 0);
}

/// True when the pager has now claimed success on the identical fault
/// `KI_FAULT_RETRY_LIMIT` times running — i.e. we are in an unbreakable
/// abort/handler/eret loop and returning again would hang the machine silently.
///
/// This is the instrument the arm64 boot wedge needed and nobody had: the loop
/// runs with DAIF masked, so no timer ever fires, no watchdog can observe it,
/// and the only evidence is a vCPU whose PC never moves. Seven captures over
/// two 400-round soaks said "FROZEN" and none of them could say WHY, because
/// the box cannot report on itself once it is in here.
unsafe fn ki_fault_retry_tripped(TrapFrame: PKTRAP_FRAME, DFSC: ULONG) -> bool {
    let va = (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress as u64;
    let pc = (*TrapFrame).Pc;
    let ttbr0: u64;
    core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0, options(nomem, nostack));

    let slot = ki_cpu_slot();
    let st = KiFaultRetryState[slot];
    let n = if st.0 == va && st.1 == pc && st.2 == ttbr0 { st.3 + 1 } else { 1 };
    KiFaultRetryState[slot] = (va, pc, ttbr0, n);
    // Report once, on the way past the reporting mark, and keep going. Printing
    // every turn would itself hang the box on the UART.
    if n != KI_FAULT_RETRY_REPORT && n != KI_FAULT_RETRY_FATAL {
        return false;
    }
    let fatal = n >= KI_FAULT_RETRY_FATAL;
    if fatal {
        KiFaultRetryState[slot] = (0, 0, 0, 0);
    }

    // Dump the contradiction itself: what the HARDWARE said (ESR/DFSC) beside
    // what SOFTWARE reads for the same VA out of the self-map. If the PTE reads
    // valid while DFSC says translation fault, the two are looking at different
    // copies or at different tables, and TTBR0 says which address space the
    // walker was actually using.
    let ttbr1: u64;
    core::arch::asm!("mrs {}, ttbr1_el1", out(reg) ttbr1, options(nomem, nostack));
    TrapPuts(if fatal {
        b"\r\n*** FAULT LOOP (FATAL): the pager reported success on the same fault ".as_slice()
    } else {
        b"\r\n[FAULTLOOP] note: the pager has reported success on the same fault ".as_slice()
    });
    TrapPutHex32(n);
    TrapPuts(b" times\r\n");
    TrapPuts(b"[FAULTLOOP] VA=");    TrapPutHex64(va);
    TrapPuts(b" PC=");               TrapPutHex64(pc);
    TrapPuts(b" ESR=");              TrapPutHex32((*TrapFrame).Esr);
    TrapPuts(b" DFSC=");             TrapPutHex32(DFSC);
    TrapPuts(b"\r\n[FAULTLOOP] TTBR0="); TrapPutHex64(ttbr0);
    TrapPuts(b" TTBR1=");            TrapPutHex64(ttbr1);
    // Same self-map formula the DFSC-0x21 diagnostic above uses. Reading it can
    // itself fault if the chain is broken; that fault is not this one, and it
    // would be a far louder failure than the silence we are replacing.
    let pte_va = 0xFFFFF68000000000u64
        .wrapping_add(((va >> 12) & 0xF_FFFF_FFFFu64) << 3);
    let pte = core::ptr::read_volatile(pte_va as *const u64);
    TrapPuts(b" PTE@");              TrapPutHex64(pte_va);
    TrapPuts(b"=");                  TrapPutHex64(pte);
    TrapPuts(b"\r\n");
    fatal
}

#[no_mangle]
pub unsafe extern "C" fn KiSynchronousExceptionHandler(
    TrapFrame: PKTRAP_FRAME,
    /* Callee-saved x19..x28 captured by EXFRAME_SAVE in trap.S; passed through
     * to KiDispatchException so KeTrapFrameToContext/KeContextToTrapFrame can
     * round-trip the full integer state (NtContinue resume, Rd>=19 fixups).
     * (This supersedes origin/master's KiCaptureCalleeSaved-in-Rust capture:
     * EXFRAME_SAVE snapshots in asm before any Rust prolog runs AND EXFRAME_RESTORE
     * writes modified regs back for the resume path — a discarded local can't.) */
    ExceptionFrame: ntkrnl_sys::ddk::PKEXCEPTION_FRAME,
) {
    let EC = ((*TrapFrame).Esr >> 26) & 0x3F;
    let ISS = (*TrapFrame).Esr & 0x1FFFFFF;

    match EC {
        0x25 | 0x24 => {
            let DFSC = ISS & 0x3F;
            let IsWrite = ((ISS >> 6) & 1) != 0;
            let FaultMode: KPROCESSOR_MODE;
            let mut FaultCode: ULONG;

            FaultMode = if (*TrapFrame).__bindgen_anon_1.__bindgen_anon_1.PreviousMode == 1 { 1i8 } else { 0i8 };

            FaultCode = 0;
            if (DFSC & 0x3C) == 0x0C {
                FaultCode |= 0x1;
            }
            if IsWrite {
                FaultCode |= 0x2;
            }

            // Alignment-fault diagnostic (DFSC 0x21): discriminates "page is
            // Device memory" (PTE AttrIndx bits [3:2] != 0) from "instruction
            // demands alignment on Normal memory" (LDAR/STLR/LDAPR — what FEX
            // emits for x86-TSO). Alignment fault ⇒ translation succeeded, so
            // the self-map PTE chain for FAR is walkable.
            if DFSC == 0x21 {
                let far = (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress;
                let pc = (*TrapFrame).Pc;
                TrapPuts(b"[ALIGN] FAR="); TrapPutHex64(far);
                TrapPuts(b" PC="); TrapPutHex64(pc);
                let pte_va = 0xFFFFF68000000000u64
                    .wrapping_add(((far as u64 >> 12) & 0xF_FFFF_FFFFu64) << 3);
                let pte = core::ptr::read_volatile(pte_va as *const u64);
                TrapPuts(b" PTE="); TrapPutHex64(pte);
                let mair: u64;
                core::arch::asm!("mrs {}, mair_el1", out(reg) mair, options(nomem, nostack));
                TrapPuts(b" MAIR="); TrapPutHex64(mair);
                let opc = core::ptr::read_volatile(pc as *const u32);
                TrapPuts(b" OPC="); TrapPutHex32(opc);
                TrapPuts(b"\r\n");
            }

            if (DFSC & 0x3C) == 0x04 ||
               (DFSC & 0x3C) == 0x08 ||
               (DFSC & 0x3C) == 0x0C
            {
                let Status = MmAccessFault(
                    FaultCode,
                    (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress as PVOID,
                    FaultMode,
                    TrapFrame as *mut core::ffi::c_void
                );
                if Status >= 0 {
                    // "Resolved" — but only believe it while it stays true. A
                    // pager that keeps succeeding on a fault that keeps
                    // recurring is not fixing anything, and returning to the
                    // faulting instruction one more time is what turns a bug
                    // into a silent whole-machine freeze.
                    if !ki_fault_retry_tripped(TrapFrame, DFSC) {
                        return;
                    }
                    // Fall through to the dispatch below: an unhandled kernel
                    // exception bugchecks with the frame, the stack bounds and
                    // the [KSTKW] stack words. A readable crash beats a hang,
                    // and this deliberately mints no new bugcheck code.
                } else {
                    ki_fault_retry_forget();
                }
            } else {
                ki_fault_retry_forget();
            }

            let mut ExRec: EXCEPTION_RECORD = core::mem::zeroed();
            // DFSC 0x21 = alignment fault: Windows semantics deliver this as
            // EXCEPTION_DATATYPE_MISALIGNMENT, not an access violation. FEX's
            // BTCpuResetToConsistentState keys on exactly this code to
            // backpatch unaligned LDAR/STLR (x86-TSO) and resume.
            ExRec.ExceptionCode = if DFSC == 0x21 {
                0x80000002u32 as i32 // STATUS_DATATYPE_MISALIGNMENT
            } else {
                STATUS_ACCESS_VIOLATION
            };
            ExRec.ExceptionFlags = 0;
            ExRec.ExceptionRecord = core::ptr::null_mut();
            ExRec.ExceptionAddress = (*TrapFrame).Pc as PVOID;
            ExRec.NumberParameters = 2;
            ExRec.ExceptionInformation[0] = if IsWrite { 1 } else { 0 };
            ExRec.ExceptionInformation[1] = (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress;
            KiDispatchException(
                &mut ExRec as *mut EXCEPTION_RECORD,
                ExceptionFrame,
                TrapFrame,
                FaultMode,
                1u8
            );
        }

        0x21 | 0x20 => {
            let IFSC = ISS & 0x3F;
            let FaultMode2: KPROCESSOR_MODE;
            let mut FaultCode2: ULONG;

            FaultMode2 = if (*TrapFrame).__bindgen_anon_1.__bindgen_anon_1.PreviousMode == 1 { 1i8 } else { 0i8 };

            FaultCode2 = 0x10;
            if (IFSC & 0x3C) == 0x0C {
                FaultCode2 |= 0x1;
            }

            // A SAME-EL instruction abort (EC 0x21) on a user-range VA, or on a
            // permission fault, is never something paging can fix, and asking
            // it to try produces a silent, unbreakable loop:
            //
            //   MmArmAccessFault's valid-PTE path bugchecks an instruction fetch
            //   only when the PTE says NOT executable (UXN, bit 54). A user page
            //   that IS user-executable passes that test and it returns
            //   STATUS_SUCCESS — "spurious, retry". But AArch64 makes every
            //   EL0-writable page implicitly PXN, so EL1 can never execute it: the
            //   retry faults at the same PC, forever, at EL1, with nothing printed
            //   and every timed wait dead. That is what the ~2% arm64 boot wedge
            //   was (soak 2026-09-05: 5/300 rounds, every vCPU sample at
            //   PC=0x10000, PSTATE EL1h, the odd one out caught in KiTrapReturn).
            //   The kernel never legitimately executes user memory, and an
            //   instruction-fetch permission fault has no paging remedy at any
            //   address (that is PXN/UXN, not a paged-out page), so both go
            //   straight to the exception path, where an unhandled kernel fault
            //   bugchecks with the frame and the stack — a crash we can read,
            //   instead of a hang we cannot.
            let FaultVa = (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress as ULONG_PTR;
            let NeverRetriable = EC == 0x21
                && (FaultVa < MmUserProbeAddress || (IFSC & 0x3C) == 0x0C);

            if !NeverRetriable && (
               (IFSC & 0x3C) == 0x04 ||
               (IFSC & 0x3C) == 0x08 ||
               (IFSC & 0x3C) == 0x0C)
            {
                let Status2 = MmAccessFault(
                    FaultCode2,
                    (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress as PVOID,
                    FaultMode2,
                    TrapFrame as *mut core::ffi::c_void
                );
                if Status2 >= 0 {
                    return;
                }
            }

            let mut ExRec: EXCEPTION_RECORD = core::mem::zeroed();
            ExRec.ExceptionCode = STATUS_ACCESS_VIOLATION;
            ExRec.ExceptionFlags = 0;
            ExRec.ExceptionRecord = core::ptr::null_mut();
            ExRec.ExceptionAddress = (*TrapFrame).Pc as PVOID;
            ExRec.NumberParameters = 2;
            ExRec.ExceptionInformation[0] = 8;
            ExRec.ExceptionInformation[1] = (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress;
            KiDispatchException(
                &mut ExRec as *mut EXCEPTION_RECORD,
                ExceptionFrame,
                TrapFrame,
                FaultMode2,
                1u8
            );
        }

        0x3C => {
            let BrkImm = ISS & 0xFFFF;
            let BrkFromKernel =
                (*TrapFrame).__bindgen_anon_1.__bindgen_anon_1.PreviousMode != 1;

            // brk #0xF000 from kernel mode is DbgBreakPoint/DbgBreakPointWithStatus
            // (DebugService moved to #0xF002, Windows ARM64's service immediate).
            // Dispatch it as STATUS_BREAKPOINT like x86 does an int3, so a kernel
            // __except can see it; KiDispatchException steps past an unhandled one
            // and continues, the amd64 policy. It used to be skipped here outright,
            // so no kernel handler ever saw a breakpoint on arm64. User-mode brk is
            // unchanged (still skipped) for now.
            if BrkImm == 0xF000 && BrkFromKernel {
                let mut ExRec: EXCEPTION_RECORD = core::mem::zeroed();
                ExRec.ExceptionCode = STATUS_BREAKPOINT;
                ExRec.ExceptionFlags = 0;
                ExRec.ExceptionRecord = core::ptr::null_mut();
                ExRec.ExceptionAddress = (*TrapFrame).Pc as PVOID;
                ExRec.NumberParameters = 1;
                ExRec.ExceptionInformation[0] = 0; // BREAKPOINT_BREAK
                KiDispatchException(
                    &mut ExRec as *mut EXCEPTION_RECORD,
                    ExceptionFrame,
                    TrapFrame,
                    0i8,
                    1u8,
                );
                return;
            }

            // Debug service: #0xF002 (DebugService), and #0xF000 from user mode for
            // anything built before the split.
            if (BrkImm == 0xF002 || BrkImm == 0xF000) && (*TrapFrame).__bindgen_anon_3.X[0] == 1 {
                let Buf = (*TrapFrame).__bindgen_anon_3.X[1] as *const u8;
                let mut Len = (*TrapFrame).__bindgen_anon_3.X[2] as ULONG;

                if Len > 512 {
                    Len = 512;
                }
                // Copy first: faulting a user buffer in can block, which must
                // not happen while holding the output lock below.
                let mut Line = [0u8; 512];
                let mut Got = 0usize;
                for i in 0..Len {
                    if MmIsAddressValid(Buf.add(i as usize) as PVOID) == 0 {
                        if (Buf.add(i as usize) as ULONG_PTR) < MmUserProbeAddress {
                            MmAccessFault(0, Buf.add(i as usize) as PVOID, 0, core::ptr::null_mut());
                        }
                        if MmIsAddressValid(Buf.add(i as usize) as PVOID) == 0 {
                            break;
                        }
                    }
                    Line[Got] = *Buf.add(i as usize);
                    Got += 1;
                }
                // One message reaches the UART whole (see uart_line_lock):
                // without it every CPU's prints shredded each other byte by
                // byte, which is how the smoke's per-CPU markers went missing.
                let daif = crate::arm64_dbg::uart_line_lock();
                for &c in &Line[..Got] {
                    if c == b'\n' {
                        crate::arm64_dbg::uart_putc_default(b'\r' as u8);
                    }
                    crate::arm64_dbg::uart_putc_default(c as u8);
                }
                crate::arm64_dbg::uart_line_unlock(daif);
            }

            (*TrapFrame).Pc += 4;
        }

        0x15 => {
            // cswitchdep taps 4/5 (user↔kernel syscall parity). Position
            // guarantees kernel stack + built trap frame (same contract as
            // amd64 KiSystemCallHandler / i386 KiSystemServiceHandler).
            // Kernel-origin SVC (Zw from EL1, Spsr.M != EL0) passes
            // user_trap=0 and is skipped — the i386 INT-2E Zw rule.
            #[cfg(feature = "cswitchdep")]
            let user_svc = ((*TrapFrame).Spsr & 0xF) == 0;
            #[cfg(feature = "cswitchdep")]
            crate::ke::cswitchdep::VibeosCswitchdepUserEntry(
                user_svc as u32,
                crate::code_addr!(),
            );

            KiSvcDispatch(TrapFrame);

            // Tap 5 pairs with tap 4 around the dispatch rather than at the
            // ERET tail: this exception path returns through EXFRAME_RESTORE
            // for user and kernel frames alike, so the user-bound branch
            // only exists here in Rust. Untapped user exits (exception
            // frames, APC-edited resumes) stay exit_no_entry orphans, as on
            // x86.
            #[cfg(feature = "cswitchdep")]
            if user_svc {
                crate::ke::cswitchdep::VibeosCswitchdepUserExit(crate::code_addr!());
            }

            // Deliver any pending user-mode APC before the eret back to EL0
            // (amd64 KiSystemServiceExit parity — see KiDeliverUserApcArm64).
            // No-op unless we're returning to user with UserApcPending set.
            KiDeliverUserApcArm64(TrapFrame, ExceptionFrame);
        }

        0x26 => {
            let FaultMode4: KPROCESSOR_MODE;

            FaultMode4 = if (*TrapFrame).__bindgen_anon_1.__bindgen_anon_1.PreviousMode == 1 { 1i8 } else { 0i8 };

            let mut ExRec: EXCEPTION_RECORD = core::mem::zeroed();
            ExRec.ExceptionCode = STATUS_DATATYPE_MISALIGNMENT;
            ExRec.ExceptionFlags = 0;
            ExRec.ExceptionRecord = core::ptr::null_mut();
            ExRec.ExceptionAddress = (*TrapFrame).Pc as PVOID;
            ExRec.NumberParameters = 0;
            KiDispatchException(
                &mut ExRec as *mut EXCEPTION_RECORD,
                ExceptionFrame,
                TrapFrame,
                FaultMode4,
                1u8
            );
        }

        _ => {
            let FaultMode3: KPROCESSOR_MODE;

            FaultMode3 = if (*TrapFrame).__bindgen_anon_1.__bindgen_anon_1.PreviousMode == 1 { 1i8 } else { 0i8 };

            let mut ExRec: EXCEPTION_RECORD = core::mem::zeroed();
            ExRec.ExceptionCode = STATUS_ILLEGAL_INSTRUCTION;
            ExRec.ExceptionFlags = 0;
            ExRec.ExceptionRecord = core::ptr::null_mut();
            ExRec.ExceptionAddress = (*TrapFrame).Pc as PVOID;
            ExRec.NumberParameters = 0;
            KiDispatchException(
                &mut ExRec as *mut EXCEPTION_RECORD,
                ExceptionFrame,
                TrapFrame,
                FaultMode3,
                1u8
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn KiIrqHandler(TrapFrame: PKTRAP_FRAME) {
    /* Dispatch the GIC interrupt: clock tick → KeUpdateSystemTime, device SPI
     * → connected ISR at DIRQL (KiDispatchInterruptArm64), then EOI.
     * (No per-IRQ UART trace — the clock fires at 100Hz and would flood.) */
    HalClockInterruptArm64(TrapFrame);

    /* Retire pending DPC work before returning, iff we interrupted code below
     * DISPATCH_LEVEL — the software-interrupt delivery an ISR's DPC queueing
     * would trigger on x86 (KiEndInterrupt). Without this, a DPC queued while
     * a busy thread runs waits for the idle loop, and interrupt-driven I/O
     * completions stall behind compute. Guards:
     *   - IRQL > DISPATCH, or non-idle DISPATCH code: the interrupted context
     *     may hold DISPATCH-level spinlocks — retiring here can re-enter a
     *     lock the interrupted code owns. The idle thread is the safe DISPATCH
     *     exception: it is pinned at DISPATCH specifically to retire DPCs, and
     *     an IRQ can arrive while it is in the open-IRQ spin window before its
     *     next poll observes the queued DPC.
     *   - DpcRoutineActive: a nested IRQ during a DPC must not recurse.
     * KiRetireDeviceDpcs expects entry at DISPATCH with IRQs masked (we're in
     * the IRQ exception, so PSTATE.I is set). On arm64 this IRQ-exit path keeps
     * IRQs masked while calling each plain device DPC; opening PSTATE.I before
     * the DPC body lets a level-triggered controller line immediately re-enter
     * the ISR path and starve the DPC that is supposed to quiesce it.
     *
     * The raise/restore MUST be the interrupt-context pair (HalpEnter/
     * HalpExitInterruptIrql), NOT KeRaise/KeLowerIrql: KfLowerIrql drains
     * pending software interrupts, i.e. delivers kernel APCs — and delivering
     * an APC from IRQ exception context wedges the machine (first hit: cmd.exe's
     * initial thread's startup APC delivered out of its bootstrap path during
     * boot — 2026-07-05). Pending APC bits stay set and drain at the thread's
     * next natural KfLowerIrql. */
    /* Scope: plain (device) DPCs ONLY — KiRetireDeviceDpcs, never the full
     * KiRetireDpcList. The full retire runs KiTimerExpiration when
     * TimerRequest is set, i.e. the thread-WAKE machinery, from exception
     * context — and that reliably wedges boot (field manual §9.19):
     * wake/swap chains rooted in a DAIF-masked exception context interact
     * with this port's SVC-masked waits and thread-lifecycle paths in more
     * ways than the KiThreadStartup daifclr fix covers (empirical:
     * winlogon→cmd handoff wedge on windows-host TCG; LpcExitThread wedge on
     * native). Timer expiration stays with the idle loop exactly as before
     * this quest. Re-attempting exception-context timer retire is gated on
     * making SVC entry re-enable IRQs (quest doc, landmine #2).
     *
     * Deliberately NOT gated on TimerRequest == 0: TimerRequest's only
     * consumer is the idle loop, and a busy thread that never blocks starves
     * idle indefinitely — TimerRequest then sits non-zero forever and a
     * TimerRequest gate here turns that into a dead device-DPC pipeline
     * (2026-07-06: exactly how virtio-input's ISR-queued DPCs never ran at
     * desktop). Device-DPC delivery must not depend on timer bookkeeping. */
    let Prcb = KeGetCurrentPrcb();
    let CurrentIrql = KeGetCurrentIrql();
    let DpcQueueDepth = (*Prcb).DpcData[0].DpcQueueDepth;
    let CanRetireDeviceDpcs = CurrentIrql < DISPATCH_LEVEL
        || (CurrentIrql == DISPATCH_LEVEL && (*Prcb).CurrentThread == (*Prcb).IdleThread);

    if CanRetireDeviceDpcs
        && (*Prcb).DpcRoutineActive == 0
        && DpcQueueDepth != 0
    {
        let OldIrql = HalpEnterInterruptIrql(DISPATCH_LEVEL);
        KI_IN_IRQ_EXCEPTION_RETIRE.store(true, core::sync::atomic::Ordering::SeqCst);
        KiRetireDeviceDpcs(Prcb);
        KI_IN_IRQ_EXCEPTION_RETIRE.store(false, core::sync::atomic::Ordering::SeqCst);
        HalpExitInterruptIrql(OldIrql);
    }
}

/// Preemption point for an IRQ that interrupted EL0, run by `KiEL0IrqEntry` after
/// `KiIrqHandler` and before the user's NEON state is restored.
///
/// arm64's only other preemption point is `KfLowerIrql` crossing below DISPATCH in
/// thread context, which a thread spinning in user mode never reaches: the clock tick
/// set `QuantumEnd` (or a timer readied a thread into `NextThread`) and nothing ever
/// acted on it. On one CPU that is a livelock, not just unfairness. ntdll_vista's NT
/// threadpool worker woke from a timed condvar wait and spun for the condvar's internal
/// lock, which the thread it had preempted held, and the whole guest stopped
/// (the `tpprobe` payload, 2026-09-30). x86 switches here via the DPC interrupt that the
/// IRQ's own lowering delivers.
///
/// This is the one IRQ exit where switching is safe. The interrupted context is user
/// mode, so PASSIVE with no kernel locks held. The handler has EOI'd and dropped its
/// interrupt-context depth, so the HAL treats the raise/lower as thread context. The
/// user's ELR/SPSR are in the trap frame, and `KiTrapReturn` re-masks before reloading
/// them. Unmasking first is what keeps this out of the DAIF-masked switch hazard, which
/// `KiDispatchPreemptArm64` refuses anyway: the thread is parked unmasked, as an SVC
/// that blocks is since `UNMASK_IRQ_FROM_SPSR`. The raise/lower then does exactly what
/// the thread would have done had it lowered IRQL itself: it drains the pending
/// DISPATCH work and then takes the quantum end or the readied thread.
#[no_mangle]
pub unsafe extern "C" fn KiEL0IrqPreempt() {
    let Prcb = KeGetCurrentPrcb();
    if (*Prcb).QuantumEnd == 0 && (*Prcb).NextThread.is_null() {
        return;
    }
    if KeGetCurrentIrql() != PASSIVE_LEVEL {
        return;
    }
    core::arch::asm!("msr daifclr, #2", options(nomem, nostack));
    let OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
    KfLowerIrql(OldIrql);
    core::arch::asm!("msr daifset, #2", options(nomem, nostack));
}

/// True while KiIrqHandler is retiring DPCs in IRQ exception context. A
/// context switch in that window is FATAL (the switched-to thread inherits
/// exception-entry DAIF, masking interrupts forever) — KiSwapContextResume
/// self-reports it with the culprit visible on serial. Diagnostic tooling for
/// the interrupt quest; keep (the check is two loads on the switch path).
pub static KI_IN_IRQ_EXCEPTION_RETIRE: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

#[no_mangle]
pub unsafe extern "C" fn KiSErrorHandler(TrapFrame: PKTRAP_FRAME) {
    TrapPuts(b"[SError] FATAL\r\n");
    KiDumpTrapFrame(TrapFrame);
    KeBugCheckEx(
        NMI_HARDWARE_FAILURE,
        (*TrapFrame).Esr as ULONG_PTR,
        (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress as ULONG_PTR,
        (*TrapFrame).Pc as ULONG_PTR,
        0
    );
}

#[no_mangle]
pub unsafe extern "C" fn KiUnexpectedHandler(TrapFrame: PKTRAP_FRAME) {
    TrapPuts(b"[UNEXPECTED EXCEPTION]\r\n");
    KiDumpTrapFrame(TrapFrame);
    KeBugCheckEx(
        UNEXPECTED_KERNEL_MODE_TRAP,
        (*TrapFrame).Esr as ULONG_PTR,
        (*TrapFrame).__bindgen_anon_2.__bindgen_anon_1.FaultAddress as ULONG_PTR,
        (*TrapFrame).Pc as ULONG_PTR,
        0
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL-2.0-or-later
//  * FILE:            ntoskrnl/ke/arm64/trap.c
//  * PURPOSE:         ARM64 Exception Handlers (Synchronous, IRQ, SError)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Verify KTRAP_FRAME offsets match assembly constants in trap.S */
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, ExceptionActive) == 0x00,  "TfExceptionActive");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, PreviousMode)    == 0x03,  "TfPreviousMode");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, PreviousIrql)    == 0x04,  "TfPreviousIrql");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, FaultAddress)    == 0x10,  "TfFaultAddress");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, Spsr)            == 0xA0,  "TfSpsr");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, Esr)             == 0xA4,  "TfEsr");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, Sp)              == 0xA8,  "TfSp");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, X0)              == 0xB0,  "TfX0");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, X18)             == 0x140, "TfX18");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, Lr)              == 0x148, "TfLr");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, Fp)              == 0x150, "TfFp");
// _Static_assert(__builtin_offsetof(KTRAP_FRAME, Pc)              == 0x158, "TfPc");
// _Static_assert(sizeof(KTRAP_FRAME)                              == 0x160, "KTRAP_FRAME_LENGTH");
//
// /* UART HELPERS **************************************************************/
//
// /* PL011 UART — use kernel VA from global (identity-mapped during early boot,
//  * remapped to kernel VA in Phase 1 via MmMapIoSpace) */
// extern volatile PVOID Arm64UartKernelBase;
// #define PL011_BASE ((volatile ULONG *)Arm64UartKernelBase)
//
// static void TrapPutChar(char c)
// {
//     *PL011_BASE = (ULONG)c;
// }
//
// static void TrapPuts(const char *s)
// {
//     while (*s)
//         TrapPutChar(*s++);
// }
//
// static void TrapPutHex4(ULONG nibble)
// {
//     TrapPutChar("0123456789ABCDEF"[nibble & 0xF]);
// }
//
// static void TrapPutHex32(ULONG val)
// {
//     int i;
//     TrapPuts("0x");
//     for (i = 28; i >= 0; i -= 4)
//         TrapPutHex4(val >> i);
// }
//
// static void TrapPutHex64(ULONG64 val)
// {
//     int i;
//     TrapPuts("0x");
//     for (i = 60; i >= 0; i -= 4)
//         TrapPutHex4((ULONG)(val >> i));
// }
//
// /* ESR exception class names */
// static const char *KiGetExceptionClassName(ULONG EC)
// {
//     switch (EC)
//     {
//         case 0x00: return "Unknown";
//         case 0x01: return "WFx trap";
//         case 0x0E: return "Illegal Execution";
//         case 0x15: return "SVC (AArch64)";
//         case 0x18: return "MSR/MRS trap";
//         case 0x20: return "Instruction Abort (lower EL)";
//         case 0x21: return "Instruction Abort (same EL)";
//         case 0x22: return "PC Alignment";
//         case 0x24: return "Data Abort (lower EL)";
//         case 0x25: return "Data Abort (same EL)";
//         case 0x26: return "SP Alignment";
//         case 0x2C: return "FP Exception";
//         case 0x3C: return "BRK (AArch64)";
//         default:   return "Other";
//     }
// }
//
// static void KiDumpTrapFrame(PKTRAP_FRAME Tf)
// {
//     ULONG EC = (Tf->Esr >> 26) & 0x3F;
//
//     TrapPuts("\r\n*** ARM64 EXCEPTION ***\r\n");
//     TrapPuts("Type: ");
//     TrapPuts(KiGetExceptionClassName(EC));
//     TrapPuts(" (EC=");
//     TrapPutHex32(EC);
//     TrapPuts(")\r\n");
//
//     TrapPuts("PC:   "); TrapPutHex64(Tf->Pc);   TrapPuts("\r\n");
//     TrapPuts("LR:   "); TrapPutHex64(Tf->Lr);   TrapPuts("\r\n");
//     TrapPuts("SP:   "); TrapPutHex64(Tf->Sp);   TrapPuts("\r\n");
//     TrapPuts("FP:   "); TrapPutHex64(Tf->Fp);   TrapPuts("\r\n");
//     TrapPuts("ESR:  "); TrapPutHex32(Tf->Esr);  TrapPuts("\r\n");
//     TrapPuts("FAR:  "); TrapPutHex64(Tf->FaultAddress); TrapPuts("\r\n");
//     TrapPuts("SPSR: "); TrapPutHex32(Tf->Spsr); TrapPuts("\r\n");
//
//     TrapPuts("X0:   "); TrapPutHex64(Tf->X0);  TrapPuts("  X1:  "); TrapPutHex64(Tf->X1);  TrapPuts("\r\n");
//     TrapPuts("X2:   "); TrapPutHex64(Tf->X2);  TrapPuts("  X3:  "); TrapPutHex64(Tf->X3);  TrapPuts("\r\n");
//     TrapPuts("X4:   "); TrapPutHex64(Tf->X4);  TrapPuts("  X5:  "); TrapPutHex64(Tf->X5);  TrapPuts("\r\n");
//     TrapPuts("X6:   "); TrapPutHex64(Tf->X6);  TrapPuts("  X7:  "); TrapPutHex64(Tf->X7);  TrapPuts("\r\n");
//     TrapPuts("X8:   "); TrapPutHex64(Tf->X8);  TrapPuts("  X9:  "); TrapPutHex64(Tf->X9);  TrapPuts("\r\n");
//     TrapPuts("X10:  "); TrapPutHex64(Tf->X10); TrapPuts("  X11: "); TrapPutHex64(Tf->X11); TrapPuts("\r\n");
//     TrapPuts("X12:  "); TrapPutHex64(Tf->X12); TrapPuts("  X13: "); TrapPutHex64(Tf->X13); TrapPuts("\r\n");
//     TrapPuts("X14:  "); TrapPutHex64(Tf->X14); TrapPuts("  X15: "); TrapPutHex64(Tf->X15); TrapPuts("\r\n");
//     TrapPuts("X16:  "); TrapPutHex64(Tf->X16); TrapPuts("  X17: "); TrapPutHex64(Tf->X17); TrapPuts("\r\n");
//     TrapPuts("X18:  "); TrapPutHex64(Tf->X18); TrapPuts("\r\n");
// }
//
// /* SVC SYSTEM CALL DISPATCHER ************************************************/
//
// /*
//  * Function pointer type for dispatching system calls with up to 15 arguments.
//  * ARM64 passes args 1-8 in x0-x7, args 9+ on the stack.
//  * The callee ignores extra args, so passing more than needed is safe.
//  */
// /* Return type must be ULONG_PTR (not NTSTATUS) because some Win32k syscalls
//  * (e.g. NtUserSetWindowLongPtr, NtUserGetWindowLongPtr) return 64-bit values.
//  * Using NTSTATUS (32-bit) truncates their return values. */
// typedef ULONG_PTR (NTAPI *PSYSCALL_ROUTINE)(
//     ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
//     ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
//     ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
//     ULONG_PTR, ULONG_PTR, ULONG_PTR);
//
// extern NTSTATUS NTAPI PsConvertToGuiThread(VOID);
//
// static VOID
// KiSvcDispatch(
//     _Inout_ PKTRAP_FRAME TrapFrame)
// {
//     ULONG ServiceNumber = (ULONG)TrapFrame->X[8];
//     ULONG TableIndex = (ServiceNumber >> 12) & 1;
//     ULONG ServiceIndex = ServiceNumber & 0xFFF;
//     PKSERVICE_TABLE_DESCRIPTOR Table;
//     PSYSCALL_ROUTINE Routine;
//     ULONG_PTR SvcResult;
//     NTSTATUS GuiConvertStatus;
//     KPROCESSOR_MODE OldMode;
//     PKTHREAD Thread;
//     PKTRAP_FRAME OldTrapFrame;
//     ULONG_PTR Args[15] = {0};
//     ULONG ArgCount, i;
//
//     /* Get current thread and its service table */
//     Thread = KeGetCurrentThread() as PKTHREAD;
//     Table = (PKSERVICE_TABLE_DESCRIPTOR)Thread->ServiceTable;
//     if (!Table)
//         Table = KeServiceDescriptorTable;
//     Table = &Table[TableIndex];
//
//     /* Validate service number */
//     if (ServiceIndex >= Table->Limit)
//     {
//         /* Check if this is a Win32k call on a non-GUI thread */
//         if (TableIndex == 1 &&
//             Thread->ServiceTable == KeServiceDescriptorTable)
//         {
//             /* GUI thread conversion trace — disabled for clean console */
//
//             /* Convert to GUI thread (allocates W32Process/W32Thread,
//              * switches ServiceTable to KeServiceDescriptorTableShadow) */
//             GuiConvertStatus = PsConvertToGuiThread();
//             if (NT_SUCCESS(GuiConvertStatus))
//             {
//                 /* Re-fetch from the now-valid Win32k SSDT */
//                 Table = (PKSERVICE_TABLE_DESCRIPTOR)Thread->ServiceTable;
//                 Table = &Table[TableIndex];
//                 if (ServiceIndex < Table->Limit)
//                     goto DispatchCall;
//             }
//             TrapPuts("[SVC] GUI conversion failed: ");
//             TrapPutHex32((ULONG)GuiConvertStatus);
//             TrapPuts("\r\n");
//         }
//         TrapFrame->X[0] = (ULONG64)STATUS_INVALID_SYSTEM_SERVICE;
//         return;
//     }
//
// DispatchCall:
//
//     /* Flush GDI batch before Win32k syscalls (matches amd64 traphandler.c:197).
//      * User-mode GDI32 batches PatBlt/DeleteObj commands in TEB->GdiTebBatch.
//      * Without this flush, batched painting runs AFTER DeleteObject destroys
//      * brushes, causing all brush colors to resolve to WHITE. */
//     if (TableIndex == 1 && (TrapFrame->Spsr & 0xF) == 0)
//     {
//         PTEB Teb = KeGetCurrentThread()->Teb;
//         if (Teb && MmIsAddressValid(&Teb->GdiBatchCount) &&
//             Teb->GdiBatchCount != 0 && KeGdiFlushUserBatch)
//         {
//             KeGdiFlushUserBatch();
//         }
//     }
//
//     /* Win32k / NtDelayExecution traces removed — explorer.exe runs */
//
//     /* Get the service routine */
//     Routine = (PSYSCALL_ROUTINE)Table->Base[ServiceIndex];
//     if (!Routine)
//     {
//         TrapFrame->X[0] = (ULONG64)STATUS_INVALID_SYSTEM_SERVICE;
//         return;
//     }
//
//     /* Copy register arguments (x0-x7 = first 8 args) */
//     for (i = 0; i < 8 && i < 15; i++)
//         Args[i] = TrapFrame->X[i];
//
//     /* Copy stack arguments if needed (args 9+) */
//     ArgCount = Table->Number[ServiceIndex] / sizeof(ULONG_PTR);
//     if (ArgCount > 8)
//     {
//         PULONG_PTR CallerStack = (PULONG_PTR)TrapFrame->Sp;
//         ULONG StackArgs = ArgCount - 8;
//         if (StackArgs > 7) StackArgs = 7; /* Limit to our buffer */
//         for (i = 0; i < StackArgs; i++)
//             Args[8 + i] = CallerStack[i];
//     }
//
//     /* Set PreviousMode from SPSR: M[3:0]==0 is EL0t (user), else kernel */
//     OldMode = Thread->PreviousMode;
//     if ((TrapFrame->Spsr & 0xF) == 0)
//         Thread->PreviousMode = UserMode;
//     else
//         Thread->PreviousMode = KernelMode;
//
//     /*
//      * Link the trap frame to the thread. System calls like NtContinue,
//      * NtGetContextThread, etc. need Thread->TrapFrame to find the current
//      * user-mode context. Save and restore the old value for nested traps.
//      */
//     OldTrapFrame = Thread->TrapFrame;
//     TrapFrame->TrapFrame = (ULONG64)OldTrapFrame;
//     Thread->TrapFrame = TrapFrame;
//
//     /* Dispatch the system call */
//     SvcResult = Routine(Args[0], Args[1], Args[2], Args[3],
//                         Args[4], Args[5], Args[6], Args[7],
//                         Args[8], Args[9], Args[10], Args[11],
//                         Args[12], Args[13], Args[14]);
//
//     /* Restore TrapFrame link and PreviousMode */
//     Thread->TrapFrame = OldTrapFrame;
//     Thread->PreviousMode = OldMode;
//     TrapFrame->X[0] = (ULONG64)SvcResult;
//
//     /* ARM64: Voluntary preemption at syscall boundary.
//      * Without timer/software interrupts, a user-mode thread calling syscalls
//      * in a tight loop (e.g. WaitForSingleObject on a signaled object) can
//      * starve higher-priority threads indefinitely.  Check if a higher-priority
//      * thread is waiting in the ready queue and yield before returning. */
//     if ((TrapFrame->Spsr & 0xF) == 0)  /* User-mode syscall */
//     {
//         PKPRCB Prcb = KeGetCurrentPrcb();
//         if (Prcb->ReadySummary)
//         {
//             ULONG HighBit;
//             BitScanReverse(&HighBit, Prcb->ReadySummary);
//             if ((SCHAR)HighBit > Thread->Priority)
//             {
//                 NtYieldExecution();
//             }
//         }
//     }
//
//     /* SVC-FAIL trace disabled — too verbose for normal operation.
//      * Re-enable for debugging by uncommenting below. */
// #if 0
//     if (OldMode == UserMode && !NT_SUCCESS((NTSTATUS)SvcResult))
//     {
//         TrapPuts("[SVC-FAIL] #");
//         TrapPutHex32(ServiceNumber);
//         TrapPuts("(");
//         TrapPutHex64(Args[0]);
//         TrapPuts(")=>");
//         TrapPutHex32((ULONG)SvcResult);
//         TrapPuts("\r\n");
//     }
// #endif
// }
//
// /* EXCEPTION HANDLERS ********************************************************/
//
// VOID
// KiSynchronousExceptionHandler(
//     _Inout_ PKTRAP_FRAME TrapFrame)
// {
//     ULONG EC = (TrapFrame->Esr >> 26) & 0x3F;
//     ULONG ISS = TrapFrame->Esr & 0x1FFFFFF;
//
//     switch (EC)
//     {
//         case 0x25: /* Data Abort (same EL) */
//         case 0x24: /* Data Abort (lower EL) */
//         {
//             ULONG DFSC = ISS & 0x3F;
//             BOOLEAN IsWrite = (ISS >> 6) & 1;
//             KPROCESSOR_MODE FaultMode;
//             ULONG FaultCode;
//
//             /* Determine fault mode from PreviousMode (set by TRAP_PROLOG_EL0 for EL0) */
//             FaultMode = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;
//
//             /*
//              * Encode FaultCode in x86-compatible format for MmAccessFault:
//              *   Bit 0: 1 if page present (permission fault), 0 if not present
//              *   Bit 1: 1 if write access
//              */
//             FaultCode = 0;
//             if ((DFSC & 0x3C) == 0x0C)  /* Permission fault → page was present */
//                 FaultCode |= 0x1;
//             if (IsWrite)
//                 FaultCode |= 0x2;
//
//             /*
//              * Handle faults that MmAccessFault can resolve:
//              *   Translation fault (DFSC 0x04-0x07): page not mapped
//              *   Access flag fault (DFSC 0x08-0x0B): access bit not set
//              *   Permission fault  (DFSC 0x0C-0x0F): wrong permissions (COW, guard, etc.)
//              */
//             if ((DFSC & 0x3C) == 0x04 ||
//                 (DFSC & 0x3C) == 0x08 ||
//                 (DFSC & 0x3C) == 0x0C)
//             {
//                 NTSTATUS Status;
//
//                 Status = MmAccessFault(FaultCode,
//                                        (PVOID)TrapFrame->FaultAddress,
//                                        FaultMode,
//                                        TrapFrame);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Page fault counter — disabled for clean console output */
//                     return; /* Resume execution */
//                 }
//
//                 /* MmAccessFault failed — dispatch as access violation
//                  * (trace disabled for clean console output) */
//             }
//
//             /* Dispatch as access violation exception */
//             {
//                 EXCEPTION_RECORD ExRec;
//                 ExRec.ExceptionCode = STATUS_ACCESS_VIOLATION;
//                 ExRec.ExceptionFlags = 0;
//                 ExRec.ExceptionRecord = NULL;
//                 ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
//                 ExRec.NumberParameters = 2;
//                 ExRec.ExceptionInformation[0] = IsWrite ? EXCEPTION_WRITE_FAULT : EXCEPTION_READ_FAULT;
//                 ExRec.ExceptionInformation[1] = TrapFrame->FaultAddress;
//                 KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode, TRUE);
//                 return;
//             }
//         }
//
//         case 0x21: /* Instruction Abort (same EL) */
//         case 0x20: /* Instruction Abort (lower EL) */
//         {
//             ULONG IFSC = ISS & 0x3F;
//             KPROCESSOR_MODE FaultMode2;
//             ULONG FaultCode2;
//
//             FaultMode2 = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;
//
//             /*
//              * Encode FaultCode for instruction fetch:
//              *   Bit 0: 1 if permission fault (page present), 0 if not present
//              *   Bit 4: 1 for instruction fetch
//              */
//             FaultCode2 = 0x10;  /* Instruction fetch */
//             if ((IFSC & 0x3C) == 0x0C)  /* Permission fault → page present */
//                 FaultCode2 |= 0x1;
//
//             /* Translation/access flag/permission fault — try MmAccessFault */
//             if ((IFSC & 0x3C) == 0x04 ||
//                 (IFSC & 0x3C) == 0x08 ||
//                 (IFSC & 0x3C) == 0x0C)
//             {
//                 NTSTATUS Status2;
//                 Status2 = MmAccessFault(FaultCode2,
//                                         (PVOID)TrapFrame->FaultAddress,
//                                         FaultMode2,
//                                         TrapFrame);
//                 if (NT_SUCCESS(Status2))
//                 {
//                     return;
//                 }
//             }
//
//             /* Dispatch as access violation */
//             {
//                 EXCEPTION_RECORD ExRec;
//                 ExRec.ExceptionCode = STATUS_ACCESS_VIOLATION;
//                 ExRec.ExceptionFlags = 0;
//                 ExRec.ExceptionRecord = NULL;
//                 ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
//                 ExRec.NumberParameters = 2;
//                 ExRec.ExceptionInformation[0] = EXCEPTION_EXECUTE_FAULT;
//                 ExRec.ExceptionInformation[1] = TrapFrame->FaultAddress;
//                 KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode2, TRUE);
//                 return;
//             }
//         }
//
//         case 0x3C: /* BRK (software breakpoint) */
//         {
//             ULONG BrkImm = ISS & 0xFFFF;
//
//             /* BRK #0xF000 = DebugService: x0=Command, x1=Arg1, x2=Arg2
//              * BREAKPOINT_PRINT (1): x1=buffer, x2=length → output to UART */
//             if (BrkImm == 0xF000 && TrapFrame->X[0] == 1)
//             {
//                 char *Buf = (char *)(ULONG_PTR)TrapFrame->X[1];
//                 ULONG Len = (ULONG)(ULONG_PTR)TrapFrame->X[2];
//                 volatile ULONG *Uart = PL011_BASE;
//                 ULONG i;
//
//                 if (Len > 512) Len = 512; /* Sanity limit */
//                 for (i = 0; i < Len; i++)
//                 {
//                     if (!MmIsAddressValid(Buf + i))
//                     {
//                         /* Try to fault in the page (user-mode demand paging) */
//                         if ((ULONG_PTR)(Buf + i) < MmUserProbeAddress)
//                             MmAccessFault(FALSE, Buf + i, KernelMode, NULL);
//                         if (!MmIsAddressValid(Buf + i)) break;
//                     }
//                     if (Buf[i] == '\n') *Uart = (ULONG)'\r';
//                     *Uart = (ULONG)(unsigned char)Buf[i];
//                 }
//             }
//
//             /* Skip past the BRK instruction (4 bytes) */
//             TrapFrame->Pc += 4;
//             return;
//         }
//
//         case 0x15: /* SVC (system call) */
//         {
//             KiSvcDispatch(TrapFrame);
//             return;
//         }
//
//         case 0x26: /* SP Alignment fault */
//         {
//             EXCEPTION_RECORD ExRec;
//             KPROCESSOR_MODE FaultMode4;
//
//             FaultMode4 = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;
//
//             ExRec.ExceptionCode = STATUS_DATATYPE_MISALIGNMENT;
//             ExRec.ExceptionFlags = 0;
//             ExRec.ExceptionRecord = NULL;
//             ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
//             ExRec.NumberParameters = 0;
//             KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode4, TRUE);
//             return;
//         }
//
//         default:
//         {
//             EXCEPTION_RECORD ExRec;
//             KPROCESSOR_MODE FaultMode3;
//
//             FaultMode3 = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;
//
//             ExRec.ExceptionCode = STATUS_ILLEGAL_INSTRUCTION;
//             ExRec.ExceptionFlags = 0;
//             ExRec.ExceptionRecord = NULL;
//             ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
//             ExRec.NumberParameters = 0;
//             KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode3, TRUE);
//             return;
//         }
//     }
// }
//
// VOID
// KiIrqHandler(
//     _Inout_ PKTRAP_FRAME TrapFrame)
// {
//     TrapPuts("[IRQ] PC=");
//     TrapPutHex64(TrapFrame->Pc);
//     TrapPuts("\r\n");
//     /* No GIC configured yet — just return */
// }
//
// VOID
// KiSErrorHandler(
//     _Inout_ PKTRAP_FRAME TrapFrame)
// {
//     TrapPuts("[SError] FATAL\r\n");
//     KiDumpTrapFrame(TrapFrame);
//     KeBugCheckEx(NMI_HARDWARE_FAILURE,
//                  TrapFrame->Esr,
//                  TrapFrame->FaultAddress,
//                  TrapFrame->Pc,
//                  0);
// }
//
// VOID
// KiUnexpectedHandler(
//     _Inout_ PKTRAP_FRAME TrapFrame)
// {
//     TrapPuts("[UNEXPECTED EXCEPTION]\r\n");
//     KiDumpTrapFrame(TrapFrame);
//     KeBugCheckEx(UNEXPECTED_KERNEL_MODE_TRAP,
//                  TrapFrame->Esr,
//                  TrapFrame->FaultAddress,
//                  TrapFrame->Pc,
//                  0);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
