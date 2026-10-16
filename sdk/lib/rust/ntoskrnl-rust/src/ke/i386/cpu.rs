use crate::ke::ke_defs::*;
// Explicit imports to disambiguate ntkrnl_sys::mm vs ntkrnl_sys::ddk glob conflicts
use ntkrnl_sys::mm::{KeGetCurrentThread, ULONG64};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    _KPROCESSOR_STATE, _KTRAP_FRAME, _KEXCEPTION_FRAME, _FLOATING_SAVE_AREA,
    _KFLOATING_SAVE, _CONTEXT, _KIDTENTRY, _KGDTENTRY, _KTSS, _CPU_INFO,
    PKPROCESSOR_STATE, PKTRAP_FRAME, PKEXCEPTION_FRAME, PFLOATING_SAVE_AREA,
    PKFLOATING_SAVE, PKIDTENTRY, PKGDTENTRY, PKTSS, KAFFINITY, PKIPCR,
    PKIPI_CONTEXT, PKTHREAD,
};

use crate::ke::krnlinit::{KeLoaderBlock, KeFeatureBits};
use crate::ke::processor::KeActiveProcessors;
extern "system" {
    fn RtlFillMemory(Destination: PVOID, Length: SIZE_T, Fill: UCHAR);
    fn KiCpuId(CpuInfo: *mut _CPU_INFO, Function: ULONG);
    fn Ke386FnInit();
    fn Ke386FxSave(Area: *mut FX_SAVE_AREA);
    fn Ke386FxStore(Area: *mut FX_SAVE_AREA);
    fn Ke386FnSave(Area: *mut _FLOATING_SAVE_AREA);
    fn Ke386SetTr(Selector: USHORT);
    fn Ke386GetSs() -> USHORT;
    fn Ke386SetCr2(Value: ULONG_PTR);
    fn Ke386SetGlobalDescriptorTable(Limit: *const USHORT);
    fn Ke386GetGlobalDescriptorTable(Limit: *mut USHORT);
    fn Ke386GetTr() -> USHORT;
    fn Ke386SetLocalDescriptorTable(Selector: USHORT);
    fn Ke386GetLocalDescriptorTable(Selector: *mut USHORT);
}
// Assembly labels from trap.s — cdecl decoration (_Name)
extern "C" {
    fn KiFastCallEntry();
    fn KiTrap08();
    fn KiTrap02();
    fn KiTrap13();
}
// Trap exit stubs from KiTrapExitStub macro — fastcall decoration (@Name@4)
extern "fastcall" {
    fn KiSystemCallTrapReturn(_dummy: PVOID) -> !;
    fn KiSystemCallSysExitReturn(_dummy: PVOID) -> !;
}
extern "system" {
    fn Ke386SaveFpuState(Area: *mut FX_SAVE_AREA);
    fn Ke386RestoreFpuState(Area: *const FX_SAVE_AREA);
    fn Ke386ClearFpExceptions();
    fn KiGetThreadNpxArea(Thread: PKTHREAD) -> *mut FX_SAVE_AREA;
    fn KiComputeIopmOffset(MapNumber: USHORT) -> USHORT;
    static mut KiDoubleFaultStack: ULONG_PTR;
    static mut KiFastCallExitHandler: PVOID;
}

#[repr(C)]
struct FX_SAVE_AREA {
    // Union of FNSAVE_FORMAT/FXSAVE_FORMAT. The C FXSAVE_FORMAT is 520 bytes,
    // NOT 512: it carries a trailing `Align16Byte[8]` (ndk/i386/ketypes.h) after
    // Reserved4[224]. Modelling U as 512 puts NpxSavedCpu/Cr0NpxState 8 bytes too
    // low (+0x200/+0x204 instead of +0x208/+0x20C) — KiFlushNPXState then reads
    // Cr0NpxState from inside the FXSAVE register/align region → pointer garbage
    // and a 0x7F/0x0A. Must match bindgen _FX_SAVE_AREA (size 528, Cr0NpxState@524).
    U: [UCHAR; 520],
    NpxSavedCpu: ULONG,
    Cr0NpxState: ULONG,
}

#[repr(C)]
struct FLOATING_SAVE_CONTEXT {
    Buffer: PVOID,
    PfxSaveArea: *mut FX_SAVE_AREA,
    CurrentThread: PKTHREAD,
    OldNpxIrql: KIRQL,
}

const TAG_FLOATING_POINT_CONTEXT: ULONG = 0x6F437046;
const TAG_FLOATING_POINT_FX: ULONG = 0x78467046;
const INVALID_FLOATING_POINT_STATE: ULONG = 0xE7;

#[no_mangle]
pub static mut KiDoubleFaultTSS: [UCHAR; KTSS_IO_MAPS as usize] = [0; KTSS_IO_MAPS as usize];
#[no_mangle]
pub static mut KiNMITSS: [UCHAR; KTSS_IO_MAPS as usize] = [0; KTSS_IO_MAPS as usize];

#[no_mangle]
pub static mut KeI386CpuType: ULONG = 0;
#[no_mangle]
pub static mut KeI386CpuStep: ULONG = 0;
#[no_mangle]
pub static mut KiFastSystemCallDisable: ULONG = 0;
#[no_mangle]
pub static mut KeI386NpxPresent: ULONG = 1;
#[no_mangle]
pub static mut KiMXCsrMask: ULONG = 0;
#[no_mangle]
pub static mut MxcsrFeatureMask: ULONG = 0;
#[no_mangle]
pub static mut KeI386XMMIPresent: ULONG = 0;
#[no_mangle]
pub static mut KeI386FxsrPresent: ULONG = 0;
#[no_mangle]
pub static mut KeI386MachineType: ULONG = 0;
#[no_mangle]
pub static mut Ke386Pae: ULONG = 0;
#[no_mangle]
pub static mut Ke386NoExecute: ULONG = 0;
#[no_mangle]
pub static mut KeLargestCacheLine: ULONG = 0x40;
#[no_mangle]
pub static mut KeDcacheFlushCount: ULONG = 0;
#[no_mangle]
pub static mut KeIcacheFlushCount: ULONG = 0;
#[no_mangle]
pub static mut KiDmaIoCoherency: ULONG = 0;
#[no_mangle]
pub static mut KePrefetchNTAGranularity: ULONG = 32;
#[no_mangle]
pub static mut KiI386PentiumLockErrataPresent: BOOLEAN = 0;
#[no_mangle]
pub static mut KiSMTProcessorsPresent: BOOLEAN = 0;

#[no_mangle]
pub static mut KiSystemCallExitAdjust: UCHAR = 0;
#[no_mangle]
pub static mut KiSystemCallExitAdjusted: UCHAR = 0;
#[no_mangle]
pub static mut KiFastCallCopyDoneOnce: BOOLEAN = 0;

static mut KiTbFlushTimeStamp: LONG = 0;

static CMP_INTEL_ID: &[u8; 13] = b"GenuineIntel\0";
static CMP_AMD_ID: &[u8; 13] = b"AuthenticAMD\0";
static CMP_CYRIX_ID: &[u8; 13] = b"CyrixInstead\0";
static CMP_TRANSMETA_ID: &[u8; 13] = b"GenuineTMx86\0";
static CMP_CENTAUR_ID: &[u8; 13] = b"CentaurHauls\0";
static CMP_RISE_ID: &[u8; 13] = b"RiseRiseRise\0";

const CPU_INTEL: ULONG = 1;
const CPU_AMD: ULONG = 2;
const CPU_CYRIX: ULONG = 3;
const CPU_TRANSMETA: ULONG = 4;
const CPU_CENTAUR: ULONG = 5;
const CPU_RISE: ULONG = 6;
const CPU_UNKNOWN: ULONG = 0;

const FXSAVE_ALIGN: usize = 15;

const X86_FEATURE_VME: ULONG = 0x00000002;
const X86_FEATURE_PSE: ULONG = 0x00000008;
const X86_FEATURE_TSC: ULONG = 0x00000010;
const X86_FEATURE_CX8: ULONG = 0x00000100;
const X86_FEATURE_SYSCALL: ULONG = 0x00000800;
const X86_FEATURE_MTRR: ULONG = 0x00001000;
const X86_FEATURE_PGE: ULONG = 0x00002000;
const X86_FEATURE_CMOV: ULONG = 0x00008000;
const X86_FEATURE_PAT: ULONG = 0x00010000;
const X86_FEATURE_DS: ULONG = 0x00200000;
const X86_FEATURE_MMX: ULONG = 0x00800000;
const X86_FEATURE_FXSR: ULONG = 0x01000000;
const X86_FEATURE_SSE: ULONG = 0x02000000;
const X86_FEATURE_SSE2: ULONG = 0x04000000;
const X86_FEATURE_HT: ULONG = 0x10000000;
const X86_FEATURE_NX: ULONG = 0x00100000;
const X86_FEATURE_PAE: ULONG = 0x00000040;
const X86_FEATURE_APIC: ULONG = 0x00000200;

const X86_FEATURE_SSE3: ULONG = 0x00000001;
const X86_FEATURE_SSSE3: ULONG = 0x00000200;
const X86_FEATURE_SSE4_1: ULONG = 0x00080000;
const X86_FEATURE_SSE4_2: ULONG = 0x00100000;
const X86_FEATURE_XSAVE: ULONG = 0x04000000;
const X86_FEATURE_RDRAND: ULONG = 0x40000000;

const KF_V86_VIS: ULONG64 = 0x00000001;
const KF_RDTSC: ULONG64 = 0x00000002;
const KF_CR4: ULONG64 = 0x00000004;
const KF_CMOV: ULONG64 = 0x00000008;
const KF_GLOBAL_PAGE: ULONG64 = 0x00000010;
const KF_LARGE_PAGE: ULONG64 = 0x00000020;
const KF_MTRR: ULONG64 = 0x00000040;
const KF_CMPXCHG8B: ULONG64 = 0x00000080;
const KF_MMX: ULONG64 = 0x00000100;
const KF_WORKING_PTE: ULONG64 = 0x00000200;
const KF_PAT: ULONG64 = 0x00000400;
const KF_FXSR: ULONG64 = 0x00000800;
const KF_FAST_SYSCALL: ULONG64 = 0x00001000;
const KF_XMMI: ULONG64 = 0x00002000;
const KF_3DNOW: ULONG64 = 0x00004000;
const KF_AMDK6MTRR: ULONG64 = 0x00008000;
const KF_XMMI64: ULONG64 = 0x00010000;
const KF_DTS: ULONG64 = 0x00020000;
const KF_NX_BIT: ULONG64 = 0x20000000;
const KF_NX_DISABLED: ULONG64 = 0x40000000;
const KF_NX_ENABLED: ULONG64 = 0x80000000;
const KF_SSE3: ULONG64 = 0x00080000;
const KF_SSSE3: ULONG64 = 0x4000000000;
const KF_SSE4_1: ULONG64 = 0x8000000000;
const KF_SSE4_2: ULONG64 = 0x10000000000;
const KF_XSTATE: ULONG64 = 0x400000;
const KF_RDRAND: ULONG64 = 0x02000000;

const CR0_WP: ULONG = 0x00010000;
const CR0_MP: ULONG = 0x00000002;
const CR0_TS: ULONG = 0x00000008;
const CR0_EM: ULONG = 0x00000004;
const CR4_DE: ULONG = 0x00000008;
const CR4_PGE: ULONG = 0x00000080;
const CR4_FXSR: ULONG = 0x00000200;
const CR4_XMMEXCPT: ULONG = 0x00000400;

const NPX_STATE_NOT_LOADED: UCHAR = 0xA;
const NPX_STATE_LOADED: UCHAR = 0x0;

const I386_TSS: UCHAR = 0x00000009;
const I386_TASK_GATE: UCHAR = 0x00000005;
const I386_INTERRUPT_GATE: UCHAR = 0x0000000E;

const KGDT_R0_CODE: USHORT = 0x08;
const KGDT_R0_DATA: USHORT = 0x10;
const KGDT_R3_DATA: USHORT = 0x20;
const KGDT_R0_PCR: USHORT = 0x30;
const KGDT_TSS: USHORT = 0x28;
const KGDT_DF_TSS: USHORT = 0x50;
const KGDT_NMI_TSS: USHORT = 0x58;
const RPL_MASK: USHORT = 3;

const KTSS_IO_MAPS: ULONG = 0x00000068;
const IOPM_COUNT: usize = 1;
const IOPM_FULL_SIZE: usize = 0x00002004;
const IOPM_DIRECTION_MAP_SIZE: usize = 32;
const IO_ACCESS_MAP_NONE: USHORT = 0;

const CX86_CCR1: UCHAR = 0xc1;

const EFLAGS_INTERRUPT_MASK: ULONG = 0x200;

#[cfg(target_arch = "x86")]
unsafe fn get_cx86(reg: UCHAR) -> UCHAR {
    let result: UCHAR;
    core::arch::asm!(
        "out 0x22, al",
        "in al, 0x23",
        inout("al") reg => result,
        options(nostack, preserves_flags)
    );
    result
}

#[cfg(target_arch = "x86")]
unsafe fn set_cx86(reg: UCHAR, data: UCHAR) {
    core::arch::asm!(
        "out 0x22, al",
        in("al") reg,
        options(nostack, preserves_flags)
    );
    core::arch::asm!(
        "out 0x23, al",
        in("al") data,
        options(nostack, preserves_flags)
    );
}

unsafe fn strcmp(s1: *const i8, s2: *const i8) -> i32 {
    let mut i = 0;
    loop {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return (c1 as u8 as i32) - (c2 as u8 as i32);
        }
        if c1 == 0 {
            return 0;
        }
        i += 1;
    }
}

// CODE_SEG("INIT")
unsafe fn KiGetCpuVendor() -> ULONG {
    let Prcb = KeGetCurrentPrcb();
    let mut CpuInfo: _CPU_INFO = core::mem::zeroed();

    KiCpuId(&mut CpuInfo, 0);

    core::ptr::write_unaligned(
        (*Prcb).VendorString.as_mut_ptr().add(0) as *mut ULONG,
        CpuInfo.__bindgen_anon_1.Ebx,
    );
    core::ptr::write_unaligned(
        (*Prcb).VendorString.as_mut_ptr().add(4) as *mut ULONG,
        CpuInfo.__bindgen_anon_1.Edx,
    );
    core::ptr::write_unaligned(
        (*Prcb).VendorString.as_mut_ptr().add(8) as *mut ULONG,
        CpuInfo.__bindgen_anon_1.Ecx,
    );
    (*Prcb).VendorString[12] = 0;

    if strcmp((*Prcb).VendorString.as_ptr(), CMP_INTEL_ID.as_ptr() as *const i8) == 0 {
        return CPU_INTEL;
    } else if strcmp((*Prcb).VendorString.as_ptr(), CMP_AMD_ID.as_ptr() as *const i8) == 0 {
        return CPU_AMD;
    } else if strcmp((*Prcb).VendorString.as_ptr(), CMP_CYRIX_ID.as_ptr() as *const i8) == 0 {
        return CPU_CYRIX;
    } else if strcmp((*Prcb).VendorString.as_ptr(), CMP_TRANSMETA_ID.as_ptr() as *const i8) == 0 {
        return CPU_TRANSMETA;
    } else if strcmp((*Prcb).VendorString.as_ptr(), CMP_CENTAUR_ID.as_ptr() as *const i8) == 0 {
        return CPU_CENTAUR;
    } else if strcmp((*Prcb).VendorString.as_ptr(), CMP_RISE_ID.as_ptr() as *const i8) == 0 {
        return CPU_RISE;
    }

    CPU_UNKNOWN
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSetProcessorType() {
    let mut CpuInfo: _CPU_INFO = core::mem::zeroed();
    let mut Stepping: ULONG;
    let mut Type: ULONG;
    let mut ExtendModel: BOOLEAN;

    KiCpuId(&mut CpuInfo, 1);

    let CpuSignature = CpuInfo.__bindgen_anon_1.Eax;
    let Step = CpuSignature & 0xF;
    let Model = (CpuSignature >> 4) & 0xF;
    let Family = (CpuSignature >> 8) & 0xF;
    let ExtendedModel = (CpuSignature >> 16) & 0xF;
    let ExtendedFamily = (CpuSignature >> 20) & 0xFF;

    Stepping = Model;
    ExtendModel = if Family == 15 { 1 } else { 0 };

    if Family == 6 {
        let Vendor = KiGetCpuVendor();
        ExtendModel |= if Vendor == CPU_INTEL { 1 } else { 0 };
    }

    if ExtendModel != 0 {
        Stepping |= ExtendedModel << 4;
    }
    Stepping = (Stepping << 8) | Step;

    Type = Family;
    if Family == 15 {
        Type += core::cmp::min(ExtendedFamily, 0xF0);
    }

    (*KeGetCurrentPrcb()).CpuID = 1;
    (*KeGetCurrentPrcb()).CpuType = Type as UCHAR;
    (*KeGetCurrentPrcb()).CpuStep = Stepping as USHORT;
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiGetFeatureBits() -> ULONG64 {
    let Prcb = KeGetCurrentPrcb();
    let Vendor: ULONG;
    let mut FeatureBits: ULONG64 = KF_WORKING_PTE;
    let mut CpuInfo: _CPU_INFO = core::mem::zeroed();
    let mut DummyCpuInfo: _CPU_INFO = core::mem::zeroed();
    let mut ExtendedCPUID: BOOLEAN = 1;
    let mut CpuFeatures: ULONG = 0;

    Vendor = KiGetCpuVendor();

    // Debug trace: vendor value
    #[cfg(target_arch = "x86")]
    {
    }

    if Vendor == 0 {
        return FeatureBits;
    }

    KiCpuId(&mut CpuInfo, 1);

    (*Prcb).InitialApicId = (CpuInfo.__bindgen_anon_1.Ebx >> 24) as UCHAR;

    match Vendor {
        CPU_INTEL => {
            if (*Prcb).CpuType == 6 {
                #[cfg(target_arch = "x86")]
                {
                    core::arch::asm!(
                        "wrmsr",
                        in("ecx") 0x8Bu32,
                        in("eax") 0u32,
                        in("edx") 0u32,
                        options(nostack, preserves_flags)
                    );
                }
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    core::arch::x86_64::_wrmsr(0x8B, 0);
                }
                KiCpuId(&mut DummyCpuInfo, 1);
                #[cfg(target_arch = "x86")]
                {
                    let low: u32;
                    let high: u32;
                    core::arch::asm!(
                        "rdmsr",
                        in("ecx") 0x8Bu32,
                        out("eax") low,
                        out("edx") high,
                        options(nostack, preserves_flags)
                    );
                    (*Prcb).UpdateSignature.QuadPart = (((high as u64) << 32) | (low as u64)) as i64;
                }
                #[cfg(target_arch = "x86_64")]
                {
                    (*Prcb).UpdateSignature.QuadPart = core::arch::x86_64::_rdmsr(0x8B) as i64;
                }
            } else if (*Prcb).CpuType == 5 {
                KiI386PentiumLockErrataPresent = 1;
            }

            if ((CpuInfo.__bindgen_anon_1.Eax & 0x0FF0) == 0x0610 && (CpuInfo.__bindgen_anon_1.Eax & 0x000F) <= 0x9)
                || ((CpuInfo.__bindgen_anon_1.Eax & 0x0FF0) == 0x0630 && (CpuInfo.__bindgen_anon_1.Eax & 0x000F) <= 0x4)
            {
                FeatureBits &= !KF_WORKING_PTE;
            }

            if ((*Prcb).CpuType < 6)
                || (((*Prcb).CpuType == 6) && ((*Prcb).CpuStep < 0x0303))
            {
                CpuInfo.__bindgen_anon_1.Edx &= !0x800;
            }
        }
        CPU_AMD => {
            if (CpuInfo.__bindgen_anon_1.Eax & 0x0F00) == 0x0500 {
                match CpuInfo.__bindgen_anon_1.Eax & 0x00F0 {
                    0x0010 => {
                        if (CpuInfo.__bindgen_anon_1.Eax & 0x000F) <= 0x03 {
                            CpuInfo.__bindgen_anon_1.Edx &= !0x2000;
                        }
                    }
                    0x0000 => {
                        CpuInfo.__bindgen_anon_1.Edx &= !0x2000;
                    }
                    0x0080 => {
                        if (CpuInfo.__bindgen_anon_1.Eax & 0x000F) >= 0x8 {
                            FeatureBits |= KF_AMDK6MTRR;
                        }
                    }
                    0x0090 | 0x00D0 => {
                        FeatureBits |= KF_AMDK6MTRR;
                    }
                    _ => {}
                }
            } else if (CpuInfo.__bindgen_anon_1.Eax & 0x0F00) < 0x0500 {
                CpuInfo.__bindgen_anon_1.Edx &= !(0x08 | 0x2000 | 0x8000);
                ExtendedCPUID = 0;
            }
        }
        CPU_CYRIX => {
            #[cfg(target_arch = "x86")]
            if (*Prcb).CpuType == 6 && (*Prcb).CpuStep <= 1 {
                let mut Ccr1 = get_cx86(CX86_CCR1);
                Ccr1 |= 0x10;
                set_cx86(CX86_CCR1, Ccr1);
            }
        }
        CPU_TRANSMETA => {
            if (CpuInfo.__bindgen_anon_1.Eax & 0x0FFF) >= 0x0542 {
                #[cfg(target_arch = "x86")]
                {
                    let msr_val: u64;
                    core::arch::asm!(
                        "rdmsr",
                        in("ecx") 0x80860004u32,
                        out("eax") _,
                        out("edx") _,
                        options(nostack)
                    );
                    core::arch::asm!(
                        "rdmsr",
                        "or eax, 0x0100",
                        "wrmsr",
                        in("ecx") 0x80860004u32,
                        options(nostack)
                    );
                }
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    let val = core::arch::x86_64::_rdmsr(0x80860004);
                    core::arch::x86_64::_wrmsr(0x80860004, val | 0x0100);
                }
                FeatureBits |= KF_CMPXCHG8B;
            }
        }
        CPU_CENTAUR | CPU_RISE => {
            FeatureBits |= KF_CMPXCHG8B;
        }
        _ => {}
    }

    if CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSE3 != 0 {
        FeatureBits |= KF_SSE3;
    }
    if CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSSE3 != 0 {
        FeatureBits |= KF_SSSE3;
    }
    if CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSE4_1 != 0 {
        FeatureBits |= KF_SSE4_1;
    }
    if CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSE4_2 != 0 {
        FeatureBits |= KF_SSE4_2;
    }
    if CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_XSAVE != 0 {
        FeatureBits |= KF_XSTATE;
    }
    if CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_RDRAND != 0 {
        FeatureBits |= KF_RDRAND;
    }

    CpuFeatures = CpuInfo.__bindgen_anon_1.Edx;

    // Debug trace: output CpuFeatures EDX hex nibbles
    #[cfg(target_arch = "x86")]
    {
        let hex = |v: u8| -> u8 { if v < 10 { b'0' + v } else { b'a' + v - 10 } };
    }

    if CpuFeatures & X86_FEATURE_VME != 0 {
        FeatureBits |= KF_V86_VIS | KF_CR4;
    }
    if CpuFeatures & X86_FEATURE_PSE != 0 {
        FeatureBits |= KF_LARGE_PAGE | KF_CR4;
    }
    if CpuFeatures & X86_FEATURE_TSC != 0 {
        FeatureBits |= KF_RDTSC;
    }
    if CpuFeatures & X86_FEATURE_CX8 != 0 {
        FeatureBits |= KF_CMPXCHG8B;
    }
    if CpuFeatures & X86_FEATURE_SYSCALL != 0 {
        FeatureBits |= KF_FAST_SYSCALL;
    }
    if CpuFeatures & X86_FEATURE_MTRR != 0 {
        FeatureBits |= KF_MTRR;
    }
    if CpuFeatures & X86_FEATURE_PGE != 0 {
        FeatureBits |= KF_GLOBAL_PAGE | KF_CR4;
    }
    if CpuFeatures & X86_FEATURE_CMOV != 0 {
        FeatureBits |= KF_CMOV;
    }
    if CpuFeatures & X86_FEATURE_PAT != 0 {
        FeatureBits |= KF_PAT;
    }
    if CpuFeatures & X86_FEATURE_DS != 0 {
        FeatureBits |= KF_DTS;
    }
    if CpuFeatures & X86_FEATURE_MMX != 0 {
        FeatureBits |= KF_MMX;
    }
    if CpuFeatures & X86_FEATURE_FXSR != 0 {
        FeatureBits |= KF_FXSR;
    }
    if CpuFeatures & X86_FEATURE_SSE != 0 {
        FeatureBits |= KF_XMMI;
    }
    if CpuFeatures & X86_FEATURE_SSE2 != 0 {
        FeatureBits |= KF_XMMI64;
    }

    if CpuFeatures & X86_FEATURE_HT != 0 {
        (*Prcb).LogicalProcessorsPerPhysicalProcessor = (CpuInfo.__bindgen_anon_1.Ebx >> 16) as UCHAR;
        if (*Prcb).LogicalProcessorsPerPhysicalProcessor > 1 {
            KiSMTProcessorsPresent = 1;
        }
    } else {
        (*Prcb).LogicalProcessorsPerPhysicalProcessor = 1;
    }

    if ExtendedCPUID != 0 {
        KiCpuId(&mut CpuInfo, 0x80000000);
        if (CpuInfo.__bindgen_anon_1.Eax & 0xffffff00) == 0x80000000 {
            if CpuInfo.__bindgen_anon_1.Eax >= 0x80000001 {
                KiCpuId(&mut CpuInfo, 0x80000001);

                if CpuInfo.__bindgen_anon_1.Edx & X86_FEATURE_NX != 0 {
                    FeatureBits |= KF_NX_BIT;
                }

                match Vendor {
                    CPU_AMD | CPU_CENTAUR => {
                        if CpuInfo.__bindgen_anon_1.Edx & 0x80000000 != 0 {
                            FeatureBits |= KF_3DNOW;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Debug trace: output FeatureBits low 32 bits
    #[cfg(target_arch = "x86")]
    {
        let hex = |v: u8| -> u8 { if v < 10 { b'0' + v } else { b'a' + v - 10 } };
        let fb = FeatureBits as u32;
    }

    FeatureBits
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiGetCacheInformation() {
    // Upstream 83a6af0773b: the descriptors come from the shared x86/x64 enumeration
    // (ke/x86x64/cacheinfo.rs). The NT5 KPRCB has no Cache[] array, so -- as upstream's
    // NTDDI < VISTA branch -- they go to a local one.
    let Pcr = KeGetPcr() as PKIPCR;
    let mut CacheDescriptors: [ntkrnl_sys::ddk::CACHE_DESCRIPTOR; 5] = core::mem::zeroed();

    (*Pcr).SecondLevelCacheSize = 0;

    let CacheCount = crate::ke::x86x64::cacheinfo::KiGetCpuCacheDescriptors(
        CacheDescriptors.as_mut_ptr(),
        CacheDescriptors.len() as ULONG,
        KiGetCpuVendor() as ntkrnl_sys::ddk::CPU_VENDORS,
    );

    for d in &CacheDescriptors[..CacheCount as usize] {
        KeLargestCacheLine = KeLargestCacheLine.max(d.LineSize as ULONG);
        if d.Level == 1 && d.Type != ntkrnl_sys::ddk::_PROCESSOR_CACHE_TYPE_CacheInstruction {
            KePrefetchNTAGranularity = d.LineSize as ULONG;
        } else if d.Level == 2 {
            (*Pcr).SecondLevelCacheSize = d.Size;
            (*Pcr).SecondLevelCacheAssociativity = d.Associativity;
        }
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSetCR0Bits() {
    #[cfg(target_arch = "x86")]
    {
        let mut Cr0: ULONG;
        core::arch::asm!("mov {}, cr0", out(reg) Cr0, options(nostack, preserves_flags));

        if (*KeGetCurrentPrcb()).CpuType > 3 {
            Cr0 |= CR0_WP;
        }

        core::arch::asm!("mov cr0, {}", in(reg) Cr0, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let mut Cr0 = core::arch::x86_64::_readcr0();
        if (*KeGetCurrentPrcb()).CpuType > 3 {
            Cr0 |= CR0_WP as u64;
        }
        core::arch::x86_64::_writecr0(Cr0);
    }
}

// CODE_SEG("INIT")
unsafe fn KiInitializeTSS2(Tss: PKTSS, TssEntry: PKGDTENTRY) {
    if !TssEntry.is_null() {
        (*TssEntry).LimitLow = (core::mem::size_of::<_KTSS>() - 1) as USHORT;
        (*TssEntry).HighWord.Bits.set_LimitHi(0);
    }

    RtlFillMemory(
        (*Tss).IoMaps[0].IoMap.as_mut_ptr() as PVOID,
        IOPM_FULL_SIZE,
        0xFF,
    );

    let p = (*Tss).IoMaps[0].DirectionMap.as_mut_ptr();
    core::ptr::write_bytes(p as *mut u8, 0, IOPM_DIRECTION_MAP_SIZE);

    *p.add(0) = 4;
    *p.add(3) = 0x18;
    *p.add(4) = 0x18;

    let p = (*Tss).IntDirectionMap.as_mut_ptr();
    core::ptr::write_bytes(p as *mut u8, 0, IOPM_DIRECTION_MAP_SIZE);

    *p.add(0) = 4;
    *p.add(3) = 0x18;
    *p.add(4) = 0x18;
}

unsafe fn KiInitializeTSS(Tss: PKTSS) {
    (*Tss).IoMapBase = KiComputeIopmOffset(IO_ACCESS_MAP_NONE);
    (*Tss).Flags = 0;
    (*Tss).LDT = 0;
    (*Tss).Ss0 = KGDT_R0_DATA;
}

/// CODE_SEG("INIT") — INIT-time TSS+GDT+IDT setup.
/// Configures the main TSS, the double-fault TSS, and the NMI TSS, then
/// loads the task register with the main TSS selector. Originally lived
/// in `ke/i386/cpu.c`; the helpers `KiInitializeTSS`/`KiInitializeTSS2`
/// are right above this fn.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn Ki386InitializeTss(
    Tss: PKTSS,
    Idt: PKIDTENTRY,
    Gdt: PKGDTENTRY,
) {
    // --- main TSS GDT entry ----------------------------------------------
    let TssEntry = Gdt.add(KGDT_TSS as usize / core::mem::size_of::<_KGDTENTRY>());
    (*TssEntry).HighWord.Bits.set_Type(I386_TSS as ULONG);
    (*TssEntry).HighWord.Bits.set_Pres(1);
    (*TssEntry).HighWord.Bits.set_Dpl(0);

    KiInitializeTSS2(Tss, TssEntry);
    KiInitializeTSS(Tss);

    // Load task register with the main TSS selector.
    core::arch::asm!("ltr ax", in("ax") KGDT_TSS, options(nostack, preserves_flags));

    // --- double-fault: IDT[8] → task gate to KGDT_DF_TSS -----------------
    let TaskGate8 = Idt.add(8) as *mut _KGDTENTRY;
    (*TaskGate8).HighWord.Bits.set_Type(I386_TASK_GATE as ULONG);
    (*TaskGate8).HighWord.Bits.set_Pres(1);
    (*TaskGate8).HighWord.Bits.set_Dpl(0);
    (*(TaskGate8 as PKIDTENTRY)).Selector = KGDT_DF_TSS;

    let DfTss = core::ptr::addr_of_mut!(KiDoubleFaultTSS) as PKTSS;
    KiInitializeTSS(DfTss);
    (*DfTss).CR3 = {
        let cr3: ULONG;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
        cr3
    };
    (*DfTss).Esp0 = KiDoubleFaultStack as ULONG;
    (*DfTss).Esp = KiDoubleFaultStack as ULONG;
    (*DfTss).Eip = KiTrap08 as ULONG;
    (*DfTss).Cs = KGDT_R0_CODE;
    (*DfTss).Fs = KGDT_R0_PCR;
    (*DfTss).Ss = {
        let ss: u32;
        core::arch::asm!("mov {}, ss", out(reg) ss, options(nostack, preserves_flags));
        ss as USHORT
    };
    (*DfTss).Es = KGDT_R3_DATA | RPL_MASK;
    (*DfTss).Ds = KGDT_R3_DATA | RPL_MASK;

    let DfTssEntry = Gdt.add(KGDT_DF_TSS as usize / core::mem::size_of::<_KGDTENTRY>());
    (*DfTssEntry).HighWord.Bits.set_Type(I386_TSS as ULONG);
    (*DfTssEntry).HighWord.Bits.set_Pres(1);
    (*DfTssEntry).HighWord.Bits.set_Dpl(0);
    (*DfTssEntry).BaseLow = (DfTss as ULONG_PTR & 0xFFFF) as USHORT;
    (*DfTssEntry).HighWord.Bytes.BaseMid = ((DfTss as ULONG_PTR >> 16) & 0xFF) as UCHAR;
    (*DfTssEntry).HighWord.Bytes.BaseHi = ((DfTss as ULONG_PTR >> 24) & 0xFF) as UCHAR;
    (*DfTssEntry).LimitLow = KTSS_IO_MAPS as USHORT;

    // --- NMI: IDT[2] → task gate to KGDT_NMI_TSS -------------------------
    let TaskGate2 = Idt.add(2) as *mut _KGDTENTRY;
    (*TaskGate2).HighWord.Bits.set_Type(I386_TASK_GATE as ULONG);
    (*TaskGate2).HighWord.Bits.set_Pres(1);
    (*TaskGate2).HighWord.Bits.set_Dpl(0);
    (*(TaskGate2 as PKIDTENTRY)).Selector = KGDT_NMI_TSS;

    let NmiTss = core::ptr::addr_of_mut!(KiNMITSS) as PKTSS;
    KiInitializeTSS(NmiTss);
    (*NmiTss).CR3 = {
        let cr3: ULONG;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
        cr3
    };
    (*NmiTss).Esp0 = KiDoubleFaultStack as ULONG;
    (*NmiTss).Esp = KiDoubleFaultStack as ULONG;
    (*NmiTss).Eip = KiTrap02 as ULONG;
    (*NmiTss).Cs = KGDT_R0_CODE;
    (*NmiTss).Fs = KGDT_R0_PCR;
    (*NmiTss).Ss = {
        let ss: USHORT;
        core::arch::asm!("mov {0:x}, ss", out(reg) ss, options(nostack, preserves_flags));
        ss
    };
    (*NmiTss).Es = KGDT_R3_DATA | RPL_MASK;
    (*NmiTss).Ds = KGDT_R3_DATA | RPL_MASK;

    let NmiTssEntry = Gdt.add(KGDT_NMI_TSS as usize / core::mem::size_of::<_KGDTENTRY>());
    (*NmiTssEntry).HighWord.Bits.set_Type(I386_TSS as ULONG);
    (*NmiTssEntry).HighWord.Bits.set_Pres(1);
    (*NmiTssEntry).HighWord.Bits.set_Dpl(0);
    (*NmiTssEntry).BaseLow = (NmiTss as ULONG_PTR & 0xFFFF) as USHORT;
    (*NmiTssEntry).HighWord.Bytes.BaseMid = ((NmiTss as ULONG_PTR >> 16) & 0xFF) as UCHAR;
    (*NmiTssEntry).HighWord.Bytes.BaseHi = ((NmiTss as ULONG_PTR >> 24) & 0xFF) as UCHAR;
    (*NmiTssEntry).LimitLow = KTSS_IO_MAPS as USHORT;
}

#[no_mangle]
pub unsafe extern "system" fn KeFlushCurrentTb() {
    #[cfg(target_arch = "x86")]
    {
        // Global (PGE) TLB entries survive a plain CR3 reload, so when global
        // pages are on the C clears CR4.PGE around the flush and restores it.
        // The translation kept only the CR3 reload, silently under-flushing on
        // every CPU with PGE enabled — and patpge.rs does enable it. Same shape
        // as hal-rust's HalpFlushTLB. (amd64's C really is CR3-only; this
        // branch is i386's alone, which is why only this copy drifted.)
        let cr3: ULONG;
        if KeFeatureBits & KF_GLOBAL_PAGE != 0 {
            let cr4: ULONG;
            core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nostack, preserves_flags));
            core::arch::asm!("mov cr4, {}", in(reg) cr4 & !CR4_PGE, options(nostack, preserves_flags));

            core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
            core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack, preserves_flags));

            core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nostack, preserves_flags));
        } else {
            core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
            core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack, preserves_flags));
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        let cr3 = core::arch::x86_64::_readcr3();
        core::arch::x86_64::_writecr3(cr3);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiRestoreProcessorControlState(ProcessorState: PKPROCESSOR_STATE) {
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("mov cr0, {}", in(reg) (*ProcessorState).SpecialRegisters.Cr0, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::x86_64::_writecr0((*ProcessorState).SpecialRegisters.Cr0 as u64);
    }
    Ke386SetCr2((*ProcessorState).SpecialRegisters.Cr2 as ULONG_PTR);
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("mov cr3, {}", in(reg) (*ProcessorState).SpecialRegisters.Cr3, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::x86_64::_writecr3((*ProcessorState).SpecialRegisters.Cr3 as u64);
    }
    if KeFeatureBits & KF_CR4 != 0 {
        #[cfg(target_arch = "x86")]
        {
            core::arch::asm!("mov cr4, {}", in(reg) (*ProcessorState).SpecialRegisters.Cr4, options(nostack, preserves_flags));
        }
        #[cfg(target_arch = "x86_64")]
        {
            core::arch::x86_64::_writecr4((*ProcessorState).SpecialRegisters.Cr4 as u64);
        }
    }

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "mov dr0, {}",
            in(reg) (*ProcessorState).SpecialRegisters.KernelDr0,
            options(nostack, preserves_flags)
        );
        core::arch::asm!(
            "mov dr1, {}",
            in(reg) (*ProcessorState).SpecialRegisters.KernelDr1,
            options(nostack, preserves_flags)
        );
        core::arch::asm!(
            "mov dr2, {}",
            in(reg) (*ProcessorState).SpecialRegisters.KernelDr2,
            options(nostack, preserves_flags)
        );
        core::arch::asm!(
            "mov dr3, {}",
            in(reg) (*ProcessorState).SpecialRegisters.KernelDr3,
            options(nostack, preserves_flags)
        );
        core::arch::asm!(
            "mov dr6, {}",
            in(reg) (*ProcessorState).SpecialRegisters.KernelDr6,
            options(nostack, preserves_flags)
        );
        core::arch::asm!(
            "mov dr7, {}",
            in(reg) (*ProcessorState).SpecialRegisters.KernelDr7,
            options(nostack, preserves_flags)
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::x86_64::_writedr(0, (*ProcessorState).SpecialRegisters.KernelDr0 as u64);
        core::arch::x86_64::_writedr(1, (*ProcessorState).SpecialRegisters.KernelDr1 as u64);
        core::arch::x86_64::_writedr(2, (*ProcessorState).SpecialRegisters.KernelDr2 as u64);
        core::arch::x86_64::_writedr(3, (*ProcessorState).SpecialRegisters.KernelDr3 as u64);
        core::arch::x86_64::_writedr(6, (*ProcessorState).SpecialRegisters.KernelDr6 as u64);
        core::arch::x86_64::_writedr(7, (*ProcessorState).SpecialRegisters.KernelDr7 as u64);
    }

    Ke386SetGlobalDescriptorTable(
        core::ptr::addr_of!((*ProcessorState).SpecialRegisters.Gdtr.Limit),
    );
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) core::ptr::addr_of!((*ProcessorState).SpecialRegisters.Idtr.Limit),
            options(nostack, preserves_flags)
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) core::ptr::addr_of!((*ProcessorState).SpecialRegisters.Idtr.Limit),
            options(nostack)
        );
    }

    let TssEntry = ((*ProcessorState).SpecialRegisters.Gdtr.Base as usize
        + (*ProcessorState).SpecialRegisters.Tr as usize)
        as PKGDTENTRY;
    (*TssEntry).HighWord.Bytes.Flags1 &= !0x2;

    Ke386SetTr((*ProcessorState).SpecialRegisters.Tr);
    Ke386SetLocalDescriptorTable((*ProcessorState).SpecialRegisters.Ldtr);
}

#[no_mangle]
pub unsafe extern "system" fn KiSaveProcessorControlState(ProcessorState: PKPROCESSOR_STATE) {
    #[cfg(target_arch = "x86")]
    {
        let cr0: ULONG;
        core::arch::asm!("mov {}, cr0", out(reg) cr0, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.Cr0 = cr0;

        let cr2: ULONG;
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.Cr2 = cr2;

        let cr3: ULONG;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.Cr3 = cr3;
    }
    #[cfg(target_arch = "x86_64")]
    {
        (*ProcessorState).SpecialRegisters.Cr0 = core::arch::x86_64::_readcr0() as ULONG_PTR;
        (*ProcessorState).SpecialRegisters.Cr2 = core::arch::x86_64::_readcr2() as ULONG_PTR;
        (*ProcessorState).SpecialRegisters.Cr3 = core::arch::x86_64::_readcr3() as ULONG_PTR;
    }

    (*ProcessorState).SpecialRegisters.Cr4 = if KeFeatureBits & KF_CR4 != 0 {
        #[cfg(target_arch = "x86")]
        {
            let cr4: ULONG;
            core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nostack, preserves_flags));
            cr4
        }
        #[cfg(target_arch = "x86_64")]
        {
            core::arch::x86_64::_readcr4() as ULONG_PTR
        }
    } else {
        0
    };

    #[cfg(target_arch = "x86")]
    {
        let dr0: ULONG;
        core::arch::asm!("mov {}, dr0", out(reg) dr0, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.KernelDr0 = dr0;

        let dr1: ULONG;
        core::arch::asm!("mov {}, dr1", out(reg) dr1, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.KernelDr1 = dr1;

        let dr2: ULONG;
        core::arch::asm!("mov {}, dr2", out(reg) dr2, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.KernelDr2 = dr2;

        let dr3: ULONG;
        core::arch::asm!("mov {}, dr3", out(reg) dr3, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.KernelDr3 = dr3;

        let dr6: ULONG;
        core::arch::asm!("mov {}, dr6", out(reg) dr6, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.KernelDr6 = dr6;

        let dr7: ULONG;
        core::arch::asm!("mov {}, dr7", out(reg) dr7, options(nostack, preserves_flags));
        (*ProcessorState).SpecialRegisters.KernelDr7 = dr7;

        core::arch::asm!("mov dr7, {}", in(reg) 0u32, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        (*ProcessorState).SpecialRegisters.KernelDr0 =
            core::arch::x86_64::_readdr(0) as ULONG_PTR;
        (*ProcessorState).SpecialRegisters.KernelDr1 =
            core::arch::x86_64::_readdr(1) as ULONG_PTR;
        (*ProcessorState).SpecialRegisters.KernelDr2 =
            core::arch::x86_64::_readdr(2) as ULONG_PTR;
        (*ProcessorState).SpecialRegisters.KernelDr3 =
            core::arch::x86_64::_readdr(3) as ULONG_PTR;
        (*ProcessorState).SpecialRegisters.KernelDr6 =
            core::arch::x86_64::_readdr(6) as ULONG_PTR;
        (*ProcessorState).SpecialRegisters.KernelDr7 =
            core::arch::x86_64::_readdr(7) as ULONG_PTR;
        core::arch::x86_64::_writedr(7, 0);
    }

    Ke386GetGlobalDescriptorTable(core::ptr::addr_of_mut!(
        (*ProcessorState).SpecialRegisters.Gdtr.Limit
    ));
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "sidt [{}]",
            in(reg) core::ptr::addr_of_mut!((*ProcessorState).SpecialRegisters.Idtr.Limit),
            options(nostack, preserves_flags)
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "sidt [{}]",
            in(reg) core::ptr::addr_of_mut!((*ProcessorState).SpecialRegisters.Idtr.Limit),
            options(nostack)
        );
    }
    (*ProcessorState).SpecialRegisters.Tr = Ke386GetTr();
    Ke386GetLocalDescriptorTable(core::ptr::addr_of_mut!(
        (*ProcessorState).SpecialRegisters.Ldtr
    ));
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeMachineType() {
    #[repr(C)]
    struct LoaderBlock {
        _pad: [u8; 0x100],
        u: LoaderBlockUnion,
    }
    #[repr(C)]
    union LoaderBlockUnion {
        I386: core::mem::ManuallyDrop<LoaderBlockI386>,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct LoaderBlockI386 {
        MachineType: ULONG,
    }

    let loader_block = KeLoaderBlock as *const LoaderBlock;
    let machine_type = core::ptr::addr_of!((*loader_block).u) as *const LoaderBlockI386;
    KeI386MachineType = (*machine_type).MachineType & 0x000FF;
}

// CODE_SEG("INIT")
unsafe extern "system" fn KiLoadFastSyscallMachineSpecificRegisters(_Context: ULONG_PTR) -> ULONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0x174u32,
            in("eax") KGDT_R0_CODE as u32,
            in("edx") 0u32,
            options(nostack, preserves_flags)
        );
        let dpc_stack = (*KeGetCurrentPrcb()).DpcStack;
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0x175u32,
            in("eax") dpc_stack as u32,
            in("edx") 0u32,
            options(nostack, preserves_flags)
        );
        let entry = KiFastCallEntry as u32;
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0x176u32,
            in("eax") entry,
            in("edx") 0u32,
            options(nostack, preserves_flags)
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::x86_64::_wrmsr(0x174, KGDT_R0_CODE as u64);
        core::arch::x86_64::_wrmsr(0x175, (*KeGetCurrentPrcb()).DpcStack as u64);
        core::arch::x86_64::_wrmsr(0x176, KiFastCallEntry as u64);
    }
    0
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiRestoreFastSyscallReturnState() {
    if KeFeatureBits & KF_FAST_SYSCALL != 0 {
        if KiFastSystemCallDisable != 0 {
            KeFeatureBits &= !KF_FAST_SYSCALL;
            KiFastCallExitHandler = KiSystemCallTrapReturn as PVOID;
        } else {
            KeIpiGenericCall(Some(KiLoadFastSyscallMachineSpecificRegisters), 0usize);
            KiFastCallExitHandler = KiSystemCallSysExitReturn as PVOID;
        }
    } else {
        KiFastCallExitHandler = KiSystemCallTrapReturn as PVOID;
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn Ki386EnableDE(_Context: ULONG_PTR) -> ULONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        let mut cr4: ULONG;
        core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nostack, preserves_flags));
        cr4 |= CR4_DE;
        core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let cr4 = core::arch::x86_64::_readcr4();
        core::arch::x86_64::_writecr4(cr4 | CR4_DE as u64);
    }
    0
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn Ki386EnableFxsr(_Context: ULONG_PTR) -> ULONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        let mut cr4: ULONG;
        core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nostack, preserves_flags));
        cr4 |= CR4_FXSR;
        core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let cr4 = core::arch::x86_64::_readcr4();
        core::arch::x86_64::_writecr4(cr4 | CR4_FXSR as u64);
    }
    0
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn Ki386EnableXMMIExceptions(_Context: ULONG_PTR) -> ULONG_PTR {
    let IdtEntry = &mut *(*(KeGetPcr() as PKIPCR)).IDT.add(0x13);

    (*IdtEntry).Selector = KGDT_R0_CODE;
    (*IdtEntry).Offset = (KiTrap13 as ULONG_PTR & 0xFFFF) as USHORT;
    (*IdtEntry).ExtendedOffset = ((KiTrap13 as ULONG_PTR >> 16) & 0xFFFF) as USHORT;
    (*IdtEntry).Access = 0;
    core::ptr::write_bytes(&mut (*IdtEntry).Access as *mut _ as *mut u8, 0, 1);
    let access_ptr = &mut (*IdtEntry).Access as *mut USHORT as *mut u8;
    *access_ptr.add(0) = I386_INTERRUPT_GATE;
    *access_ptr.add(1) |= 0x80;

    #[cfg(target_arch = "x86")]
    {
        let mut cr4: ULONG;
        core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nostack, preserves_flags));
        cr4 |= CR4_XMMEXCPT;
        core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let cr4 = core::arch::x86_64::_readcr4();
        core::arch::x86_64::_writecr4(cr4 | CR4_XMMEXCPT as u64);
    }
    0
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiI386PentiumLockErrataFixup() {
    let mut IdtDescriptor: [USHORT; 3] = [0, 0, 0];
    let mut NewIdt2: PKIDTENTRY;

    let NewIdt = ExAllocatePool(NonPagedPool, 2 * PAGE_SIZE as ULONG) as *mut UCHAR;

    NewIdt2 = NewIdt.add(PAGE_SIZE - (7 * core::mem::size_of::<_KIDTENTRY>())) as PKIDTENTRY;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "sidt [{}]",
            in(reg) IdtDescriptor.as_mut_ptr(),
            options(nostack, preserves_flags)
        );
        core::arch::asm!("cli", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "sidt [{}]",
            in(reg) IdtDescriptor.as_mut_ptr(),
            options(nostack)
        );
        core::arch::asm!("cli", options(nostack));
    }

    let idt_base = ((IdtDescriptor[2] as u32) << 16) | (IdtDescriptor[1] as u32);
    let idt_limit = IdtDescriptor[0] as usize;

    core::ptr::copy_nonoverlapping(
        idt_base as *const u8,
        NewIdt2 as *mut u8,
        idt_limit + 1,
    );

    IdtDescriptor[1] = (NewIdt2 as u32 & 0xFFFF) as USHORT;
    IdtDescriptor[2] = ((NewIdt2 as u32 >> 16) & 0xFFFF) as USHORT;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) IdtDescriptor.as_ptr(),
            options(nostack, preserves_flags)
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) IdtDescriptor.as_ptr(),
            options(nostack)
        );
    }

    (*(KeGetPcr() as PKIPCR)).IDT = NewIdt2;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("sti", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("sti", options(nostack));
    }

    let PointerPte = MiAddressToPte(NewIdt as PVOID);
    (*PointerPte).Long &= !2; // Clear Write bit (bit 1)
    KeInvalidateTlbEntry(NewIdt as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn KeInvalidateAllCaches() -> BOOLEAN {
    if KeI386CpuType < 6 {
        return 0;
    }

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("wbinvd", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("wbinvd", options(nostack));
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn KiSaveProcessorState(
    TrapFrame: PKTRAP_FRAME,
    _ExceptionFrame: PKEXCEPTION_FRAME,
) {
    let Prcb = KeGetCurrentPrcb();

    (*Prcb).ProcessorState.ContextFrame.ContextFlags =
        CONTEXT_FULL | CONTEXT_DEBUG_REGISTERS;
    KeTrapFrameToContext(
        TrapFrame,
        core::ptr::null_mut(),
        &mut (*Prcb).ProcessorState.ContextFrame,
    );

    KiSaveProcessorControlState(&mut (*Prcb).ProcessorState);
}

/// KiRestoreProcessorState — inverse of KiSaveProcessorState (freeze/thaw
/// resume path). Upstream i386 never had one (its freeze support was
/// UNIMPLEMENTED); mirrors the amd64 body (ke/amd64/cpu.c KiRestoreProcessorState)
/// against the state KiSaveProcessorState above captures.
#[no_mangle]
pub unsafe extern "system" fn KiRestoreProcessorState(
    TrapFrame: PKTRAP_FRAME,
    _ExceptionFrame: PKEXCEPTION_FRAME,
) {
    let Prcb = KeGetCurrentPrcb();

    // KernelMode: the freeze/thaw path only ever restores a kernel-mode
    // capture (i386 KTRAP_FRAME has no PreviousMode field to consult).
    ntkrnl_sys::ddk::KeContextToTrapFrame(
        &mut (*Prcb).ProcessorState.ContextFrame,
        core::ptr::null_mut(),
        TrapFrame,
        (*Prcb).ProcessorState.ContextFrame.ContextFlags,
        0, // KernelMode
    );

    KiRestoreProcessorControlState(&mut (*Prcb).ProcessorState);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiIsNpxErrataPresent() -> BOOLEAN {
    static mut VALUE1: f64 = 4195835.0;
    static mut VALUE2: f64 = 3145727.0;
    let mut ErrataPresent: i32 = 0;
    let Cr0: ULONG;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("mov {}, cr0", out(reg) Cr0, options(nostack, preserves_flags));
        core::arch::asm!("mov cr0, {}", in(reg) Cr0 & !(CR0_MP | CR0_TS | CR0_EM), options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let Cr0 = core::arch::x86_64::_readcr0() as ULONG;
        core::arch::x86_64::_writecr0((Cr0 & !(CR0_MP | CR0_TS | CR0_EM)) as u64);
    }

    Ke386FnInit();

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "fld qword ptr [{0}]",
            "fdiv qword ptr [{1}]",
            "fmul qword ptr [{1}]",
            "fld qword ptr [{0}]",
            "fsubp",
            "fistp dword ptr [{2}]",
            in(reg) &VALUE1 as *const f64,
            in(reg) &VALUE2 as *const f64,
            in(reg) &mut ErrataPresent as *mut i32,
            options(nostack)
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        ErrataPresent = 0;
    }

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("mov cr0, {}", in(reg) Cr0, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::x86_64::_writecr0(Cr0 as u64);
    }

    if ErrataPresent != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiFlushNPXState(SaveArea: PFLOATING_SAVE_AREA) {
    let EFlags: ULONG;
    let mut Cr0: ULONG;
    let Thread: PKTHREAD;
    let NpxThread: PKTHREAD;
    let mut FxSaveArea: *mut FX_SAVE_AREA;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("pushfd", "pop {}", out(reg) EFlags, options(nostack, preserves_flags));
        core::arch::asm!("cli", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let rflags: u64;
        core::arch::asm!("pushfq", "pop {}", out(reg) rflags, options(nostack));
        let EFlags = rflags as ULONG;
        core::arch::asm!("cli", options(nostack));
    }

    Thread = KeGetCurrentThread() as PKTHREAD;

    if (*Thread).NpxState != NPX_STATE_LOADED {
        if SaveArea.is_null() {
            #[cfg(target_arch = "x86")]
            {
                core::arch::asm!("push {}", "popfd", in(reg) EFlags, options(nostack, preserves_flags));
            }
            #[cfg(target_arch = "x86_64")]
            {
                core::arch::asm!("push {}", "popfq", in(reg) EFlags as u64, options(nostack));
            }
            return;
        }

        #[cfg(target_arch = "x86")]
        {
            core::arch::asm!("mov {}, cr0", out(reg) Cr0, options(nostack, preserves_flags));
        }
        #[cfg(target_arch = "x86_64")]
        {
            Cr0 = core::arch::x86_64::_readcr0() as ULONG;
        }
        if Cr0 & (CR0_MP | CR0_TS | CR0_EM) != 0 {
            #[cfg(target_arch = "x86")]
            {
                core::arch::asm!("mov cr0, {}", in(reg) Cr0 & !(CR0_MP | CR0_TS | CR0_EM), options(nostack, preserves_flags));
            }
            #[cfg(target_arch = "x86_64")]
            {
                core::arch::x86_64::_writecr0((Cr0 & !(CR0_MP | CR0_TS | CR0_EM)) as u64);
            }
        }

        NpxThread = (*KeGetCurrentPrcb()).NpxThread;
        if !NpxThread.is_null() && (*NpxThread).NpxState == NPX_STATE_LOADED {
            FxSaveArea = KiGetThreadNpxArea(NpxThread);
            Ke386FxSave(FxSaveArea);
            (*NpxThread).NpxState = NPX_STATE_NOT_LOADED;
        }

        FxSaveArea = KiGetThreadNpxArea(Thread);
        Ke386FxStore(FxSaveArea);
    } else {
        #[cfg(target_arch = "x86")]
        {
            core::arch::asm!("mov {}, cr0", out(reg) Cr0, options(nostack, preserves_flags));
        }
        #[cfg(target_arch = "x86_64")]
        {
            Cr0 = core::arch::x86_64::_readcr0() as ULONG;
        }
        if Cr0 & (CR0_MP | CR0_TS | CR0_EM) != 0 {
            #[cfg(target_arch = "x86")]
            {
                core::arch::asm!("mov cr0, {}", in(reg) Cr0 & !(CR0_MP | CR0_TS | CR0_EM), options(nostack, preserves_flags));
            }
            #[cfg(target_arch = "x86_64")]
            {
                core::arch::x86_64::_writecr0((Cr0 & !(CR0_MP | CR0_TS | CR0_EM)) as u64);
            }
        }

        FxSaveArea = KiGetThreadNpxArea(Thread);
        (*Thread).NpxState = NPX_STATE_NOT_LOADED;

        if KeI386FxsrPresent != 0 {
            Ke386FxSave(FxSaveArea);
        }
    }

    if !SaveArea.is_null() {
        Ke386FnSave(SaveArea);
    }

    (*KeGetCurrentPrcb()).NpxThread = core::ptr::null_mut();

    Cr0 |= NPX_STATE_NOT_LOADED as ULONG;
    Cr0 |= (*FxSaveArea).Cr0NpxState;
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("mov cr0, {}", in(reg) Cr0, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::x86_64::_writecr0(Cr0 as u64);
    }

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("push {}", "popfd", in(reg) EFlags, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("push {}", "popfq", in(reg) EFlags as u64, options(nostack));
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KiCoprocessorError() {
    let NpxArea = KiGetThreadNpxArea(KeGetCurrentThread() as PKTHREAD);

    (*NpxArea).Cr0NpxState = CR0_TS;
    #[cfg(target_arch = "x86")]
    {
        let cr0: ULONG;
        core::arch::asm!("mov {}, cr0", out(reg) cr0, options(nostack, preserves_flags));
        core::arch::asm!("mov cr0, {}", in(reg) cr0 | CR0_TS, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        let cr0 = core::arch::x86_64::_readcr0();
        core::arch::x86_64::_writecr0(cr0 | CR0_TS as u64);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeSaveFloatingPointState(
    Save: PKFLOATING_SAVE,
) -> NTSTATUS {
    let mut FsContext: *mut FLOATING_SAVE_CONTEXT;
    let mut FxSaveAreaFrame: *mut FX_SAVE_AREA;
    let CurrentPrcb: PKPRCB;

    FsContext = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<FLOATING_SAVE_CONTEXT>(),
        TAG_FLOATING_POINT_CONTEXT,
    ) as *mut FLOATING_SAVE_CONTEXT;
    if FsContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*FsContext).Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<FX_SAVE_AREA>() + FXSAVE_ALIGN,
        TAG_FLOATING_POINT_FX,
    );
    if (*FsContext).Buffer.is_null() {
        ExFreePoolWithTag(FsContext as PVOID, TAG_FLOATING_POINT_CONTEXT);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let ptr_val = (*FsContext).Buffer as usize;
    (*FsContext).PfxSaveArea = ((ptr_val + 15) & !15) as *mut FX_SAVE_AREA;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("cli", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("cli", options(nostack));
    }
    CurrentPrcb = KeGetCurrentPrcb();

    (*FsContext).CurrentThread = KeGetCurrentThread() as PKTHREAD;

    if (*FsContext).CurrentThread != (*CurrentPrcb).NpxThread {
        if !(*CurrentPrcb).NpxThread.is_null()
            && (*(*CurrentPrcb).NpxThread).NpxState == NPX_STATE_LOADED
        {
            FxSaveAreaFrame = KiGetThreadNpxArea((*CurrentPrcb).NpxThread);
            Ke386SaveFpuState(FxSaveAreaFrame);
            (*(*CurrentPrcb).NpxThread).NpxState = NPX_STATE_NOT_LOADED;
            (*FxSaveAreaFrame).NpxSavedCpu = 0;
        }

        (*CurrentPrcb).NpxThread = (*FsContext).CurrentThread;
    }

    Ke386SaveFpuState((*FsContext).PfxSaveArea);

    (*FsContext).OldNpxIrql = (*(*FsContext).CurrentThread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.NpxIrql;
    (*(*FsContext).CurrentThread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.NpxIrql = KeGetCurrentIrql();

    Ke386FnInit();

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("sti", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("sti", options(nostack));
    }

    *(Save as *mut *mut FLOATING_SAVE_CONTEXT) = FsContext;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KeRestoreFloatingPointState(
    Save: PKFLOATING_SAVE,
) -> NTSTATUS {
    let FsContext: *mut FLOATING_SAVE_CONTEXT;

    FsContext = *(Save as *const *mut FLOATING_SAVE_CONTEXT);

    if (*FsContext).CurrentThread != KeGetCurrentThread() as PKTHREAD {
        KeBugCheckEx(
            INVALID_FLOATING_POINT_STATE,
            0x2,
            (*FsContext).CurrentThread as ULONG_PTR,
            KeGetCurrentThread() as PKTHREAD as ULONG_PTR,
            0,
        );
    }

    if (*(*FsContext).CurrentThread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.NpxIrql != KeGetCurrentIrql() {
        KeBugCheckEx(
            INVALID_FLOATING_POINT_STATE,
            0x1,
            (*(*FsContext).CurrentThread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.NpxIrql as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
        );
    }

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("cli", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("cli", options(nostack));
    }

    Ke386ClearFpExceptions();
    Ke386RestoreFpuState((*FsContext).PfxSaveArea);

    (*(*FsContext).CurrentThread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.NpxIrql = (*FsContext).OldNpxIrql;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("sti", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("sti", options(nostack));
    }

    ExFreePoolWithTag((*FsContext).Buffer, TAG_FLOATING_POINT_FX);
    ExFreePoolWithTag(FsContext as PVOID, TAG_FLOATING_POINT_CONTEXT);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KeGetRecommendedSharedDataAlignment() -> ULONG {
    KeLargestCacheLine
}

// extern "stdcall" to slot into the PKIPI_WORKER function-pointer type
// (bindgen spells it "stdcall", which rustc type-distinguishes from "system")
// when handed to KiIpiSendPacket under smp. i386-only file, so bare stdcall
// is fine.
unsafe extern "system" fn KiFlushTargetEntireTb(
    PacketContext: PKIPI_CONTEXT,
    _Ignored1: PVOID,
    _Ignored2: PVOID,
    _Ignored3: PVOID,
) {
    KiIpiSignalPacketDone(PacketContext);
    KeFlushCurrentTb();
}

// Single-VA twin (NT KiFlushTargetSingleTb). The VA arrives as the packet's
// Context, which KiIpiServiceRoutine hands us as Parameter2 — it reads every
// packet field from the sender's Prcb BEFORE invoking us, so ack-first is
// slot-safe. Ack precedes the invlpg like the entire-TB worker: we are in the
// 0xE1 ISR, and the interrupted code cannot touch the stale mapping again
// until the ISR returns, by which point the entry is gone.
#[cfg(feature = "smp")]
unsafe extern "system" fn KiFlushTargetSingleTb(
    PacketContext: PKIPI_CONTEXT,
    _Ignored1: PVOID,
    Virtual: PVOID,
    _Ignored3: PVOID,
) {
    KiIpiSignalPacketDone(PacketContext);
    core::arch::asm!("invlpg [{}]", in(reg) Virtual, options(nostack, preserves_flags));
}

// Sender-side packet-completion drain (NT KiIpiStallOnPacketTargets): spin
// until every target has read + acked our packet (our Prcb->TargetSet -> 0).
// Until then the sender packet slots (WorkerRoutine/CurrentPacket/TargetSet)
// are live and must not be reused — returning without draining lets the NEXT
// KiIpiSendPacket from this CPU clobber a packet an unserviced target hasn't
// read yet (it would run the new worker with the new arguments, and a stale
// ack could corrupt the new packet's TargetSet bookkeeping).
// Contract: SYNCH_LEVEL with interrupts enabled — a concurrent sender
// targeting US is serviced by the 0xE1 ISR (IPI_LEVEL > SYNCH_LEVEL) while
// we spin, which is what makes cross-sends deadlock-free. Guarded loud,
// matching the amd64 transport's 0x101 sub-coded bugchecks.
#[cfg(feature = "smp")]
unsafe fn KiIpiStallOnPacketTargets(Prcb: ntkrnl_sys::ddk::PKPRCB, Subcode: ULONG_PTR) {
    use core::sync::atomic::Ordering;
    let target_set = core::sync::atomic::AtomicUsize::from_ptr(
        core::ptr::addr_of_mut!((*Prcb).TargetSet) as *mut usize,
    );
    let mut guard: u64 = 0;
    while target_set.load(Ordering::SeqCst) != 0 {
        core::hint::spin_loop();
        guard += 1;
        if guard > 4_000_000_000 {
            // nolint: silent-failure (loud unresponsive-processor bugcheck)
            KeBugCheckEx(
                ntkrnl_sys::bugcodes::CLOCK_WATCHDOG_TIMEOUT,
                Subcode,
                target_set.load(Ordering::SeqCst) as ULONG_PTR,
                0,
                0,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Remote-only TLB shootdown entry points — same names and contracts as the
// amd64 descriptor-transport versions (ke/amd64/ipi.rs), so the arch dispatch
// in ntkrnl-sys inline::mm (KeInvalidateTlbEntry / KeFlushProcessTb) is one
// shared extern for both x86 flavors. i386 rides the classic NT packet path
// (KiIpiSendPacket over 0xE1) that KeFlushEntireTb already validated on this
// arch instead of a bespoke descriptor protocol. The LOCAL invlpg/cr3-reload
// is the caller's job; these only make the remote TLBs coherent.
// Contract: IRQL <= SYNCH_LEVEL, interrupts enabled (see the drain above).
// Callable before APs start: with no other active processor they no-op.
// ---------------------------------------------------------------------------
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbShootdownSingle(Address: u64) {
    let OldIrql = KeRaiseIrqlToSynchLevel();
    let Prcb = KeGetCurrentPrcb();
    let TargetAffinity = KeActiveProcessors & !(*Prcb).SetMember;
    if TargetAffinity != 0 {
        crate::ke::ipi::KiIpiSendPacket(
            TargetAffinity,
            Some(KiFlushTargetSingleTb),
            None,
            Address as ULONG_PTR,
            core::ptr::null_mut(),
        );
        KiIpiStallOnPacketTargets(Prcb, 7);
    }
    KeLowerIrql(OldIrql);
}

#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbShootdownBroadcast() {
    let OldIrql = KeRaiseIrqlToSynchLevel();
    let Prcb = KeGetCurrentPrcb();
    let TargetAffinity = KeActiveProcessors & !(*Prcb).SetMember;
    if TargetAffinity != 0 {
        crate::ke::ipi::KiIpiSendPacket(
            TargetAffinity,
            Some(KiFlushTargetEntireTb),
            None,
            0,
            core::ptr::null_mut(),
        );
        KiIpiStallOnPacketTargets(Prcb, 8);
    }
    KeLowerIrql(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KeFlushEntireTb(
    _Invalid: BOOLEAN,
    _AllProcessors: BOOLEAN,
) {
    // DBG: KeFlushEntireTb entry

    let OldIrql: KIRQL;

    OldIrql = KeRaiseIrqlToSynchLevel();

    // DBG: after KeRaiseIrqlToSynchLevel

    #[cfg(feature = "smp")]
    {
        let Prcb = KeGetCurrentPrcb();
        let mut TargetAffinity = KeActiveProcessors;
        TargetAffinity &= !(*Prcb).SetMember;

        if TargetAffinity != 0 {
            crate::ke::ipi::KiIpiSendPacket(
                TargetAffinity,
                Some(KiFlushTargetEntireTb),
                None,
                0,
                core::ptr::null_mut(),
            );
        }
    }

    KeFlushCurrentTb();

    // DBG: after KeFlushCurrentTb

    #[cfg(feature = "smp")]
    {
        // Drain our packet before the slots can be reused. This block used to
        // be an EMPTY "// Wait for IPI completion" — without the drain, the
        // next KiIpiSendPacket from this CPU could clobber a packet an
        // unserviced target hadn't read yet (see KiIpiStallOnPacketTargets).
        // No-sent-packet case: TargetSet is already 0 and this returns at once.
        KiIpiStallOnPacketTargets(KeGetCurrentPrcb(), 9);
    }

    InterlockedExchangeAdd(core::ptr::addr_of_mut!(KiTbFlushTimeStamp), 1);

    // DBG: after InterlockedExchangeAdd

    KeLowerIrql(OldIrql);

    // DBG: KeFlushEntireTb done
}

#[no_mangle]
pub unsafe extern "system" fn KeSetDmaIoCoherency(Coherency: ULONG) {
    KiDmaIoCoherency = Coherency;
}

#[no_mangle]
pub unsafe extern "C" fn KeSaveStateForHibernate(State: PKPROCESSOR_STATE) {
    RtlCaptureContext(&mut (*State).ContextFrame);
    KiSaveProcessorControlState(State);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/i386/cpu.c
//  * PURPOSE:         Routines for CPU-level support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <xmmintrin.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* The TSS to use for Double Fault Traps (INT 0x9) */
// UCHAR KiDoubleFaultTSS[KTSS_IO_MAPS];
//
// /* The TSS to use for NMI Fault Traps (INT 0x2) */
// UCHAR KiNMITSS[KTSS_IO_MAPS];
//
// /* CPU Features and Flags */
// ULONG KeI386CpuType;
// ULONG KeI386CpuStep;
// ULONG KiFastSystemCallDisable = 0;
// ULONG KeI386NpxPresent = TRUE;
// ULONG KiMXCsrMask = 0;
// ULONG MxcsrFeatureMask = 0;
// ULONG KeI386XMMIPresent = 0;
// ULONG KeI386FxsrPresent = 0;
// ULONG KeI386MachineType;
// ULONG Ke386Pae = FALSE;
// ULONG Ke386NoExecute = FALSE;
// ULONG KeLargestCacheLine = 0x40;
// ULONG KeDcacheFlushCount = 0;
// ULONG KeIcacheFlushCount = 0;
// ULONG KiDmaIoCoherency = 0;
// ULONG KePrefetchNTAGranularity = 32;
// BOOLEAN KiI386PentiumLockErrataPresent;
// BOOLEAN KiSMTProcessorsPresent;
//
// /* The distance between SYSEXIT and IRETD return modes */
// UCHAR KiSystemCallExitAdjust;
//
// /* The offset that was applied -- either 0 or the value above */
// UCHAR KiSystemCallExitAdjusted;
//
// /* Whether the adjustment was already done once */
// BOOLEAN KiFastCallCopyDoneOnce;
//
// /* Flush data */
// volatile LONG KiTbFlushTimeStamp;
//
// /* CPU Signatures */
// static const CHAR CmpIntelID[]       = "GenuineIntel";
// static const CHAR CmpAmdID[]         = "AuthenticAMD";
// static const CHAR CmpCyrixID[]       = "CyrixInstead";
// static const CHAR CmpTransmetaID[]   = "GenuineTMx86";
// static const CHAR CmpCentaurID[]     = "CentaurHauls";
// static const CHAR CmpRiseID[]        = "RiseRiseRise";
//
// typedef union _CPU_SIGNATURE
// {
//     struct
//     {
//         ULONG Step : 4;
//         ULONG Model : 4;
//         ULONG Family : 4;
//         ULONG Unused : 4;
//         ULONG ExtendedModel : 4;
//         ULONG ExtendedFamily : 8;
//         ULONG Unused2 : 4;
//     };
//     ULONG AsULONG;
// } CPU_SIGNATURE;
//
// /* FX area alignment size */
// #define FXSAVE_ALIGN 15
//
// /* SUPPORT ROUTINES FOR MSVC COMPATIBILITY ***********************************/
//
// /* NSC/Cyrix CPU configuration register index */
// #define CX86_CCR1 0xc1
//
// /* NSC/Cyrix CPU indexed register access macros */
// static __inline
// UCHAR
// getCx86(UCHAR reg)
// {
//     WRITE_PORT_UCHAR((PUCHAR)(ULONG_PTR)0x22, reg);
//     return READ_PORT_UCHAR((PUCHAR)(ULONG_PTR)0x23);
// }
//
// static __inline
// void
// setCx86(UCHAR reg, UCHAR data)
// {
//     WRITE_PORT_UCHAR((PUCHAR)(ULONG_PTR)0x22, reg);
//     WRITE_PORT_UCHAR((PUCHAR)(ULONG_PTR)0x23, data);
// }
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// ULONG
// NTAPI
// KiGetCpuVendor(VOID)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     CPU_INFO CpuInfo;
//
//     /* Get the Vendor ID */
//     KiCpuId(&CpuInfo, 0);
//
//     /* Copy it to the PRCB and null-terminate it */
//     *(ULONG*)&Prcb->VendorString[0] = CpuInfo.__bindgen_anon_1.Ebx;
//     *(ULONG*)&Prcb->VendorString[4] = CpuInfo.__bindgen_anon_1.Edx;
//     *(ULONG*)&Prcb->VendorString[8] = CpuInfo.__bindgen_anon_1.Ecx;
//     Prcb->VendorString[12] = 0;
//
//     /* Now check the CPU Type */
//     if (!strcmp(Prcb->VendorString, CmpIntelID))
//     {
//         return CPU_INTEL;
//     }
//     else if (!strcmp(Prcb->VendorString, CmpAmdID))
//     {
//         return CPU_AMD;
//     }
//     else if (!strcmp(Prcb->VendorString, CmpCyrixID))
//     {
//         DPRINT1("Cyrix CPU support not fully tested!\n");
//         return CPU_CYRIX;
//     }
//     else if (!strcmp(Prcb->VendorString, CmpTransmetaID))
//     {
//         DPRINT1("Transmeta CPU support not fully tested!\n");
//         return CPU_TRANSMETA;
//     }
//     else if (!strcmp(Prcb->VendorString, CmpCentaurID))
//     {
//         DPRINT1("Centaur CPU support not fully tested!\n");
//         return CPU_CENTAUR;
//     }
//     else if (!strcmp(Prcb->VendorString, CmpRiseID))
//     {
//         DPRINT1("Rise CPU support not fully tested!\n");
//         return CPU_RISE;
//     }
//
//     /* Unknown CPU */
//     DPRINT1("%s CPU support not fully tested!\n", Prcb->VendorString);
//     return CPU_UNKNOWN;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiSetProcessorType(VOID)
// {
//     CPU_INFO CpuInfo;
//     CPU_SIGNATURE CpuSignature;
//     BOOLEAN ExtendModel;
//     ULONG Stepping, Type;
//
//     /* Do CPUID 1 now */
//     KiCpuId(&CpuInfo, 1);
//
//     /*
//      * Get the Stepping and Type. The stepping contains both the
//      * Model and the Step, while the Type contains the returned Family.
//      *
//      * For the stepping, we convert this: zzzzzzxy into this: x0y
//      */
//     CpuSignature.AsULONG = CpuInfo.__bindgen_anon_1.Eax;
//     Stepping = CpuSignature.Model;
//     ExtendModel = (CpuSignature.Family == 15);
// #if ( (NTDDI_VERSION >= NTDDI_WINXPSP2) && (NTDDI_VERSION < NTDDI_WS03) ) || (NTDDI_VERSION >= NTDDI_WS03SP1)
//     if (CpuSignature.Family == 6)
//     {
//         ULONG Vendor = KiGetCpuVendor();
//         ExtendModel |= (Vendor == CPU_INTEL);
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         ExtendModel |= (Vendor == CPU_CENTAUR);
// #endif
//     }
// #endif
//     if (ExtendModel)
//     {
//         /* Add ExtendedModel to distinguish from non-extended values. */
//         Stepping |= (CpuSignature.ExtendedModel << 4);
//     }
//     Stepping = (Stepping << 8) | CpuSignature.Step;
//     Type = CpuSignature.Family;
//     if (CpuSignature.Family == 15)
//     {
//         /* Add ExtendedFamily to distinguish from non-extended values.
//          * It must not be larger than 0xF0 to avoid overflow. */
//         Type += min(CpuSignature.ExtendedFamily, 0xF0);
//     }
//
//     /* Save them in the PRCB */
//     KeGetCurrentPrcb()->CpuID = TRUE;
//     KeGetCurrentPrcb()->CpuType = (UCHAR)Type;
//     KeGetCurrentPrcb()->CpuStep = (USHORT)Stepping;
// }
//
// CODE_SEG("INIT")
// ULONG64
// NTAPI
// KiGetFeatureBits(VOID)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     ULONG Vendor;
//     ULONG64 FeatureBits = KF_WORKING_PTE;
//     CPU_INFO CpuInfo, DummyCpuInfo;
//     UCHAR Ccr1;
//     BOOLEAN ExtendedCPUID = TRUE;
//     ULONG CpuFeatures = 0;
//
//     /* Get the Vendor ID */
//     Vendor = KiGetCpuVendor();
//
//     /* Make sure we got a valid vendor ID at least. */
//     if (!Vendor) return FeatureBits;
//
//     /* Get the CPUID Info. Features are in Reg[3]. */
//     KiCpuId(&CpuInfo, 1);
//
//     /* Set the initial APIC ID */
//     Prcb->InitialApicId = (UCHAR)(CpuInfo.__bindgen_anon_1.Ebx >> 24);
//
//     switch (Vendor)
//     {
//         /* Intel CPUs */
//         case CPU_INTEL:
//
//             /* Check if it's a P6 */
//             if (Prcb->CpuType == 6)
//             {
//                 /* Perform the special sequence to get the MicroCode Signature */
//                 __writemsr(0x8B, 0);
//                 KiCpuId(&DummyCpuInfo, 1);
//                 Prcb->UpdateSignature.QuadPart = __readmsr(0x8B);
//             }
//             else if (Prcb->CpuType == 5)
//             {
//                 /* On P5, enable workaround for the LOCK errata. */
//                 KiI386PentiumLockErrataPresent = TRUE;
//             }
//
//             /* Check for broken P6 with bad SMP PTE implementation */
//             if (((CpuInfo.__bindgen_anon_1.Eax & 0x0FF0) == 0x0610 && (CpuInfo.__bindgen_anon_1.Eax & 0x000F) <= 0x9) ||
//                 ((CpuInfo.__bindgen_anon_1.Eax & 0x0FF0) == 0x0630 && (CpuInfo.__bindgen_anon_1.Eax & 0x000F) <= 0x4))
//             {
//                 /* Remove support for correct PTE support. */
//                 FeatureBits &= ~KF_WORKING_PTE;
//             }
//
//             /* Check if the CPU is too old to support SYSENTER */
//             if ((Prcb->CpuType < 6) ||
//                 ((Prcb->CpuType == 6) && (Prcb->CpuStep < 0x0303)))
//             {
//                 /* Disable it */
//                 CpuInfo.__bindgen_anon_1.Edx &= ~0x800;
//             }
//
//             break;
//
//         /* AMD CPUs */
//         case CPU_AMD:
//
//             /* Check if this is a K5 or K6. (family 5) */
//             if ((CpuInfo.__bindgen_anon_1.Eax & 0x0F00) == 0x0500)
//             {
//                 /* Get the Model Number */
//                 switch (CpuInfo.__bindgen_anon_1.Eax & 0x00F0)
//                 {
//                     /* Model 1: K5 - 5k86 (initial models) */
//                     case 0x0010:
//
//                         /* Check if this is Step 0 or 1. They don't support PGE */
//                         if ((CpuInfo.__bindgen_anon_1.Eax & 0x000F) > 0x03) break;
//
//                     /* Model 0: K5 - SSA5 */
//                     case 0x0000:
//
//                         /* Model 0 doesn't support PGE at all. */
//                         CpuInfo.__bindgen_anon_1.Edx &= ~0x2000;
//                         break;
//
//                     /* Model 8: K6-2 */
//                     case 0x0080:
//
//                         /* K6-2, Step 8 and over have support for MTRR. */
//                         if ((CpuInfo.__bindgen_anon_1.Eax & 0x000F) >= 0x8) FeatureBits |= KF_AMDK6MTRR;
//                         break;
//
//                     /* Model 9: K6-III
//                        Model D: K6-2+, K6-III+ */
//                     case 0x0090:
//                     case 0x00D0:
//
//                         FeatureBits |= KF_AMDK6MTRR;
//                         break;
//                 }
//             }
//             else if((CpuInfo.__bindgen_anon_1.Eax & 0x0F00) < 0x0500)
//             {
//                 /* Families below 5 don't support PGE, PSE or CMOV at all */
//                 CpuInfo.__bindgen_anon_1.Edx &= ~(0x08 | 0x2000 | 0x8000);
//
//                 /* They also don't support advanced CPUID functions. */
//                 ExtendedCPUID = FALSE;
//             }
//
//             break;
//
//         /* Cyrix CPUs */
//         case CPU_CYRIX:
//
//             /* Workaround the "COMA" bug on 6x family of Cyrix CPUs */
//             if (Prcb->CpuType == 6 &&
//                 Prcb->CpuStep <= 1)
//             {
//                 /* Get CCR1 value */
//                 Ccr1 = getCx86(CX86_CCR1);
//
//                 /* Enable the NO_LOCK bit */
//                 Ccr1 |= 0x10;
//
//                 /* Set the new CCR1 value */
//                 setCx86(CX86_CCR1, Ccr1);
//             }
//
//             break;
//
//         /* Transmeta CPUs */
//         case CPU_TRANSMETA:
//
//             /* Enable CMPXCHG8B if the family (>= 5), model and stepping (>= 4.2) support it */
//             if ((CpuInfo.__bindgen_anon_1.Eax & 0x0FFF) >= 0x0542)
//             {
//                 __writemsr(0x80860004, __readmsr(0x80860004) | 0x0100);
//                 FeatureBits |= KF_CMPXCHG8B;
//             }
//
//             break;
//
//         /* Centaur, IDT, Rise and VIA CPUs */
//         case CPU_CENTAUR:
//         case CPU_RISE:
//
//             /* These CPUs don't report the presence of CMPXCHG8B through CPUID.
//                However, this feature exists and operates properly without any additional steps. */
//             FeatureBits |= KF_CMPXCHG8B;
//
//             break;
//     }
//
//     /* Get some features from ECX */
//     if (CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSE3) FeatureBits |= KF_SSE3;
//     if (CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSSE3) FeatureBits |= KF_SSSE3;
//     if (CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSE4_1) FeatureBits |= KF_SSE4_1;
//     if (CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_SSE4_2) FeatureBits |= KF_SSE4_2;
//     if (CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_XSAVE) FeatureBits |= KF_XSTATE;
//     if (CpuInfo.__bindgen_anon_1.Ecx & X86_FEATURE_RDRAND) FeatureBits |= KF_RDRAND;
//
//     /* Set the current features */
//     CpuFeatures = CpuInfo.__bindgen_anon_1.Edx;
//
//     /* Convert all CPUID Feature bits into our format */
//     if (CpuFeatures & X86_FEATURE_VME)     FeatureBits |= KF_V86_VIS | KF_CR4;
//     if (CpuFeatures & X86_FEATURE_PSE)     FeatureBits |= KF_LARGE_PAGE | KF_CR4;
//     if (CpuFeatures & X86_FEATURE_TSC)     FeatureBits |= KF_RDTSC;
//     if (CpuFeatures & X86_FEATURE_CX8)     FeatureBits |= KF_CMPXCHG8B;
//     if (CpuFeatures & X86_FEATURE_SYSCALL) FeatureBits |= KF_FAST_SYSCALL;
//     if (CpuFeatures & X86_FEATURE_MTRR)    FeatureBits |= KF_MTRR;
//     if (CpuFeatures & X86_FEATURE_PGE)     FeatureBits |= KF_GLOBAL_PAGE | KF_CR4;
//     if (CpuFeatures & X86_FEATURE_CMOV)    FeatureBits |= KF_CMOV;
//     if (CpuFeatures & X86_FEATURE_PAT)     FeatureBits |= KF_PAT;
//     if (CpuFeatures & X86_FEATURE_DS)      FeatureBits |= KF_DTS;
//     if (CpuFeatures & X86_FEATURE_MMX)     FeatureBits |= KF_MMX;
//     if (CpuFeatures & X86_FEATURE_FXSR)    FeatureBits |= KF_FXSR;
//     if (CpuFeatures & X86_FEATURE_SSE)     FeatureBits |= KF_XMMI;
//     if (CpuFeatures & X86_FEATURE_SSE2)    FeatureBits |= KF_XMMI64;
//
//     /* Check if the CPU has hyper-threading */
//     if (CpuFeatures & X86_FEATURE_HT)
//     {
//         /* Set the number of logical CPUs */
//         Prcb->LogicalProcessorsPerPhysicalProcessor = (UCHAR)(CpuInfo.__bindgen_anon_1.Ebx >> 16);
//         if (Prcb->LogicalProcessorsPerPhysicalProcessor > 1)
//         {
//             /* We're on dual-core */
//             KiSMTProcessorsPresent = TRUE;
//         }
//     }
//     else
//     {
//         /* We only have a single CPU */
//         Prcb->LogicalProcessorsPerPhysicalProcessor = 1;
//     }
//
//     /* Check if CPUID 0x80000000 is supported */
//     if (ExtendedCPUID)
//     {
//         /* Do the call */
//         KiCpuId(&CpuInfo, 0x80000000);
//         if ((CpuInfo.__bindgen_anon_1.Eax & 0xffffff00) == 0x80000000)
//         {
//             /* Check if CPUID 0x80000001 is supported */
//             if (CpuInfo.__bindgen_anon_1.Eax >= 0x80000001)
//             {
//                 /* Check which extended features are available. */
//                 KiCpuId(&CpuInfo, 0x80000001);
//
//                 /* Check if NX-bit is supported */
//                 if (CpuInfo.__bindgen_anon_1.Edx & X86_FEATURE_NX) FeatureBits |= KF_NX_BIT;
//
//                 /* Now handle each features for each CPU Vendor */
//                 switch (Vendor)
//                 {
//                     case CPU_AMD:
//                     case CPU_CENTAUR:
//                         if (CpuInfo.__bindgen_anon_1.Edx & 0x80000000) FeatureBits |= KF_3DNOW;
//                         break;
//                 }
//             }
//         }
//     }
//
//     /* Return the Feature Bits */
//     return FeatureBits;
// }
//
// #if DBG
// CODE_SEG("INIT")
// VOID
// KiReportCpuFeatures(VOID)
// {
//     ULONG CpuFeatures = 0;
//     CPU_INFO CpuInfo;
//
//     if (KiGetCpuVendor())
//     {
//         KiCpuId(&CpuInfo, 1);
//         CpuFeatures = CpuInfo.__bindgen_anon_1.Edx;
//     }
//
//     DPRINT1("Supported CPU features:");
//
// #define print_kf_bit(kf_value) if (KeFeatureBits & kf_value) DbgPrint(" " #kf_value)
//     print_kf_bit(KF_V86_VIS);
//     print_kf_bit(KF_RDTSC);
//     print_kf_bit(KF_CR4);
//     print_kf_bit(KF_CMOV);
//     print_kf_bit(KF_GLOBAL_PAGE);
//     print_kf_bit(KF_LARGE_PAGE);
//     print_kf_bit(KF_MTRR);
//     print_kf_bit(KF_CMPXCHG8B);
//     print_kf_bit(KF_MMX);
//     print_kf_bit(KF_WORKING_PTE);
//     print_kf_bit(KF_PAT);
//     print_kf_bit(KF_FXSR);
//     print_kf_bit(KF_FAST_SYSCALL);
//     print_kf_bit(KF_XMMI);
//     print_kf_bit(KF_3DNOW);
//     print_kf_bit(KF_AMDK6MTRR);
//     print_kf_bit(KF_XMMI64);
//     print_kf_bit(KF_DTS);
//     print_kf_bit(KF_NX_BIT);
//     print_kf_bit(KF_NX_DISABLED);
//     print_kf_bit(KF_NX_ENABLED);
// #undef print_kf_bit
//
// #define print_cf(cpu_flag) if (CpuFeatures & cpu_flag) DbgPrint(" " #cpu_flag)
//     print_cf(X86_FEATURE_PAE);
//     print_cf(X86_FEATURE_APIC);
//     print_cf(X86_FEATURE_HT);
// #undef print_cf
//
//     DbgPrint("\n");
// }
// #endif // DBG
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiGetCacheInformation(VOID)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
//     CPU_INFO CpuInfo;
//     ULONG CacheRequests = 0, i;
//     ULONG CurrentRegister;
//     UCHAR RegisterByte, Associativity = 0;
//     ULONG Size, CacheLine = 64, CurrentSize = 0;
//     BOOLEAN FirstPass = TRUE;
//
//     /* Set default L2 size */
//     Pcr->SecondLevelCacheSize = 0;
//
//     /* Check the Vendor ID */
//     switch (KiGetCpuVendor())
//     {
//         /* Handle Intel case */
//         case CPU_INTEL:
//
//             /* Check if we support CPUID 2 */
//             KiCpuId(&CpuInfo, 0);
//             if (CpuInfo.__bindgen_anon_1.Eax >= 2)
//             {
//                 /* We need to loop for the number of times CPUID will tell us to */
//                 do
//                 {
//                     /* Do the CPUID call */
//                     KiCpuId(&CpuInfo, 2);
//
//                     /* Check if it was the first call */
//                     if (FirstPass)
//                     {
//                         /*
//                          * The number of times to loop is the first byte. Read
//                          * it and then destroy it so we don't get confused.
//                          */
//                         CacheRequests = CpuInfo.__bindgen_anon_1.Eax & 0xFF;
//                         CpuInfo.__bindgen_anon_1.Eax &= 0xFFFFFF00;
//
//                         /* Don't go over this again */
//                         FirstPass = FALSE;
//                     }
//
//                     /* Loop all 4 registers */
//                     for (i = 0; i < 4; i++)
//                     {
//                         /* Get the current register */
//                         CurrentRegister = CpuInfo.AsUINT32[i];
//
//                         /*
//                          * If the upper bit is set, then this register should
//                          * be skipped.
//                          */
//                         if (CurrentRegister & 0x80000000) continue;
//
//                         /* Keep looping for every byte inside this register */
//                         while (CurrentRegister)
//                         {
//                             /* Read a byte, skip a byte. */
//                             RegisterByte = (UCHAR)(CurrentRegister & 0xFF);
//                             CurrentRegister >>= 8;
//                             if (!RegisterByte) continue;
//
//                             Size = 0;
//                             switch (RegisterByte)
//                             {
//                                 case 0x06:
//                                 case 0x08:
//                                     KePrefetchNTAGranularity = 32;
//                                     break;
//                                 case 0x09:
//                                     KePrefetchNTAGranularity = 64;
//                                     break;
//                                 case 0x0a:
//                                 case 0x0c:
//                                     KePrefetchNTAGranularity = 32;
//                                     break;
//                                 case 0x0d:
//                                 case 0x0e:
//                                     KePrefetchNTAGranularity = 64;
//                                     break;
//                                 case 0x1d:
//                                     Size = 128 * 1024;
//                                     Associativity = 2;
//                                     break;
//                                 case 0x21:
//                                     Size = 256 * 1024;
//                                     Associativity = 8;
//                                     break;
//                                 case 0x24:
//                                     Size = 1024 * 1024;
//                                     Associativity = 16;
//                                     break;
//                                 case 0x2c:
//                                 case 0x30:
//                                     KePrefetchNTAGranularity = 64;
//                                     break;
//                                 case 0x41:
//                                 case 0x42:
//                                 case 0x43:
//                                 case 0x44:
//                                 case 0x45:
//                                     Size = (1 << (RegisterByte - 0x41)) * 128 * 1024;
//                                     Associativity = 4;
//                                     break;
//                                 case 0x48:
//                                     Size = 3 * 1024 * 1024;
//                                     Associativity = 12;
//                                     break;
//                                 case 0x49:
//                                     Size = 4 * 1024 * 1024;
//                                     Associativity = 16;
//                                     break;
//                                 case 0x4e:
//                                     Size = 6 * 1024 * 1024;
//                                     Associativity = 24;
//                                     break;
//                                 case 0x60:
//                                 case 0x66:
//                                 case 0x67:
//                                 case 0x68:
//                                     KePrefetchNTAGranularity = 64;
//                                     break;
//                                 case 0x78:
//                                     Size = 1024 * 1024;
//                                     Associativity = 4;
//                                     break;
//                                 case 0x79:
//                                 case 0x7a:
//                                 case 0x7b:
//                                 case 0x7c:
//                                 case 0x7d:
//                                     Size = (1 << (RegisterByte - 0x79)) * 128 * 1024;
//                                     Associativity = 8;
//                                     break;
//                                 case 0x7f:
//                                     Size = 512 * 1024;
//                                     Associativity = 2;
//                                     break;
//                                 case 0x80:
//                                     Size = 512 * 1024;
//                                     Associativity = 8;
//                                     break;
//                                 case 0x82:
//                                 case 0x83:
//                                 case 0x84:
//                                 case 0x85:
//                                     Size = (1 << (RegisterByte - 0x82)) * 256 * 1024;
//                                     Associativity = 8;
//                                     break;
//                                 case 0x86:
//                                     Size = 512 * 1024;
//                                     Associativity = 4;
//                                     break;
//                                 case 0x87:
//                                     Size = 1024 * 1024;
//                                     Associativity = 8;
//                                     break;
//                                 case 0xf0:
//                                     KePrefetchNTAGranularity = 64;
//                                     break;
//                                 case 0xf1:
//                                     KePrefetchNTAGranularity = 128;
//                                     break;
//                             }
//                             if (Size && (Size / Associativity) > CurrentSize)
//                             {
//                                 /* Set the L2 Cache Size and Associativity */
//                                 CurrentSize = Size / Associativity;
//                                 Pcr->SecondLevelCacheSize = Size;
//                                 Pcr->SecondLevelCacheAssociativity = Associativity;
//                             }
//                         }
//                     }
//                 } while (--CacheRequests);
//             }
//             break;
//
//         case CPU_AMD:
//
//             /* Check if we support CPUID 0x80000005 */
//             KiCpuId(&CpuInfo, 0x80000000);
//             if (CpuInfo.__bindgen_anon_1.Eax >= 0x80000005)
//             {
//                 /* Get L1 size first */
//                 KiCpuId(&CpuInfo, 0x80000005);
//                 KePrefetchNTAGranularity = CpuInfo.__bindgen_anon_1.Ecx & 0xFF;
//
//                 /* Check if we support CPUID 0x80000006 */
//                 KiCpuId(&CpuInfo, 0x80000000);
//                 if (CpuInfo.__bindgen_anon_1.Eax >= 0x80000006)
//                 {
//                     /* Get 2nd level cache and tlb size */
//                     KiCpuId(&CpuInfo, 0x80000006);
//
//                     /* Cache line size */
//                     CacheLine = CpuInfo.__bindgen_anon_1.Ecx & 0xFF;
//
//                     /* Hardcode associativity */
//                     RegisterByte = (CpuInfo.__bindgen_anon_1.Ecx >> 12) & 0xFF;
//                     switch (RegisterByte)
//                     {
//                         case 2:
//                             Associativity = 2;
//                             break;
//
//                         case 4:
//                             Associativity = 4;
//                             break;
//
//                         case 6:
//                             Associativity = 8;
//                             break;
//
//                         case 8:
//                         case 15:
//                             Associativity = 16;
//                             break;
//
//                         default:
//                             Associativity = 1;
//                             break;
//                     }
//
//                     /* Compute size */
//                     Size = (CpuInfo.__bindgen_anon_1.Ecx >> 16) << 10;
//
//                     /* Hack for Model 6, Steping 300 */
//                     if ((KeGetCurrentPrcb()->CpuType == 6) &&
//                         (KeGetCurrentPrcb()->CpuStep == 0x300))
//                     {
//                         /* Stick 64K in there */
//                         Size = 64 * 1024;
//                     }
//
//                     /* Set the L2 Cache Size and associativity */
//                     Pcr->SecondLevelCacheSize = Size;
//                     Pcr->SecondLevelCacheAssociativity = Associativity;
//                 }
//             }
//             break;
//
//         case CPU_CYRIX:
//         case CPU_TRANSMETA:
//         case CPU_CENTAUR:
//         case CPU_RISE:
//
//             /* FIXME */
//             break;
//     }
//
//     /* Set the cache line */
//     if (CacheLine > KeLargestCacheLine) KeLargestCacheLine = CacheLine;
//     DPRINT1("Prefetch Cache: %lu bytes\tL2 Cache: %lu bytes\tL2 Cache Line: %lu bytes\tL2 Cache Associativity: %lu\n",
//             KePrefetchNTAGranularity,
//             Pcr->SecondLevelCacheSize,
//             KeLargestCacheLine,
//             Pcr->SecondLevelCacheAssociativity);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiSetCR0Bits(VOID)
// {
//     ULONG Cr0;
//
//     /* Save current CR0 */
//     Cr0 = __readcr0();
//
//     /* If this is a 486, enable Write-Protection */
//     if (KeGetCurrentPrcb()->CpuType > 3) Cr0 |= CR0_WP;
//
//     /* Set new Cr0 */
//     __writecr0(Cr0);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeTSS2(IN PKTSS Tss,
//                  IN PKGDTENTRY TssEntry OPTIONAL)
// {
//     PUCHAR p;
//
//     /* Make sure the GDT Entry is valid */
//     if (TssEntry)
//     {
//         /* Set the Limit */
//         TssEntry->LimitLow = sizeof(KTSS) - 1;
//         TssEntry->HighWord.Bits.LimitHi = 0;
//     }
//
//     /* Now clear the I/O Map */
//     ASSERT(IOPM_COUNT == 1);
//     RtlFillMemory(Tss->IoMaps[0].IoMap, IOPM_FULL_SIZE, 0xFF);
//
//     /* Initialize Interrupt Direction Maps */
//     p = (PUCHAR)(Tss->IoMaps[0].DirectionMap);
//     RtlZeroMemory(p, IOPM_DIRECTION_MAP_SIZE);
//
//     /* Add DPMI support for interrupts */
//     p[0] = 4;
//     p[3] = 0x18;
//     p[4] = 0x18;
//
//     /* Initialize the default Interrupt Direction Map */
//     p = Tss->IntDirectionMap;
//     RtlZeroMemory(Tss->IntDirectionMap, IOPM_DIRECTION_MAP_SIZE);
//
//     /* Add DPMI support */
//     p[0] = 4;
//     p[3] = 0x18;
//     p[4] = 0x18;
// }
//
// VOID
// NTAPI
// KiInitializeTSS(IN PKTSS Tss)
// {
//     /* Set an invalid map base */
//     Tss->IoMapBase = KiComputeIopmOffset(IO_ACCESS_MAP_NONE);
//
//     /* Disable traps during Task Switches */
//     Tss->Flags = 0;
//
//     /* Set LDT and Ring 0 SS */
//     Tss->LDT = 0;
//     Tss->Ss0 = KGDT_R0_DATA;
// }
//
// CODE_SEG("INIT")
// VOID
// FASTCALL
// Ki386InitializeTss(IN PKTSS Tss,
//                    IN PKIDTENTRY Idt,
//                    IN PKGDTENTRY Gdt)
// {
//     PKGDTENTRY TssEntry, TaskGateEntry;
//
//     /* Initialize the boot TSS. */
//     TssEntry = &Gdt[KGDT_TSS / sizeof(KGDTENTRY)];
//     TssEntry->HighWord.Bits.Type = I386_TSS;
//     TssEntry->HighWord.Bits.Pres = 1;
//     TssEntry->HighWord.Bits.Dpl = 0;
//     KiInitializeTSS2(Tss, TssEntry);
//     KiInitializeTSS(Tss);
//
//     /* Load the task register */
//     Ke386SetTr(KGDT_TSS);
//
//     /* Setup the Task Gate for Double Fault Traps */
//     TaskGateEntry = (PKGDTENTRY)&Idt[8];
//     TaskGateEntry->HighWord.Bits.Type = I386_TASK_GATE;
//     TaskGateEntry->HighWord.Bits.Pres = 1;
//     TaskGateEntry->HighWord.Bits.Dpl = 0;
//     ((PKIDTENTRY)TaskGateEntry)->Selector = KGDT_DF_TSS;
//
//     /* Initialize the TSS used for handling double faults. */
//     Tss = (PKTSS)KiDoubleFaultTSS;
//     KiInitializeTSS(Tss);
//     Tss->CR3 = __readcr3();
//     Tss->Esp0 = KiDoubleFaultStack;
//     Tss->Esp = KiDoubleFaultStack;
//     Tss->Eip = PtrToUlong(KiTrap08);
//     Tss->Cs = KGDT_R0_CODE;
//     Tss->Fs = KGDT_R0_PCR;
//     Tss->Ss = Ke386GetSs();
//     Tss->Es = KGDT_R3_DATA | RPL_MASK;
//     Tss->Ds = KGDT_R3_DATA | RPL_MASK;
//
//     /* Setup the Double Trap TSS entry in the GDT */
//     TssEntry = &Gdt[KGDT_DF_TSS / sizeof(KGDTENTRY)];
//     TssEntry->HighWord.Bits.Type = I386_TSS;
//     TssEntry->HighWord.Bits.Pres = 1;
//     TssEntry->HighWord.Bits.Dpl = 0;
//     TssEntry->BaseLow = (USHORT)((ULONG_PTR)Tss & 0xFFFF);
//     TssEntry->HighWord.Bytes.BaseMid = (UCHAR)((ULONG_PTR)Tss >> 16);
//     TssEntry->HighWord.Bytes.BaseHi = (UCHAR)((ULONG_PTR)Tss >> 24);
//     TssEntry->LimitLow = KTSS_IO_MAPS;
//
//     /* Now setup the NMI Task Gate */
//     TaskGateEntry = (PKGDTENTRY)&Idt[2];
//     TaskGateEntry->HighWord.Bits.Type = I386_TASK_GATE;
//     TaskGateEntry->HighWord.Bits.Pres = 1;
//     TaskGateEntry->HighWord.Bits.Dpl = 0;
//     ((PKIDTENTRY)TaskGateEntry)->Selector = KGDT_NMI_TSS;
//
//     /* Initialize the actual TSS */
//     Tss = (PKTSS)KiNMITSS;
//     KiInitializeTSS(Tss);
//     Tss->CR3 = __readcr3();
//     Tss->Esp0 = KiDoubleFaultStack;
//     Tss->Esp = KiDoubleFaultStack;
//     Tss->Eip = PtrToUlong(KiTrap02);
//     Tss->Cs = KGDT_R0_CODE;
//     Tss->Fs = KGDT_R0_PCR;
//     Tss->Ss = Ke386GetSs();
//     Tss->Es = KGDT_R3_DATA | RPL_MASK;
//     Tss->Ds = KGDT_R3_DATA | RPL_MASK;
//
//     /* And its associated TSS Entry */
//     TssEntry = &Gdt[KGDT_NMI_TSS / sizeof(KGDTENTRY)];
//     TssEntry->HighWord.Bits.Type = I386_TSS;
//     TssEntry->HighWord.Bits.Pres = 1;
//     TssEntry->HighWord.Bits.Dpl = 0;
//     TssEntry->BaseLow = (USHORT)((ULONG_PTR)Tss & 0xFFFF);
//     TssEntry->HighWord.Bytes.BaseMid = (UCHAR)((ULONG_PTR)Tss >> 16);
//     TssEntry->HighWord.Bytes.BaseHi = (UCHAR)((ULONG_PTR)Tss >> 24);
//     TssEntry->LimitLow = KTSS_IO_MAPS;
// }
//
// VOID
// NTAPI
// KeFlushCurrentTb(VOID)
// {
//
// #if !defined(_GLOBAL_PAGES_ARE_AWESOME_)
//
//     /* Flush the TLB by resetting CR3 */
//     __writecr3(__readcr3());
//
// #else
//
//     /* Check if global pages are enabled */
//     if (KeFeatureBits & KF_GLOBAL_PAGE)
//     {
//         ULONG Cr4;
//
//         /* Disable PGE (Note: may not have been enabled yet) */
//         Cr4 = __readcr4();
//         __writecr4(Cr4 & ~CR4_PGE);
//
//         /* Flush everything */
//         __writecr3(__readcr3());
//
//         /* Re-enable PGE */
//         __writecr4(Cr4);
//     }
//     else
//     {
//         /* No global pages, resetting CR3 is enough */
//         __writecr3(__readcr3());
//     }
//
// #endif
//
// }
//
// VOID
// NTAPI
// KiRestoreProcessorControlState(PKPROCESSOR_STATE ProcessorState)
// {
//     PKGDTENTRY TssEntry;
//
//     //
//     // Restore the CR registers
//     //
//     __writecr0(ProcessorState->SpecialRegisters.Cr0);
//     Ke386SetCr2(ProcessorState->SpecialRegisters.Cr2);
//     __writecr3(ProcessorState->SpecialRegisters.Cr3);
//     if (KeFeatureBits & KF_CR4) __writecr4(ProcessorState->SpecialRegisters.Cr4);
//
//     //
//     // Restore the DR registers
//     //
//     __writedr(0, ProcessorState->SpecialRegisters.KernelDr0);
//     __writedr(1, ProcessorState->SpecialRegisters.KernelDr1);
//     __writedr(2, ProcessorState->SpecialRegisters.KernelDr2);
//     __writedr(3, ProcessorState->SpecialRegisters.KernelDr3);
//     __writedr(6, ProcessorState->SpecialRegisters.KernelDr6);
//     __writedr(7, ProcessorState->SpecialRegisters.KernelDr7);
//
//     //
//     // Restore GDT and IDT
//     //
//     Ke386SetGlobalDescriptorTable(&ProcessorState->SpecialRegisters.Gdtr.Limit);
//     __lidt(&ProcessorState->SpecialRegisters.Idtr.Limit);
//
//     //
//     // Clear the busy flag so we don't crash if we reload the same selector
//     //
//     TssEntry = (PKGDTENTRY)(ProcessorState->SpecialRegisters.Gdtr.Base +
//                             ProcessorState->SpecialRegisters.Tr);
//     TssEntry->HighWord.Bytes.Flags1 &= ~0x2;
//
//     //
//     // Restore TSS and LDT
//     //
//     Ke386SetTr(ProcessorState->SpecialRegisters.Tr);
//     Ke386SetLocalDescriptorTable(ProcessorState->SpecialRegisters.Ldtr);
// }
//
// VOID
// NTAPI
// KiSaveProcessorControlState(OUT PKPROCESSOR_STATE ProcessorState)
// {
//     /* Save the CR registers */
//     ProcessorState->SpecialRegisters.Cr0 = __readcr0();
//     ProcessorState->SpecialRegisters.Cr2 = __readcr2();
//     ProcessorState->SpecialRegisters.Cr3 = __readcr3();
//     ProcessorState->SpecialRegisters.Cr4 = (KeFeatureBits & KF_CR4) ?
//                                            __readcr4() : 0;
//
//     /* Save the DR registers */
//     ProcessorState->SpecialRegisters.KernelDr0 = __readdr(0);
//     ProcessorState->SpecialRegisters.KernelDr1 = __readdr(1);
//     ProcessorState->SpecialRegisters.KernelDr2 = __readdr(2);
//     ProcessorState->SpecialRegisters.KernelDr3 = __readdr(3);
//     ProcessorState->SpecialRegisters.KernelDr6 = __readdr(6);
//     ProcessorState->SpecialRegisters.KernelDr7 = __readdr(7);
//     __writedr(7, 0);
//
//     /* Save GDT, IDT, LDT and TSS */
//     Ke386GetGlobalDescriptorTable(&ProcessorState->SpecialRegisters.Gdtr.Limit);
//     __sidt(&ProcessorState->SpecialRegisters.Idtr.Limit);
//     ProcessorState->SpecialRegisters.Tr = Ke386GetTr();
//     Ke386GetLocalDescriptorTable(&ProcessorState->SpecialRegisters.Ldtr);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeMachineType(VOID)
// {
//     /* Set the Machine Type we got from NTLDR */
//     KeI386MachineType = KeLoaderBlock->u.I386.MachineType & 0x000FF;
// }
//
// CODE_SEG("INIT")
// ULONG_PTR
// NTAPI
// KiLoadFastSyscallMachineSpecificRegisters(IN ULONG_PTR Context)
// {
//     /* Set CS and ESP */
//     __writemsr(0x174, KGDT_R0_CODE);
//     __writemsr(0x175, (ULONG_PTR)KeGetCurrentPrcb()->DpcStack);
//
//     /* Set LSTAR */
//     __writemsr(0x176, (ULONG_PTR)KiFastCallEntry);
//     return 0;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiRestoreFastSyscallReturnState(VOID)
// {
//     /* Check if the CPU Supports fast system call */
//     if (KeFeatureBits & KF_FAST_SYSCALL)
//     {
//         /* Check if it has been disabled */
//         if (KiFastSystemCallDisable)
//         {
//             /* Disable fast system call */
//             KeFeatureBits &= ~KF_FAST_SYSCALL;
//             KiFastCallExitHandler = KiSystemCallTrapReturn;
//             DPRINT1("Support for SYSENTER disabled.\n");
//         }
//         else
//         {
//             /* Do an IPI to enable it */
//             KeIpiGenericCall(KiLoadFastSyscallMachineSpecificRegisters, 0);
//
//             /* It's enabled, so use the proper exit stub */
//             KiFastCallExitHandler = KiSystemCallSysExitReturn;
//             DPRINT("Support for SYSENTER detected.\n");
//         }
//     }
//     else
//     {
//         /* Use the IRET handler */
//         KiFastCallExitHandler = KiSystemCallTrapReturn;
//         DPRINT1("No support for SYSENTER detected.\n");
//     }
// }
//
// CODE_SEG("INIT")
// ULONG_PTR
// NTAPI
// Ki386EnableDE(IN ULONG_PTR Context)
// {
//     /* Enable DE */
//     __writecr4(__readcr4() | CR4_DE);
//     return 0;
// }
//
// CODE_SEG("INIT")
// ULONG_PTR
// NTAPI
// Ki386EnableFxsr(IN ULONG_PTR Context)
// {
//     /* Enable FXSR */
//     __writecr4(__readcr4() | CR4_FXSR);
//     return 0;
// }
//
// CODE_SEG("INIT")
// ULONG_PTR
// NTAPI
// Ki386EnableXMMIExceptions(IN ULONG_PTR Context)
// {
//     PKIDTENTRY IdtEntry;
//
//     /* Get the IDT Entry for Interrupt 0x13 */
//     IdtEntry = &((PKIPCR)KeGetPcr())->IDT[0x13];
//
//     /* Set it up */
//     IdtEntry->Selector = KGDT_R0_CODE;
//     IdtEntry->Offset = ((ULONG_PTR)KiTrap13 & 0xFFFF);
//     IdtEntry->ExtendedOffset = ((ULONG_PTR)KiTrap13 >> 16) & 0xFFFF;
//     ((PKIDT_ACCESS)&IdtEntry->Access)->Dpl = 0;
//     ((PKIDT_ACCESS)&IdtEntry->Access)->Present = 1;
//     ((PKIDT_ACCESS)&IdtEntry->Access)->SegmentType = I386_INTERRUPT_GATE;
//
//     /* Enable XMMI exceptions */
//     __writecr4(__readcr4() | CR4_XMMEXCPT);
//     return 0;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiI386PentiumLockErrataFixup(VOID)
// {
//     KDESCRIPTOR IdtDescriptor = {0, 0, 0};
//     PKIDTENTRY NewIdt, NewIdt2;
//     PMMPTE PointerPte;
//
//     /* Allocate memory for a new IDT */
//     NewIdt = ExAllocatePool(NonPagedPool, 2 * PAGE_SIZE);
//
//     /* Put everything after the first 7 entries on a new page */
//     NewIdt2 = (PVOID)((ULONG_PTR)NewIdt + PAGE_SIZE - (7 * sizeof(KIDTENTRY)));
//
//     /* Disable interrupts */
//     _disable();
//
//     /* Get the current IDT and copy it */
//     __sidt(&IdtDescriptor.Limit);
//     RtlCopyMemory(NewIdt2,
//                   (PVOID)IdtDescriptor.Base,
//                   IdtDescriptor.Limit + 1);
//     IdtDescriptor.Base = (ULONG)NewIdt2;
//
//     /* Set the new IDT */
//     __lidt(&IdtDescriptor.Limit);
//     ((PKIPCR)KeGetPcr())->IDT = NewIdt2;
//
//     /* Restore interrupts */
//     _enable();
//
//     /* Set the first 7 entries as read-only to produce a fault */
//     PointerPte = MiAddressToPte(NewIdt);
//     ASSERT(PointerPte->u.Hard.Write == 1);
//     PointerPte->u.Hard.Write = 0;
//     KeInvalidateTlbEntry(NewIdt);
// }
//
// BOOLEAN
// NTAPI
// KeInvalidateAllCaches(VOID)
// {
//     /* Only supported on Pentium Pro and higher */
//     if (KeI386CpuType < 6) return FALSE;
//
//     /* Invalidate all caches */
//     __wbinvd();
//     return TRUE;
// }
//
// VOID
// NTAPI
// KiSaveProcessorState(IN PKTRAP_FRAME TrapFrame,
//                      IN PKEXCEPTION_FRAME ExceptionFrame)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     //
//     // Save full context
//     //
//     Prcb->ProcessorState.ContextFrame.ContextFlags = CONTEXT_FULL |
//                                                      CONTEXT_DEBUG_REGISTERS;
//     KeTrapFrameToContext(TrapFrame, NULL, &Prcb->ProcessorState.ContextFrame);
//
//     //
//     // Save control registers
//     //
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// KiIsNpxErrataPresent(VOID)
// {
//     static double Value1 = 4195835.0, Value2 = 3145727.0;
//     INT ErrataPresent;
//     ULONG Cr0;
//
//     /* Interrupts have to be disabled here. */
//     ASSERT(!(__readeflags() & EFLAGS_INTERRUPT_MASK));
//
//     /* Read CR0 and remove FPU flags */
//     Cr0 = __readcr0();
//     __writecr0(Cr0 & ~(CR0_MP | CR0_TS | CR0_EM));
//
//     /* Initialize FPU state */
//     Ke386FnInit();
//
//     /* Multiply the magic values and divide, we should get the result back */
// #ifdef __GNUC__
//     __asm__ __volatile__
//     (
//         "fldl %1\n\t"
//         "fdivl %2\n\t"
//         "fmull %2\n\t"
//         "fldl %1\n\t"
//         "fsubp\n\t"
//         "fistpl %0\n\t"
//         : "=m" (ErrataPresent)
//         : "m" (Value1),
//           "m" (Value2)
//     );
// #else
//     __asm
//     {
//         fld Value1
//         fdiv Value2
//         fmul Value2
//         fld Value1
//         fsubp st(1), st(0)
//         fistp ErrataPresent
//     };
// #endif
//
//     /* Restore CR0 */
//     __writecr0(Cr0);
//
//     /* Return if there's an errata */
//     return ErrataPresent != 0;
// }
//
// VOID
// NTAPI
// KiFlushNPXState(IN PFLOATING_SAVE_AREA SaveArea)
// {
//     ULONG EFlags, Cr0;
//     PKTHREAD Thread, NpxThread;
//     PFX_SAVE_AREA FxSaveArea;
//
//     /* Save volatiles and disable interrupts */
//     EFlags = __readeflags();
//     _disable();
//
//     /* Save the PCR and get the current thread */
//     Thread = KeGetCurrentThread();
//
//     /* Check if we're already loaded */
//     if (Thread->NpxState != NPX_STATE_LOADED)
//     {
//         /* If there's nothing to load, quit */
//         if (!SaveArea)
//         {
//             /* Restore interrupt state and return */
//             __writeeflags(EFlags);
//             return;
//         }
//
//         /* Need FXSR support for this */
//         ASSERT(KeI386FxsrPresent == TRUE);
//
//         /* Check for sane CR0 */
//         Cr0 = __readcr0();
//         if (Cr0 & (CR0_MP | CR0_TS | CR0_EM))
//         {
//             /* Mask out FPU flags */
//             __writecr0(Cr0 & ~(CR0_MP | CR0_TS | CR0_EM));
//         }
//
//         /* Get the NPX thread and check its FPU state */
//         NpxThread = KeGetCurrentPrcb()->NpxThread;
//         if ((NpxThread) && (NpxThread->NpxState == NPX_STATE_LOADED))
//         {
//             /* Get the FX frame and store the state there */
//             FxSaveArea = KiGetThreadNpxArea(NpxThread);
//             Ke386FxSave(FxSaveArea);
//
//             /* NPX thread has lost its state */
//             NpxThread->NpxState = NPX_STATE_NOT_LOADED;
//         }
//
//         /* Now load NPX state from the NPX area */
//         FxSaveArea = KiGetThreadNpxArea(Thread);
//         Ke386FxStore(FxSaveArea);
//     }
//     else
//     {
//         /* Check for sane CR0 */
//         Cr0 = __readcr0();
//         if (Cr0 & (CR0_MP | CR0_TS | CR0_EM))
//         {
//             /* Mask out FPU flags */
//             __writecr0(Cr0 & ~(CR0_MP | CR0_TS | CR0_EM));
//         }
//
//         /* Get FX frame */
//         FxSaveArea = KiGetThreadNpxArea(Thread);
//         Thread->NpxState = NPX_STATE_NOT_LOADED;
//
//         /* Save state if supported by CPU */
//         if (KeI386FxsrPresent) Ke386FxSave(FxSaveArea);
//     }
//
//     /* Now save the FN state wherever it was requested */
//     if (SaveArea) Ke386FnSave(SaveArea);
//
//     /* Clear NPX thread */
//     KeGetCurrentPrcb()->NpxThread = NULL;
//
//     /* Add the CR0 from the NPX frame */
//     Cr0 |= NPX_STATE_NOT_LOADED;
//     Cr0 |= FxSaveArea->Cr0NpxState;
//     __writecr0(Cr0);
//
//     /* Restore interrupt state */
//     __writeeflags(EFlags);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KiCoprocessorError(VOID)
// {
//     PFX_SAVE_AREA NpxArea;
//
//     /* Get the FPU area */
//     NpxArea = KiGetThreadNpxArea(KeGetCurrentThread());
//
//     /* Set CR0_TS */
//     NpxArea->Cr0NpxState = CR0_TS;
//     __writecr0(__readcr0() | CR0_TS);
// }
//
// /**
//  * @brief
//  * Saves the current floating point unit state
//  * context of the current calling thread.
//  *
//  * @param[out] Save
//  * The saved floating point context given to the
//  * caller at the end of function's operations.
//  * The structure whose data contents are opaque
//  * to the calling thread.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has
//  * successfully completed its operations.
//  * STATUS_INSUFFICIENT_RESOURCES is returned
//  * if the function couldn't allocate memory
//  * for FPU state information.
//  *
//  * @remarks
//  * The function performs a FPU state save
//  * in two ways. A normal FPU save (FNSAVE)
//  * is performed if the system doesn't have
//  * SSE/SSE2, otherwise the function performs
//  * a save of FPU, MMX and SSE states save (FXSAVE).
//  */
// #if defined(__clang__)
// __attribute__((__target__("sse")))
// #endif
// NTSTATUS
// NTAPI
// KeSaveFloatingPointState(
//     _Out_ PKFLOATING_SAVE Save)
// {
//     PFLOATING_SAVE_CONTEXT FsContext;
//     PFX_SAVE_AREA FxSaveAreaFrame;
//     PKPRCB CurrentPrcb;
//
//     /* Sanity checks */
//     ASSERT(Save);
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//     ASSERT(KeI386NpxPresent);
//
//     /* Initialize the floating point context */
//     FsContext = ExAllocatePoolWithTag(NonPagedPool,
//                                       sizeof(FLOATING_SAVE_CONTEXT),
//                                       TAG_FLOATING_POINT_CONTEXT);
//     if (!FsContext)
//     {
//         /* Bail out if we failed */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /*
//      * Allocate some memory pool for the buffer. The size
//      * of this allocated buffer is the FX area plus the
//      * alignment requirement needed for FXSAVE as a 16-byte
//      * aligned pointer is compulsory in order to save the
//      * FPU state.
//      */
//     FsContext->Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                               sizeof(FX_SAVE_AREA) + FXSAVE_ALIGN,
//                                               TAG_FLOATING_POINT_FX);
//     if (!FsContext->Buffer)
//     {
//         /* Bail out if we failed */
//         ExFreePoolWithTag(FsContext, TAG_FLOATING_POINT_CONTEXT);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /*
//      * Now cache the allocated buffer into the save area
//      * and align the said area to a 16-byte boundary. Why
//      * do we have to do this is because of ExAllocate function.
//      * We gave the necessary alignment requirement in the pool
//      * allocation size although the function will always return
//      * a 8-byte aligned pointer. Aligning the given pointer directly
//      * can cause issues when freeing it from memory afterwards. With
//      * that said, we have to cache the buffer to the area so that we
//      * do not touch or mess the allocated buffer any further.
//      */
//     FsContext->PfxSaveArea = ALIGN_UP_POINTER_BY(FsContext->Buffer, 16);
//
//     /* Disable interrupts and get the current processor control region */
//     _disable();
//     CurrentPrcb = KeGetCurrentPrcb();
//
//     /* Store the current thread to context */
//     FsContext->CurrentThread = KeGetCurrentThread();
//
//     /*
//      * Save the previous NPX thread state registers (aka Numeric
//      * Processor eXtension) into the current context so that
//      * we are informing the scheduler the current FPU state
//      * belongs to this thread.
//      */
//     if (FsContext->CurrentThread != CurrentPrcb->NpxThread)
//     {
//         if ((CurrentPrcb->NpxThread != NULL) &&
//             (CurrentPrcb->NpxThread->NpxState == NPX_STATE_LOADED))
//         {
//             /* Get the FX frame */
//             FxSaveAreaFrame = KiGetThreadNpxArea(CurrentPrcb->NpxThread);
//
//             /* Save the FPU state */
//             Ke386SaveFpuState(FxSaveAreaFrame);
//
//             /* NPX thread has lost its state */
//             CurrentPrcb->NpxThread->NpxState = NPX_STATE_NOT_LOADED;
//             FxSaveAreaFrame->NpxSavedCpu = 0;
//         }
//
//         /* The new NPX thread is the current thread */
//         CurrentPrcb->NpxThread = FsContext->CurrentThread;
//     }
//
//     /* Perform the save */
//     Ke386SaveFpuState(FsContext->PfxSaveArea);
//
//     /* Store the NPX IRQL */
//     FsContext->OldNpxIrql = FsContext->CurrentThread->Header.NpxIrql;
//
//     /* Set the current IRQL to NPX */
//     FsContext->CurrentThread->Header.NpxIrql = KeGetCurrentIrql();
//
//     /* Initialize the FPU */
//     Ke386FnInit();
//
//     /* Enable interrupts back */
//     _enable();
//
//     /* Give the saved FPU context to the caller */
//     *((PVOID *) Save) = FsContext;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Restores the original FPU state context that has
//  * been saved by a API call of KeSaveFloatingPointState.
//  * Callers are expected to restore the floating point
//  * state by calling this function when they've finished
//  * doing FPU operations.
//  *
//  * @param[in] Save
//  * The saved floating point context that is to be given
//  * to the function to restore the FPU state.
//  *
//  * @return
//  * Returns STATUS_SUCCESS indicating the function
//  * has fully completed its operations.
//  */
// #if defined(__clang__)
// __attribute__((__target__("sse")))
// #endif
// NTSTATUS
// NTAPI
// KeRestoreFloatingPointState(
//     _In_ PKFLOATING_SAVE Save)
// {
//     PFLOATING_SAVE_CONTEXT FsContext;
//
//     /* Sanity checks */
//     ASSERT(Save);
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//     ASSERT(KeI386NpxPresent);
//
//     /* Cache the saved FS context */
//     FsContext = *((PVOID *) Save);
//
//     /*
//      * We have to restore the regular saved FPU
//      * state. For this we must first do some
//      * validation checks so that we are sure
//      * ourselves the state context is saved
//      * properly. Check if we are in the same
//      * calling thread.
//      */
//     if (FsContext->CurrentThread != KeGetCurrentThread())
//     {
//         /*
//          * This isn't the thread that saved the
//          * FPU state context, crash the system!
//          */
//         KeBugCheckEx(INVALID_FLOATING_POINT_STATE,
//                      0x2,
//                      (ULONG_PTR)FsContext->CurrentThread,
//                      (ULONG_PTR)KeGetCurrentThread(),
//                      0);
//     }
//
//     /* Are we under the same NPX interrupt level? */
//     if (FsContext->CurrentThread->Header.NpxIrql != KeGetCurrentIrql())
//     {
//         /* The interrupt level has changed, crash the system! */
//         KeBugCheckEx(INVALID_FLOATING_POINT_STATE,
//                      0x1,
//                      (ULONG_PTR)FsContext->CurrentThread->Header.NpxIrql,
//                      (ULONG_PTR)KeGetCurrentIrql(),
//                      0);
//     }
//
//     /* Disable interrupts */
//     _disable();
//
//     /*
//      * The saved FPU state context is valid,
//      * it's time to restore the state. First,
//      * clear FPU exceptions now.
//      */
//     Ke386ClearFpExceptions();
//
//     /* Restore the state */
//     Ke386RestoreFpuState(FsContext->PfxSaveArea);
//
//     /* Give the saved NPX IRQL back to the NPX thread */
//     FsContext->CurrentThread->Header.NpxIrql = FsContext->OldNpxIrql;
//
//     /* Enable interrupts back */
//     _enable();
//
//     /* We're done, free the allocated area and context */
//     ExFreePoolWithTag(FsContext->Buffer, TAG_FLOATING_POINT_FX);
//     ExFreePoolWithTag(FsContext, TAG_FLOATING_POINT_CONTEXT);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// KeGetRecommendedSharedDataAlignment(VOID)
// {
//     /* Return the global variable */
//     return KeLargestCacheLine;
// }
//
// VOID
// NTAPI
// KiFlushTargetEntireTb(IN PKIPI_CONTEXT PacketContext,
//                       IN PVOID Ignored1,
//                       IN PVOID Ignored2,
//                       IN PVOID Ignored3)
// {
//     /* Signal this packet as done */
//     KiIpiSignalPacketDone(PacketContext);
//
//     /* Flush the TB for the Current CPU */
//     KeFlushCurrentTb();
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeFlushEntireTb(IN BOOLEAN Invalid,
//                 IN BOOLEAN AllProcessors)
// {
//     KIRQL OldIrql;
// #ifdef CONFIG_SMP
//     KAFFINITY TargetAffinity;
//     PKPRCB Prcb = KeGetCurrentPrcb();
// #endif
//
//     /* Raise the IRQL for the TB Flush */
//     OldIrql = KeRaiseIrqlToSynchLevel();
//
// #ifdef CONFIG_SMP
//     /* FIXME: Use KiTbFlushTimeStamp to synchronize TB flush */
//
//     /* Get the current processor affinity, and exclude ourselves */
//     TargetAffinity = KeActiveProcessors;
//     TargetAffinity &= ~Prcb->SetMember;
//
//     /* Make sure this is MP */
//     if (TargetAffinity)
//     {
//         /* Send an IPI TB flush to the other processors */
//         KiIpiSendPacket(TargetAffinity,
//                         KiFlushTargetEntireTb,
//                         NULL,
//                         0,
//                         NULL);
//     }
// #endif
//
//     /* Flush the TB for the Current CPU, and update the flush stamp */
//     KeFlushCurrentTb();
//
// #ifdef CONFIG_SMP
//     /* If this is MP, wait for the other processors to finish */
//     if (TargetAffinity)
//     {
//         /* Sanity check */
//         ASSERT(Prcb == KeGetCurrentPrcb());
//
//         /* FIXME: TODO */
//         ASSERTMSG("Not yet implemented\n", FALSE);
//     }
// #endif
//
//     /* Update the flush stamp and return to original IRQL */
//     InterlockedExchangeAdd(&KiTbFlushTimeStamp, 1);
//     KeLowerIrql(OldIrql);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeSetDmaIoCoherency(IN ULONG Coherency)
// {
//     /* Save the coherency globally */
//     KiDmaIoCoherency = Coherency;
// }
//
// /*
//  * @implemented
//  */
// VOID
// __cdecl
// KeSaveStateForHibernate(IN PKPROCESSOR_STATE State)
// {
//     /* Capture the context */
//     RtlCaptureContext(&State->ContextFrame);
//
//     /* Capture the control state */
//     KiSaveProcessorControlState(State);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
