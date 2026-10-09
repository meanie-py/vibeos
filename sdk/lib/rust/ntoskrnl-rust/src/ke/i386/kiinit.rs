use crate::ke::ke_defs::*;
// Explicit imports to disambiguate glob conflicts on i386
use crate::ke::ke_defs::NO_PAGES_AVAILABLE;
type ULONG64 = u64;
use ntkrnl_sys::mm::KeGetCurrentThread;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _ETHREAD,
    _KGDTENTRY,
    _KIDTENTRY,
    _KIPCR,
    _KPCR,
    _KPRCB,
    _KPROCESS,
    _KTHREAD,
    _KTSS,
    _LOADER_PARAMETER_BLOCK,
    PHARDWARE_PTE_X86,
    PEPROCESS,
    PETHREAD,
    PKGDTENTRY,
    PKIDTENTRY,
    PKIPCR,
    PKPRCB,
    PKPROCESS,
    PKTHREAD,
    PKTSS,
    PLOADER_PARAMETER_BLOCK,
    CPU_INFO,
    LARGE_INTEGER,
    KI_SAMPLE_MAP,
    PKI_SAMPLE_MAP,
    PKIPI_BROADCAST_WORKER,
    ExpInitializeExecutive,
    HalInitializeProcessor,
    KeDelayExecutionThread,
    KeInitExceptions,
    KeInitializeProcess,
    KeInitializeSpinLock,
    KeInitializeThread,
    KeIpiGenericCall,
    KeMemoryBarrier,
    KeQueryPerformanceCounter,
    KeRevertToUserAffinityThread,
    KeSetPriorityThread,
    KeSetSystemAffinityThread,
    KiAcquirePrcbLock,
    KiComputeReciprocal,
    KiIdleLoop,
    KiInitSpinLocks,
    KiInitSystem,
    KiInitializeMachineType,
    KiReleasePrcbLock,
    KiSaveProcessorControlState,
    MmCreateKernelStack,
    RtlPrefetchMemoryNonTemporal,
};

const KERNEL_STACK_SIZE: usize = 12288;
const EXCEPTION_CHAIN_END: *mut core::ffi::c_void = !0usize as *mut _;
const PCR_MAJOR_VERSION: ULONG = 1;
const PCR_MINOR_VERSION: ULONG = 1;
const PRCB_MAJOR_VERSION: USHORT = 1;
const PRCB_MINOR_VERSION: USHORT = 1;
// Canonical Windows values (ke.h): DEBUG=1, UNIPROCESSOR=2. These were swapped
// here, which was masked on UP (both bits set → BuildType==3 either way, and the
// HAL's UP HalpBuildType is also 3) but unmasked the instant SMP sets DEBUG-only:
// the kernel published BuildType=2 while the SMP HAL's HalpBuildType=DEBUG=1, so
// halinit.rs:178's build-type check bugchecked 0x79 MISMATCHED_HAL(2,2,1). The
// HAL, the ntkrnl-sys bindgen headers, and the amd64 kernel all use DEBUG=1/UP=2.
const PRCB_BUILD_UNIPROCESSOR: USHORT = 2;
const PRCB_BUILD_DEBUG: USHORT = 1;
const HIGH_PRIORITY: i8 = 31;
const MAXCHAR: i8 = 127;
const EFLAGS_ID: ULONG = 0x200000;
const CR0_EM: ULONG_PTR = 0x00000004;
const CR0_MP: ULONG_PTR = 0x00000002;
const CR0_NE: ULONG_PTR = 0x00000020;
const KGDT_R0_PCR: USHORT = 0x30;
// 0x20 (ndk/i386/ketypes.h:126). Was 0x23, i.e. KGDT_R3_DATA|RPL_MASK folded into the
// constant. Every use site here spells `KGDT_R3_DATA | RPL_MASK` anyway, so OR-ing 3
// onto 0x23 was idempotent and the value landed right BY LUCK -- latent, not live.
// Any site that did not OR, or that compared against the bare selector, would differ.
const KGDT_R3_DATA: USHORT = 0x20;
const RPL_MASK: USHORT = 3;
const IOPM_SIZE: ULONG = 8192;
const TAG_KERNEL: ULONG = 0x2020654B;
const KPCR_PROCESSOR_NUMBER: ULONG = 0x130; // PrcbData.Number (KPCR+0x120+0x10)
const KPCR_PRCB: ULONG = 0x20;              // KPCR.Prcb pointer (NOT PrcbData!)
const KPCR_TEB: ULONG = 0x18;
const KPCR_DR6: ULONG = 0x428;              // KPCR PrcbData debug register
const KPCR_DR7: ULONG = 0x42C;              // KPCR PrcbData debug register
const KPCR_NUMBER: ULONG = 0x51;            // KPCR.Number (UCHAR)
const KPCR_SET_MEMBER: ULONG = 0x48;        // KPCR.SetMember
const KPCR_SET_MEMBER_COPY: ULONG = 0x14;   // KPCR.SetMemberCopy
const KPCR_PRCB_SET_MEMBER: ULONG = 0x134;  // PrcbData.SetMember (KPCR+0x120+0x14)
const DBG_STATUS_CONTROL_C: ULONG = 1;
const NX_SUPPORT_POLICY_OPTIN: ULONG = 0x02;
const NX_SUPPORT_POLICY_OPTOUT: ULONG = 0x03;
const NX_SUPPORT_POLICY_ALWAYSON: ULONG = 0x01;
const NX_SUPPORT_POLICY_ALWAYSOFF: ULONG = 0x00;
const PROCESSOR_ARCHITECTURE_INTEL: USHORT = 0;
const PF_FLOATING_POINT_PRECISION_ERRATA: ULONG = 0;
const PF_MMX_INSTRUCTIONS_AVAILABLE: ULONG = 3;
const PF_COMPARE_EXCHANGE_DOUBLE: ULONG = 2;
const PF_XMMI_INSTRUCTIONS_AVAILABLE: ULONG = 6;
const PF_XMMI64_INSTRUCTIONS_AVAILABLE: ULONG = 10;
const PF_3DNOW_INSTRUCTIONS_AVAILABLE: ULONG = 7;
const PF_RDTSC_INSTRUCTION_AVAILABLE: ULONG = 8;
const PF_RDRAND_INSTRUCTION_AVAILABLE: ULONG = 28;
const PF_SSE3_INSTRUCTIONS_AVAILABLE: ULONG = 13;
const PF_SSSE3_INSTRUCTIONS_AVAILABLE: ULONG = 36;
const PF_SSE4_1_INSTRUCTIONS_AVAILABLE: ULONG = 37;
const PF_SSE4_2_INSTRUCTIONS_AVAILABLE: ULONG = 38;

#[repr(C, align(4096))]
struct AlignedStack([u8; KERNEL_STACK_SIZE]);

static mut P0BootStackData: AlignedStack = AlignedStack([0; KERNEL_STACK_SIZE]);
static mut KiDoubleFaultStackData: AlignedStack = AlignedStack([0; KERNEL_STACK_SIZE]);

#[no_mangle]
pub static mut P0BootStack: ULONG_PTR = 0;
#[no_mangle]
pub static mut KiDoubleFaultStack: ULONG_PTR = 0;

#[no_mangle]
pub static mut KiFreezeExecutionLock: KSPIN_LOCK = 0;
#[no_mangle]
pub static mut Ki486CompatibilityLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut ProcessCount: ULONG = 0;
#[no_mangle]
pub static mut BootCycles: ULONGLONG = 0;
#[no_mangle]
pub static mut BootCyclesEnd: ULONGLONG = 0;

// Use actual definitions from sibling modules, not extern declarations (lto=false splits them)
use crate::ke::clock::KeMaximumIncrement;
use crate::ke::dpc::{KiMaximumDpcQueueDepth, KiMinimumDpcRate, KiAdjustDpcThreshold,
    KiTimeLimitIsrMicroseconds};
use crate::ke::timerobj::{KiTimeIncrementReciprocal, KiTimeIncrementShiftCount};
use crate::ke::krnlinit::{KeFeatureBits, KeLoaderBlock, KiProcessorBlock, KeNodeBlock,
    KeProcessorArchitecture, KeProcessorLevel, KeProcessorRevision,
    KiInitialProcess, KiInitialThread};
use crate::ke::processor::KeNumberProcessors;
use crate::ke::thrdschd::KiIdleSummary;
use crate::ke::procobj::KiProcessListHead;
use super::v86vdm::Ki386IopmSaveArea;
use super::cpu::{KeI386CpuType, KeI386CpuStep, KeI386FxsrPresent, KeI386XMMIPresent,
    KiMXCsrMask, KiI386PentiumLockErrataPresent};

extern "system" {
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
    static mut KiIdtDescriptor: KDESCRIPTOR;
    fn Ki386CreateIdentityMap(IdentityMap: PVOID, Function: PVOID, Arg: ULONG) -> BOOLEAN;
    fn Ki386EnableCurrentLargePage(StartAddress: ULONG_PTR, Cr3: ULONG);
    fn Ki386FreeIdentityMap(IdentityMap: PVOID);
    fn HalQuerySystemInformation(InfoClass: ULONG, BufferSize: ULONG, Buffer: PVOID, ReturnLength: *mut ULONG) -> NTSTATUS;
    fn KiInitializePAT();
    fn KiRestoreFastSyscallReturnState();
    fn KiCpuId(CpuInfo: *mut CPU_INFO, Function: ULONG);
    fn KiInitializeMTRR(First: BOOLEAN);
    fn KiAmdK6InitializeMTRR();
    fn KiI386PentiumLockErrataFixup();
    fn KiGetThreadNpxArea(Thread: PKTHREAD) -> *mut FX_SAVE_AREA;
    fn Ke386SaveFpuState(FxSaveArea: *mut FX_SAVE_AREA);
    fn KiSetCR0Bits();
    fn Ke386GetGlobalDescriptorTable(Limit: *mut USHORT);
    fn Ke386GetTr() -> USHORT;
    fn Ke386GetFs() -> USHORT;
    fn Ke386SetFs(Selector: USHORT);
    fn Ke386SetDs(Selector: USHORT);
    fn Ke386SetEs(Selector: USHORT);
    fn PoInitializePrcb(Prcb: PKPRCB);
    fn KiIsNpxErrataPresent() -> BOOLEAN;
    fn KiReportCpuFeatures();
    fn KdInitSystem(Phase: ULONG, LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN;
    fn KdPollBreakIn() -> BOOLEAN;
    fn DbgBreakPointWithStatus(Status: ULONG);
    fn KiSwitchToBootStack(Stack: ULONG_PTR) -> !;
    fn __readeflags() -> ULONG;
    fn __writeeflags(Value: ULONG);
    fn __rdtsc() -> ULONGLONG;
    fn __invlpg(Address: PVOID);
    fn _enable();
    fn __readfsdword(Offset: ULONG) -> ULONG;
    fn __writefsdword(Offset: ULONG, Value: ULONG);
}
extern "C" {
    fn YieldProcessor();
    fn InterlockedBitTestAndSet(Base: *mut LONG, Bit: LONG) -> UCHAR;
    fn __readcr0() -> ULONG_PTR;
    fn __writecr0(Value: ULONG_PTR);
    fn __sidt(Limit: *mut USHORT);
}
extern "fastcall" {
    fn Ki386InitializeTss(Tss: PKTSS, Idt: PKIDTENTRY, Gdt: PKGDTENTRY);
}
extern "C" {
    fn strstr(str1: *const i8, str2: *const i8) -> *const i8;
}

// KeIpiGenericCall callback functions — `system` is stdcall here (this module is i386-only),
// and it is how patpge.rs actually defines them and how the KeIpiGenericCall slot is spelled,
// so all three agree nominally instead of only lowering the same.
extern "system" {
    fn Ki386EnableTargetLargePage(Arg: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong;
    fn Ki386EnableGlobalPage(CpuCount: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong;
    fn Ki386EnableDE(CpuCount: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong;
    fn Ki386EnableFxsr(CpuCount: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong;
    fn Ki386EnableXMMIExceptions(CpuCount: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong;
}
extern "system" {
    fn KiGetCacheInformation();
    fn KiSetProcessorType();
    fn KiGetFeatureBits() -> ULONG64;
}

// EPROCESS_WRAPPER/ETHREAD_WRAPPER removed — use _EPROCESS/_ETHREAD directly via imports

#[repr(C)]
struct FX_SAVE_AREA {
    U: FX_SAVE_AREA_UNION,
}

#[repr(C)]
#[derive(Copy, Clone)]
union FX_SAVE_AREA_UNION {
    FxArea: FX_AREA,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FX_AREA {
    // MXCsrMask sits at +0x1C (28) in the C FXSAVE_FORMAT: after the 8-byte
    // {Control,Status,Tag,ErrorOpcode} words, ErrorOffset(4), ErrorSelector(4),
    // DataOffset(4), DataSelector(4), MXCsr(4) → 28. A 24-byte pad read MXCsr
    // (0x1F80) as the mask and stashed it in KiMXCsrMask.
    _padding: [u8; 28],
    MXCsrMask: ULONG,
}

#[repr(C)]
struct KDESCRIPTOR {
    Pad: USHORT,
    Limit: USHORT,
    Base: ULONG,
}

#[repr(C)]
struct LARGE_IDENTITY_MAP {
    _opaque: [u8; 64],
}

// KUSER_SHARED_DATA: bindgen layout. The shadow struct here put
// ProcessorFeatures at +0x2D4 and NXSupportPolicy at +0x330 (real: +0x274 /
// +0x2D5) — writes landed on ActiveConsoleId..Cookie. Masked by boot ordering
// on UP; same landmine class as the amd64 -smp 2 NtSystemRoot chop (see
// ke/amd64/kiinit.rs note, 2026-07-11).
use ntkrnl_sys::ddk::_KUSER_SHARED_DATA as KUSER_SHARED_DATA;

#[repr(C)]
struct KNODE {
    ProcessorMask: ULONG_PTR,
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitMachineDependent() {
    let mut CpuCount: ULONG;
    let mut FbCaching: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;
    let mut ReturnLength: ULONG = 0;
    let mut i: ULONG;
    let mut Affinity: ULONG;
    let mut Sample: ULONG = 0;
    let mut FxSaveArea: *mut FX_SAVE_AREA;
    let mut MXCsrMask: ULONG = 0xFFBF;
    let mut CpuInfo: CPU_INFO = core::mem::zeroed();
    let mut Samples: [KI_SAMPLE_MAP; 10] = core::mem::zeroed();
    let mut CurrentSample: PKI_SAMPLE_MAP = Samples.as_mut_ptr();
    let mut IdentityMap: LARGE_IDENTITY_MAP = core::mem::zeroed();

    if (KeFeatureBits & KF_LARGE_PAGE as u64) != 0 {
        if Ki386CreateIdentityMap(
            core::ptr::addr_of_mut!(IdentityMap) as PVOID,
            Ki386EnableCurrentLargePage as PVOID,
            2,
        ) != 0
        {
            KeIpiGenericCall(
                Some(Ki386EnableTargetLargePage),
                core::ptr::addr_of_mut!(IdentityMap) as u32 as _,
            );
        }

        Ki386FreeIdentityMap(core::ptr::addr_of_mut!(IdentityMap) as PVOID);
    }

    if (KeFeatureBits & KF_GLOBAL_PAGE as u64) != 0 {
        CpuCount = KeNumberProcessors as ULONG;
        KeIpiGenericCall(
            Some(Ki386EnableGlobalPage),
            core::ptr::addr_of_mut!(CpuCount) as u32,
        );
    }

    if (KeFeatureBits & ((KF_PAT | KF_MTRR) as u64)) != 0 {
        Status = HalQuerySystemInformation(
            0x15,
            core::mem::size_of::<BOOLEAN>() as ULONG,
            core::ptr::addr_of_mut!(FbCaching) as PVOID,
            core::ptr::addr_of_mut!(ReturnLength),
        );
        if Status >= 0 && FbCaching != 0 {
            KeFeatureBits &= !((KF_PAT | KF_MTRR) as u64);
        }
    }

    if (KeFeatureBits & KF_PAT as u64) != 0 {
        KiInitializePAT();
    }

    if (KeFeatureBits & KF_CR4 as u64) != 0 {
        CpuCount = KeNumberProcessors as ULONG;
        KeIpiGenericCall(
            Some(Ki386EnableDE),
            core::ptr::addr_of_mut!(CpuCount) as u32,
        );
    }

    if (KeFeatureBits & KF_FXSR as u64) != 0 {
        CpuCount = KeNumberProcessors as ULONG;
        KeIpiGenericCall(
            Some(Ki386EnableFxsr),
            core::ptr::addr_of_mut!(CpuCount) as u32,
        );

        if (KeFeatureBits & KF_XMMI as u64) != 0 {
            CpuCount = KeNumberProcessors as ULONG;
            KeIpiGenericCall(
                Some(Ki386EnableXMMIExceptions),
                core::ptr::addr_of_mut!(CpuCount) as u32,
            );

            *(RtlPrefetchMemoryNonTemporal as *mut u8) = 0x90;
        }
    }

    KiRestoreFastSyscallReturnState();

    i = KeActiveProcessors as ULONG;
    Affinity = 1;
    while i != 0 {
        if (i & Affinity as ULONG) != 0 {
            i &= !(Affinity as ULONG);
            KeSetSystemAffinityThread(Affinity);

            // CR0.WP on EVERY processor, while we run on it. The C sets it once, after
            // this loop, so only on the CPU we return to; the APs were started earlier in
            // Phase 1 (KeStartAllProcessors precedes KeInitSystem) with the boot CPU's CR0
            // from before WP, so on them a kernel-mode write ignored read-only PTEs:
            // kmtest ZwMapViewOfSection :492 wrote a PAGE_READONLY view with no access
            // violation at i386 -smp 2, every sweep. amd64 sets WP per CPU in KiInitializeCpu.
            KiSetCR0Bits();

            (*KeGetCurrentPrcb()).MHz = 0;

            if (KeFeatureBits & KF_RDTSC as u64) != 0 {
                loop {
                    KiCpuId(core::ptr::addr_of_mut!(CpuInfo), 0);

                    (*CurrentSample).PerfStart = KeQueryPerformanceCounter(core::ptr::null_mut());
                    (*CurrentSample).TSCStart = __rdtsc() as i64;
                    (*CurrentSample).PerfFreq.QuadPart = -50000;

                    KeDelayExecutionThread(
                        0i8,
                        0u8,
                        core::ptr::addr_of_mut!((*CurrentSample).PerfFreq),
                    );

                    KiCpuId(core::ptr::addr_of_mut!(CpuInfo), 0);

                    (*CurrentSample).PerfEnd = KeQueryPerformanceCounter(
                        core::ptr::addr_of_mut!((*CurrentSample).PerfFreq),
                    );
                    (*CurrentSample).TSCEnd = __rdtsc() as i64;

                    (*CurrentSample).PerfDelta = (*CurrentSample).PerfEnd.QuadPart
                        - (*CurrentSample).PerfStart.QuadPart;
                    (*CurrentSample).TSCDelta =
                        (*CurrentSample).TSCEnd - (*CurrentSample).TSCStart;

                    (*CurrentSample).MHz = (((*CurrentSample).TSCDelta
                        * (*CurrentSample).PerfFreq.QuadPart
                        + 500000i64)
                        / ((*CurrentSample).PerfDelta * 1000000i64))
                        as ULONG;

                    if Sample != 0 {
                        if (*CurrentSample).MHz == (*CurrentSample.offset(-1)).MHz
                            || (*CurrentSample).MHz == (*CurrentSample.offset(-1)).MHz + 1
                            || (*CurrentSample).MHz == (*CurrentSample.offset(-1)).MHz - 1
                        {
                            break;
                        }
                    }

                    CurrentSample = CurrentSample.offset(1);
                    Sample += 1;

                    if Sample == 10 {
                        let mut TotalMHz: ULONG = 0;
                        let mut s = Sample;
                        while s > 0 {
                            s -= 1;
                            TotalMHz += Samples[s as usize].MHz;
                        }
                        (*CurrentSample.offset(-1)).MHz = TotalMHz / 10;
                        break;
                    }
                }

                (*KeGetCurrentPrcb()).MHz = (*CurrentSample.offset(-1)).MHz;
            }

            if (KeFeatureBits & KF_MTRR as u64) != 0 {
                KiInitializeMTRR(if i != 0 { 0u8 } else { 1u8 });
            }

            if (KeFeatureBits & KF_AMDK6MTRR as u64) != 0 {
                KiAmdK6InitializeMTRR();
            }

            if KiI386PentiumLockErrataPresent != 0 {
                KiI386PentiumLockErrataFixup();
            }

            if (KeFeatureBits & KF_FXSR as u64) != 0 {
                FxSaveArea = KiGetThreadNpxArea(ntkrnl_sys::ddk::KeGetCurrentThread());

                (*FxSaveArea).U.FxArea.MXCsrMask = 0;

                Ke386SaveFpuState(FxSaveArea);

                if (*FxSaveArea).U.FxArea.MXCsrMask != 0 {
                    MXCsrMask = (*FxSaveArea).U.FxArea.MXCsrMask;
                }

                if KiMXCsrMask == 0 {
                    KiMXCsrMask = MXCsrMask;
                } else {
                    if KiMXCsrMask != MXCsrMask {
                        KeBugCheckEx(
                            MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED,
                            KF_FXSR as ULONG_PTR,
                            KiMXCsrMask as ULONG_PTR,
                            MXCsrMask as ULONG_PTR,
                            0,
                        );
                    }
                }

                KiMXCsrMask &= MXCsrMask;
            }
        }
        Affinity <<= 1;
    }

    KeRevertToUserAffinityThread();

    if KiTimeLimitIsrMicroseconds != 0 {}

    KiSetCR0Bits();

}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializePcr(
    ProcessorNumber: ULONG,
    Pcr: PKIPCR,
    Idt: PKIDTENTRY,
    Gdt: PKGDTENTRY,
    Tss: PKTSS,
    IdleThread: PKTHREAD,
    DpcStack: PVOID,
) {
    let IdleThread = opaque_ptr(IdleThread);
    let DpcStack = opaque_ptr(DpcStack);
    (*Pcr).__bindgen_anon_1.NtTib.ExceptionList = EXCEPTION_CHAIN_END as *mut _;
    (*Pcr).__bindgen_anon_1.NtTib.StackBase = core::ptr::null_mut();
    (*Pcr).__bindgen_anon_1.NtTib.StackLimit = core::ptr::null_mut();
    (*Pcr).__bindgen_anon_1.NtTib.Self_ = core::ptr::null_mut();

    (*Pcr).PrcbData.CurrentThread = IdleThread;

    (*Pcr).SelfPcr = Pcr as *mut _KPCR;
    (*Pcr).Prcb = core::ptr::addr_of_mut!((*Pcr).PrcbData);

    (*Pcr).MajorVersion = PCR_MAJOR_VERSION as USHORT;
    (*Pcr).MinorVersion = PCR_MINOR_VERSION as USHORT;

    (*Pcr).PrcbData.MajorVersion = PRCB_MAJOR_VERSION;
    (*Pcr).PrcbData.MinorVersion = PRCB_MINOR_VERSION;

    (*Pcr).PrcbData.BuildType = 0;
    #[cfg(not(feature = "smp"))]
    {
        (*Pcr).PrcbData.BuildType |= PRCB_BUILD_UNIPROCESSOR;
    }
    // Always set PRCB_BUILD_DEBUG to match the C HAL which is built with DBG=1.
    // Rust's debug_assertions is only set in debug profile, but we build with
    // --release, so #[cfg(debug_assertions)] is always false.
    (*Pcr).PrcbData.BuildType |= PRCB_BUILD_DEBUG;

    (*Pcr).PrcbData.Number = ProcessorNumber as UCHAR;
    (*Pcr).PrcbData.SetMember = 1 << ProcessorNumber;

    *(core::ptr::addr_of_mut!(KiProcessorBlock) as *mut PKPRCB)
        .add(ProcessorNumber as usize) = (*Pcr).Prcb;

    (*Pcr).Irql = PASSIVE_LEVEL as u8;

    (*Pcr).GDT = Gdt;
    (*Pcr).IDT = Idt;
    (*Pcr).TSS = Tss;
    (*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssCopy = Tss as PVOID;
    (*Pcr).PrcbData.DpcStack = DpcStack;

    (*Pcr).PrcbData.MultiThreadProcessorSet = (*Pcr).PrcbData.SetMember;
}

// CODE_SEG("INIT")
unsafe fn KiVerifyCpuFeatures(Prcb: PKPRCB) {
    let mut CpuInfo: CPU_INFO = core::mem::zeroed();

    let EFlags = __readeflags();
    let NewEFlags = EFlags ^ EFLAGS_ID;
    __writeeflags(NewEFlags);
    let NewEFlags = __readeflags();

    if NewEFlags == EFlags {
        KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x1, 0, 0, 0);
    } else {
        let EFlags = EFlags | EFLAGS_ID;
        __writeeflags(EFlags);
    }

    KiCpuId(core::ptr::addr_of_mut!(CpuInfo), 0);
    if CpuInfo.__bindgen_anon_1.Eax == 0 {
        KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x1, 0, 0, 0);
    }

    KiSetProcessorType();

    if (*Prcb).CpuType == 3 {
        KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x386, 0, 0, 0);
    }

    let FeatureBits = KiGetFeatureBits();

    if (FeatureBits & KF_RDTSC as u64) == 0 {
        KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000010, 0, 0);
    }

    if (FeatureBits & KF_CMPXCHG8B as u64) == 0 {
        KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000100, 0, 0);
    }

    KiCpuId(core::ptr::addr_of_mut!(CpuInfo), 1);

    if (CpuInfo.__bindgen_anon_1.Edx & 0x00000001) == 0 {
        KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000001, 0, 0);
    }

    let mut Cr0 = __readcr0();
    Cr0 &= !(CR0_EM | CR0_MP);
    Cr0 |= CR0_NE;
    __writecr0(Cr0);

    if KiIsNpxErrataPresent() != 0 {
        KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000001, 0, 0);
    }

    (*Prcb).FeatureBits = FeatureBits as ULONG;
    (*Prcb).FeatureBitsHigh = (FeatureBits >> 32) as ULONG;
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeKernel(
    InitProcess: PKPROCESS,
    InitThread: PKTHREAD,
    IdleStack: PVOID,
    Prcb: PKPRCB,
    Number: i8,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    let InitProcess = opaque_ptr(InitProcess);
    let InitThread = opaque_ptr(InitThread);
    let IdleStack = opaque_ptr(IdleStack);
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut PageDirectory: [ULONG; 2] = [0, 0];
    let DpcStack: PVOID;
    let mut DummyIrql: KIRQL = 0;

    PoInitializePrcb(Prcb);

    if Number == 0 {
        KeFeatureBits =
            (*Prcb).FeatureBits as u64 | (((*Prcb).FeatureBitsHigh as u64) << 32);
    }

    (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_OPTIN as u8);

    if !strstr((*KeLoaderBlock).LoadOptions, b"NOEXECUTE=ALWAYSON\0".as_ptr() as *const i8)
        .is_null()
    {
        (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_ALWAYSON as u8);
        KeFeatureBits |= KF_NX_ENABLED as u64;
    } else if !strstr(
        (*KeLoaderBlock).LoadOptions,
        b"NOEXECUTE=OPTOUT\0".as_ptr() as *const i8,
    )
    .is_null()
    {
        (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_OPTOUT as u8);
        KeFeatureBits |= KF_NX_ENABLED as u64;
    } else if !strstr(
        (*KeLoaderBlock).LoadOptions,
        b"NOEXECUTE=OPTIN\0".as_ptr() as *const i8,
    )
    .is_null()
        || !strstr((*KeLoaderBlock).LoadOptions, b"NOEXECUTE\0".as_ptr() as *const i8).is_null()
    {
        KeFeatureBits |= KF_NX_ENABLED as u64;
    } else if !strstr(
        (*KeLoaderBlock).LoadOptions,
        b"NOEXECUTE=ALWAYSOFF\0".as_ptr() as *const i8,
    )
    .is_null()
        || !strstr((*KeLoaderBlock).LoadOptions, b"EXECUTE\0".as_ptr() as *const i8).is_null()
    {
        (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy(NX_SUPPORT_POLICY_ALWAYSOFF as u8);
        KeFeatureBits |= KF_NX_DISABLED as u64;
    }

    KiSaveProcessorControlState(core::ptr::addr_of_mut!((*Prcb).ProcessorState));

    // NOT TRANSLATED — see the identical note in ke/amd64/kiinit.rs.
    // ke/i386/cpu.rs:2417 carries only the signature as a comment.
    // #[cfg(debug_assertions)]
    // if Number == 0 { KiReportCpuFeatures(); }

    KiGetCacheInformation();

    KiInitSpinLocks(Prcb, Number);

    (*Prcb).ParentNode = KeNodeBlock[0] as *mut _;
    (*(*Prcb).ParentNode).ProcessorMask |= (*Prcb).SetMember;

    if Number == 0 {
        KeI386CpuType = (*Prcb).CpuType as ULONG;
        KeI386CpuStep = (*Prcb).CpuStep as ULONG;
        KeProcessorArchitecture = PROCESSOR_ARCHITECTURE_INTEL;
        KeProcessorLevel = (*Prcb).CpuType as USHORT;
        if (*Prcb).CpuID != 0 {
            KeProcessorRevision = (*Prcb).CpuStep as USHORT;
        }
        KeI386FxsrPresent = if (KeFeatureBits & KF_FXSR as u64) != 0 {
            1
        } else {
            0
        };
        KeI386XMMIPresent = if (KeFeatureBits & KF_XMMI as u64) != 0 {
            1
        } else {
            0
        };

        (*Prcb).MultiThreadSetMaster = Prcb;

        KeLowerIrql(APC_LEVEL as u8);

        KeInitializeSpinLock(core::ptr::addr_of_mut!(KiFreezeExecutionLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(Ki486CompatibilityLock) as *mut _);

        KiInitSystem();

        InitializeListHead(core::ptr::addr_of_mut!(KiProcessListHead));
        PageDirectory[0] = 0;
        PageDirectory[1] = 0;
        KeInitializeProcess(
            InitProcess,
            0,
            !0u32,
            PageDirectory.as_mut_ptr() as *mut _,
            0u8,
        );
        (*InitProcess).QuantumReset = MAXCHAR;
    } else {
        // i386 M5 Phase C: the AP enters KiInitializeKernel at HIGH_LEVEL (raised
        // in KiSystemStartup before KiSwitchToBootStack) and, unlike P0, skips the
        // APC_LEVEL lower above — but it STILL runs ExpInitializeExecutive(Cpu!=0)
        // below (→ ExInitPoolLookasidePointers + HalInitSystem, which allocate
        // pool) and then KeRaiseIrql(DISPATCH_LEVEL). At HIGH_LEVEL both are
        // illegal: pool ops require IRQL <= DISPATCH (else 0xC2 BAD_POOL_CALLER
        // Arg1=8/Arg2=0x1f, the exact AP crash observed), and KeRaiseIrql can't
        // "raise" from HIGH_LEVEL down to DISPATCH. ReactOS's i386 AP branch is a
        // `/* FIXME */ "you are brave"` stub — never implemented — so this lower
        // is greenfield, mirroring the amd64/arm64 AP (arm64 KiSystemStartupBootStack
        // does KeLowerIrql(APC_LEVEL) for both CPUs). Lower the AP to APC_LEVEL so
        // its executive/HAL init runs at a legal IRQL, matching P0.
        KeLowerIrql(APC_LEVEL as u8);
    }

    KeInitializeThread(
        InitProcess,
        InitThread,
        None,
        None,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        IdleStack,
    );
    (*InitThread).__bindgen_anon_1.__bindgen_anon_1.NextProcessor = Number as UCHAR;
    (*InitThread).Priority = HIGH_PRIORITY;
    (*InitThread).State = 2u8;
    (*InitThread).Affinity = 1 << Number;
    (*InitThread).WaitIrql = DISPATCH_LEVEL as u8;
    (*InitProcess).ActiveProcessors |= 1 << Number;

    (*(InitThread as PETHREAD)).ThreadsProcess = InitProcess as PEPROCESS;

    (*SharedUserData).ProcessorFeatures[PF_FLOATING_POINT_PRECISION_ERRATA as usize] = 0u8;
    (*SharedUserData).ProcessorFeatures[PF_MMX_INSTRUCTIONS_AVAILABLE as usize] =
        if (KeFeatureBits & KF_MMX as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_COMPARE_EXCHANGE_DOUBLE as usize] =
        if (KeFeatureBits & KF_CMPXCHG8B as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_XMMI_INSTRUCTIONS_AVAILABLE as usize] =
        if ((KeFeatureBits & KF_FXSR as u64) != 0) && ((KeFeatureBits & KF_XMMI as u64) != 0) {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_XMMI64_INSTRUCTIONS_AVAILABLE as usize] =
        if ((KeFeatureBits & KF_FXSR as u64) != 0) && ((KeFeatureBits & KF_XMMI64 as u64) != 0) {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_3DNOW_INSTRUCTIONS_AVAILABLE as usize] =
        if (KeFeatureBits & KF_3DNOW as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_RDTSC_INSTRUCTION_AVAILABLE as usize] =
        if (KeFeatureBits & KF_RDTSC as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_RDRAND_INSTRUCTION_AVAILABLE as usize] =
        if (KeFeatureBits & KF_RDRAND as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_SSE3_INSTRUCTIONS_AVAILABLE as usize] =
        if (KeFeatureBits & KF_SSE3 as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_SSSE3_INSTRUCTIONS_AVAILABLE as usize] =
        if (KeFeatureBits & KF_SSSE3 as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_SSE4_1_INSTRUCTIONS_AVAILABLE as usize] =
        if (KeFeatureBits & KF_SSE4_1 as u64) != 0 {
            1u8
        } else {
            0u8
        };
    (*SharedUserData).ProcessorFeatures[PF_SSE4_2_INSTRUCTIONS_AVAILABLE as usize] =
        if (KeFeatureBits & KF_SSE4_2 as u64) != 0 {
            1u8
        } else {
            0u8
        };

    (*Prcb).CurrentThread = InitThread;
    (*Prcb).NextThread = core::ptr::null_mut();
    (*Prcb).IdleThread = InitThread;

    ExpInitializeExecutive(Number as ULONG, LoaderBlock);

    if Number == 0 {
        KiTimeIncrementReciprocal = KiComputeReciprocal(
            KeMaximumIncrement as LONG,
            core::ptr::addr_of_mut!(KiTimeIncrementShiftCount),
        );

        (*Prcb).MaximumDpcQueueDepth = KiMaximumDpcQueueDepth;
        (*Prcb).MinimumDpcRate = KiMinimumDpcRate;
        (*Prcb).AdjustDpcThreshold = KiAdjustDpcThreshold;

        DpcStack = MmCreateKernelStack(0u8, 0);
        if DpcStack.is_null() {
            KeBugCheckEx(NO_PAGES_AVAILABLE, 1, 0, 0, 0);
        }
        (*Prcb).DpcStack = DpcStack;

        Ki386IopmSaveArea = ExAllocatePoolWithTag(PagedPool, IOPM_SIZE as usize, TAG_KERNEL);
        if Ki386IopmSaveArea.is_null() {
            KeBugCheckEx(NO_PAGES_AVAILABLE, 2, IOPM_SIZE as ULONG_PTR, 0, 0);
        }

    }

    KeRaiseIrql(DISPATCH_LEVEL as u8, core::ptr::addr_of_mut!(DummyIrql));

    KeSetPriorityThread(InitThread, 0);

    KiAcquirePrcbLock(Prcb);
    if (*Prcb).NextThread.is_null() {
        KiIdleSummary |= 1 << Number;
    }
    KiReleasePrcbLock(Prcb);

    KeRaiseIrql(HIGH_LEVEL as u8, core::ptr::addr_of_mut!(DummyIrql));

    (*LoaderBlock).Prcb = 0;
}

// CODE_SEG("INIT")
// KiGetMachineBootPointers is FASTCALL on i386 (in the i386/ module, always x86)
#[no_mangle]
pub unsafe extern "fastcall" fn KiGetMachineBootPointers(
    Gdt: *mut PKGDTENTRY,
    Idt: *mut PKIDTENTRY,
    Pcr: *mut PKIPCR,
    Tss: *mut PKTSS,
) {
    let Gdt = opaque_ptr(Gdt);
    let Idt = opaque_ptr(Idt);
    let Pcr = opaque_ptr(Pcr);
    let Tss = opaque_ptr(Tss);
    let TssSelector: _KGDTENTRY;
    let PcrSelector: _KGDTENTRY;

    // Use raw inline asm for SGDT/SIDT/STR/FS instead of C shims,
    // which have a broken __sgdt intrinsic under the Phase 1 rename pattern.
    let mut raw_sgdt: [u8; 6] = [0u8; 6];
    core::arch::asm!("sgdt [{}]", in(reg) raw_sgdt.as_mut_ptr(), options(nostack));
    let gdt_base = u32::from_le_bytes([raw_sgdt[2], raw_sgdt[3], raw_sgdt[4], raw_sgdt[5]]);

    let mut raw_sidt: [u8; 6] = [0u8; 6];
    core::arch::asm!("sidt [{}]", in(reg) raw_sidt.as_mut_ptr(), options(nostack));
    let idt_base = u32::from_le_bytes([raw_sidt[2], raw_sidt[3], raw_sidt[4], raw_sidt[5]]);

    let Tr: u16;
    core::arch::asm!("str {:x}", out(reg) Tr, options(nomem, nostack));

    let Fs: u16;
    core::arch::asm!("mov {:x}, fs", out(reg) Fs, options(nomem, nostack));

    core::ptr::write_volatile(Gdt, gdt_base as PKGDTENTRY);
    core::ptr::write_volatile(Idt, idt_base as PKIDTENTRY);

    PcrSelector = *((gdt_base as ULONG_PTR + (Fs & !RPL_MASK) as ULONG_PTR)
        as *const _KGDTENTRY);

    core::ptr::write_volatile(
        Pcr,
        (PcrSelector.BaseLow as ULONG_PTR
            | ((PcrSelector.HighWord.Bytes.BaseMid as ULONG_PTR) << 16)
            | ((PcrSelector.HighWord.Bytes.BaseHi as ULONG_PTR) << 24))
            as PKIPCR,
    );

    TssSelector = *((gdt_base as ULONG_PTR + (Tr & !RPL_MASK) as ULONG_PTR)
        as *const _KGDTENTRY);

    core::ptr::write_volatile(
        Tss,
        (TssSelector.BaseLow as ULONG_PTR
            | ((TssSelector.HighWord.Bytes.BaseMid as ULONG_PTR) << 16)
            | ((TssSelector.HighWord.Bytes.BaseHi as ULONG_PTR) << 24))
            as PKTSS,
    );
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSystemStartupBootStack() -> ! {
    let Thread: PKTHREAD;

    KiInitializeKernel(
        core::ptr::addr_of_mut!(KiInitialProcess.Pcb),
        (*KeLoaderBlock).Thread as PKTHREAD,
        ((*KeLoaderBlock).KernelStack & !3) as PVOID,
        __readfsdword(KPCR_PRCB) as PKPRCB,
        (KeNumberProcessors - 1) as i8,
        KeLoaderBlock,
    );

    Thread = KeGetCurrentThread() as PKTHREAD;
    (*Thread).Priority = 0;

    _enable();
    KeLowerIrql(DISPATCH_LEVEL as u8);

    (*Thread).WaitIrql = DISPATCH_LEVEL as u8;

    KiIdleLoop();
}

unsafe fn KiMarkPageAsReadOnly(Address: PVOID) {
    let PointerPte: PHARDWARE_PTE_X86;

    // C: ((PHARDWARE_PTE_X86)PTE_BASE) + ((ULONG_PTR)Address / PAGE_SIZE)
    // sizeof(HARDWARE_PTE_X86) = 4 on i386 (non-PAE 32-bit PTE)
    PointerPte = ((PTE_BASE as usize + (Address as usize / PAGE_SIZE) * 4) as PHARDWARE_PTE_X86);

    (*PointerPte).set_Write(0);

    __invlpg(Address);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSystemStartup(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> ! {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let Cpu: ULONG;
    let InitialThread: PKTHREAD;
    let InitialStack: ULONG;
    let mut Gdt: PKGDTENTRY = core::ptr::null_mut();
    let mut Idt: PKIDTENTRY = core::ptr::null_mut();
    let mut NmiEntry: _KIDTENTRY = core::mem::zeroed();
    let mut DoubleFaultEntry: _KIDTENTRY = core::mem::zeroed();
    let mut Tss: PKTSS = core::ptr::null_mut();
    let mut Pcr: PKIPCR = core::ptr::null_mut();
    let mut DummyIrql: KIRQL = 0;

    BootCycles = __rdtsc();

    KeLoaderBlock = LoaderBlock;
    Cpu = KeNumberProcessors as ULONG;
    if Cpu == 0 {
        Ke386SetFs(KGDT_R0_PCR);
        __writefsdword(KPCR_PROCESSOR_NUMBER, Cpu);

        P0BootStack =
            core::ptr::addr_of!(P0BootStackData).cast::<u8>().add(KERNEL_STACK_SIZE) as ULONG_PTR; // nolint: magic-offset (stack size, not a field offset)
        KiDoubleFaultStack = core::ptr::addr_of!(KiDoubleFaultStackData)
            .cast::<u8>()
            .add(KERNEL_STACK_SIZE) as ULONG_PTR; // nolint: magic-offset (stack size, not a field offset)

        (*LoaderBlock).KernelStack = P0BootStack as ULONG;
        (*LoaderBlock).Thread = core::ptr::addr_of!(KiInitialThread) as ULONG;
    }

    InitialStack = (*LoaderBlock).KernelStack;
    InitialThread = (*LoaderBlock).Thread as PKTHREAD;

    InitializeListHead(core::ptr::addr_of_mut!(
        (*InitialThread).__bindgen_anon_1.ApcState.ApcListHead[0]
    ));

    KiInitializeMachineType();

    // i386 M5 Phase C: the AP (Cpu != 0) now flows the FULL per-CPU init path
    // instead of parking. Everything below is per-CPU-parameterized (Cpu/Pcr
    // args, `if Cpu == 0` guards on the global-init parts), so the AP sets up
    // its own PCR/TSS/IDT/exceptions, acquires+releases the freeze lock,
    // publishes its KeActiveProcessors bit, HalInitializeProcessor, and
    // KiSwitchToBootStack → KiSystemStartupBootStack → KiInitializeKernel(AP,
    // Number!=0) → KiIdleLoop. KiInitializeKernel clears LoaderBlock->Prcb at
    // its end (the KeStartAllProcessors handshake). The M5a spin-stub that
    // used to sit here (KeMemoryBarrier + LoaderBlock->Prcb=0 + spin) is
    // removed; the handshake now happens at the ReactOS-faithful point.

    KiGetMachineBootPointers(
        core::ptr::addr_of_mut!(Gdt),
        core::ptr::addr_of_mut!(Idt),
        core::ptr::addr_of_mut!(Pcr),
        core::ptr::addr_of_mut!(Tss),
    );
    Ki386InitializeTss(Tss, Idt, Gdt);

    RtlZeroMemory(Pcr as PVOID, PAGE_SIZE);
    KiInitializePcr(
        Cpu,
        Pcr,
        Idt,
        Gdt,
        Tss,
        InitialThread,
        KiDoubleFaultStack as PVOID,
    );

    (*InitialThread).__bindgen_anon_1.ApcState.Process = core::ptr::addr_of_mut!(KiInitialProcess.Pcb);

    __writefsdword(KPCR_TEB, 0);
    __writefsdword(KPCR_DR6, 0);
    __writefsdword(KPCR_DR7, 0);

    // KeInitExceptions ONLY flips KiIdt's Selector/ExtendedOffset into hardware
    // layout (P0-only, one-shot prep for the KiIdtDescriptor recopy below). It is
    // unguarded, so an AP re-running it would DOUBLE-flip (corrupt) the global
    // template. The AP skips both the recopy and this (see the Cpu==0 gate on the
    // recopy block below) — its IDT is the already-correct mproc copy of P0's live
    // IDT. (i386 M5 Phase C.)
    if Cpu == 0 {
        KeInitExceptions();
    }

    Ke386SetDs(KGDT_R3_DATA | RPL_MASK);
    Ke386SetEs(KGDT_R3_DATA | RPL_MASK);

    // i386 M5 Phase C: this block re-copies the KERNEL IDT TEMPLATE
    // (KiIdtDescriptor.Base = &KiIdt) over the live IDT, preserving only the
    // NMI/double-fault task gates. On P0 that's correct — the HAL then wires its
    // interrupt vectors (clock IPI 0xd2 → HalpClockIpi, etc.) into P0's live IDT
    // afterward. But the AP's IDT was already copied from P0's LIVE, fully-wired
    // IDT in KeStartAllProcessors (ke/i386/mproc.rs, via __sidt); re-applying the
    // bare template here UNWIRES those HAL vectors (0xd2 reverts to
    // KiUnexpectedInterrupt210), and the AP never re-runs HAL init to re-wire them
    // (its per-CPU HAL is HalInitializeProcessor, which doesn't touch the IDT). So
    // the AP's FIRST clock IPI would hit the unexpected-interrupt stub and fault →
    // #DF (GDB: v=d2 → #PF@0x8048 → #DF). Skip the recopy on the AP and keep the
    // mproc copy, which already carries the correct traps + HAL vectors + NMI/DF
    // task gates (the gates resolve through the AP's own GDT TSS selectors).
    if Cpu == 0 {
        RtlCopyMemory(
            core::ptr::addr_of_mut!(NmiEntry) as PVOID,
            Idt.add(2) as *const core::ffi::c_void,
            core::mem::size_of::<_KIDTENTRY>(),
        );
        RtlCopyMemory(
            core::ptr::addr_of_mut!(DoubleFaultEntry) as PVOID,
            Idt.add(8) as *const core::ffi::c_void,
            core::mem::size_of::<_KIDTENTRY>(),
        );

        RtlCopyMemory(
            Idt as PVOID,
            KiIdtDescriptor.Base as *const core::ffi::c_void,
            (KiIdtDescriptor.Limit + 1) as usize,
        );

        RtlCopyMemory(
            Idt.add(2) as PVOID,
            core::ptr::addr_of!(NmiEntry) as *const core::ffi::c_void,
            core::mem::size_of::<_KIDTENTRY>(),
        );
        RtlCopyMemory(
            Idt.add(8) as PVOID,
            core::ptr::addr_of!(DoubleFaultEntry) as *const core::ffi::c_void,
            core::mem::size_of::<_KIDTENTRY>(),
        );
    }

    loop {
        while (core::ptr::read_volatile(core::ptr::addr_of!(KiFreezeExecutionLock)) as usize) != 0 {}

        if InterlockedBitTestAndSet(
            core::ptr::addr_of_mut!(KiFreezeExecutionLock) as *mut LONG,
            0,
        ) == 0
        {
            break;
        }
    }

    __writefsdword(KPCR_NUMBER, Cpu);
    __writefsdword(KPCR_SET_MEMBER, 1 << Cpu);
    __writefsdword(KPCR_SET_MEMBER_COPY, 1 << Cpu);
    __writefsdword(KPCR_PRCB_SET_MEMBER, 1 << Cpu);

    KiVerifyCpuFeatures((*Pcr).Prcb);

    HalInitializeProcessor(Cpu, KeLoaderBlock);

    KeActiveProcessors |= __readfsdword(KPCR_SET_MEMBER);
    KeNumberProcessors += 1;

    // Release the freeze-execution lock so the NEXT processor can acquire it
    // and continue its own KiSystemStartup. amd64 does this here
    // (ntoskrnl/ke/i386/../amd64/kiinit.c:566 — InterlockedAnd64(&lock, 0));
    // the i386 KiSwitchToBootStack asm translation DROPPED the upstream
    // release, which is latent while the AP parks at the M5a spin-stub (it
    // never reaches this acquire) but deadlocks the moment the AP flows the
    // full path: it would spin forever at the acquire loop above and P0's
    // KeStartAllProcessors handshake (mproc.rs waits on LoaderBlock->Prcb==0)
    // would never complete. Atomic Release store matches the acquire's
    // InterlockedBitTestAndSet/read_volatile pairing. (i386 M5 Phase C.)
    core::sync::atomic::AtomicUsize::from_ptr(
        core::ptr::addr_of_mut!(KiFreezeExecutionLock) as *mut usize,
    )
    .store(0, core::sync::atomic::Ordering::Release);

    if Cpu == 0 {
        KdInitSystem(0, KeLoaderBlock as PLOADER_PARAMETER_BLOCK);

        if KdPollBreakIn() != 0 {
            DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
        }

        KiMarkPageAsReadOnly(core::ptr::addr_of!(P0BootStackData) as PVOID);
        KiMarkPageAsReadOnly(core::ptr::addr_of!(KiDoubleFaultStackData) as PVOID);
    }

    KeRaiseIrql(HIGH_LEVEL as u8, core::ptr::addr_of_mut!(DummyIrql));

    KiSwitchToBootStack((InitialStack & !3) as ULONG_PTR);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/i386/kiinit.c
//  * PURPOSE:         Kernel Initialization for x86 CPUs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
// #include "internal/i386/trap_x.h"
//
// /* GLOBALS *******************************************************************/
//
// /* Boot and double-fault/NMI/DPC stack */
// UCHAR DECLSPEC_ALIGN(PAGE_SIZE) P0BootStackData[KERNEL_STACK_SIZE] = {0};
// UCHAR DECLSPEC_ALIGN(PAGE_SIZE) KiDoubleFaultStackData[KERNEL_STACK_SIZE] = {0};
// ULONG_PTR P0BootStack = (ULONG_PTR)&P0BootStackData[KERNEL_STACK_SIZE];
// ULONG_PTR KiDoubleFaultStack = (ULONG_PTR)&KiDoubleFaultStackData[KERNEL_STACK_SIZE];
//
// /* Spinlocks used only on X86 */
// KSPIN_LOCK KiFreezeExecutionLock;
// KSPIN_LOCK Ki486CompatibilityLock;
//
// /* Perf */
// ULONG ProcessCount;
// ULONGLONG BootCycles, BootCyclesEnd;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitMachineDependent(VOID)
// {
//     ULONG CpuCount;
//     BOOLEAN FbCaching = FALSE;
//     NTSTATUS Status;
//     ULONG ReturnLength;
//     ULONG i, Affinity, Sample = 0;
//     PFX_SAVE_AREA FxSaveArea;
//     ULONG MXCsrMask = 0xFFBF;
//     CPU_INFO CpuInfo;
//     KI_SAMPLE_MAP Samples[10];
//     PKI_SAMPLE_MAP CurrentSample = Samples;
//     LARGE_IDENTITY_MAP IdentityMap;
//
//     /* Check for large page support */
//     if (KeFeatureBits & KF_LARGE_PAGE)
//     {
//         /* Do an IPI to enable it on all CPUs */
//         if (Ki386CreateIdentityMap(&IdentityMap, Ki386EnableCurrentLargePage, 2))
//             KeIpiGenericCall(Ki386EnableTargetLargePage, (ULONG_PTR)&IdentityMap);
//
//         /* Free the pages allocated for identity map */
//         Ki386FreeIdentityMap(&IdentityMap);
//     }
//
//     /* Check for global page support */
//     if (KeFeatureBits & KF_GLOBAL_PAGE)
//     {
//         /* Do an IPI to enable it on all CPUs */
//         CpuCount = KeNumberProcessors;
//         KeIpiGenericCall(Ki386EnableGlobalPage, (ULONG_PTR)&CpuCount);
//     }
//
//     /* Check for PAT and/or MTRR support */
//     if (KeFeatureBits & (KF_PAT | KF_MTRR))
//     {
//         /* Query the HAL to make sure we can use it */
//         Status = HalQuerySystemInformation(HalFrameBufferCachingInformation,
//                                            sizeof(BOOLEAN),
//                                            &FbCaching,
//                                            &ReturnLength);
//         if ((NT_SUCCESS(Status)) && (FbCaching))
//         {
//             /* We can't, disable it */
//             KeFeatureBits &= ~(KF_PAT | KF_MTRR);
//         }
//     }
//
//     /* Check for PAT support and enable it */
//     if (KeFeatureBits & KF_PAT) KiInitializePAT();
//
//     /* Check for CR4 support */
//     if (KeFeatureBits & KF_CR4)
//     {
//         /* Do an IPI call to enable the Debug Exceptions */
//         CpuCount = KeNumberProcessors;
//         KeIpiGenericCall(Ki386EnableDE, (ULONG_PTR)&CpuCount);
//     }
//
//     /* Check if FXSR was found */
//     if (KeFeatureBits & KF_FXSR)
//     {
//         /* Do an IPI call to enable the FXSR */
//         CpuCount = KeNumberProcessors;
//         KeIpiGenericCall(Ki386EnableFxsr, (ULONG_PTR)&CpuCount);
//
//         /* Check if XMM was found too */
//         if (KeFeatureBits & KF_XMMI)
//         {
//             /* Do an IPI call to enable XMMI exceptions */
//             CpuCount = KeNumberProcessors;
//             KeIpiGenericCall(Ki386EnableXMMIExceptions, (ULONG_PTR)&CpuCount);
//
//             /* FIXME: Implement and enable XMM Page Zeroing for Mm */
//
//             /* Patch the RtlPrefetchMemoryNonTemporal routine to enable it */
//             *(PCHAR)RtlPrefetchMemoryNonTemporal = 0x90; // NOP
//         }
//     }
//
//     /* Check for, and enable SYSENTER support */
//     KiRestoreFastSyscallReturnState();
//
//     /* Loop every CPU */
//     i = KeActiveProcessors;
//     for (Affinity = 1; i; Affinity <<= 1)
//     {
//         /* Check if this is part of the set */
//         if (i & Affinity)
//         {
//             /* Run on this CPU */
//             i &= ~Affinity;
//             KeSetSystemAffinityThread(Affinity);
//
//             /* Reset MHz to 0 for this CPU */
//             KeGetCurrentPrcb()->MHz = 0;
//
//             /* Check if we can use RDTSC */
//             if (KeFeatureBits & KF_RDTSC)
//             {
//                 /* Start sampling loop */
//                 for (;;)
//                 {
//                     /* Do a dummy CPUID to start the sample */
//                     KiCpuId(&CpuInfo, 0);
//
//                     /* Fill out the starting data */
//                     CurrentSample->PerfStart = KeQueryPerformanceCounter(NULL);
//                     CurrentSample->TSCStart = __rdtsc();
//                     CurrentSample->PerfFreq.QuadPart = -50000;
//
//                     /* Sleep for this sample */
//                     KeDelayExecutionThread(KernelMode,
//                                            FALSE,
//                                            &CurrentSample->PerfFreq);
//
//                     /* Do another dummy CPUID */
//                     KiCpuId(&CpuInfo, 0);
//
//                     /* Fill out the ending data */
//                     CurrentSample->PerfEnd =
//                         KeQueryPerformanceCounter(&CurrentSample->PerfFreq);
//                     CurrentSample->TSCEnd = __rdtsc();
//
//                     /* Calculate the differences */
//                     CurrentSample->PerfDelta = CurrentSample->PerfEnd.QuadPart -
//                                                CurrentSample->PerfStart.QuadPart;
//                     CurrentSample->TSCDelta = CurrentSample->TSCEnd -
//                                               CurrentSample->TSCStart;
//
//                     /* Compute CPU Speed */
//                     CurrentSample->MHz = (ULONG)((CurrentSample->TSCDelta *
//                                                   CurrentSample->
//                                                   PerfFreq.QuadPart + 500000) /
//                                                  (CurrentSample->PerfDelta *
//                                                   1000000));
//
//                     /* Check if this isn't the first sample */
//                     if (Sample)
//                     {
//                         /* Check if we got a good precision within 1MHz */
//                         if ((CurrentSample->MHz == CurrentSample[-1].MHz) ||
//                             (CurrentSample->MHz == CurrentSample[-1].MHz + 1) ||
//                             (CurrentSample->MHz == CurrentSample[-1].MHz - 1))
//                         {
//                             /* We did, stop sampling */
//                             break;
//                         }
//                     }
//
//                     /* Move on */
//                     CurrentSample++;
//                     Sample++;
//
//                     if (Sample == RTL_NUMBER_OF(Samples))
//                     {
//                         /* No luck. Average the samples and be done */
//                         ULONG TotalMHz = 0;
//                         while (Sample--)
//                         {
//                             TotalMHz += Samples[Sample].MHz;
//                         }
//                         CurrentSample[-1].MHz = TotalMHz / RTL_NUMBER_OF(Samples);
//                         DPRINT1("Sampling CPU frequency failed. Using average of %lu MHz\n", CurrentSample[-1].MHz);
//                         break;
//                     }
//                 }
//
//                 /* Save the CPU Speed */
//                 KeGetCurrentPrcb()->MHz = CurrentSample[-1].MHz;
//             }
//
//             /* Check if we have MTRR */
//             if (KeFeatureBits & KF_MTRR)
//             {
//                 /* Then manually initialize MTRR for the CPU */
//                 KiInitializeMTRR(i ? FALSE : TRUE);
//             }
//
//             /* Check if we have AMD MTRR and initialize it for the CPU */
//             if (KeFeatureBits & KF_AMDK6MTRR) KiAmdK6InitializeMTRR();
//
//             /* Check if this is a buggy Pentium and apply the fixup if so */
//             if (KiI386PentiumLockErrataPresent) KiI386PentiumLockErrataFixup();
//
//             /* Check if the CPU supports FXSR */
//             if (KeFeatureBits & KF_FXSR)
//             {
//                 /* Get the current thread NPX state */
//                 FxSaveArea = KiGetThreadNpxArea(KeGetCurrentThread());
//
//                 /* Clear initial MXCsr mask */
//                 FxSaveArea->U.FxArea.MXCsrMask = 0;
//
//                 /* Save the current NPX State */
//                 Ke386SaveFpuState(FxSaveArea);
//
//                 /* Check if the current mask doesn't match the reserved bits */
//                 if (FxSaveArea->U.FxArea.MXCsrMask != 0)
//                 {
//                     /* Then use whatever it's holding */
//                     MXCsrMask = FxSaveArea->U.FxArea.MXCsrMask;
//                 }
//
//                 /* Check if nobody set the kernel-wide mask */
//                 if (!KiMXCsrMask)
//                 {
//                     /* Then use the one we calculated above */
//                     KiMXCsrMask = MXCsrMask;
//                 }
//                 else
//                 {
//                     /* Was it set to the same value we found now? */
//                     if (KiMXCsrMask != MXCsrMask)
//                     {
//                         /* No, something is definitely wrong */
//                         KeBugCheckEx(MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED,
//                                      KF_FXSR,
//                                      KiMXCsrMask,
//                                      MXCsrMask,
//                                      0);
//                     }
//                 }
//
//                 /* Now set the kernel mask */
//                 KiMXCsrMask &= MXCsrMask;
//             }
//         }
//     }
//
//     /* Return affinity back to where it was */
//     KeRevertToUserAffinityThread();
//
//     /* NT allows limiting the duration of an ISR with a registry key */
//     if (KiTimeLimitIsrMicroseconds)
//     {
//         /* FIXME: TODO */
//         DPRINT1("ISR Time Limit not yet supported\n");
//     }
//
//     /* Set CR0 features based on detected CPU */
//     KiSetCR0Bits();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializePcr(IN ULONG ProcessorNumber,
//                 IN PKIPCR Pcr,
//                 IN PKIDTENTRY Idt,
//                 IN PKGDTENTRY Gdt,
//                 IN PKTSS Tss,
//                 IN PKTHREAD IdleThread,
//                 IN PVOID DpcStack)
// {
//     /* Setup the TIB */
//     Pcr->NtTib.ExceptionList = EXCEPTION_CHAIN_END;
//     Pcr->NtTib.StackBase = 0;
//     Pcr->NtTib.StackLimit = 0;
//     Pcr->NtTib.Self = NULL;
//
//     /* Set the Current Thread */
//     Pcr->PrcbData.CurrentThread = IdleThread;
//
//     /* Set pointers to ourselves */
//     Pcr->SelfPcr = (PKPCR)Pcr;
//     Pcr->Prcb = &Pcr->PrcbData;
//
//     /* Set the PCR Version */
//     Pcr->MajorVersion = PCR_MAJOR_VERSION;
//     Pcr->MinorVersion = PCR_MINOR_VERSION;
//
//     /* Set the PCRB Version */
//     Pcr->PrcbData.MajorVersion = PRCB_MAJOR_VERSION;
//     Pcr->PrcbData.MinorVersion = PRCB_MINOR_VERSION;
//
//     /* Set the Build Type */
//     Pcr->PrcbData.BuildType = 0;
// #ifndef CONFIG_SMP
//     Pcr->PrcbData.BuildType |= PRCB_BUILD_UNIPROCESSOR;
// #endif
// #if DBG
//     Pcr->PrcbData.BuildType |= PRCB_BUILD_DEBUG;
// #endif
//
//     /* Set the Processor Number and current Processor Mask */
//     Pcr->PrcbData.Number = (UCHAR)ProcessorNumber;
//     Pcr->PrcbData.SetMember = 1 << ProcessorNumber;
//
//     /* Set the PRCB for this Processor */
//     KiProcessorBlock[ProcessorNumber] = Pcr->Prcb;
//
//     /* Start us out at PASSIVE_LEVEL */
//     Pcr->Irql = PASSIVE_LEVEL;
//
//     /* Set the GDI, IDT, TSS and DPC Stack */
//     Pcr->GDT = (PVOID)Gdt;
//     Pcr->IDT = Idt;
//     Pcr->TSS = Tss;
//     Pcr->TssCopy = Tss;
//     Pcr->PrcbData.DpcStack = DpcStack;
//
//     /* Setup the processor set */
//     Pcr->PrcbData.MultiThreadProcessorSet = Pcr->PrcbData.SetMember;
// }
//
// static
// CODE_SEG("INIT")
// VOID
// KiVerifyCpuFeatures(PKPRCB Prcb)
// {
//     CPU_INFO CpuInfo;
//
//     // 1. Check CPUID support
//     ULONG EFlags = __readeflags();
//
//     /* XOR out the ID bit and update EFlags */
//     ULONG NewEFlags = EFlags ^ EFLAGS_ID;
//     __writeeflags(NewEFlags);
//
//     /* Get them back and see if they were modified */
//     NewEFlags = __readeflags();
//
//     if (NewEFlags == EFlags)
//     {
//         /* The modification did not work, so CPUID is not supported. */
//         KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x1, 0, 0, 0);
//     }
//     else
//     {
//         /* CPUID is supported. Set the ID Bit again. */
//         EFlags |= EFLAGS_ID;
//         __writeeflags(EFlags);
//     }
//
//     /* Peform CPUID 0 to see if CPUID 1 is supported */
//     KiCpuId(&CpuInfo, 0);
//     if (CpuInfo.Eax == 0)
//     {
//         // 0x1 - missing CPUID instruction
//         KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x1, 0, 0, 0);
//     }
//
//     // 2. Detect and set the CPU Type
//     KiSetProcessorType();
//
//     if (Prcb->CpuType == 3)
//         KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x386, 0, 0, 0);
//
//     // 3. Finally, obtain CPU features.
//     ULONG64 FeatureBits = KiGetFeatureBits();
//
//     // 4. Verify it supports everything we need.
//     if (!(FeatureBits & KF_RDTSC))
//     {
//         // 0x2 - missing CPUID features
//         // second paramenter - edx flag which is missing
//         KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000010, 0, 0);
//     }
//
//     if (!(FeatureBits & KF_CMPXCHG8B))
//     {
//         KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000100, 0, 0);
//     }
//
//     // Check x87 FPU is present. FIXME: put this into FeatureBits?
//     KiCpuId(&CpuInfo, 1);
//
//     if (!(CpuInfo.Edx & 0x00000001))
//     {
//         KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000001, 0, 0);
//     }
//
//     // Set up FPU-related CR0 flags.
//     ULONG Cr0 = __readcr0();
//     // Disable emulation and monitoring.
//     Cr0 &= ~(CR0_EM | CR0_MP);
//     // Enable FPU exceptions.
//     Cr0 |= CR0_NE;
//
//     __writecr0(Cr0);
//
//     // Check for Pentium FPU bug.
//     if (KiIsNpxErrataPresent())
//     {
//         KeBugCheckEx(UNSUPPORTED_PROCESSOR, 0x2, 0x00000001, 0, 0);
//     }
//
//     // 5. Save feature bits.
//     Prcb->FeatureBits = (ULONG)FeatureBits;
//     Prcb->FeatureBitsHigh = FeatureBits >> 32;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeKernel(IN PKPROCESS InitProcess,
//                    IN PKTHREAD InitThread,
//                    IN PVOID IdleStack,
//                    IN PKPRCB Prcb,
//                    IN CCHAR Number,
//                    IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     ULONG PageDirectory[2];
//     PVOID DpcStack;
//     KIRQL DummyIrql;
//
//     /* Initialize the Power Management Support for this PRCB */
//     PoInitializePrcb(Prcb);
//
//     /* Set boot-level flags */
//     if (Number == 0)
//         KeFeatureBits = Prcb->FeatureBits | (ULONG64)Prcb->FeatureBitsHigh << 32;
//
//     /* Set the default NX policy (opt-in) */
//     SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_OPTIN;
//
//     /* Check if NPX is always on */
//     if (strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=ALWAYSON"))
//     {
//         /* Set it always on */
//         SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_ALWAYSON;
//         KeFeatureBits |= KF_NX_ENABLED;
//     }
//     else if (strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=OPTOUT"))
//     {
//         /* Set it in opt-out mode */
//         SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_OPTOUT;
//         KeFeatureBits |= KF_NX_ENABLED;
//     }
//     else if ((strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=OPTIN")) ||
//              (strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE")))
//     {
//         /* Set the feature bits */
//         KeFeatureBits |= KF_NX_ENABLED;
//     }
//     else if ((strstr(KeLoaderBlock->LoadOptions, "NOEXECUTE=ALWAYSOFF")) ||
//              (strstr(KeLoaderBlock->LoadOptions, "EXECUTE")))
//     {
//         /* Set disabled mode */
//         SharedUserData->NXSupportPolicy = NX_SUPPORT_POLICY_ALWAYSOFF;
//         KeFeatureBits |= KF_NX_DISABLED;
//     }
//
//     /* Save CPU state */
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
//
// #if DBG
//     /* Print applied kernel features/policies and boot CPU features */
//     if (Number == 0)
//         KiReportCpuFeatures();
// #endif
//
//     /* Get cache line information for this CPU */
//     KiGetCacheInformation();
//
//     /* Initialize spinlocks and DPC data */
//     KiInitSpinLocks(Prcb, Number);
//
//     /* Set Node Data */
//     Prcb->ParentNode = KeNodeBlock[0];
//     Prcb->ParentNode->ProcessorMask |= Prcb->SetMember;
//
//     /* Check if this is the Boot CPU */
//     if (!Number)
//     {
//         /* Set boot-level flags */
//         KeI386CpuType = Prcb->CpuType;
//         KeI386CpuStep = Prcb->CpuStep;
//         KeProcessorArchitecture = PROCESSOR_ARCHITECTURE_INTEL;
//         KeProcessorLevel = (USHORT)Prcb->CpuType;
//         if (Prcb->CpuID) KeProcessorRevision = Prcb->CpuStep;
//         KeI386FxsrPresent = (KeFeatureBits & KF_FXSR) ? TRUE : FALSE;
//         KeI386XMMIPresent = (KeFeatureBits & KF_XMMI) ? TRUE : FALSE;
//
//         /* Set the current MP Master KPRCB to the Boot PRCB */
//         Prcb->MultiThreadSetMaster = Prcb;
//
//         /* Lower to APC_LEVEL */
//         KeLowerIrql(APC_LEVEL);
//
//         /* Initialize some spinlocks */
//         KeInitializeSpinLock(&KiFreezeExecutionLock);
//         KeInitializeSpinLock(&Ki486CompatibilityLock);
//
//         /* Initialize portable parts of the OS */
//         KiInitSystem();
//
//         /* Initialize the Idle Process and the Process Listhead */
//         InitializeListHead(&KiProcessListHead);
//         PageDirectory[0] = 0;
//         PageDirectory[1] = 0;
//         KeInitializeProcess(InitProcess,
//                             0,
//                             MAXULONG_PTR,
//                             PageDirectory,
//                             FALSE);
//         InitProcess->QuantumReset = MAXCHAR;
//     }
//     else
//     {
//         /* FIXME */
//         DPRINT1("Starting CPU#%u - you are brave\n", Number);
//     }
//
//     /* Setup the Idle Thread */
//     KeInitializeThread(InitProcess,
//                        InitThread,
//                        NULL,
//                        NULL,
//                        NULL,
//                        NULL,
//                        NULL,
//                        IdleStack);
//     InitThread->NextProcessor = Number;
//     InitThread->Priority = HIGH_PRIORITY;
//     InitThread->State = Running;
//     InitThread->Affinity = 1 << Number;
//     InitThread->WaitIrql = DISPATCH_LEVEL;
//     InitProcess->ActiveProcessors |= 1 << Number;
//
//     /* HACK for MmUpdatePageDir */
//     ((PETHREAD)InitThread)->ThreadsProcess = (PEPROCESS)InitProcess;
//
//     /* Set basic CPU Features that user mode can read */
//     SharedUserData->ProcessorFeatures[PF_FLOATING_POINT_PRECISION_ERRATA] = FALSE;
//     SharedUserData->ProcessorFeatures[PF_MMX_INSTRUCTIONS_AVAILABLE] =
//         (KeFeatureBits & KF_MMX) ? TRUE: FALSE;
//     SharedUserData->ProcessorFeatures[PF_COMPARE_EXCHANGE_DOUBLE] =
//         (KeFeatureBits & KF_CMPXCHG8B) ? TRUE: FALSE;
//     SharedUserData->ProcessorFeatures[PF_XMMI_INSTRUCTIONS_AVAILABLE] =
//         ((KeFeatureBits & KF_FXSR) && (KeFeatureBits & KF_XMMI)) ? TRUE: FALSE;
//     SharedUserData->ProcessorFeatures[PF_XMMI64_INSTRUCTIONS_AVAILABLE] =
//         ((KeFeatureBits & KF_FXSR) && (KeFeatureBits & KF_XMMI64)) ? TRUE: FALSE;
//     SharedUserData->ProcessorFeatures[PF_3DNOW_INSTRUCTIONS_AVAILABLE] =
//         (KeFeatureBits & KF_3DNOW) ? TRUE: FALSE;
//     SharedUserData->ProcessorFeatures[PF_RDTSC_INSTRUCTION_AVAILABLE] =
//         (KeFeatureBits & KF_RDTSC) ? TRUE: FALSE;
//     SharedUserData->ProcessorFeatures[PF_RDRAND_INSTRUCTION_AVAILABLE] =
//         (KeFeatureBits & KF_RDRAND) ? TRUE : FALSE;
//     // Note: On x86 we lack support for saving/restoring SSE state
//     SharedUserData->ProcessorFeatures[PF_SSE3_INSTRUCTIONS_AVAILABLE] =
//         (KeFeatureBits & KF_SSE3) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_SSSE3_INSTRUCTIONS_AVAILABLE] =
//         (KeFeatureBits & KF_SSSE3) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_SSE4_1_INSTRUCTIONS_AVAILABLE] =
//         (KeFeatureBits & KF_SSE4_1) ? TRUE : FALSE;
//     SharedUserData->ProcessorFeatures[PF_SSE4_2_INSTRUCTIONS_AVAILABLE] =
//         (KeFeatureBits & KF_SSE4_2) ? TRUE : FALSE;
//
//     /* Set up the thread-related fields in the PRCB */
//     Prcb->CurrentThread = InitThread;
//     Prcb->NextThread = NULL;
//     Prcb->IdleThread = InitThread;
//
//     /* Initialize the Kernel Executive */
//     ExpInitializeExecutive(Number, LoaderBlock);
//
//     /* Only do this on the boot CPU */
//     if (!Number)
//     {
//         /* Calculate the time reciprocal */
//         KiTimeIncrementReciprocal =
//             KiComputeReciprocal(KeMaximumIncrement,
//                                 &KiTimeIncrementShiftCount);
//
//         /* Update DPC Values in case they got updated by the executive */
//         Prcb->MaximumDpcQueueDepth = KiMaximumDpcQueueDepth;
//         Prcb->MinimumDpcRate = KiMinimumDpcRate;
//         Prcb->AdjustDpcThreshold = KiAdjustDpcThreshold;
//
//         /* Allocate the DPC Stack */
//         DpcStack = MmCreateKernelStack(FALSE, 0);
//         if (!DpcStack) KeBugCheckEx(NO_PAGES_AVAILABLE, 1, 0, 0, 0);
//         Prcb->DpcStack = DpcStack;
//
//         /* Allocate the IOPM save area */
//         Ki386IopmSaveArea = ExAllocatePoolWithTag(PagedPool,
//                                                   IOPM_SIZE,
//                                                   TAG_KERNEL);
//         if (!Ki386IopmSaveArea)
//         {
//             /* Bugcheck. We need this for V86/VDM support. */
//             KeBugCheckEx(NO_PAGES_AVAILABLE, 2, IOPM_SIZE, 0, 0);
//         }
//     }
//
//     /* Raise to Dispatch */
//     KeRaiseIrql(DISPATCH_LEVEL, &DummyIrql);
//
//     /* Set the Idle Priority to 0. This will jump into Phase 1 */
//     KeSetPriorityThread(InitThread, 0);
//
//     /* If there's no thread scheduled, put this CPU in the Idle summary */
//     KiAcquirePrcbLock(Prcb);
//     if (!Prcb->NextThread) KiIdleSummary |= 1 << Number;
//     KiReleasePrcbLock(Prcb);
//
//     /* Raise back to HIGH_LEVEL and clear the PRCB for the loader block */
//     KeRaiseIrql(HIGH_LEVEL, &DummyIrql);
//     LoaderBlock->Prcb = 0;
// }
//
// CODE_SEG("INIT")
// VOID
// FASTCALL
// KiGetMachineBootPointers(IN PKGDTENTRY *Gdt,
//                          IN PKIDTENTRY *Idt,
//                          IN PKIPCR *Pcr,
//                          IN PKTSS *Tss)
// {
//     KDESCRIPTOR GdtDescriptor, IdtDescriptor;
//     KGDTENTRY TssSelector, PcrSelector;
//     USHORT Tr, Fs;
//
//     /* Get GDT and IDT descriptors */
//     Ke386GetGlobalDescriptorTable(&GdtDescriptor.Limit);
//     __sidt(&IdtDescriptor.Limit);
//
//     /* Save IDT and GDT */
//     *Gdt = (PKGDTENTRY)GdtDescriptor.Base;
//     *Idt = (PKIDTENTRY)IdtDescriptor.Base;
//
//     /* Get TSS and FS Selectors */
//     Tr = Ke386GetTr();
//     Fs = Ke386GetFs();
//
//     /* Get PCR Selector, mask it and get its GDT Entry */
//     PcrSelector = *(PKGDTENTRY)((ULONG_PTR)*Gdt + (Fs & ~RPL_MASK));
//
//     /* Get the KPCR itself */
//     *Pcr = (PKIPCR)(ULONG_PTR)(PcrSelector.BaseLow |
//                                PcrSelector.HighWord.Bytes.BaseMid << 16 |
//                                PcrSelector.HighWord.Bytes.BaseHi << 24);
//
//     /* Get TSS Selector, mask it and get its GDT Entry */
//     TssSelector = *(PKGDTENTRY)((ULONG_PTR)*Gdt + (Tr & ~RPL_MASK));
//
//     /* Get the KTSS itself */
//     *Tss = (PKTSS)(ULONG_PTR)(TssSelector.BaseLow |
//                               TssSelector.HighWord.Bytes.BaseMid << 16 |
//                               TssSelector.HighWord.Bytes.BaseHi << 24);
// }
//
// CODE_SEG("INIT")
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KiSystemStartupBootStack(VOID)
// {
//     PKTHREAD Thread;
//
//     /* Initialize the kernel for the current CPU */
//     KiInitializeKernel(&KiInitialProcess.Pcb,
//                        (PKTHREAD)KeLoaderBlock->Thread,
//                        (PVOID)(KeLoaderBlock->KernelStack & ~3),
//                        (PKPRCB)__readfsdword(KPCR_PRCB),
//                        KeNumberProcessors - 1,
//                        KeLoaderBlock);
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
//     /* Jump into the idle loop */
//     KiIdleLoop();
// }
//
// static
// VOID
// KiMarkPageAsReadOnly(
//     PVOID Address)
// {
//     PHARDWARE_PTE_X86 PointerPte;
//
//     /* Make sure the address is page aligned */
//     ASSERT(ALIGN_DOWN_POINTER_BY(Address, PAGE_SIZE) == Address);
//
//     /* Get the PTE address */
//     PointerPte = ((PHARDWARE_PTE_X86)PTE_BASE) + ((ULONG_PTR)Address / PAGE_SIZE);
//     ASSERT(PointerPte->Valid);
//     ASSERT(PointerPte->Write);
//
//     /* Set as read-only */
//     PointerPte->Write = 0;
//
//     /* Flush the TLB entry */
//     __invlpg(Address);
// }
//
// CODE_SEG("INIT")
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KiSystemStartup(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     ULONG Cpu;
//     PKTHREAD InitialThread;
//     ULONG InitialStack;
//     PKGDTENTRY Gdt;
//     PKIDTENTRY Idt;
//     KIDTENTRY NmiEntry, DoubleFaultEntry;
//     PKTSS Tss;
//     PKIPCR Pcr;
//     KIRQL DummyIrql;
//
//     /* Boot cycles timestamp */
//     BootCycles = __rdtsc();
//
//     /* Save the loader block and get the current CPU */
//     KeLoaderBlock = LoaderBlock;
//     Cpu = KeNumberProcessors;
//     if (!Cpu)
//     {
//         /* If this is the boot CPU, set FS and the CPU Number*/
//         Ke386SetFs(KGDT_R0_PCR);
//         __writefsdword(KPCR_PROCESSOR_NUMBER, Cpu);
//
//         /* Set the initial stack and idle thread as well */
//         LoaderBlock->KernelStack = (ULONG_PTR)P0BootStack;
//         LoaderBlock->Thread = (ULONG_PTR)&KiInitialThread;
//     }
//
//     /* Save the initial thread and stack */
//     InitialStack = LoaderBlock->KernelStack;
//     InitialThread = (PKTHREAD)LoaderBlock->Thread;
//
//     /* Clean the APC List Head */
//     InitializeListHead(&InitialThread->ApcState.ApcListHead[KernelMode]);
//
//     /* Initialize the machine type */
//     KiInitializeMachineType();
//
//     /* Skip initial setup if this isn't the Boot CPU */
//     if (Cpu) goto AppCpuInit;
//
//     /* Get GDT, IDT, PCR and TSS pointers */
//     KiGetMachineBootPointers(&Gdt, &Idt, &Pcr, &Tss);
//
//     /* Setup the TSS descriptors and entries */
//     Ki386InitializeTss(Tss, Idt, Gdt);
//
//     /* Initialize the PCR */
//     RtlZeroMemory(Pcr, PAGE_SIZE);
//     KiInitializePcr(Cpu,
//                     Pcr,
//                     Idt,
//                     Gdt,
//                     Tss,
//                     InitialThread,
//                     (PVOID)KiDoubleFaultStack);
//
//     /* Set us as the current process */
//     InitialThread->ApcState.Process = &KiInitialProcess.Pcb;
//
//     /* Clear DR6/7 to cleanup bootloader debugging */
//     __writefsdword(KPCR_TEB, 0);
//     __writefsdword(KPCR_DR6, 0);
//     __writefsdword(KPCR_DR7, 0);
//
//     /* Setup the IDT */
//     KeInitExceptions();
//
//     /* Load Ring 3 selectors for DS/ES */
//     Ke386SetDs(KGDT_R3_DATA | RPL_MASK);
//     Ke386SetEs(KGDT_R3_DATA | RPL_MASK);
//
//     /* Save NMI and double fault traps */
//     RtlCopyMemory(&NmiEntry, &Idt[2], sizeof(KIDTENTRY));
//     RtlCopyMemory(&DoubleFaultEntry, &Idt[8], sizeof(KIDTENTRY));
//
//     /* Copy kernel's trap handlers */
//     RtlCopyMemory(Idt,
//                   (PVOID)KiIdtDescriptor.Base,
//                   KiIdtDescriptor.Limit + 1);
//
//     /* Restore NMI and double fault */
//     RtlCopyMemory(&Idt[2], &NmiEntry, sizeof(KIDTENTRY));
//     RtlCopyMemory(&Idt[8], &DoubleFaultEntry, sizeof(KIDTENTRY));
//
// AppCpuInit:
//     //TODO: We don't setup IPIs yet so freeze other processors here.
//     if (Cpu)
//     {
//         KeMemoryBarrier();
//         LoaderBlock->Prcb = 0;
//
//         for (;;)
//         {
//             YieldProcessor();
//         }
//     }
//
//     /* Loop until we can release the freeze lock */
//     do
//     {
//         /* Loop until execution can continue */
//         while (*(volatile PKSPIN_LOCK*)&KiFreezeExecutionLock == (PVOID)1);
//     } while(InterlockedBitTestAndSet((PLONG)&KiFreezeExecutionLock, 0));
//
//     /* Setup CPU-related fields */
//     __writefsdword(KPCR_NUMBER, Cpu);
//     __writefsdword(KPCR_SET_MEMBER, 1 << Cpu);
//     __writefsdword(KPCR_SET_MEMBER_COPY, 1 << Cpu);
//     __writefsdword(KPCR_PRCB_SET_MEMBER, 1 << Cpu);
//
//     KiVerifyCpuFeatures(Pcr->Prcb);
//
//     /* Initialize the Processor with HAL */
//     HalInitializeProcessor(Cpu, KeLoaderBlock);
//
//     /* Set active processors */
//     KeActiveProcessors |= __readfsdword(KPCR_SET_MEMBER);
//     KeNumberProcessors++;
//
//     /* Check if this is the boot CPU */
//     if (!Cpu)
//     {
//         /* Initialize debugging system */
//         KdInitSystem(0, KeLoaderBlock);
//
//         /* Check for break-in */
//         if (KdPollBreakIn()) DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
//
//         /* Make the lowest page of the boot and double fault stack read-only */
//         KiMarkPageAsReadOnly(P0BootStackData);
//         KiMarkPageAsReadOnly(KiDoubleFaultStackData);
//     }
//
//     /* Raise to HIGH_LEVEL */
//     KeRaiseIrql(HIGH_LEVEL, &DummyIrql);
//
//     /* Switch to new kernel stack and start kernel bootstrapping */
//     KiSwitchToBootStack(InitialStack & ~3);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
