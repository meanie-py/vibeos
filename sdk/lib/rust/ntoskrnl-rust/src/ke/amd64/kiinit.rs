use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    _KIPCR,
    PKIPCR,
    PKPCR,
    PKTHREAD,
    PLOADER_PARAMETER_BLOCK,
    PLDR_DATA_TABLE_ENTRY,
    LDR_DATA_TABLE_ENTRY,
    HalInitializeProcessor,
    KeBugCheck,
    KeInitExceptions,
    PLONG64,
};

// Rust-defined globals — import to avoid extern split under LTO=false
use crate::ke::krnlinit::{
    KeLoaderBlock, KiProcessorBlock, KeFeatureBits, KiInitialThread, KiInitialProcess,
    KeProcessorArchitecture, KeProcessorLevel, KeProcessorRevision,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use crate::ke::processor::{KeNumberProcessors, KeActiveProcessors};
use crate::ARM3::sysldr::PsLoadedModuleList;
use super::cpu::{KeI386CpuType, KeI386CpuStep};
use super::stubs::KeXStateLength;

extern "system" {
    fn KiInitializeXStateConfiguration(Cpu: ULONG);
    fn KiSwitchToBootStack(InitialStack: u64) -> !;
    fn KdInitSystem(Phase: u32, LoaderBlock: PLOADER_PARAMETER_BLOCK) -> u8;
    fn KdPollBreakIn() -> u8;
    fn DbgBreakPointWithStatus(Status: u32);
    fn KiReportCpuFeatures(Prcb: PKPRCB);
    fn KiInitializeSegments();
    fn KiSystemCallEntry64();
    fn KiSystemCallEntry32();
    static mut KdDebuggerEnabled: BOOLEAN;
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
}
extern "C" {
    fn KiGetGdtEntry(GdtBase: PKGDTENTRY64, Selector: u16) -> PKGDTENTRY64;
    fn KiGetGdtDescriptorBase(Entry: PKGDTENTRY64) -> PKTSS64;
    fn KiInitGdtEntry(Entry: PKGDTENTRY64, Base: u64, Limit: u32, Type: u8, Dpl: u8);
}
extern "system" {
    fn KiSetProcessorType();
}
extern "system" {
    fn KiGetFeatureBits() -> u64;
}

type PKGDTENTRY64 = *mut u8;
type PKIDTENTRY64 = *mut u8;
type PKTSS64 = *mut KTSS64;

#[repr(C, packed(4))]
struct KTSS64 {
    Reserved0: u32,
    Rsp0: u64,
    Rsp1: u64,
    Rsp2: u64,
    Ist: [u64; 8],
    Reserved1: u64,
    Reserved2: u16,
    IoMapBase: u16,
}
const _: () = assert!(core::mem::size_of::<KTSS64>() == 0x68);

#[repr(C)]
struct KDESCRIPTOR {
    Pad: [u16; 3],
    Limit: u16,
    Base: *mut u8,
}

// KUSER_SHARED_DATA: use the size-verified bindgen layout. A hand-rolled
// shadow struct here previously put ProcessorFeatures at +0x00 and
// NXSupportPolicy at +0x40 (real: +0x274 / +0x2D5) — the +0x40 store landed
// inside NtSystemRoot. Harmless on the BSP (runs before the string exists),
// but the AP's KiInitializeCpu chopped "C:\reactos" to "C:\react" at -smp 2
// (2026-07-11, first -smp 2 bring-up: SMSS 0xC000021A via KnownDlls
// path-not-found).
use ntkrnl_sys::ddk::_KUSER_SHARED_DATA as KUSER_SHARED_DATA;

#[repr(C)]
struct _ETHREAD {
    Tcb: KTHREAD,
}

#[repr(C)]
struct KTHREAD {
    ApcState: _KAPC_STATE,
}

#[repr(C)]
struct _KAPC_STATE {
    Process: PVOID,
}

#[repr(C)]
struct _EPROCESS {
    Pcb: KPROCESS,
}

#[repr(C)]
struct KPROCESS {
    _unused: u8,
}

// From sdk/include/ndk/amd64/ketypes.h — canonical amd64 KF_ values
const KF_SMEP: u64 = 0x00000001;
const KF_RDTSC: u64 = 0x00000002;
const KF_CR4: u64 = 0x00000004;
const KF_CMOV: u64 = 0x00000008;
const KF_GLOBAL_PAGE: u64 = 0x00000010;
const KF_LARGE_PAGE: u64 = 0x00000020;
const KF_MTRR: u64 = 0x00000040;
const KF_CMPXCHG8B: u64 = 0x00000080;
const KF_MMX: u64 = 0x00000100;
const KF_DTS: u64 = 0x00000200;
const KF_PAT: u64 = 0x00000400;
const KF_FXSR: u64 = 0x00000800;
const KF_FAST_SYSCALL: u64 = 0x00001000;
const KF_XMMI: u64 = 0x00002000;
const KF_3DNOW: u64 = 0x00004000;
const KF_XMMI64: u64 = 0x00010000;
const KF_BRANCH: u64 = 0x00020000;
const KF_SSE3: u64 = 0x00080000;
const KF_CMPXCHG16B: u64 = 0x00100000;
const KF_XSTATE: u64 = 0x00800000;
const KF_GENUINE_INTEL: u64 = 0x01000000;
const KF_SLAT: u64 = 0x04000000;
const KF_VIRT_FIRMWARE_ENABLED: u64 = 0x08000000;
const KF_RDWRFSGSBASE: u64 = 0x10000000;
const KF_NX_BIT: u64 = 0x20000000;
const KF_NX_DISABLED: u64 = 0x40000000;
const KF_NX_ENABLED: u64 = 0x80000000;
const KF_RDRAND: u64 = 0x0000000100000000;
const KF_SMAP: u64 = 0x0000000200000000;
const KF_RDTSCP: u64 = 0x0000000400000000;
const KF_XSAVES: u64 = 0x0000004000000000;
const KF_SSSE3: u64 = 0x0000800000000000;
const KF_SSE4_1: u64 = 0x0001000000000000;
const KF_SSE4_2: u64 = 0x0002000000000000;
const KF_AVX: u64 = 0x1000000000000000;
const KF_AVX2: u64 = 0x2000000000000000;
const KF_AVX512F: u64 = 0x4000000000000000;

const REQUIRED_FEATURE_BITS: u64 = KF_RDTSC | KF_CR4 | KF_CMPXCHG8B | KF_XMMI | KF_XMMI64 |
                                    KF_LARGE_PAGE | KF_FAST_SYSCALL | KF_GLOBAL_PAGE |
                                    KF_CMOV | KF_PAT | KF_MMX | KF_FXSR | KF_NX_BIT | KF_MTRR;

// 0xC000, xdk/amd64/ke.h:182. usercall.rs in this same crate already had it right;
// 0x6000 is half a kernel stack, and this constant sizes the allocation.
const KERNEL_STACK_SIZE: usize = 0xC000;
const PCR_MAJOR_VERSION: u32 = 1;
const PCR_MINOR_VERSION: u32 = 1;
const PRCB_MAJOR_VERSION: u16 = 1;
const PRCB_MINOR_VERSION: u16 = 1;
const PRCB_BUILD_UNIPROCESSOR: u16 = 0x2;
const PRCB_BUILD_DEBUG: u16 = 0x1;
const INITIAL_MXCSR: u32 = 0x1F80;

const MSR_GS_BASE: u32 = 0xC0000101;
const MSR_GS_SWAP: u32 = 0xC0000102;
const MSR_LSTAR: u32 = 0xC0000082;
const MSR_CSTAR: u32 = 0xC0000083;
const MSR_STAR: u32 = 0xC0000081;
const MSR_SYSCALL_MASK: u32 = 0xC0000084;
const MSR_EFER: u32 = 0xC0000080;
const MSR_PAT: u32 = 0x277;
const MSR_SCE: u64 = 0x01;
const MSR_NXE: u64 = 0x800;

const CR4_FXSR: u64 = 0x200;
const CR4_XMMEXCPT: u64 = 0x400;
const CR4_XSAVE: u64 = 0x40000;
const CR0_WP: u64 = 0x10000;
const CR0_MP: u64 = 0x02;
const CR0_NE: u64 = 0x20;

const KGDT64_R0_CODE: u64 = 0x10;
const KGDT64_R3_CMCODE: u64 = 0x20; // ndk/amd64/ketypes.h
const KGDT64_SYS_TSS: u16 = 0x40;
const RPL_MASK: u64 = 0x03;

const EFLAGS_IF_MASK: u64 = 0x200;
const EFLAGS_TF: u64 = 0x100;
const EFLAGS_DF: u64 = 0x400;
const EFLAGS_NESTED_TASK: u64 = 0x4000;

const PAT_WB: u64 = 0x06;
const PAT_WC: u64 = 0x01;
const PAT_UCM: u64 = 0x07;
const PAT_UC: u64 = 0x00;

// internal/amd64/ke.h:85: 9 = available 64-bit TSS. It is the 5-bit Bits.Type field;
// Present is set separately by KiInitGdtEntry (0x89 = P|9 was the raw access byte).
const AMD64_TSS: u8 = 0x9;

const NX_SUPPORT_POLICY_ALWAYSON: u32 = 0x01;
const NX_SUPPORT_POLICY_OPTIN: u32 = 0x02;
const NX_SUPPORT_POLICY_OPTOUT: u32 = 0x03;
const NX_SUPPORT_POLICY_ALWAYSOFF: u32 = 0x00;

const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;

const PF_FLOATING_POINT_PRECISION_ERRATA: usize = 0;
const PF_FLOATING_POINT_EMULATED: usize = 1;
const PF_COMPARE_EXCHANGE_DOUBLE: usize = 2;
const PF_MMX_INSTRUCTIONS_AVAILABLE: usize = 3;
const PF_XMMI_INSTRUCTIONS_AVAILABLE: usize = 6;
const PF_3DNOW_INSTRUCTIONS_AVAILABLE: usize = 7;
const PF_RDTSC_INSTRUCTION_AVAILABLE: usize = 8;
const PF_PAE_ENABLED: usize = 9;
const PF_XMMI64_INSTRUCTIONS_AVAILABLE: usize = 10;
const PF_SSE_DAZ_MODE_AVAILABLE: usize = 11;
const PF_NX_ENABLED: usize = 12;
const PF_SSE3_INSTRUCTIONS_AVAILABLE: usize = 13;
const PF_COMPARE_EXCHANGE128: usize = 14;
const PF_COMPARE64_EXCHANGE128: usize = 15;
const PF_CHANNELS_ENABLED: usize = 16;
const PF_XSAVE_ENABLED: usize = 17;
const PF_SECOND_LEVEL_ADDRESS_TRANSLATION: usize = 20;
const PF_VIRT_FIRMWARE_ENABLED: usize = 21;
const PF_RDWRFSGSBASE_AVAILABLE: usize = 22;
const PF_FASTFAIL_AVAILABLE: usize = 23;
const PF_RDRAND_INSTRUCTION_AVAILABLE: usize = 28;
const PF_RDTSCP_INSTRUCTION_AVAILABLE: usize = 32;
const PF_RDPID_INSTRUCTION_AVAILABLE: usize = 33;
const PF_SSSE3_INSTRUCTIONS_AVAILABLE: usize = 36;
const PF_SSE4_1_INSTRUCTIONS_AVAILABLE: usize = 37;
const PF_SSE4_2_INSTRUCTIONS_AVAILABLE: usize = 38;
const PF_AVX_INSTRUCTIONS_AVAILABLE: usize = 39;
const PF_AVX2_INSTRUCTIONS_AVAILABLE: usize = 40;
const PF_AVX512F_INSTRUCTIONS_AVAILABLE: usize = 41;

const DBG_STATUS_CONTROL_C: u32 = 1;

/// Inline-asm wrapper for the x86 `wrmsr` instruction.
#[inline(always)]
unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high);
}

/// Inline-asm wrapper for the x86 `rdmsr` instruction.
#[inline(always)]
unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    core::arch::asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high);
    ((high as u64) << 32) | (low as u64)
}

/// Inline-asm wrapper for reading CR0.
#[inline(always)]
unsafe fn readcr0() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr0", out(reg) val);
    val
}

/// Inline-asm wrapper for writing CR0.
#[inline(always)]
unsafe fn writecr0(val: u64) {
    core::arch::asm!("mov cr0, {}", in(reg) val);
}

/// Inline-asm wrapper for reading CR4.
#[inline(always)]
unsafe fn readcr4() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr4", out(reg) val);
    val
}

/// Inline-asm wrapper for writing CR4.
#[inline(always)]
unsafe fn writecr4(val: u64) {
    core::arch::asm!("mov cr4, {}", in(reg) val);
}

#[no_mangle]
pub static mut FrLdrDbgPrint: Option<unsafe extern "C" fn(*const i8, ...) -> u32> = None;

#[no_mangle]
pub static mut KiFreezeExecutionLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut KiInitialPcr: _KIPCR = unsafe { core::mem::zeroed() };

#[repr(align(16))]
struct AlignedStack([u8; KERNEL_STACK_SIZE]);

static mut KiP0BootStackData: AlignedStack = AlignedStack([0u8; KERNEL_STACK_SIZE]);
static mut KiP0DoubleFaultStackData: AlignedStack = AlignedStack([0u8; KERNEL_STACK_SIZE]);

#[no_mangle]
pub static mut KiP0BootStack: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut KiP0DoubleFaultStack: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut BootCycles: u64 = 0;
#[no_mangle]
pub static mut BootCyclesEnd: u64 = 0;

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitMachineDependent() {
    if KeFeatureBits & KF_LARGE_PAGE != 0 {
    }

    if KeFeatureBits & KF_GLOBAL_PAGE != 0 {
    }

    if KeFeatureBits & KF_MTRR != 0 {
    }

    if KeFeatureBits & KF_PAT != 0 {
    }
}

unsafe fn KiInitializePcr(
    Pcr: PKIPCR,
    ProcessorNumber: u32,
    GdtBase: PKGDTENTRY64,
    IdtBase: PKIDTENTRY64,
    TssBase: PKTSS64,
    IdleThread: PKTHREAD,
    DpcStack: PVOID,
) {
    RtlZeroMemory(Pcr as PVOID, core::mem::size_of::<_KIPCR>());

    (*Pcr).__bindgen_anon_1.__bindgen_anon_1.Self_ = Pcr as PKPCR;
    (*Pcr).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb = core::ptr::addr_of_mut!((*Pcr).Prcb);

    (*Pcr).MajorVersion = PCR_MAJOR_VERSION as u16;
    (*Pcr).MinorVersion = PCR_MINOR_VERSION as u16;

    (*Pcr).Prcb.MajorVersion = PRCB_MAJOR_VERSION;
    (*Pcr).Prcb.MinorVersion = PRCB_MINOR_VERSION;

    (*Pcr).Prcb.BuildType = 0;
    #[cfg(not(feature = "smp"))]
    {
        (*Pcr).Prcb.BuildType |= PRCB_BUILD_UNIPROCESSOR as u8;
    }
    // C kernel uses `#if DBG` which is always TRUE in our Debug build.
    // Rust `#[cfg(debug_assertions)]` is NOT set in --release mode.
    // Must match HAL's HalpBuildType or we get MISMATCHED_HAL bugcheck.
    (*Pcr).Prcb.BuildType |= PRCB_BUILD_DEBUG as u8;

    (*Pcr).Prcb.Number = ProcessorNumber as u8;
    (*Pcr).Prcb.SetMember = 1u64 << ProcessorNumber;

    (*Pcr).__bindgen_anon_1.__bindgen_anon_1.GdtBase = GdtBase as *mut _;
    (*Pcr).IdtBase = IdtBase as *mut _;

    (*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase = TssBase as *mut _;

    (*Pcr).Prcb.RspBase = (*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).Rsp0;

    (*Pcr).Prcb.DpcStack = DpcStack;

    (*Pcr).Prcb.MultiThreadProcessorSet = (*Pcr).Prcb.SetMember;

    (*Pcr).Prcb.ProcessorState.SpecialRegisters.KernelDr6 = 0;
    (*Pcr).Prcb.ProcessorState.SpecialRegisters.KernelDr7 = 0;

    (*Pcr).Prcb.MxCsr = INITIAL_MXCSR;

    (*Pcr).Prcb.CurrentThread = IdleThread;

    (*Pcr).Irql = PASSIVE_LEVEL;
}

#[no_mangle]
pub unsafe extern "system" fn KiInitializeCpu(Pcr: PKIPCR) {
    let Pat: u64;
    let FeatureBits: u64;

    KiInitializeSegments();

    wrmsr(MSR_GS_BASE, Pcr as u64);
    wrmsr(MSR_GS_SWAP, Pcr as u64);

    KiSetProcessorType();

    FeatureBits = KiGetFeatureBits();

    if (FeatureBits & REQUIRED_FEATURE_BITS) != REQUIRED_FEATURE_BITS {
        if let Some(print) = FrLdrDbgPrint {
            print(
                b"CPU doesn't have needed features! Has: 0x%x, required: 0x%x\n\0".as_ptr() as *const i8,
                FeatureBits,
                REQUIRED_FEATURE_BITS,
            );
        }
        KeBugCheck(0); // nolint: bugcheck-code (faithful: C kiinit.c says KeBugCheck(0))
    }

    (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_ALWAYSON as u8);
    let mut FeatureBits = FeatureBits | KF_NX_ENABLED;

    (*Pcr).Prcb.FeatureBits = FeatureBits as u32;
    (*Pcr).Prcb.FeatureBitsHigh = (FeatureBits >> 32) as u32;

    // Enable fx save restore support
    writecr4(readcr4() | CR4_FXSR);

    // Enable XMMI exceptions
    writecr4(readcr4() | CR4_XMMEXCPT);

    // Enable Write-Protection
    writecr0(readcr0() | CR0_WP);

    // Disable fpu monitoring
    writecr0(readcr0() & !CR0_MP);

    // Disable x87 fpu exceptions
    writecr0(readcr0() & !CR0_NE);

    // Check if XSAVE is supported, enable CR4.OSXSAVE
    if FeatureBits & KF_XSTATE != 0 {
        writecr4(readcr4() | CR4_XSAVE);
    }

    core::arch::asm!("lldt ax", in("ax") 0u16);

    wrmsr(MSR_LSTAR, KiSystemCallEntry64 as u64);
    wrmsr(MSR_CSTAR, KiSystemCallEntry32 as u64);

    wrmsr(
        MSR_STAR,
        (KGDT64_R0_CODE << 32) | ((KGDT64_R3_CMCODE | RPL_MASK) << 48),
    );

    wrmsr(
        MSR_SYSCALL_MASK,
        EFLAGS_IF_MASK | EFLAGS_TF | EFLAGS_DF | EFLAGS_NESTED_TASK,
    );

    wrmsr(
        MSR_EFER,
        rdmsr(MSR_EFER) | MSR_SCE | MSR_NXE,
    );

    Pat = (PAT_WB << 0) | (PAT_WC << 8) | (PAT_UCM << 16) | (PAT_UC << 24) |
          (PAT_WB << 32) | (PAT_WC << 40) | (PAT_UCM << 48) | (PAT_UC << 56);
    wrmsr(MSR_PAT, Pat);

    core::arch::asm!("ldmxcsr [{}]", in(reg) &INITIAL_MXCSR, options(nostack));

    KeSetCurrentIrql(PASSIVE_LEVEL);
}

unsafe fn KiInitializeTss(
    Pcr: PKIPCR,
    Tss: PKTSS64,
    InitialStack: PVOID,
    DoubleFaultStack: PVOID,
    NmiStack: PVOID,
) {
    let TssEntry: PKGDTENTRY64;

    TssEntry = KiGetGdtEntry((*Pcr).__bindgen_anon_1.__bindgen_anon_1.GdtBase as PKGDTENTRY64, KGDT64_SYS_TSS);

    KiInitGdtEntry(TssEntry, Tss as u64, core::mem::size_of::<KTSS64>() as u32, AMD64_TSS, 0);

    RtlZeroMemory(Tss as PVOID, core::mem::size_of::<KTSS64>());

    (*Tss).IoMapBase = 0x68;

    (*Tss).Rsp0 = InitialStack as u64;

    (*Tss).Ist[1] = DoubleFaultStack as u64;

    (*Tss).Ist[2] = DoubleFaultStack as u64;

    (*Tss).Ist[3] = NmiStack as u64;
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeProcessorBootStructures(
    ProcessorNumber: u32,
    Pcr: PKIPCR,
    GdtBase: PKGDTENTRY64,
    IdtBase: PKIDTENTRY64,
    TssBase: PKTSS64,
    IdleThread: PKTHREAD,
    KernelStack: PVOID,
    DpcStack: PVOID,
    DoubleFaultStack: PVOID,
    NmiStack: PVOID,
) {
    let IdleThread = opaque_ptr(IdleThread);
    let KernelStack = opaque_ptr(KernelStack);
    let DpcStack = opaque_ptr(DpcStack);
    let DoubleFaultStack = opaque_ptr(DoubleFaultStack);
    let NmiStack = opaque_ptr(NmiStack);
    KiInitializePcr(
        Pcr,
        ProcessorNumber,
        GdtBase,
        IdtBase,
        TssBase,
        IdleThread,
        DpcStack,
    );

    KiInitializeTss(Pcr, TssBase, KernelStack, DoubleFaultStack, NmiStack);
}

// CODE_SEG("INIT")
unsafe fn KiInitializeP0BootStructures(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    // Initialize boot stack pointers — in C these are initialized at declaration
    // time (PVOID KiP0BootStack = &KiP0BootStackData[KERNEL_STACK_SIZE]), but Rust
    // can't take the address of a static in a const initializer.
    KiP0BootStack = core::ptr::addr_of_mut!(KiP0BootStackData) as *mut u8
        as *mut core::ffi::c_void;
    KiP0BootStack = (KiP0BootStack as *mut u8).add(KERNEL_STACK_SIZE) as PVOID; // nolint: magic-offset (stack size, not a field offset)
    KiP0DoubleFaultStack = core::ptr::addr_of_mut!(KiP0DoubleFaultStackData) as *mut u8
        as *mut core::ffi::c_void;
    KiP0DoubleFaultStack = (KiP0DoubleFaultStack as *mut u8).add(KERNEL_STACK_SIZE) as PVOID; // nolint: magic-offset (stack size, not a field offset)

    let mut GdtDescriptor = KDESCRIPTOR {
        Pad: [0u16; 3],
        Limit: 0,
        Base: core::ptr::null_mut(),
    };
    let mut IdtDescriptor = KDESCRIPTOR {
        Pad: [0u16; 3],
        Limit: 0,
        Base: core::ptr::null_mut(),
    };
    let TssEntry: PKGDTENTRY64;
    let TssBase: PKTSS64;

    (*LoaderBlock).KernelStack = KiP0BootStack as u64;
    (*LoaderBlock).Thread = core::ptr::addr_of_mut!(KiInitialThread) as u64;
    (*LoaderBlock).Process = core::ptr::addr_of_mut!(KiInitialProcess) as u64;
    (*LoaderBlock).Prcb = core::ptr::addr_of_mut!(KiInitialPcr.Prcb) as u64;

    core::arch::asm!(
        "sgdt [{}]",
        in(reg) core::ptr::addr_of_mut!(GdtDescriptor.Limit),
    );
    core::arch::asm!(
        "sidt [{}]",
        in(reg) core::ptr::addr_of_mut!(IdtDescriptor.Limit),
    );

    TssEntry = KiGetGdtEntry(GdtDescriptor.Base, KGDT64_SYS_TSS);
    TssBase = KiGetGdtDescriptorBase(TssEntry);

    KiInitializeProcessorBootStructures(
        0,
        core::ptr::addr_of_mut!(KiInitialPcr),
        GdtDescriptor.Base,
        IdtDescriptor.Base,
        TssBase,
        core::ptr::addr_of_mut!(KiInitialThread.Tcb) as PKTHREAD,
        KiP0BootStack,
        KiP0DoubleFaultStack,
        KiP0DoubleFaultStack,
        KiP0DoubleFaultStack,
    );
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeKernelMachineDependent(
    Prcb: PKPRCB,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let FeatureBits = KeFeatureBits;

    KeI386CpuType = (*Prcb).CpuType as u32;
    KeI386CpuStep = (*Prcb).CpuStep as u32;
    KeProcessorArchitecture = PROCESSOR_ARCHITECTURE_AMD64;
    KeProcessorLevel = (*Prcb).CpuType as u16;
    if (*Prcb).CpuID != 0 {
        KeProcessorRevision = (*Prcb).CpuStep as u16;
    }

    (*SharedUserData).ProcessorFeatures[PF_FLOATING_POINT_PRECISION_ERRATA] = 0u8;
    (*SharedUserData).ProcessorFeatures[PF_FLOATING_POINT_EMULATED] = 0u8;
    (*SharedUserData).ProcessorFeatures[PF_COMPARE_EXCHANGE_DOUBLE] = 1u8;
    (*SharedUserData).ProcessorFeatures[PF_MMX_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_MMX != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_XMMI_INSTRUCTIONS_AVAILABLE] =
        if (FeatureBits & KF_FXSR != 0) && (FeatureBits & KF_XMMI != 0) { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_3DNOW_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_3DNOW != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_RDTSC_INSTRUCTION_AVAILABLE] = 1u8;
    (*SharedUserData).ProcessorFeatures[PF_PAE_ENABLED] = 1u8;
    (*SharedUserData).ProcessorFeatures[PF_XMMI64_INSTRUCTIONS_AVAILABLE] =
        if (FeatureBits & KF_FXSR != 0) && (FeatureBits & KF_XMMI64 != 0) { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_SSE_DAZ_MODE_AVAILABLE] = 0u8;
    (*SharedUserData).ProcessorFeatures[PF_NX_ENABLED] = 1u8;
    (*SharedUserData).ProcessorFeatures[PF_SSE3_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_SSE3 != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_COMPARE_EXCHANGE128] =
        if FeatureBits & KF_CMPXCHG16B != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_COMPARE64_EXCHANGE128] = 0u8;
    (*SharedUserData).ProcessorFeatures[PF_CHANNELS_ENABLED] = 0u8;
    (*SharedUserData).ProcessorFeatures[PF_XSAVE_ENABLED] =
        if FeatureBits & KF_XSTATE != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_SECOND_LEVEL_ADDRESS_TRANSLATION] =
        if FeatureBits & KF_SLAT != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_VIRT_FIRMWARE_ENABLED] =
        if FeatureBits & KF_VIRT_FIRMWARE_ENABLED != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_RDWRFSGSBASE_AVAILABLE] =
        if FeatureBits & KF_RDWRFSGSBASE != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_FASTFAIL_AVAILABLE] = 1u8;
    (*SharedUserData).ProcessorFeatures[PF_RDRAND_INSTRUCTION_AVAILABLE] =
        if FeatureBits & KF_RDRAND != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_RDTSCP_INSTRUCTION_AVAILABLE] =
        if FeatureBits & KF_RDTSCP != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_RDPID_INSTRUCTION_AVAILABLE] = 0u8;
    (*SharedUserData).ProcessorFeatures[PF_SSSE3_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_SSSE3 != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_SSE4_1_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_SSE4_1 != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_SSE4_2_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_SSE4_2 != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_AVX_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_AVX != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_AVX2_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_AVX2 != 0 { 1u8 } else { 0u8 };
    (*SharedUserData).ProcessorFeatures[PF_AVX512F_INSTRUCTIONS_AVAILABLE] =
        if FeatureBits & KF_AVX512F != 0 { 1u8 } else { 0u8 };

    (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_OPTIN as u8);

    let load_options = (*KeLoaderBlock).LoadOptions as *const u8;
    if !load_options.is_null() {
        if strstr(load_options, b"NOEXECUTE=ALWAYSON\0".as_ptr()) {
            (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_ALWAYSON as u8);
            (*Prcb).FeatureBits |= KF_NX_ENABLED as u32;
        } else if strstr(load_options, b"NOEXECUTE=OPTOUT\0".as_ptr()) {
            (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_OPTOUT as u8);
            (*Prcb).FeatureBits |= KF_NX_ENABLED as u32;
        } else if strstr(load_options, b"NOEXECUTE=OPTIN\0".as_ptr())
            || strstr(load_options, b"NOEXECUTE\0".as_ptr())
        {
            (*Prcb).FeatureBits |= KF_NX_ENABLED as u32;
        } else if strstr(load_options, b"NOEXECUTE=ALWAYSOFF\0".as_ptr())
            || strstr(load_options, b"EXECUTE\0".as_ptr())
        {
            (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_ALWAYSOFF as u8);
            (*Prcb).FeatureBits |= KF_NX_DISABLED as u32;
        }
    }

    // KiReportCpuFeatures (ke/amd64/cpu.c:354) is NOT TRANSLATED — ke/amd64/cpu.rs
    // carries only its signature as a comment, and the C is cohab-dead, so the
    // symbol exists in no artifact. The call survived because it sits behind
    // `cfg(debug_assertions)`, which the release profile never set: an undefined
    // symbol that nothing ever tried to link. Arming ex/ntoskrnl assertions made
    // it a hard LINK_FAIL on both x86 arches.
    //
    // Left disabled rather than stubbed: a body invented here would report CPU
    // features we never decoded. Translating it is owed work, tracked in the
    // roadmap with the other never-compiled `cfg(debug_assertions)` blocks.
    // #[cfg(debug_assertions)]
    // KiReportCpuFeatures(Prcb);
}

static mut LdrCoreEntries: [LDR_DATA_TABLE_ENTRY; 3] = unsafe { core::mem::zeroed() };

unsafe fn KiInitModuleList(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut i: u32;

    InitializeListHead(core::ptr::addr_of_mut!(PsLoadedModuleList));

    Entry = (*LoaderBlock).LoadOrderListHead.Flink;
    i = 0;
    while Entry != core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead) && i < 3 {
        LdrEntry = crate::CONTAINING_RECORD!(Entry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as PLDR_DATA_TABLE_ENTRY;

        LdrCoreEntries[i as usize] = *LdrEntry;

        InsertTailList(
            core::ptr::addr_of_mut!(PsLoadedModuleList),
            core::ptr::addr_of_mut!(LdrCoreEntries[i as usize].InLoadOrderLinks),
        );

        Entry = (*Entry).Flink;
        i += 1;
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSystemStartup(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> ! {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let Cpu: i8;
    let InitialThread: PKTHREAD;
    let InitialStack: u64;
    let Pcr: PKIPCR;

    BootCycles = core::arch::x86_64::_rdtsc();

    FrLdrDbgPrint = core::mem::transmute((*LoaderBlock).u.I386.CommonDataArea);

    Cpu = KeNumberProcessors as i8;
    KeNumberProcessors += 1;

    if Cpu == 0 {
        KeLoaderBlock = LoaderBlock;

        KiInitializeP0BootStructures(LoaderBlock);
    }

    Pcr = crate::CONTAINING_RECORD!((*LoaderBlock).Prcb as PVOID, _KIPCR, Prcb) as PKIPCR;

    let prcb_ptr = (core::ptr::addr_of!(KiProcessorBlock) as *mut PKPRCB).add(Cpu as usize);
    *prcb_ptr = core::ptr::addr_of_mut!((*Pcr).Prcb);

    InitialThread = (*LoaderBlock).Thread as PKTHREAD;

    (*InitialThread).__bindgen_anon_1.ApcState.Process = (*LoaderBlock).Process as *mut _;

    KiInitializeCpu(Pcr);

    if Cpu == 0 {
        KeFeatureBits = ((*Pcr).Prcb.FeatureBitsHigh as u64) << 32 | (*Pcr).Prcb.FeatureBits as u64;

        KiInitModuleList(LoaderBlock);

        KeInitExceptions();
    }

    while InterlockedBitTestAndSet64(
        core::ptr::addr_of_mut!(KiFreezeExecutionLock) as PLONG64,
        0,
    ) != 0
    {
        while (*(core::ptr::addr_of!(KiFreezeExecutionLock) as *const KSPIN_LOCK)) & 1 != 0 {}
    }

    HalInitializeProcessor(Cpu as u32, KeLoaderBlock);

    if Cpu == 0 {
        // KD init MUST follow HalInitializeProcessor: a PCI-based KD transport's
        // module depends on the functions HalpRegisterKdSupportFunctions() registers,
        // so initialising the debugger first leaves those transports dead. i386 and
        // Win7 x64 already order it this way. Upstream-Commit: 8aac5c4c872
        let _kd_ret = KdInitSystem(0, KeLoaderBlock);

        if KdPollBreakIn() != 0 {
            DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
        }
    }

    // A CPU must not advertise itself in KeActiveProcessors until it can
    // actually run work — the dispatcher / DPC targeting iterates this mask,
    // and routing a ready thread to a CPU that never schedules wedges the
    // BSP (observed 2026-07-10 with the M2 parked AP). The BSP sets its bit
    // here (it proceeds straight into full kernel init); an AP sets its bit
    // in KiSystemStartupBootStack right before entering KiIdleLoop.
    #[cfg(not(feature = "smp"))]
    {
        KeActiveProcessors |= 1u64 << Cpu;
    }
    #[cfg(feature = "smp")]
    if Cpu == 0 {
        KeActiveProcessors |= 1u64 << Cpu;
    }

    InterlockedAnd64(core::ptr::addr_of_mut!(KiFreezeExecutionLock) as PLONG64, 0);

    KfRaiseIrql(HIGH_LEVEL);

    if Cpu == 0 {
        KiInitializeKernelMachineDependent(core::ptr::addr_of_mut!((*Pcr).Prcb), LoaderBlock);
    }

    KiInitializeXStateConfiguration(Cpu as ULONG);

    InitialStack = ALIGN_DOWN_BY(((*LoaderBlock).KernelStack - KeXStateLength as u64) as usize, 64) as u64;

    // M3: the AP falls through into KiSwitchToBootStack →
    // KiSystemStartupBootStack, whose Number!=0 branch runs the hand-built
    // idle-thread init and drops into KiIdleLoop. The LoaderBlock->Prcb = 0
    // release (which un-blocks KeStartAllProcessors on the BSP) happens
    // there, at the proper NT point — after every per-AP LoaderBlock field
    // has been consumed. (M2 parked here instead; the park is retired.)
    #[cfg(feature = "smp")]
    if Cpu != 0 {
        crate::dbg_print!(b"[SMP] AP %u through KiSystemStartup, entering boot stack\n\0", Cpu as u32);
    }

    KiSwitchToBootStack(InitialStack);
}

unsafe fn strstr(haystack: *const u8, needle: *const u8) -> bool {
    let mut h = haystack;
    let mut n = needle;
    
    if needle.is_null() || *needle == 0 {
        return true;
    }
    
    while *h != 0 {
        let mut h2 = h;
        n = needle;
        
        while *h2 != 0 && *n != 0 && *h2 == *n {
            h2 = h2.add(1);
            n = n.add(1);
        }
        
        if *n == 0 {
            return true;
        }
        
        h = h.add(1);
    }
    
    false
}

#[no_mangle]
pub unsafe extern "C" fn __chkstk() {
}

#[no_mangle]
pub unsafe extern "system" fn KeSetCurrentIrql(irql: u8) {
}

// KfRaiseIrql — provided by shim_exports.rs (correct cdecl + return value)

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/amd64/kiinit.c
//  * PURPOSE:         Kernel Initialization for x86 CPUs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define REQUIRED_FEATURE_BITS (KF_RDTSC|KF_CR4|KF_CMPXCHG8B|KF_XMMI|KF_XMMI64| \
//                                KF_LARGE_PAGE|KF_FAST_SYSCALL|KF_GLOBAL_PAGE| \
//                                KF_CMOV|KF_PAT|KF_MMX|KF_FXSR|KF_NX_BIT|KF_MTRR)
//
// /* GLOBALS *******************************************************************/
//
// /* Function pointer for early debug prints */
// ULONG (*FrLdrDbgPrint)(const char *Format, ...);
//
// /* Spinlocks used only on X86 */
// KSPIN_LOCK KiFreezeExecutionLock;
//
//
// KIPCR KiInitialPcr;
//
// /* Boot and double-fault/NMI/DPC stack */
// UCHAR DECLSPEC_ALIGN(16) KiP0BootStackData[KERNEL_STACK_SIZE] = {0};
// UCHAR DECLSPEC_ALIGN(16) KiP0DoubleFaultStackData[KERNEL_STACK_SIZE] = {0};
// PVOID KiP0BootStack = &KiP0BootStackData[KERNEL_STACK_SIZE];
// PVOID KiP0DoubleFaultStack = &KiP0DoubleFaultStackData[KERNEL_STACK_SIZE];
//
// ULONGLONG BootCycles, BootCyclesEnd;
//
// void KiInitializeSegments();
// void KiSystemCallEntry64();
// void KiSystemCallEntry32();
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitMachineDependent(VOID)
// {
//     /* Check for large page support */
//     if (KeFeatureBits & KF_LARGE_PAGE)
//     {
//         /* FIXME: Support this */
//         DPRINT("Large Page support detected but not yet taken advantage of!\n");
//     }
//
//     /* Check for global page support */
//     if (KeFeatureBits & KF_GLOBAL_PAGE)
//     {
//         /* FIXME: Support this */
//         DPRINT("Global Page support detected but not yet taken advantage of!\n");
//     }
//
//     /* Check if we have MTRR */
//     if (KeFeatureBits & KF_MTRR)
//     {
//         /* FIXME: Support this */
//         DPRINT("MTRR support detected but not yet taken advantage of!\n");
//     }
//
//     /* Check for PAT and/or MTRR support */
//     if (KeFeatureBits & KF_PAT)
//     {
//         /* FIXME: Support this */
//         DPRINT("PAT support detected but not yet taken advantage of!\n");
//     }
//
// //        /* Allocate the IOPM save area */
// //        Ki386IopmSaveArea = ExAllocatePoolWithTag(PagedPool,
// //                                                  IOPM_SIZE,
// //                                                  '  eK');
// //        if (!Ki386IopmSaveArea)
// //        {
// //            /* Bugcheck. We need this for V86/VDM support. */
// //            KeBugCheckEx(NO_PAGES_AVAILABLE, 2, IOPM_SIZE, 0, 0);
// //        }
//
// }
//
// static
// VOID
// KiInitializePcr(
//     _Out_ PKIPCR Pcr,
//     _In_ ULONG ProcessorNumber,
//     _In_ PKGDTENTRY64 GdtBase,
//     _In_ PKIDTENTRY64 IdtBase,
//     _In_ PKTSS64 TssBase,
//     _In_ PKTHREAD IdleThread,
//     _In_ PVOID DpcStack)
// {
//     /* Zero out the PCR */
//     RtlZeroMemory(Pcr, sizeof(KIPCR));
//
//     /* Set pointers to ourselves */
//     Pcr->Self = (PKPCR)Pcr;
//     Pcr->CurrentPrcb = &Pcr->Prcb;
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
//     /* Set GDT and IDT base */
//     Pcr->GdtBase = GdtBase;
//     Pcr->IdtBase = IdtBase;
//
//     /* Set TssBase */
//     Pcr->TssBase = TssBase;
//
//     Pcr->Prcb.RspBase = Pcr->TssBase->Rsp0; // FIXME
//
//     /* Set DPC Stack */
//     Pcr->Prcb.DpcStack = DpcStack;
//
//     /* Setup the processor set */
//     Pcr->Prcb.MultiThreadProcessorSet = Pcr->Prcb.SetMember;
//
//     /* Clear DR6/7 to cleanup bootloader debugging */
//     Pcr->Prcb.ProcessorState.SpecialRegisters.KernelDr6 = 0;
//     Pcr->Prcb.ProcessorState.SpecialRegisters.KernelDr7 = 0;
//
//     /* Initialize MXCSR (all exceptions masked) */
//     Pcr->Prcb.MxCsr = INITIAL_MXCSR;
//
//     /* Set the Current Thread */
//     Pcr->Prcb.CurrentThread = IdleThread;
//
//     /* Start us out at PASSIVE_LEVEL */
//     Pcr->Irql = PASSIVE_LEVEL;
// }
//
// VOID
// NTAPI
// KiInitializeCpu(PKIPCR Pcr)
// {
//     ULONG64 Pat;
//     ULONG64 FeatureBits;
//
//     /* Initialize gs */
//     KiInitializeSegments();
//
//     /* Set GS base */
//     __writemsr(MSR_GS_BASE, (ULONG64)Pcr);
//     __writemsr(MSR_GS_SWAP, (ULONG64)Pcr);
//
//     /* Detect and set the CPU Type */
//     KiSetProcessorType();
//
//     /* Get the processor features for this CPU */
//     FeatureBits = KiGetFeatureBits();
//
//     /* Check if we support all needed features */
//     if ((FeatureBits & REQUIRED_FEATURE_BITS) != REQUIRED_FEATURE_BITS)
//     {
//         /* If not, bugcheck system */
//         FrLdrDbgPrint("CPU doesn't have needed features! Has: 0x%x, required: 0x%x\n",
//                 FeatureBits, REQUIRED_FEATURE_BITS);
//         KeBugCheck(0);
//     }
//
//     /* Set DEP to always on */
//     SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_ALWAYSON;
//     FeatureBits |= KF_NX_ENABLED;
//
//     /* Save feature bits */
//     Pcr->Prcb.FeatureBits = (ULONG)FeatureBits;
//     Pcr->Prcb.FeatureBitsHigh = FeatureBits >> 32;
//
//     /* Enable fx save restore support */
//     __writecr4(__readcr4() | CR4_FXSR);
//
//     /* Enable XMMI exceptions */
//     __writecr4(__readcr4() | CR4_XMMEXCPT);
//
//     /* Enable Write-Protection */
//     __writecr0(__readcr0() | CR0_WP);
//
//     /* Disable fpu monitoring */
//     __writecr0(__readcr0() & ~CR0_MP);
//
//     /* Disable x87 fpu exceptions */
//     __writecr0(__readcr0() & ~CR0_NE);
//
//     /* Check if XSAVE is supported */
//     if (FeatureBits & KF_XSTATE)
//     {
//         /* Enable CR4.OSXSAVE[Bit 18] */
//         __writecr4(__readcr4() | CR4_XSAVE);
//     }
//
//     /* LDT is unused */
//     __lldt(0);
//
//     /* Set the systemcall entry points */
//     __writemsr(MSR_LSTAR, (ULONG64)KiSystemCallEntry64);
//     __writemsr(MSR_CSTAR, (ULONG64)KiSystemCallEntry32);
//
//     __writemsr(MSR_STAR, ((ULONG64)KGDT64_R0_CODE << 32) |
//                          ((ULONG64)(KGDT64_R3_CMCODE|RPL_MASK) << 48));
//
//     /* Set the flags to be cleared when doing a syscall */
//     __writemsr(MSR_SYSCALL_MASK, EFLAGS_IF_MASK | EFLAGS_TF | EFLAGS_DF | EFLAGS_NESTED_TASK);
//
//     /* Enable syscall instruction and no-execute support */
//     __writemsr(MSR_EFER, __readmsr(MSR_EFER) | MSR_SCE | MSR_NXE);
//
//     /* LSTAR and syscall MSRs configured above */
//
//     /* Initialize the PAT */
//     Pat = (PAT_WB << 0)  | (PAT_WC << 8) | (PAT_UCM << 16) | (PAT_UC << 24) |
//           (PAT_WB << 32) | (PAT_WC << 40) | (PAT_UCM << 48) | (PAT_UC << 56);
//     __writemsr(MSR_PAT, Pat);
//
//     /* Initialize MXCSR */
//     _mm_setcsr(INITIAL_MXCSR);
//
//     KeSetCurrentIrql(PASSIVE_LEVEL);
// }
//
// static
// VOID
// KiInitializeTss(
//     _In_ PKIPCR Pcr,
//     _Out_ PKTSS64 Tss,
//     _In_ PVOID InitialStack,
//     _In_ PVOID DoubleFaultStack,
//     _In_ PVOID NmiStack)
// {
//     PKGDTENTRY64 TssEntry;
//
//     /* Get pointer to the GDT entry */
//     TssEntry = KiGetGdtEntry(Pcr->GdtBase, KGDT64_SYS_TSS);
//
//     /* Initialize the GDT entry */
//     KiInitGdtEntry(TssEntry, (ULONG64)Tss, sizeof(KTSS64), AMD64_TSS, 0);
//
//     /* Zero out the TSS */
//     RtlZeroMemory(Tss, sizeof(KTSS64));
//
//     /* FIXME: I/O Map? */
//     Tss->IoMapBase = 0x68;
//
//     /* Setup ring 0 stack pointer */
//     Tss->Rsp0 = (ULONG64)InitialStack;
//
//     /* Setup a stack for Double Fault Traps */
//     Tss->Ist[1] = (ULONG64)DoubleFaultStack;
//
//     /* Setup a stack for CheckAbort Traps */
//     Tss->Ist[2] = (ULONG64)DoubleFaultStack;
//
//     /* Setup a stack for NMI Traps */
//     Tss->Ist[3] = (ULONG64)NmiStack;
// }
//
// CODE_SEG("INIT")
// VOID
// KiInitializeProcessorBootStructures(
//     _In_ ULONG ProcessorNumber,
//     _Out_ PKIPCR Pcr,
//     _In_ PKGDTENTRY64 GdtBase,
//     _In_ PKIDTENTRY64 IdtBase,
//     _In_ PKTSS64 TssBase,
//     _In_ PKTHREAD IdleThread,
//     _In_ PVOID KernelStack,
//     _In_ PVOID DpcStack,
//     _In_ PVOID DoubleFaultStack,
//     _In_ PVOID NmiStack)
// {
//     /* Initialize the PCR */
//     KiInitializePcr(Pcr,
//                     ProcessorNumber,
//                     GdtBase,
//                     IdtBase,
//                     TssBase,
//                     IdleThread,
//                     DpcStack);
//
//
//     /* Setup the TSS descriptor and entries */
//     KiInitializeTss(Pcr,
//                     TssBase,
//                     KernelStack,
//                     DoubleFaultStack,
//                     NmiStack);
// }
//
// CODE_SEG("INIT")
// static
// VOID
// KiInitializeP0BootStructures(
//     _Inout_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     KDESCRIPTOR GdtDescriptor = {{0},0,0}, IdtDescriptor = {{0},0,0};
//     PKGDTENTRY64 TssEntry;
//     PKTSS64 TssBase;
//
//     /* Set the initial stack, idle thread and process for processor 0 */
//     LoaderBlock->KernelStack = (ULONG_PTR)KiP0BootStack;
//     LoaderBlock->Thread = (ULONG_PTR)&KiInitialThread;
//     LoaderBlock->Process = (ULONG_PTR)&KiInitialProcess.Pcb;
//     LoaderBlock->Prcb = (ULONG_PTR)&KiInitialPcr.Prcb;
//
//     /* Get GDT and IDT descriptors */
//     __sgdt(&GdtDescriptor.Limit);
//     __sidt(&IdtDescriptor.Limit);
//
//     /* Get the boot TSS from the GDT */
//     TssEntry = KiGetGdtEntry(GdtDescriptor.Base, KGDT64_SYS_TSS);
//     TssBase = KiGetGdtDescriptorBase(TssEntry);
//
//     /* Initialize PCR and TSS */
//     KiInitializeProcessorBootStructures(0,
//                                         &KiInitialPcr,
//                                         GdtDescriptor.Base,
//                                         IdtDescriptor.Base,
//                                         TssBase,
//                                         &KiInitialThread.Tcb,
//                                         KiP0BootStack,
//                                         KiP0DoubleFaultStack,
//                                         KiP0DoubleFaultStack,
//                                         KiP0DoubleFaultStack);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeKernelMachineDependent(
//     IN PKPRCB Prcb,
//     IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     ULONG64 FeatureBits = KeFeatureBits;
//
//     /* Set boot-level flags */
//     KeI386CpuType = Prcb->CpuType;
//     KeI386CpuStep = Prcb->CpuStep;
//     KeProcessorArchitecture = PROCESSOR_ARCHITECTURE_AMD64;
//     KeProcessorLevel = (USHORT)Prcb->CpuType;
//     if (Prcb->CpuID)
//         KeProcessorRevision = Prcb->CpuStep;
//
//     /* Set basic CPU Features that user mode can read */
//     SharedUserData->ProcessorFeatures[PF_FLOATING_POINT_PRECISION_ERRATA] = FALSE;
//     SharedUserData->ProcessorFeatures[PF_FLOATING_POINT_EMULATED] = FALSE;
//     SharedUserData->ProcessorFeatures[PF_COMPARE_EXCHANGE_DOUBLE] = TRUE;
//     SharedUserData->ProcessorFeatures[PF_MMX_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_MMX) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_XMMI_INSTRUCTIONS_AVAILABLE] =
//         ((FeatureBits & KF_FXSR) && (FeatureBits & KF_XMMI)) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_3DNOW_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_3DNOW) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_RDTSC_INSTRUCTION_AVAILABLE] = TRUE;
//     SharedUserData->ProcessorFeatures[PF_PAE_ENABLED] = TRUE; // ???
//     SharedUserData->ProcessorFeatures[PF_XMMI64_INSTRUCTIONS_AVAILABLE] =
//         ((FeatureBits & KF_FXSR) && (FeatureBits & KF_XMMI64)) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_SSE_DAZ_MODE_AVAILABLE] = FALSE; // ???
//     SharedUserData->ProcessorFeatures[PF_NX_ENABLED] = TRUE;
//     SharedUserData->ProcessorFeatures[PF_SSE3_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_SSE3) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_COMPARE_EXCHANGE128] =
//         (FeatureBits & KF_CMPXCHG16B) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_COMPARE64_EXCHANGE128] = FALSE; // ???
//     SharedUserData->ProcessorFeatures[PF_CHANNELS_ENABLED] = FALSE; // ???
//     SharedUserData->ProcessorFeatures[PF_XSAVE_ENABLED] =
//         (FeatureBits & KF_XSTATE) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_SECOND_LEVEL_ADDRESS_TRANSLATION] =
//         (FeatureBits & KF_SLAT) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_VIRT_FIRMWARE_ENABLED] =
//         (FeatureBits & KF_VIRT_FIRMWARE_ENABLED) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_RDWRFSGSBASE_AVAILABLE] =
//         (FeatureBits & KF_RDWRFSGSBASE) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_FASTFAIL_AVAILABLE] = TRUE;
//     SharedUserData->ProcessorFeatures[PF_RDRAND_INSTRUCTION_AVAILABLE] =
//         (FeatureBits & KF_RDRAND) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_RDTSCP_INSTRUCTION_AVAILABLE] =
//         (FeatureBits & KF_RDTSCP) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_RDPID_INSTRUCTION_AVAILABLE] = FALSE; // ???
//     SharedUserData->ProcessorFeatures[PF_SSSE3_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_SSSE3) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_SSE4_1_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_SSE4_1) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_SSE4_2_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_SSE4_2) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_AVX_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_AVX) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_AVX2_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_AVX2) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_AVX512F_INSTRUCTIONS_AVAILABLE] =
//         (FeatureBits & KF_AVX512F) ? TRUE : FALSE;
//
//     /* Set the default NX policy (opt-in) */
//     SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_OPTIN;
//
//     /* Check if NPX is always on */
//     if (strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=ALWAYSON"))
//     {
//         /* Set it always on */
//         SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_ALWAYSON;
//         Prcb->FeatureBits |= KF_NX_ENABLED;
//     }
//     else if (strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=OPTOUT"))
//     {
//         /* Set it in opt-out mode */
//         SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_OPTOUT;
//         Prcb->FeatureBits |= KF_NX_ENABLED;
//     }
//     else if ((strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=OPTIN")) ||
//              (strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE")))
//     {
//         /* Set the feature bits */
//         Prcb->FeatureBits |= KF_NX_ENABLED;
//     }
//     else if ((strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=ALWAYSOFF")) ||
//              (strstr(KeLoaderBlock->LoadOptions, "EXECUTE")))
//     {
//         /* Set disabled mode */
//         SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_ALWAYSOFF;
//         Prcb->FeatureBits |= KF_NX_DISABLED;
//     }
//
// #if DBG
//     /* Print applied kernel features/policies and boot CPU features */
//     KiReportCpuFeatures(Prcb);
// #endif
// }
//
// static LDR_DATA_TABLE_ENTRY LdrCoreEntries[3];
//
// void
// KiInitModuleList(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY Entry;
//     ULONG i;
//
//     /* Initialize the list head */
//     InitializeListHead(&PsLoadedModuleList);
//
//     /* Loop the first 3 entries */
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
// CODE_SEG("INIT")
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KiSystemStartup(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     CCHAR Cpu;
//     PKTHREAD InitialThread;
//     ULONG64 InitialStack;
//     PKIPCR Pcr;
//
//     /* Boot cycles timestamp */
//     BootCycles = __rdtsc();
//
//     /* HACK */
//     FrLdrDbgPrint = LoaderBlock->u.I386.CommonDataArea;
//     //FrLdrDbgPrint("Hello from KiSystemStartup!!!\n");
//
//     /* Get the current CPU number */
//     Cpu = KeNumberProcessors++; // FIXME
//
//     /* LoaderBlock initialization for Cpu 0 */
//     if (Cpu == 0)
//     {
//         /* Save the loader block */
//         KeLoaderBlock = LoaderBlock;
//
//         /* Prepare LoaderBlock, PCR, TSS with the P0 boot data */
//         KiInitializeP0BootStructures(LoaderBlock);
//     }
//
//     /* Get Pcr from loader block */
//     Pcr = CONTAINING_RECORD(LoaderBlock->Prcb, KIPCR, Prcb);
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
//     /* Initialize the CPU features */
//     KiInitializeCpu(Pcr);
//
//     /* Initial setup for the boot CPU */
//     if (Cpu == 0)
//     {
//         /* Set global feature bits */
//         KeFeatureBits = (ULONG64)Pcr->Prcb.FeatureBitsHigh << 32 |
//                         Pcr->Prcb.FeatureBits;
//
//         /* Initialize the module list (ntos, hal, kdcom) */
//         KiInitModuleList(LoaderBlock);
//
//         /* Setup the IDT */
//         KeInitExceptions();
//
//          /* Initialize debugging system */
//         KdInitSystem(0, KeLoaderBlock);
//
//         /* Check for break-in */
//         if (KdPollBreakIn()) DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
//     }
//
//     DPRINT1("Pcr = %p, Gdt = %p, Idt = %p, Tss = %p\n",
//            Pcr, Pcr->GdtBase, Pcr->IdtBase, Pcr->TssBase);
//
//     /* Acquire lock */
//     while (InterlockedBitTestAndSet64((PLONG64)&KiFreezeExecutionLock, 0))
//     {
//         /* Loop until lock is free */
//         while ((*(volatile KSPIN_LOCK*)&KiFreezeExecutionLock) & 1);
//     }
//
//     /* Initialize the Processor with HAL */
//     HalInitializeProcessor(Cpu, KeLoaderBlock);
//
//     /* Set processor as active */
//     KeActiveProcessors |= 1ULL << Cpu;
//
//     /* Release lock */
//     InterlockedAnd64((PLONG64)&KiFreezeExecutionLock, 0);
//
//     /* Raise to HIGH_LEVEL */
//     KfRaiseIrql(HIGH_LEVEL);
//
//     /* Machine specific kernel initialization */
//     if (Cpu == 0) KiInitializeKernelMachineDependent(&Pcr->Prcb, LoaderBlock);
//
//     /* Initialize extended state management */
//     KiInitializeXStateConfiguration(Cpu);
//
//     /* Calculate the initial stack pointer */
//     InitialStack = ALIGN_DOWN_BY(LoaderBlock->KernelStack - KeXStateLength, 64);
//
//     /* Switch to new kernel stack and start kernel bootstrapping */
//     KiSwitchToBootStack(InitialStack);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
