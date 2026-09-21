use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm::* removed to avoid KeGetCurrentThread ambiguity — use ke_defs version
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _KIPCR, _KPCR, _KPRCB, _KPROCESS, _KTHREAD, _LDR_DATA_TABLE_ENTRY,
    PKIPCR, PKPCR, PKPRCB, PKPROCESS, PKTHREAD, PLDR_DATA_TABLE_ENTRY,
    PLOADER_PARAMETER_BLOCK, KAFFINITY, PULONG_PTR,
    ExInitializeFastMutex, ExpInitializeExecutive, HalInitializeProcessor,
    KeInitializeDpc, KeInitializeEvent, KeInitializeProcess,
    KeInitializeSpinLock, KeSetPriorityThread, KeSetTargetProcessorDpc,
    KiAcquirePrcbLock, KiIdleLoop, KiInitSpinLocks,
    KiReleasePrcbLock, KiSaveProcessorControlState,
    KiTimerExpiration,
};

// Rust-defined globals — import to avoid extern split under LTO=false
use crate::ke::krnlinit::{
    KiInitialThread, KiInitialProcess, KeLoaderBlock, KiProcessorBlock,
    KeProcessorArchitecture, KeProcessorLevel, KeProcessorRevision,
    KeFeatureBits, KiServiceLimit,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use crate::ke::processor::{KeNumberProcessors, KeActiveProcessors};
use crate::ke::thrdschd::KiIdleSummary;
use crate::ke::bug::{KeBugcheckCallbackListHead, KeBugcheckReasonCallbackListHead, BugCheckCallbackLock};
use crate::ke::dpc::{KiTimerExpireDpc, KiGenericCallDpcMutex, KiMaximumDpcQueueDepth, KiMinimumDpcRate, KiAdjustDpcThreshold};
use crate::ke::profobj::{KiProfileLock, KiProfileListHead, KiProfileSourceListHead};
use crate::ke::timerobj::KiTimerTableListHead;
use crate::ke::procobj::{
    KiSwapEvent, KiProcessInSwapListHead, KiProcessOutSwapListHead, KiStackInSwapListHead,
    KeServiceDescriptorTable, KeServiceDescriptorTableShadow,
};
use crate::ke::krnlinit::KeNodeBlock;
use crate::ARM3::sysldr::PsLoadedModuleList;
use super::stubs::{Arm64FwCfgPhysical, Arm64GicCpuInterfacePhysical, Arm64GicDistributorPhysical, Arm64UartKernelBase};

use crate::ke::ssdt::{MainSSDT, MainSSPT};
extern "system" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    static KiArm64VectorTable: [u64; 0];
    fn KiSwitchToBootStack(InitialStack: ULONG_PTR) -> !;
    fn KiInitializeHandBuiltThread(Thread: PKTHREAD, Process: PKPROCESS, Stack: PVOID);
    fn _enable();
}

const PCR_MAJOR_VERSION: USHORT = 1;
const PCR_MINOR_VERSION: USHORT = 1;
const PRCB_MAJOR_VERSION: USHORT = 1;
const PRCB_MINOR_VERSION: USHORT = 1;
// ndk/arm64/ketypes.h:46-47 — DEBUG is 1, UNIPROCESSOR is 2 (same on all three
// arches). These two were SWAPPED here until 2026-07-25, so every arm64 boot
// published a BuildType meaning the opposite of what it wrote: the UP kernel
// told kmtest "MP, checked" and the MP kernel told it "UP, free". That is what
// BUGCHECK 0x9 in the KeSpinLock kmtest was — a free-build claim lets the test
// legitimately call `…AtDpcLevel` at PASSIVE, where our assert fires.
const PRCB_BUILD_DEBUG: USHORT = 1;
const PRCB_BUILD_UNIPROCESSOR: USHORT = 2;
// 0xC000, xdk/arm64/ke.h:49 ("48KB, matches amd64").
const KERNEL_STACK_SIZE: usize = 0xC000;
const TIMER_TABLE_SIZE: usize = 512;

#[no_mangle]
pub static mut KiInitialPcr: _KIPCR = unsafe { core::mem::zeroed() };

#[repr(align(16))]
struct AlignedStack([u8; KERNEL_STACK_SIZE]);

#[no_mangle]
pub static mut KiP0BootStackData: AlignedStack = AlignedStack([0; KERNEL_STACK_SIZE]);

#[no_mangle]
pub static mut KiP0DoubleFaultStackData: AlignedStack = AlignedStack([0; KERNEL_STACK_SIZE]);

#[no_mangle]
pub static mut KiP0BootStack: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut KiFreezeExecutionLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut FrLdrDbgPrint: Option<unsafe extern "C" fn(*const u8, ...) -> ULONG> = None;

static mut LdrCoreEntries: [_LDR_DATA_TABLE_ENTRY; 3] = unsafe { core::mem::zeroed() };

// SMP AP progress beacon (smp-bringup-quest.md M5a debug). The AP writes its
// current milestone here (memory only — no UART, so no BSP/AP serial interleave
// and no TCG slowdown that shifts the wedge); the BSP prints it on handshake
// timeout and re-reads to tell "hung" (stage frozen) from "slow" (still rising).
#[no_mangle]
pub static KiArm64ApStage: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

#[inline(always)]
fn ap_stage(cpu_nonzero: bool, n: u32) {
    if cpu_nonzero {
        KiArm64ApStage.store(n, core::sync::atomic::Ordering::SeqCst);
    }
}

pub(crate) unsafe fn KiInitializePcr(
    Pcr: PKIPCR,
    ProcessorNumber: ULONG,
    IdleThread: PKTHREAD,
) {
    core::ptr::write_bytes(Pcr as *mut u8, 0, core::mem::size_of::<_KIPCR>());

    (*Pcr).__bindgen_anon_1.__bindgen_anon_1.Self_ = Pcr as PKPCR;

    (*(Pcr as PKPCR)).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb = core::ptr::addr_of_mut!((*Pcr).Prcb);

    (*Pcr).MajorVersion = PCR_MAJOR_VERSION;
    (*Pcr).MinorVersion = PCR_MINOR_VERSION;

    (*Pcr).Prcb.MajorVersion = PRCB_MAJOR_VERSION;
    (*Pcr).Prcb.MinorVersion = PRCB_MINOR_VERSION;

    (*Pcr).Prcb.BuildType = 0;
    #[cfg(not(feature = "smp"))]
    {
        (*Pcr).Prcb.BuildType |= PRCB_BUILD_UNIPROCESSOR as UCHAR;
    }
    // C kernel uses `#if DBG`, which is always TRUE in our Debug build (-DDBG=1).
    // Rust `#[cfg(debug_assertions)]` is NOT set in --release mode, which is how
    // the kernel is built — so gating on it published a FREE-build PRCB and left
    // arm64 the odd arch out (i386/amd64 both set this bit unconditionally, with
    // this same note). Two things rode on that: kmtest's `KmtIsCheckedBuild` was
    // FALSE for the whole arm64 corpus (so every test took its free-build path),
    // and on the MP kernel that let KeSpinLock call `…AtDpcLevel` at PASSIVE —
    // legal on a free build, but our smp-gated asserts BUGCHECK 0x9 there.
    (*Pcr).Prcb.BuildType |= PRCB_BUILD_DEBUG as UCHAR;

    (*Pcr).Prcb.Number = ProcessorNumber;
    (*Pcr).Prcb.SetMember = 1u64 << ProcessorNumber;

    (*Pcr).Prcb.CurrentThread = IdleThread;

    (*Pcr).CurrentIrql = PASSIVE_LEVEL;

    (*Pcr).Prcb.MultiThreadProcessorSet = (*Pcr).Prcb.SetMember;
}

// CODE_SEG("INIT")
unsafe fn KiInitializeP0BootStructures(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    // C: PVOID KiP0BootStack = &KiP0BootStackData[KERNEL_STACK_SIZE];
    // Stack grows down — KiP0BootStack points to the TOP (end) of the array.
    KiP0BootStack = (core::ptr::addr_of_mut!(KiP0BootStackData) as *mut u8)
        .add(KERNEL_STACK_SIZE) as PVOID;
    (*LoaderBlock).KernelStack = KiP0BootStack as u64;
    (*LoaderBlock).Thread = core::ptr::addr_of_mut!(KiInitialThread) as u64;
    (*LoaderBlock).Process = core::ptr::addr_of_mut!(KiInitialProcess) as u64;
    (*LoaderBlock).Prcb = core::ptr::addr_of_mut!(KiInitialPcr.Prcb) as u64;

    KiInitializePcr(
        core::ptr::addr_of_mut!(KiInitialPcr),
        0,
        core::ptr::addr_of_mut!(KiInitialThread) as PKTHREAD,
    );
}

unsafe fn KiInitModuleList(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    InitializeListHead(core::ptr::addr_of_mut!(PsLoadedModuleList));

    let mut entry = (*LoaderBlock).LoadOrderListHead.Flink;
    let list_head = core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead);
    
    for i in 0..3 {
        if entry == list_head {
            break;
        }

        let offset = core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
        let ldr_entry = (entry as *mut u8).sub(offset) as PLDR_DATA_TABLE_ENTRY;

        LdrCoreEntries[i] = *ldr_entry;

        InsertTailList(
            core::ptr::addr_of_mut!(PsLoadedModuleList) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(LdrCoreEntries[i].InLoadOrderLinks) as PLIST_ENTRY,
        );

        entry = (*entry).Flink;
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSystemStartup(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> ! {
    let LoaderBlock = opaque_ptr(LoaderBlock);

    // Enable FP/SIMD access (CPACR_EL1.FPEN = 0b11). The BSP inherits an
    // FP-enabled CPACR from the UEFI firmware, but a PSCI-started AP (SMP) comes
    // up at the trapping reset value (FPEN=00) — so the first NEON-vectorized
    // store the AP executes (e.g. KiInitSpinLocks' LockQueue block) takes an
    // EC=0x07 SIMD-disabled abort, and the exception path re-enters the vector on
    // a corrupted SP and wedges the core (GDB: CPU1 PC=KiArm64VectorTable+0x200,
    // "FPU disabled"; smp-bringup-quest.md M5a). Set it for every CPU here
    // (idempotent on the BSP) before any code that may vectorize.
    core::arch::asm!(
        "mrs {t}, cpacr_el1",
        "orr {t}, {t}, #(3 << 20)",
        "msr cpacr_el1, {t}",
        "isb",
        t = out(reg) _,
        options(nomem, nostack),
    );

    // Publish the firmware-discovered platform addresses BEFORE the first debug
    // write below — this is the earliest point they can be known, and every
    // Arm64UartKernelBase consumer in the kernel reads this one global. The
    // loader identity-maps the UART page, so the physical address is usable
    // until Phase 1 swaps in an MmMapIoSpace kernel VA.
    Arm64UartKernelBase = (*LoaderBlock).u.Arm.Arm64UartBase as PVOID;
    Arm64GicDistributorPhysical = (*LoaderBlock).u.Arm.Arm64GicDistributorBase;
    Arm64GicCpuInterfacePhysical = (*LoaderBlock).u.Arm.Arm64GicCpuInterfaceBase;
    // Same inference the loader makes when deciding whether to map the page.
    const QEMU_VIRT_UART: u64 = 0x0900_0000;
    const QEMU_VIRT_FW_CFG: u64 = 0x0902_0000;
    Arm64FwCfgPhysical = if (*LoaderBlock).u.Arm.Arm64UartBase == QEMU_VIRT_UART {
        QEMU_VIRT_FW_CFG
    } else {
        0
    };

    // Direct PL011 UART write to confirm kernel entry (bypasses KD)
    {
        let uart = Arm64UartKernelBase as *mut ULONG;
        let msg = b"[KERNEL] KiSystemStartup entered!\r\n";
        for &b in msg {
            core::ptr::write_volatile(uart, b as ULONG);
        }
    }

    // Get the current CPU number
    let Cpu = KeNumberProcessors;
    KeNumberProcessors += 1;

    // LoaderBlock initialization for Cpu 0
    if Cpu == 0 {
        KeLoaderBlock = LoaderBlock;
        KiInitializeP0BootStructures(LoaderBlock);
    }

    // Get Pcr from loader block — CONTAINING_RECORD(LoaderBlock->Prcb, KIPCR, Prcb)
    let Pcr = ((*LoaderBlock).Prcb as *mut u8)
        .offset(-(core::mem::offset_of!(_KIPCR, Prcb) as isize))
        as PKIPCR;

    // Write PCR pointer to TPIDR_EL1 for KeGetPcr()
    core::arch::asm!("msr tpidr_el1, {}", in(reg) Pcr as u64);

    // Grant EL0 read access to the generic-timer counters (CNTKCTL_EL1).
    // Reset value traps every EL0 counter read to EL1 as an illegal
    // instruction (ESR EC=0x18). vibeos's own user-mode never reads the
    // timers directly, but real ARM64 user binaries do — e.g. FEX-Emu's
    // SpinWaitLock ctor does `mrs CNTFRQ_EL0` to calibrate its spin, and
    // libc++/ucrt read CNTVCT_EL0 for high-res time. Set EL0PCTEN (bit0) +
    // EL0VCTEN (bit1) so CNTFRQ_EL0/CNTPCT_EL0/CNTVCT_EL0 are readable at EL0;
    // leave the EL0*TEN timer-programming bits clear (reads only). See
    // docs/engineering/fex-bringup-plan.md rung 2.
    core::arch::asm!(
        "mrs {tmp}, cntkctl_el1",
        "orr {tmp}, {tmp}, #0x3",
        "msr cntkctl_el1, {tmp}",
        "isb",
        tmp = out(reg) _,
    );

    // Same family, SCTLR_EL1 edition: grant EL0 the cache-maintenance surface
    // production Windows-on-ARM64 exposes. The reset value leaves UCI/UCT/DZE
    // clear, so EL0 `dc cvau` (FEX's inline JIT icache flush on the guest-SMC
    // path), `mrs CTR_EL0` (cache geometry, read by ARM64 memcpy/JIT runtimes)
    // and `dc zva` (memset fast path) all trap to EL1 as EC=0x18 and die as
    // STATUS_ILLEGAL_INSTRUCTION. That was the task-#46 FEX startup-death
    // flake: rare only because QEMU's CTR_EL0 DIC/IDC lets vixl skip
    // maintenance on the common flush path, so `dc cvau` executed maybe once
    // per boot. UCI (bit 26) + UCT (bit 15) + DZE (bit 14). fexprobe.exe
    // executes all three ops every boot as the deterministic tripwire.
    core::arch::asm!(
        "mrs {tmp}, sctlr_el1",
        "orr {tmp}, {tmp}, #0x04000000", // UCI
        "orr {tmp}, {tmp}, #0xC000",     // UCT | DZE
        "msr sctlr_el1, {tmp}",
        "isb",
        tmp = out(reg) _,
    );

    // Install ARM64 exception vector table (VBAR_EL1)
    {
        let vbar = KiArm64VectorTable.as_ptr() as u64;
        core::arch::asm!("msr vbar_el1, {}", "isb", in(reg) vbar);

        let uart = Arm64UartKernelBase as *mut ULONG;
        let msg = b"[KERNEL] VBAR_EL1 installed\r\n";
        for &b in msg {
            core::ptr::write_volatile(uart, b as ULONG);
        }
    }

    // Set the PRCB for this Processor
    *KiProcessorBlock.as_mut_ptr().add(Cpu as usize) = core::ptr::addr_of_mut!((*Pcr).Prcb);

    // Save the initial thread
    let InitialThread = (*LoaderBlock).Thread as PKTHREAD;

    // Set us as the current process
    (*InitialThread).__bindgen_anon_1.ApcState.Process = (*LoaderBlock).Process as _;

    {
        let uart = Arm64UartKernelBase as *mut ULONG;
        let msg = b"[KERNEL] pre-HAL\r\n";
        for &b in msg { core::ptr::write_volatile(uart, b as ULONG); }
    }
    ap_stage(Cpu != 0, 3);

    // Initial setup for the boot CPU
    if Cpu == 0 {
        KeProcessorArchitecture = 12; // ARM64
        KeProcessorLevel = 8;         // ARMv8
        KeProcessorRevision = 0;

        KiInitModuleList(LoaderBlock);
    }

    // Initialize the Processor with HAL
    HalInitializeProcessor(Cpu as ULONG, KeLoaderBlock);

    // Set processor as active
    KeActiveProcessors |= (1 as KAFFINITY) << Cpu;
    ap_stage(Cpu != 0, 4);

    // Raise to HIGH_LEVEL
    KfRaiseIrql(HIGH_LEVEL);

    // Machine specific kernel initialization
    if Cpu == 0 {
        KiInitMachineDependent();
    }

    // Switch to boot stack and continue initialization
    ap_stage(Cpu != 0, 5);
    KiSwitchToBootStack((*LoaderBlock).KernelStack as ULONG_PTR);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSystemStartupBootStack() -> ! {
    let LoaderBlock = KeLoaderBlock;
    let Prcb = KeGetCurrentPrcb();
    let Thread = (*KeLoaderBlock).Thread as PKTHREAD;
    let Process = (*Thread).__bindgen_anon_1.ApcState.Process as PKPROCESS;
    let KernelStack = (*KeLoaderBlock).KernelStack as PVOID;

    let ap = (*Prcb).Number != 0;
    ap_stage(ap, 10);

    (*Prcb).ParentNode = *(core::ptr::addr_of!(KeNodeBlock) as *const *mut core::ffi::c_void).add(0) as *mut ntkrnl_sys::ddk::_KNODE;
    (*((*Prcb).ParentNode as *mut _KNODE)).ProcessorMask |= (*Prcb).SetMember;

    KiSaveProcessorControlState(core::ptr::addr_of_mut!((*Prcb).ProcessorState));
    ap_stage(ap, 11);

    KiInitSpinLocks(Prcb, (*Prcb).Number as CCHAR);
    ap_stage(ap, 13);

    (*Prcb).CurrentThread = Thread;
    (*Prcb).NextThread = core::ptr::null_mut();
    (*Prcb).IdleThread = Thread;

    KeLowerIrql(APC_LEVEL);
    ap_stage(ap, 14);

    if (*Prcb).Number == 0 {
        KiInitializeKernel(Process, Thread, KernelStack, Prcb, LoaderBlock);
    }

    KfRaiseIrql(DISPATCH_LEVEL);

    KeSetPriorityThread(Thread, 0);
    ap_stage(ap, 16);

    KiAcquirePrcbLock(Prcb);
    if (*Prcb).NextThread.is_null() {
        KiIdleSummary |= (1 as KAFFINITY) << (*Prcb).Number;
    }
    KiReleasePrcbLock(Prcb);
    ap_stage(ap, 17);

    KfRaiseIrql(HIGH_LEVEL);
    (*LoaderBlock).Prcb = 0;
    ap_stage(ap, 19);

    let Thread = KeGetCurrentThread() as PKTHREAD;
    (*Thread).Priority = 0;

    _enable();
    KeLowerIrql(DISPATCH_LEVEL);

    (*Thread).__bindgen_anon_5.__bindgen_anon_6.WaitIrql = DISPATCH_LEVEL;

    {
        let uart = Arm64UartKernelBase as *mut ULONG;
        let msg = b"[KERNEL] Entering KiIdleLoop\r\n";
        for &ch in msg.iter() {
            core::ptr::write_volatile(uart, ch as ULONG);
        }
        // Dump KiIdleLoop function address for symbolication
        let fn_addr = KiIdleLoop as *const () as u64;
        let msg2 = b"[KERNEL] KiIdleLoop@0x";
        for &ch in msg2.iter() { core::ptr::write_volatile(uart, ch as ULONG); }
        for i in (0..16).rev() {
            let nibble = ((fn_addr >> (i * 4)) & 0xF) as u8;
            let c = if nibble < 10 { b'0' + nibble } else { b'a' + nibble - 10 };
            core::ptr::write_volatile(uart, c as ULONG);
        }
        for &ch in b"\r\n" { core::ptr::write_volatile(uart, ch as ULONG); }
    }

    KiIdleLoop();
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeKernel(
    InitProcess: PKPROCESS,
    InitThread: PKTHREAD,
    IdleStack: PVOID,
    Prcb: PKPRCB,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    let InitProcess = opaque_ptr(InitProcess);
    let InitThread = opaque_ptr(InitThread);
    let IdleStack = opaque_ptr(IdleStack);
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut PageDirectory: [u64; 2] = [0, 0];

    // One-time SCTLR_EL1 dump: bit 1 (0x2) = A (strict alignment check). If set,
    // every unaligned access faults (DFSC 0x21) — which is the cascade of
    // vectorized wide-store alignment faults seen in early init (NLS, ObpAllocate).
    // If clear, the faults come from Device/strict-mapped early regions instead.
    {
        extern "C" {
            fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ULONG_PTR);
        }
        let sctlr: u64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
        ARM64_UART_TRACE_HEX(b"[KISTART] SCTLR_EL1=\0".as_ptr(), sctlr as ULONG_PTR);
    }

    (*Prcb).MultiThreadSetMaster = Prcb;

    InitializeListHead(core::ptr::addr_of_mut!(KeBugcheckCallbackListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KeBugcheckReasonCallbackListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(BugCheckCallbackLock) as *mut u64);

    KeInitializeDpc(
        core::ptr::addr_of_mut!(KiTimerExpireDpc) as *mut ntkrnl_sys::ddk::_KDPC,
        Some(KiTimerExpiration),
        core::ptr::null_mut(),
    );
    KeSetTargetProcessorDpc(core::ptr::addr_of_mut!(KiTimerExpireDpc) as *mut ntkrnl_sys::ddk::_KDPC, 0);

    KeInitializeSpinLock(core::ptr::addr_of_mut!(KiProfileLock) as *mut u64);
    InitializeListHead(core::ptr::addr_of_mut!(KiProfileListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiProfileSourceListHead));

    for i in 0..TIMER_TABLE_SIZE {
        InitializeListHead(core::ptr::addr_of_mut!(KiTimerTableListHead[i].Entry));
        KiTimerTableListHead[i].Time.u.HighPart = 0xFFFFFFFFu32;
        KiTimerTableListHead[i].Time.u.LowPart = 0;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(KiSwapEvent) as *mut ntkrnl_sys::ddk::_KEVENT,
        1, // SynchronizationEvent (EVENT_TYPE: Notification=0, Synchronization=1)
        0u8,
    );
    InitializeListHead(core::ptr::addr_of_mut!(KiProcessInSwapListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiProcessOutSwapListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiStackInSwapListHead));

    ExInitializeFastMutex(core::ptr::addr_of_mut!(KiGenericCallDpcMutex) as *mut ntkrnl_sys::ddk::_FAST_MUTEX);

    KeServiceDescriptorTable[0].Base = core::ptr::addr_of!(MainSSDT) as PULONG_PTR;
    KeServiceDescriptorTable[0].Count = core::ptr::null_mut();
    KeServiceDescriptorTable[0].Limit = KiServiceLimit;
    KeServiceDescriptorTable[1].Limit = 0;
    KeServiceDescriptorTable[0].Number = core::ptr::addr_of!(MainSSPT) as *mut u8;

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!(KeServiceDescriptorTable) as *const u8,
        core::ptr::addr_of_mut!(KeServiceDescriptorTableShadow) as *mut u8,
        core::mem::size_of_val(&KeServiceDescriptorTable),
    );

    InitializeListHead(core::ptr::addr_of_mut!(KiProcessListHead));
    PageDirectory[0] = 0;
    PageDirectory[1] = 0;
    KeInitializeProcess(
        InitProcess,
        0,
        u64::MAX,
        PageDirectory.as_mut_ptr(),
        0u8,
    );
    (*InitProcess).QuantumReset = 127i8;

    KiInitializeHandBuiltThread(InitThread, InitProcess, IdleStack);

    {
        let uart = Arm64UartKernelBase as *mut ULONG;
        let msg = b"[KERNEL] Calling ExpInitializeExecutive\r\n";
        for &ch in msg.iter() {
            core::ptr::write_volatile(uart, ch as ULONG);
        }
    }

    ExpInitializeExecutive(0, LoaderBlock);

    (*Prcb).MaximumDpcQueueDepth = KiMaximumDpcQueueDepth as i32;
    (*Prcb).MinimumDpcRate = KiMinimumDpcRate;
    (*Prcb).AdjustDpcThreshold = KiAdjustDpcThreshold;

    {
        let uart = Arm64UartKernelBase as *mut ULONG;
        let msg = b"[KERNEL] ExpInitializeExecutive returned!\r\n";
        for &ch in msg.iter() {
            core::ptr::write_volatile(uart, ch as ULONG);
        }
    }
}

use crate::ke::procobj::KiProcessListHead;

// C macro constants — NOT extern variables
const MAXCHAR: i8 = 0x7f;

#[repr(C)]
struct _KDPC {
    _opaque: [u8; 0],
}

#[repr(C)]
struct _KTIMER_TABLE_ENTRY {
    Time: LARGE_INTEGER,
    Entry: LIST_ENTRY,
}

#[repr(C)]
struct _KEVENT {
    _opaque: [u8; 0],
}

#[repr(C)]
struct _FAST_MUTEX {
    _opaque: [u8; 0],
}

#[repr(C)]
struct _KSERVICE_TABLE_DESCRIPTOR {
    Base: *mut isize,
    Count: *mut ULONG,
    Limit: ULONG,
    Number: *mut u8,
}

#[repr(C)]
struct _KNODE {
    ProcessorMask: u64, // KAFFINITY = ULONG_PTR = c_ulonglong on arm64
}

#[no_mangle]
pub extern "system" fn __chkstk() {}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub extern "system" fn _allshl() {}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub extern "system" fn _allshr() {}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub extern "system" fn _aullshr() {}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL-2.0-or-later
//  * FILE:            ntoskrnl/ke/arm64/kiinit.c
//  * PURPOSE:         ARM64 Kernel Initialization
//  * PROGRAMMERS:     ReactOS ARM64 Port
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern ULONG_PTR MainSSDT[];
// extern UCHAR MainSSPT[];
//
// /* Forward declarations */
// DECLSPEC_NORETURN VOID NTAPI KiSystemStartupBootStack(VOID);
// DECLSPEC_NORETURN VOID KiSwitchToBootStack(ULONG_PTR InitialStack);
// DECLSPEC_NORETURN VOID FASTCALL KiIdleLoop(VOID);
// VOID NTAPI KiInitializeHandBuiltThread(PKTHREAD Thread, PKPROCESS Process, PVOID Stack);
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeKernel(
//     _In_ PKPROCESS InitProcess,
//     _In_ PKTHREAD InitThread,
//     _In_ PVOID IdleStack,
//     _In_ PKPRCB Prcb,
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock);
//
// /* GLOBALS *******************************************************************/
//
// /* Boot PCR for processor 0 */
// KIPCR KiInitialPcr;
//
// /* Boot and double-fault stack */
// UCHAR DECLSPEC_ALIGN(16) KiP0BootStackData[KERNEL_STACK_SIZE] = {0};
// UCHAR DECLSPEC_ALIGN(16) KiP0DoubleFaultStackData[KERNEL_STACK_SIZE] = {0};
// PVOID KiP0BootStack = &KiP0BootStackData[KERNEL_STACK_SIZE];
//
// /* Spinlock for freezing execution during init */
// KSPIN_LOCK KiFreezeExecutionLock;
//
// /* Function pointer for early debug prints (passed from FreeLDR) */
// ULONG (*FrLdrDbgPrint)(const char *Format, ...);
//
// /* Module list entries for core modules */
// static LDR_DATA_TABLE_ENTRY LdrCoreEntries[3];
//
// /* FUNCTIONS *****************************************************************/
//
// static
// VOID
// KiInitializePcr(
//     _Out_ PKIPCR Pcr,
//     _In_ ULONG ProcessorNumber,
//     _In_ PKTHREAD IdleThread)
// {
//     /* Zero out the PCR */
//     RtlZeroMemory(Pcr, sizeof(KIPCR));
//
//     /* Set self-pointer (KIPCR.Self maps to KPCR.Self) */
//     Pcr->Self = (PKPCR)Pcr;
//
//     /*
//      * Set CurrentPrcb via the KPCR view.
//      * KIPCR doesn't name it CurrentPrcb — it's PcrReserved0 at the same offset.
//      * Cast to PKPCR to write the member that KeGetCurrentPrcb() reads.
//      */
//     ((PKPCR)Pcr)->CurrentPrcb = &Pcr->Prcb;
//
//     /* Set the PCR Version */
//     Pcr->MajorVersion = PCR_MAJOR_VERSION;
//     Pcr->MinorVersion = PCR_MINOR_VERSION;
//
//     /* Set the PRCB Version */
//     Pcr->Prcb.MajorVersion = PRCB_MAJOR_VERSION;
//     Pcr->Prcb.MinorVersion = PRCB_MINOR_VERSION;
//
//     /* Set the Build Type */
//     Pcr->Prcb.BuildType = 0;
// #ifndef CONFIG_SMP
//     Pcr->Prcb.BuildType |= PRCB_BUILD_UNIPROCESSOR;
// #endif
// #if DBG
//     Pcr->Prcb.BuildType |= PRCB_BUILD_DEBUG;
// #endif
//
//     /* Set the Processor Number and current Processor Mask */
//     Pcr->Prcb.Number = (UCHAR)ProcessorNumber;
//     Pcr->Prcb.SetMember = 1ULL << ProcessorNumber;
//
//     /* Set the Current Thread */
//     Pcr->Prcb.CurrentThread = IdleThread;
//
//     /* Start us out at PASSIVE_LEVEL */
//     Pcr->CurrentIrql = PASSIVE_LEVEL;
//
//     /* Setup the processor set */
//     Pcr->Prcb.MultiThreadProcessorSet = Pcr->Prcb.SetMember;
// }
//
// CODE_SEG("INIT")
// static
// VOID
// KiInitializeP0BootStructures(
//     _Inout_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     /* Set the initial stack, idle thread and process for processor 0 */
//     LoaderBlock->KernelStack = (ULONG_PTR)KiP0BootStack;
//     LoaderBlock->Thread = (ULONG_PTR)&KiInitialThread;
//     LoaderBlock->Process = (ULONG_PTR)&KiInitialProcess.Pcb;
//     LoaderBlock->Prcb = (ULONG_PTR)&KiInitialPcr.Prcb;
//
//     /* Initialize PCR for processor 0 */
//     KiInitializePcr(&KiInitialPcr,
//                     0,
//                     &KiInitialThread.Tcb);
// }
//
// static
// VOID
// KiInitModuleList(
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY Entry;
//     ULONG i;
//
//     /* Initialize the list head */
//     InitializeListHead(&PsLoadedModuleList);
//
//     /* Loop the first 3 entries (ntoskrnl, hal, kdcom) */
//     for (Entry = LoaderBlock->LoadOrderListHead.Flink, i = 0;
//          Entry != &LoaderBlock->LoadOrderListHead && i < 3;
//          Entry = Entry->Flink, i++)
//     {
//         /* Get the data table entry */
//         LdrEntry = CONTAINING_RECORD(Entry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//
//         /* Copy the entry */
//         LdrCoreEntries[i] = *LdrEntry;
//
//         /* Insert the copy into the list */
//         InsertTailList(&PsLoadedModuleList, &LdrCoreEntries[i].InLoadOrderLinks);
//     }
// }
//
#[no_mangle]
pub unsafe extern "system" fn KiInitMachineDependent() {
    // ARM64-specific hardware initialization — stub
}

// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitMachineDependent(VOID)
// {
//     /* ARM64-specific hardware initialization */
//     DPRINT1("KiInitMachineDependent: ARM64\n");
// }
//
// CODE_SEG("INIT")
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KiSystemStartup(
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     CCHAR Cpu;
//     PKTHREAD InitialThread;
//     PKIPCR Pcr;
//
//     /* Direct PL011 UART write to confirm kernel entry (bypasses KD) */
//     {
//         volatile ULONG *uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *msg = "[KERNEL] KiSystemStartup entered!\r\n";
//         while (*msg) { *uart = (ULONG)*msg++; }
//     }
//
//     /* Get the current CPU number */
//     Cpu = KeNumberProcessors++;
//
//     /* LoaderBlock initialization for Cpu 0 */
//     if (Cpu == 0)
//     {
//         /* Save the loader block */
//         KeLoaderBlock = LoaderBlock;
//
//         /* Prepare LoaderBlock, PCR with the P0 boot data */
//         KiInitializeP0BootStructures(LoaderBlock);
//     }
//
//     /* Get Pcr from loader block */
//     Pcr = CONTAINING_RECORD(LoaderBlock->Prcb, KIPCR, Prcb);
//
//     /* Write PCR pointer to TPIDR_EL1 for KeGetPcr() */
//     __asm__ __volatile__("msr tpidr_el1, %0" :: "r"((ULONG64)Pcr));
//
//     /* Install ARM64 exception vector table (VBAR_EL1) */
//     {
//         extern ULONG64 KiArm64VectorTable[];
//         volatile ULONG *uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *msg = "[KERNEL] VBAR_EL1 installed\r\n";
//
//         __asm__ __volatile__("msr vbar_el1, %0" :: "r"((ULONG64)&KiArm64VectorTable));
//         __asm__ __volatile__("isb");
//
//         while (*msg) { *uart = (ULONG)*msg++; }
//     }
//
//     /* Set the PRCB for this Processor */
//     KiProcessorBlock[Cpu] = &Pcr->Prcb;
//
//     /* Save the initial thread */
//     InitialThread = (PKTHREAD)LoaderBlock->Thread;
//
//     /* Set us as the current process */
//     InitialThread->ApcState.Process = (PVOID)LoaderBlock->Process;
//
//     /* Initial setup for the boot CPU */
//     if (Cpu == 0)
//     {
//         /* Set processor architecture (12 = ARM64) */
//         KeProcessorArchitecture = 12;
//         KeProcessorLevel = 8;   /* ARMv8 */
//         KeProcessorRevision = 0;
//
//         /* Initialize the module list (ntos, hal, kdcom) */
//         KiInitModuleList(LoaderBlock);
//
//         /* TODO: KdInitSystem hangs with kernel VA page tables active.
//          * Skip for now to continue debugging boot sequence. */
//         //KdInitSystem(0, KeLoaderBlock);
//
//         /* Check for break-in - skip since KD not initialized */
//         //if (KdPollBreakIn()) DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
//     }
//
//     /* Initialize the Processor with HAL */
//     HalInitializeProcessor(Cpu, KeLoaderBlock);
//
//     /* Set processor as active */
//     KeActiveProcessors |= 1ULL << Cpu;
//
//     /* Raise to HIGH_LEVEL */
//     KfRaiseIrql(HIGH_LEVEL);
//
//     /* Machine specific kernel initialization */
//     if (Cpu == 0)
//         KiInitMachineDependent();
//
//     {
//         volatile ULONG *uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *msg = "[KERNEL] Switching to boot stack\r\n";
//         while (*msg) { *uart = (ULONG)*msg++; }
//     }
//
//     /* Switch to boot stack and continue initialization */
//     KiSwitchToBootStack((ULONG_PTR)LoaderBlock->KernelStack);
// }
//
// CODE_SEG("INIT")
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KiSystemStartupBootStack(VOID)
// {
//     PLOADER_PARAMETER_BLOCK LoaderBlock = KeLoaderBlock;
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD Thread = (PKTHREAD)KeLoaderBlock->Thread;
//     PKPROCESS Process = Thread->ApcState.Process;
//     PVOID KernelStack = (PVOID)KeLoaderBlock->KernelStack;
//
//     {
//         volatile ULONG *uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *msg = "[KERNEL] KiSystemStartupBootStack entered\r\n";
//         while (*msg) { *uart = (ULONG)*msg++; }
//     }
//
//     /* Set Node Data */
//     Prcb->ParentNode = KeNodeBlock[0];
//     Prcb->ParentNode->ProcessorMask |= Prcb->SetMember;
//
//     /* Save CPU state (stub on ARM64) */
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
//
//     /* Initialize spinlocks and DPC data */
//     KiInitSpinLocks(Prcb, Prcb->Number);
//
//     /* Set up the thread-related fields in the PRCB */
//     Prcb->CurrentThread = Thread;
//     Prcb->NextThread = NULL;
//     Prcb->IdleThread = Thread;
//
//     /* Lower to APC_LEVEL */
//     KeLowerIrql(APC_LEVEL);
//
//     /* Check if this is the boot cpu */
//     if (Prcb->Number == 0)
//     {
//         /* Initialize the kernel */
//         KiInitializeKernel(Process, Thread, KernelStack, Prcb, LoaderBlock);
//     }
//
//     /* Raise to Dispatch */
//     KfRaiseIrql(DISPATCH_LEVEL);
//
//     /* Set the Idle Priority to 0. This will cause the scheduler to notice
//      * the Phase1 initialization thread created by PsInitSystem. */
//     KeSetPriorityThread(Thread, 0);
//
//     /* If there's no thread scheduled, put this CPU in the Idle summary */
//     KiAcquirePrcbLock(Prcb);
//     if (!Prcb->NextThread) KiIdleSummary |= (ULONG_PTR)1 << Prcb->Number;
//     KiReleasePrcbLock(Prcb);
//
//     /* Raise back to HIGH_LEVEL and clear the loader PRCB */
//     KfRaiseIrql(HIGH_LEVEL);
//     LoaderBlock->Prcb = 0;
//
//     /* Set the priority of this thread to 0 */
//     Thread = KeGetCurrentThread();
//     Thread->Priority = 0;
//
//     /* Force interrupts enabled and lower IRQL back to DISPATCH_LEVEL */
//     _enable();
//     KeLowerIrql(DISPATCH_LEVEL);
//
//     /* Set the right wait IRQL */
//     Thread->WaitIrql = DISPATCH_LEVEL;
//
//     {
//         volatile ULONG *uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *msg = "[KERNEL] Entering KiIdleLoop\r\n";
//         while (*msg) { *uart = (ULONG)*msg++; }
//     }
//
//     /* Jump into the idle loop (never returns) */
//     KiIdleLoop();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeKernel(
//     _In_ PKPROCESS InitProcess,
//     _In_ PKTHREAD InitThread,
//     _In_ PVOID IdleStack,
//     _In_ PKPRCB Prcb,
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     ULONG_PTR PageDirectory[2];
//     ULONG i;
//
//     DPRINT1("ARM64 KiInitializeKernel: Process=%p Thread=%p Stack=%p Prcb=%p\n",
//             InitProcess, InitThread, IdleStack, Prcb);
//
//     /* Set the current MP Master KPRCB to the Boot PRCB */
//     Prcb->MultiThreadSetMaster = Prcb;
//
//     /* Initialize Bugcheck Callback data */
//     InitializeListHead(&KeBugcheckCallbackListHead);
//     InitializeListHead(&KeBugcheckReasonCallbackListHead);
//     KeInitializeSpinLock(&BugCheckCallbackLock);
//
//     /* Initialize the Timer Expiration DPC */
//     KeInitializeDpc(&KiTimerExpireDpc, KiTimerExpiration, NULL);
//     KeSetTargetProcessorDpc(&KiTimerExpireDpc, 0);
//
//     /* Initialize Profiling data */
//     KeInitializeSpinLock(&KiProfileLock);
//     InitializeListHead(&KiProfileListHead);
//     InitializeListHead(&KiProfileSourceListHead);
//
//     /* Loop the timer table */
//     for (i = 0; i < TIMER_TABLE_SIZE; i++)
//     {
//         /* Initialize the list and entries */
//         InitializeListHead(&KiTimerTableListHead[i].Entry);
//         KiTimerTableListHead[i].Time.HighPart = 0xFFFFFFFF;
//         KiTimerTableListHead[i].Time.LowPart = 0;
//     }
//
//     /* Initialize the Swap event and all swap lists */
//     KeInitializeEvent(&KiSwapEvent, SynchronizationEvent, FALSE);
//     InitializeListHead(&KiProcessInSwapListHead);
//     InitializeListHead(&KiProcessOutSwapListHead);
//     InitializeListHead(&KiStackInSwapListHead);
//
//     /* Initialize the mutex for generic DPC calls */
//     ExInitializeFastMutex(&KiGenericCallDpcMutex);
//
//     /* Initialize the syscall table */
//     KeServiceDescriptorTable[0].Base = MainSSDT;
//     KeServiceDescriptorTable[0].Count = NULL;
//     KeServiceDescriptorTable[0].Limit = KiServiceLimit;
//     KeServiceDescriptorTable[1].Limit = 0;
//     KeServiceDescriptorTable[0].Number = MainSSPT;
//
//     /* Copy the current table into the shadow table for win32k */
//     RtlCopyMemory(KeServiceDescriptorTableShadow,
//                   KeServiceDescriptorTable,
//                   sizeof(KeServiceDescriptorTable));
//
//     /* Initialize the Idle Process and the Process Listhead */
//     InitializeListHead(&KiProcessListHead);
//     PageDirectory[0] = 0;
//     PageDirectory[1] = 0;
//     KeInitializeProcess(InitProcess,
//                         0,
//                         MAXULONG_PTR,
//                         PageDirectory,
//                         FALSE);
//     InitProcess->QuantumReset = MAXCHAR;
//
//     /* Initialize the idle thread (the boot thread) properly */
//     KiInitializeHandBuiltThread(InitThread, InitProcess, IdleStack);
//
//     {
//         volatile ULONG *uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *msg = "[KERNEL] Calling ExpInitializeExecutive\r\n";
//         while (*msg) { *uart = (ULONG)*msg++; }
//     }
//
//     /* Initialize the Kernel Executive */
//     ExpInitializeExecutive(0, LoaderBlock);
//
//     /* Update DPC Values in case they got updated by the executive */
//     Prcb->MaximumDpcQueueDepth = KiMaximumDpcQueueDepth;
//     Prcb->MinimumDpcRate = KiMinimumDpcRate;
//     Prcb->AdjustDpcThreshold = KiAdjustDpcThreshold;
//
//     /* If we get here, Phase 0 init succeeded! */
//     {
//         volatile ULONG *uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *msg = "[KERNEL] ExpInitializeExecutive returned!\r\n";
//         while (*msg) { *uart = (ULONG)*msg++; }
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
