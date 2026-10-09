use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PLOADER_PARAMETER_BLOCK};
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderMemoryData as LoaderMemoryData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupPcrPage as LoaderStartupPcrPage;

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn ERR(fmt: *const i8, ...);
    // boot/freeldr/freeldr/include/mm.h: ARC loader allocator. amd64 winldr.rs
    // declares it the same way file-locally. Returns PVOID; arm64 NTAPI = plain
    // cdecl-ish (no @N decoration).
    fn MmAllocateMemoryWithType(size: usize, mem_type: u32) -> PVOID;
}

// ---------------------------------------------------------------------------
// arm64 file-scope preamble restored faithfully from
// boot/freeldr/freeldr/ntldr/arch/arm64/winldr.c (rustconv dropped all
// non-function content: #defines, typedefs, and the two file-scope globals).
// Values are verbatim from the C macros above. PAGE_SIZE / MM_HAL_VA_START
// match sdk/include/ndk/arm64/{mmtypes,ketypes}.h.
// ---------------------------------------------------------------------------

/* typedef ULONG64 ARM64_PTE;  typedef ARM64_PTE *PARM64_PTE; */
#[allow(non_camel_case_types)]
type ARM64_PTE = ULONG64;
#[allow(non_camel_case_types)]
type PARM64_PTE = *mut ARM64_PTE;

/* ARM64 uses 4KB pages with 4-level page tables (like x86-64) */
#[allow(dead_code)]
const ARM64_PAGE_SHIFT: u32 = 12;
#[allow(dead_code)]
const ARM64_PAGE_SIZE: u64 = 1u64 << ARM64_PAGE_SHIFT;

/* freeldr mm.h PAGE_SIZE (== ndk/arm64/mmtypes.h 0x1000). The bare PAGE_SIZE
 * from ntkrnl_sys::base is `usize`; this winldr does ULONG64 page arithmetic,
 * so keep a file-local u64 named PAGE_SIZE matching the C usage exactly. */
const PAGE_SIZE: u64 = 0x1000;

/* Page table entry indices */
const ARM64_PGD_SHIFT: u32 = 39;
const ARM64_PUD_SHIFT: u32 = 30;
const ARM64_PMD_SHIFT: u32 = 21;
const ARM64_PTE_SHIFT: u32 = 12;

#[allow(dead_code)]
const ARM64_PTRS_PER_LEVEL: u32 = 512;

/* Get index for each level — macros -> file-local fns (returns ULONG like the
 * C use sites which pass the result to MempGetOrCreatePageDir(.., ULONG)). */
#[inline]
fn ARM64_PGD_INDEX(va: u64) -> ULONG { (((va) >> ARM64_PGD_SHIFT) & 0x1FF) as ULONG }
#[inline]
fn ARM64_PUD_INDEX(va: u64) -> ULONG { (((va) >> ARM64_PUD_SHIFT) & 0x1FF) as ULONG }
#[inline]
fn ARM64_PMD_INDEX(va: u64) -> ULONG { (((va) >> ARM64_PMD_SHIFT) & 0x1FF) as ULONG }
#[inline]
fn ARM64_PTE_INDEX(va: u64) -> ULONG { (((va) >> ARM64_PTE_SHIFT) & 0x1FF) as ULONG }

/* Page table entry attributes */
const ARM64_PTE_VALID: u64 = 1u64 << 0;
const ARM64_PTE_TABLE: u64 = 1u64 << 1;
const ARM64_PTE_PAGE: u64 = 1u64 << 1;
const ARM64_PTE_AF: u64 = 1u64 << 10; /* Access Flag */
const ARM64_PTE_SH_INNER: u64 = 3u64 << 8; /* Inner Shareable */
const ARM64_PTE_AP_RW: u64 = 0u64 << 6; /* Read-Write */
#[allow(dead_code)]
const ARM64_PTE_AP_RO: u64 = 1u64 << 7; /* Read-Only */
const ARM64_PTE_ATTR_NORMAL: u64 = 0u64 << 2; /* Normal memory */
const ARM64_PTE_ATTR_DEVICE: u64 = 1u64 << 2; /* Device-nGnRnE (MAIR Attr1) */

/* Standard page entry for kernel memory */
const ARM64_KERNEL_PAGE_FLAGS: u64 = ARM64_PTE_VALID
    | ARM64_PTE_PAGE
    | ARM64_PTE_AF
    | ARM64_PTE_SH_INNER
    | ARM64_PTE_AP_RW
    | ARM64_PTE_ATTR_NORMAL;

/* Device MMIO page entry (Device-nGnRnE via MAIR Attr1, no shareability bits) */
const ARM64_DEVICE_PAGE_FLAGS: u64 =
    ARM64_PTE_VALID | ARM64_PTE_PAGE | ARM64_PTE_AF | ARM64_PTE_AP_RW | ARM64_PTE_ATTR_DEVICE;
/* Table descriptors (L0/L1/L2) — see C comment for the AF/SH_INNER rationale. */
const ARM64_KERNEL_TABLE_FLAGS: u64 =
    ARM64_PTE_VALID | ARM64_PTE_TABLE | ARM64_PTE_AF | ARM64_PTE_SH_INNER;

/* ndk/arm64/ketypes.h: #define MM_HAL_VA_START 0xFFFFFFFFFFC00000ULL.
 * amd64 winldr.rs keeps the identical value as a file-local const. */
const MM_HAL_VA_START: u64 = 0xFFFFFFFFFFC00000;

/* Level 0 page table (PGD) — file-scope global. */
static mut Arm64Pgd: PARM64_PTE = core::ptr::null_mut();
/* Shared user data page frame number — file-scope global. */
static mut SharedUserDataPfn: PFN_NUMBER = 0;

// Minimal EL1 exception vectors for the loader.
//
// Dropping to EL1 costs us the firmware's handler: its vectors are EL2's, so
// after the drop a fault at EL1 lands on whatever VBAR_EL1 happens to hold and
// the board simply stops - the "Synchronous Exception at 0x..." line that
// located every earlier bug disappears. Installing these keeps faults legible
// until the kernel installs KiArm64VectorTable.
//
// 16 entries of 128 bytes at a 2048-byte alignment, as the architecture
// requires; every one funnels to the same reporter because the loader has
// nothing to recover to.
core::arch::global_asm!(
    ".balign 2048",
    ".globl Arm64LoaderVectorTable",
    "Arm64LoaderVectorTable:",
    ".rept 16",
    ".balign 128",
    "b Arm64LoaderFaultEntry",
    ".endr",
);

extern "C" {
    static Arm64LoaderVectorTable: u8;
}

/// Report an EL1 fault and stop. No recovery is possible here, so this does not
/// preserve registers - it exists so a fault says what and where.
#[no_mangle]
pub unsafe extern "C" fn Arm64LoaderFaultEntry() -> ! {
    let (esr, far, elr, spsr): (ULONG64, ULONG64, ULONG64, ULONG64);
    core::arch::asm!("mrs {}, esr_el1", out(reg) esr, options(nomem, nostack));
    core::arch::asm!("mrs {}, far_el1", out(reg) far, options(nomem, nostack));
    core::arch::asm!("mrs {}, elr_el1", out(reg) elr, options(nomem, nostack));
    core::arch::asm!("mrs {}, spsr_el1", out(reg) spsr, options(nomem, nostack));
    crate::frldr::dbgcon_str(b"\r\n!!FLDR-EL1-FAULT esr=0x");
    crate::frldr::dbgcon_hex64(esr);
    crate::frldr::dbgcon_str(b" far=0x");
    crate::frldr::dbgcon_hex64(far);
    crate::frldr::dbgcon_str(b" elr=0x");
    crate::frldr::dbgcon_hex64(elr);
    crate::frldr::dbgcon_str(b" spsr=0x");
    crate::frldr::dbgcon_hex64(spsr);
    // EC is the top 6 bits of ESR: 0x25 data abort, 0x21 instruction abort.
    crate::frldr::dbgcon_str(b" ec=0x");
    crate::frldr::dbgcon_hex8(((esr >> 26) & 0x3F) as u32);
    crate::frldr::dbgcon_str(b"\r\n");
    loop {
        core::arch::asm!("wfi", options(nomem, nostack));
    }
}

/// Point VBAR_EL1 at the loader's vectors.
pub unsafe fn Arm64InstallEl1Vectors() {
    let base = core::ptr::addr_of!(Arm64LoaderVectorTable) as ULONG64;
    core::arch::asm!("msr vbar_el1, {}", "isb", in(reg) base, options(nostack));
}

/// Drop from EL2 to EL1 so the EL1 translation regime the loader programs (and
/// the kernel requires) is the one that actually translates.
///
/// Measured 2026-09-20 on a Pi 4B: pftf/RPi4 UEFI hands off at **EL2 with
/// HCR_EL2.E2H == 0**, where writes to TTBR0_EL1/TTBR1_EL1/TCR_EL1/MAIR_EL1 are
/// inert for the current regime - EL2 keeps translating through TTBR0_EL2 under
/// the firmware's identity map. The loader therefore ran with correct page
/// tables that nothing consulted: identity addresses worked, the first store
/// through a PaToVa KSEG0 VA faulted, and no error was reported anywhere.
/// QEMU hands off at EL1, which is why this was never seen there.
///
/// EL1 is also where the kernel must end up regardless - KiSystemStartup writes
/// CPACR_EL1 and the trap vectors are EL1 - so dropping is the fix, not
/// programming EL2 translation instead.
///
/// Caller must already have taken the system down to a state where firmware
/// boot services are no longer used, and must enable the EL1 MMU afterwards:
/// EL1 arrives with SCTLR_EL1.M clear, so the window between `eret` and the
/// MMU enable runs flat-addressed and is kept as short as possible.
pub unsafe fn Arm64DropToEl1() {
    let sp: ULONG64;
    core::arch::asm!("mov {}, sp", out(reg) sp, options(nomem, nostack));
    core::arch::asm!(
        // EL1h takes its stack from SP_EL1, not the EL2 stack we are on.
        "msr sp_el1, {sp}",
        // RW: EL1 is AArch64. Writing the whole register also clears TGE, which
        // must be 0 to execute at EL1 at all.
        "mov {t}, #(1 << 31)",
        "msr hcr_el2, {t}",
        // EL1PCTEN|EL1PCEN: let EL1 read the counter and drive the timer. Without
        // this the HAL's generic-timer clock traps to EL2 the moment it starts.
        "mov {t}, #3",
        "msr cnthctl_el2, {t}",
        "msr cntvoff_el2, xzr",
        // No stage-2 translation; report this CPU to EL1 as itself.
        "msr vttbr_el2, xzr",
        "mrs {t}, midr_el1",
        "msr vpidr_el2, {t}",
        "mrs {t}, mpidr_el1",
        "msr vmpidr_el2, {t}",
        // RES1 bits with TFP clear, so EL1 FP/SIMD does not trap to EL2 (the
        // kernel enables FP via CPACR_EL1 immediately on entry).
        "mov {t}, #0x33ff",
        "msr cptr_el2, {t}",
        // Return to the next instruction, at EL1h, with DAIF masked.
        "mov {t}, #0x3c5",
        "msr spsr_el2, {t}",
        "adr {t}, 99f",
        "msr elr_el2, {t}",
        "dsb sy",
        "isb",
        "eret",
        "99:",
        sp = in(reg) sp,
        t = out(reg) _,
        options(nostack)
    );
}

/// Report the exception level and the translation control the loader is
/// actually running under.
///
/// WinLdrSetProcessorContext programs MAIR_EL1/TCR_EL1/TTBR0_EL1/TTBR1_EL1 and
/// invalidates with `tlbi vmalle1`. That is only the EL1 regime. If firmware
/// hands off at EL2 WITHOUT VHE (HCR_EL2.E2H == 0), every one of those writes is
/// inert for the regime we execute in: EL2 translates through TTBR0_EL2, the
/// firmware's own identity map stays live, and the symptom is exactly what this
/// board shows — identity addresses work, the KSEG0 VA faults, our tables are
/// correct but unused, and nothing reports an error. With VHE (E2H == 1) the
/// _EL1 accesses are redirected to the _EL2 registers and the same code works,
/// which is why QEMU has never caught this.
pub unsafe fn Arm64ReportExceptionLevel(Label: &[u8]) {
    let current_el: u64;
    core::arch::asm!("mrs {}, currentel", out(reg) current_el, options(nomem, nostack));
    let el = (current_el >> 2) & 3;
    crate::frldr::dbgcon_str(Label);
    crate::frldr::dbgcon_str(b" EL");
    crate::frldr::dbgcon_hex8(el as u32);
    let sctlr: u64;
    core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr, options(nomem, nostack));
    crate::frldr::dbgcon_str(b" SCTLR_EL1=0x");
    crate::frldr::dbgcon_hex64(sctlr);
    if el == 2 {
        // Only readable at EL2; reading it from EL1 would trap.
        let hcr: u64;
        core::arch::asm!("mrs {}, hcr_el2", out(reg) hcr, options(nomem, nostack));
        crate::frldr::dbgcon_str(b" HCR_EL2=0x");
        crate::frldr::dbgcon_hex64(hcr);
        crate::frldr::dbgcon_str(if hcr & (1 << 34) != 0 { b" E2H=1" } else { b" E2H=0" });
    }
    crate::frldr::dbgcon_str(b"\r\n");
}

/// Walk the live page tables for one VA and report each level, so an unmapped
/// address says WHERE translation stopped instead of only that it faulted.
/// Reads the tables through their physical addresses, which is what the loader
/// has been doing all along; safe both before and after the TTBR switch.
pub unsafe fn Arm64ReportMapping(Label: &[u8], VirtualAddress: ULONG64) {
    crate::frldr::dbgcon_str(Label);
    crate::frldr::dbgcon_str(b" va=0x");
    crate::frldr::dbgcon_hex64(VirtualAddress);
    if Arm64Pgd.is_null() {
        crate::frldr::dbgcon_str(b" PGD=null\r\n");
        return;
    }
    let mut table = Arm64Pgd;
    let indices = [
        ARM64_PGD_INDEX(VirtualAddress),
        ARM64_PUD_INDEX(VirtualAddress),
        ARM64_PMD_INDEX(VirtualAddress),
        ARM64_PTE_INDEX(VirtualAddress),
    ];
    for (level, index) in indices.iter().enumerate() {
        let entry = *table.add(*index as usize);
        crate::frldr::dbgcon_str(b" L");
        crate::frldr::dbgcon_hex8(level as u32);
        crate::frldr::dbgcon_str(b"[");
        crate::frldr::dbgcon_hex32(*index);
        crate::frldr::dbgcon_str(b"]=0x");
        crate::frldr::dbgcon_hex64(entry);
        if entry & ARM64_PTE_VALID == 0 {
            crate::frldr::dbgcon_str(b" INVALID\r\n");
            return;
        }
        if level == 3 {
            break;
        }
        table = (entry & !0xFFFu64) as PARM64_PTE;
    }
    crate::frldr::dbgcon_str(b" MAPPED\r\n");
}

unsafe fn MempAllocatePageTables() -> u8 {
    TRACE(b">>> MempAllocatePageTables\n\0".as_ptr() as _);

    Arm64Pgd = MmAllocateMemoryWithType(PAGE_SIZE as usize, LoaderMemoryData) as PARM64_PTE;
    if Arm64Pgd.is_null() {
        ERR(b"Failed to allocate ARM64 PGD\n\0".as_ptr() as _);
        return 0u8;
    }

    RtlZeroMemory(Arm64Pgd as *mut core::ffi::c_void, PAGE_SIZE as usize);

    const ARM64_PXE_SELFMAP_INDEX: usize = 0x1ED;
    *Arm64Pgd.add(ARM64_PXE_SELFMAP_INDEX) =
        ((Arm64Pgd as u64) & !0xFFFu64) | ARM64_KERNEL_PAGE_FLAGS;

    TRACE(
        b">>> ARM64 PGD allocated at %p, self-map at index 0x%x\n\0".as_ptr() as _,
        Arm64Pgd,
        ARM64_PXE_SELFMAP_INDEX,
    );
    return 1u8;
}

unsafe fn MempGetOrCreatePageDir(ParentTable: PARM64_PTE, Index: ULONG) -> PARM64_PTE {
    let SubDir: PARM64_PTE;
    let Entry: ARM64_PTE;

    if ParentTable.is_null() {
        return core::ptr::null_mut();
    }

    Entry = *ParentTable.add(Index as usize);

    if (Entry & ARM64_PTE_VALID) == 0 {
        SubDir = MmAllocateMemoryWithType(PAGE_SIZE as usize, LoaderMemoryData) as PARM64_PTE;
        if SubDir.is_null() {
            return core::ptr::null_mut();
        }

        RtlZeroMemory(SubDir as PVOID, PAGE_SIZE as usize);

        *ParentTable.add(Index as usize) = ((SubDir as ULONG64) & !0xFFFu64) | ARM64_KERNEL_TABLE_FLAGS;
    } else {
        SubDir = (Entry & !0xFFFu64) as PARM64_PTE;
    }

    SubDir
}

/// Count of page-table allocation failures inside MempMapSinglePage. In a
/// release build ERR() compiles to nothing, so a mapping that quietly fails
/// here surfaces only much later as a fault on an unmapped VA. Reported at the
/// loader/kernel handoff so "some mapping failed" is never invisible again.
pub static mut Arm64MapFailures: u32 = 0;

unsafe fn MempMapSinglePage(VirtualAddress: ULONG64, PhysicalAddress: ULONG64) -> BOOLEAN {
    let Pud: PARM64_PTE;
    let Pmd: PARM64_PTE;
    let Pte: PARM64_PTE;
    let Index: ULONG;

    Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(VirtualAddress));
    if Pud.is_null() {
        Arm64MapFailures += 1;
        return 0u8;
    }

    Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(VirtualAddress));
    if Pmd.is_null() {
        Arm64MapFailures += 1;
        return 0u8;
    }

    Pte = MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(VirtualAddress));
    if Pte.is_null() {
        Arm64MapFailures += 1;
        return 0u8;
    }

    Index = ARM64_PTE_INDEX(VirtualAddress);

    if (*Pte.add(Index as usize) & ARM64_PTE_VALID) != 0 {
        *Pte.add(Index as usize) = (PhysicalAddress & !0xFFFu64) | ARM64_KERNEL_PAGE_FLAGS;
        return 1u8;
    }

    *Pte.add(Index as usize) = (PhysicalAddress & !0xFFFu64) | ARM64_KERNEL_PAGE_FLAGS;

    return 1u8;
}

/// Identity-map one MMIO page with Device-nGnRnE attributes. Port of the C
/// `MempMapDevicePage` (winldr.c) — the 2026-07-05 tripwire commit added it to
/// the by-then-inert C file, so it never shipped until this port. The kernel's
/// early boot (pre-MmMapIoSpace) reaches the PL011 UART, fw_cfg, and — for SMP
/// AP bring-up (M5b) — the GICv2 through these identity mappings; mapping them
/// explicitly here removes the dependence on whatever the UEFI memory-map walk
/// happened to cover (and gives them proper Device attributes instead of
/// accidental Normal-WB).
unsafe fn MempMapDevicePage(PhysicalAddress: ULONG64) -> BOOLEAN {
    let Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(PhysicalAddress));
    if Pud.is_null() {
        return 0u8;
    }

    let Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(PhysicalAddress));
    if Pmd.is_null() {
        return 0u8;
    }

    let Pte = MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(PhysicalAddress));
    if Pte.is_null() {
        return 0u8;
    }

    *Pte.add(ARM64_PTE_INDEX(PhysicalAddress) as usize) =
        (PhysicalAddress & !0xFFFu64) | ARM64_DEVICE_PAGE_FLAGS;

    1u8
}

unsafe fn MempIsPageMapped(VirtualAddress: PVOID) -> BOOLEAN {
    let Pud: PARM64_PTE;
    let Pmd: PARM64_PTE;
    let Pte: PARM64_PTE;
    let Va: ULONG64 = VirtualAddress as ULONG64;

    if (*Arm64Pgd.offset(ARM64_PGD_INDEX(Va) as isize) & ARM64_PTE_VALID) == 0 {
        return 0u8;
    }

    Pud = (*Arm64Pgd.offset(ARM64_PGD_INDEX(Va) as isize) & !0xFFFu64) as PARM64_PTE;
    if (*Pud.offset(ARM64_PUD_INDEX(Va) as isize) & ARM64_PTE_VALID) == 0 {
        return 0u8;
    }

    Pmd = (*Pud.offset(ARM64_PUD_INDEX(Va) as isize) & !0xFFFu64) as PARM64_PTE;
    if (*Pmd.offset(ARM64_PMD_INDEX(Va) as isize) & ARM64_PTE_VALID) == 0 {
        return 0u8;
    }

    Pte = (*Pmd.offset(ARM64_PMD_INDEX(Va) as isize) & !0xFFFu64) as PARM64_PTE;
    if (*Pte.offset(ARM64_PTE_INDEX(Va) as isize) & ARM64_PTE_VALID) == 0 {
        return 0u8;
    }

    1u8
}

unsafe fn MempMapRangeOfPages(mut VirtualAddress: ULONG64, mut PhysicalAddress: ULONG64, cPages: PFN_NUMBER) -> PFN_NUMBER {
    let mut i: PFN_NUMBER = 0;

    while i < cPages {
        if MempMapSinglePage(VirtualAddress, PhysicalAddress) == 0u8 {
            ERR(b"Failed to map page %ld from %p to %p\n\0".as_ptr() as PCHAR,
                i, VirtualAddress as PVOID, PhysicalAddress as PVOID);
            return i;
        }
        VirtualAddress += PAGE_SIZE as ULONG64;
        PhysicalAddress += PAGE_SIZE as ULONG64;
        i += 1;
    }
    return i;
}

#[no_mangle]
pub unsafe extern "C" fn MempSetupPaging(StartPage: PFN_NUMBER, NumberOfPages: PFN_NUMBER, KernelMapping: BOOLEAN) -> BOOLEAN {

    TRACE(b">>> MempSetupPaging(0x%lx, %ld, KernelMapping=%d)\n\0".as_ptr() as PCHAR,
          StartPage, NumberOfPages, KernelMapping as i32);

    if MempMapRangeOfPages(StartPage as ULONG64 * PAGE_SIZE,
                           StartPage as ULONG64 * PAGE_SIZE,
                           NumberOfPages) != NumberOfPages
    {
        ERR(b"Failed to identity map pages %ld, %ld\n\0".as_ptr() as PCHAR, StartPage, NumberOfPages);
        return 0u8;
    }

    if KernelMapping != 0 {
        if MempMapRangeOfPages(StartPage as ULONG64 * PAGE_SIZE + KSEG0_BASE as ULONG64,
                               StartPage as ULONG64 * PAGE_SIZE,
                               NumberOfPages) != NumberOfPages
        {
            ERR(b"Failed to kernel-map pages\n\0".as_ptr() as PCHAR);
            return 0u8;
        }
    }

    return 1u8;
}

unsafe fn MempUnmapPage(Page: PFN_NUMBER) {
    TRACE(b">>> MempUnmapPage(%ld)\n\0".as_ptr() as *const i8, Page);
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrSetProcessorContext(
    _OperatingSystemVersion: USHORT)
{
    let tcr: ULONG64;
    let mair: ULONG64;
    let ttbr_val: ULONG64 = Arm64Pgd as ULONG64;

    ERR(b">>> WinLdrSetProcessorContext for ARM64\n\0".as_ptr() as *const i8);
    ERR(b"  PGD at physical %p\n\0".as_ptr() as *const i8, Arm64Pgd);
    Arm64ReportExceptionLevel(b"[FLDR] ctx-pre");

    core::arch::asm!("msr daifset, #0xF");

    // Firmware may hand off at EL2 (Pi 4B) or EL1 (QEMU virt). Everything below
    // programs the EL1 regime, so get to EL1 first when there is no VHE to
    // redirect those accesses for us; otherwise they configure a regime we are
    // not executing in and the MMU silently keeps using firmware's tables.
    {
        let current_el: ULONG64;
        core::arch::asm!("mrs {}, currentel", out(reg) current_el, options(nomem, nostack));
        if (current_el >> 2) & 3 == 2 {
            let hcr: ULONG64;
            core::arch::asm!("mrs {}, hcr_el2", out(reg) hcr, options(nomem, nostack));
            if hcr & (1 << 34) == 0 {
                Arm64DropToEl1();
                // Before anything else can fault at EL1 without a handler.
                Arm64InstallEl1Vectors();
                Arm64ReportExceptionLevel(b"[FLDR] ctx-el1");
            }
        }
    }

    mair = 0x00000000004400FFu64;
    core::arch::asm!("msr mair_el1, {}", in(reg) mair);

    tcr = (16u64 << 0)  |
          (16u64 << 16) |
          (0u64  << 14) |
          (2u64  << 30) |
          (1u64  << 8)  |
          (1u64  << 10) |
          (3u64  << 12) |
          (1u64  << 24) |
          (1u64  << 26) |
          (3u64  << 28) |
          (2u64  << 32);
    core::arch::asm!("msr tcr_el1, {}", in(reg) tcr);

    core::arch::asm!("msr ttbr0_el1, {}", in(reg) ttbr_val);
    core::arch::asm!("msr ttbr1_el1, {}", in(reg) ttbr_val);

    core::arch::asm!(
        "isb",
        "tlbi vmalle1",
        "dsb sy",
        "isb",
        options(nostack)
    );

    ERR(b"<<< WinLdrSetProcessorContext done: TTBR0=TTBR1=%p TCR=0x%llx MAIR=0x%llx\n\0".as_ptr() as *const i8,
        Arm64Pgd, tcr, mair);
    // EL1 arrives from an EL2 handoff with the MMU off, so the tables above are
    // configured but not yet in use. (When firmware handed off at EL1 the MMU is
    // already on with firmware's tables and this is a no-op - the TTBR swap
    // above is what takes effect there, which is the QEMU path.)
    {
        let mut sctlr: ULONG64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr, options(nomem, nostack));
        if sctlr & 1 == 0 {
            // M | C | I. Our mappings are Normal-WB, so caches on is correct;
            // the loader code and stack are identity-mapped, which is what makes
            // enabling translation mid-function survivable.
            sctlr |= (1 << 0) | (1 << 2) | (1 << 12);
            core::arch::asm!(
                "dsb sy",
                "isb",
                "msr sctlr_el1, {s}",
                "isb",
                s = in(reg) sctlr,
                options(nostack)
            );
            crate::frldr::dbgcon_str(b"[FLDR] el1 mmu enabled\r\n");
        }
    }

    Arm64ReportExceptionLevel(b"[FLDR] ctx-post");
    // Read TTBR1_EL1 back: at EL2 without VHE the register still holds what we
    // wrote, so a matching readback does NOT prove the MMU is using it. The EL
    // reported above is what decides that.
    let ttbr1_readback: ULONG64;
    core::arch::asm!("mrs {}, ttbr1_el1", out(reg) ttbr1_readback, options(nomem, nostack));
    crate::frldr::dbgcon_str(b"[FLDR] ttbr1_el1=0x");
    crate::frldr::dbgcon_hex64(ttbr1_readback);
    crate::frldr::dbgcon_str(b" pgd=0x");
    crate::frldr::dbgcon_hex64(Arm64Pgd as ULONG64);
    crate::frldr::dbgcon_str(b"\r\n");
}

/// GICv2 (distributor, CPU interface) physical bases from the ACPI MADT, or
/// (0, 0) when firmware describes no GICv2. Filled by WinLdrSetupMachineDependent
/// before WinLdrMapSpecialPages runs, and handed to the kernel in the loader block.
/// On a GICv3 the distributor is here with a zero CPU interface, and the
/// redistributors are in Arm64GicRedistributors.
static mut Arm64GicBases: (ULONG64, ULONG64) = (0, 0);
/// GICv3: (base, length) of the redistributor frames that exist, or (0, 0).
static mut Arm64GicRedistributors: (ULONG64, ULONG64) = (0, 0);

/// GICR_TYPER, and its bits that delimit a frame.
const GICR_TYPER: ULONG64 = 0x8;
const GICR_TYPER_VLPIS: u64 = 1 << 1;
const GICR_TYPER_LAST: u64 = 1 << 4;
/// RD_base + SGI_base; a GICv4 with virtual LPIs adds two more 64 KiB pages.
const GICR_FRAME: ULONG64 = 0x2_0000;
/// More than any machine we boot on; it bounds the walk if firmware's range
/// holds no frame marked Last.
const GICR_MAX_FRAMES: u32 = 256;

/// The length of the frames that actually exist in a MADT discovery range.
///
/// The range is routinely larger than its frames - QEMU virt describes room for
/// 123 CPUs - and the space past the last frame need not decode at all, so the
/// kernel must never walk it. Reads through firmware's own mapping: UEFI's GIC
/// driver walked these same TYPERs to find the boot CPU's frame.
unsafe fn WinLdrGicRedistributorLength(Base: ULONG64, Range: ULONG64) -> ULONG64 {
    let mut Offset: ULONG64 = 0;
    let mut Frames = 0u32;
    while Offset + GICR_FRAME <= Range && Frames < GICR_MAX_FRAMES {
        let Typer = core::ptr::read_volatile((Base + Offset + GICR_TYPER) as *const u64);
        Offset += if Typer & GICR_TYPER_VLPIS != 0 { 2 * GICR_FRAME } else { GICR_FRAME };
        Frames += 1;
        if Typer & GICR_TYPER_LAST != 0 {
            break;
        }
    }
    Offset
}

/// QEMU virt's fw_cfg MMIO block, which carries the Phase-0 tripwire channel.
/// It is a QEMU device with no ACPI description, so it cannot be discovered;
/// it is probed ONLY when the firmware's console tables describe QEMU virt's
/// PL011, which is what makes it safe — on a physical board this stays zero and
/// nothing touches it.
const QEMU_VIRT_FW_CFG: ULONG64 = 0x0902_0000;
static mut Arm64FwCfgBase: ULONG64 = 0;

unsafe fn WinLdrMapSpecialPages() -> BOOLEAN {
    let Pud: PARM64_PTE;
    let Pmd: PARM64_PTE;

    if MempMapSinglePage(KI_USER_SHARED_DATA as ULONG64, SharedUserDataPfn as ULONG64 * PAGE_SIZE) == 0 {
        ERR(b"Could not map KI_USER_SHARED_DATA\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(MM_HAL_VA_START));
    Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(MM_HAL_VA_START));
    MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(MM_HAL_VA_START));
    MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(MM_HAL_VA_START + 2 * 1024 * 1024));

    /* Give the KSEG0 window over ALL of physical RAM a complete L0/L1/L2 chain,
     * even where no individual page is mapped.
     *
     * MiInitializePfnDatabase reads the PDE for each page's kernel VA. Reading a
     * PDE only needs the table that CONTAINS it to exist - an absent entry then
     * reads as zero, which the kernel copes with; an absent TABLE faults. On
     * QEMU virt all RAM sits inside one 1GB L1 entry whose L2 table the loader's
     * own allocations already created, so every PDE read lands somewhere valid
     * and this was invisible. The Pi 4B has 3GB spanning three L1 entries while
     * the loader only ever allocates in the first, so PFN init faulted reading
     * the PDE for physical 0xBF700000 (translation fault at level 3).
     *
     * This is deliberately NOT "map all of RAM": the tables cost three pages per
     * GB instead of ~2MB per GB of leaf PTEs, and mapping pages the loader does
     * not own would be a much larger change to what the kernel inherits. */
    let HighestPage = MmHighestPhysicalPage as ULONG64;
    let mut Va = KSEG0_BASE as ULONG64;
    let EndVa = KSEG0_BASE as ULONG64 + HighestPage * PAGE_SIZE as ULONG64;
    while Va < EndVa {
        let TablePud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(Va));
        if TablePud.is_null() {
            ERR(b"Could not create KSEG0 PUD\n\0".as_ptr() as PCHAR);
            break;
        }
        if MempGetOrCreatePageDir(TablePud, ARM64_PUD_INDEX(Va)).is_null() {
            ERR(b"Could not create KSEG0 PMD\n\0".as_ptr() as PCHAR);
            break;
        }
        /* One L2 table covers 1GB of VA, so step by that. */
        Va += 1024 * 1024 * 1024;
    }
    crate::frldr::dbgcon_str(b"[FLDR] kseg0 tables to 0x");
    crate::frldr::dbgcon_hex64(EndVa);
    crate::frldr::dbgcon_str(b"\r\n");

    /* Identity-map the device pages the kernel touches before MmMapIoSpace
     * exists (Device attrs; see MempMapDevicePage):
     *   PL011 UART  — Arm64UartKernelBase early serial
     *   GICv2 GICD  — early-SMP IPI send (KeStartAllProcessors runs BEFORE
     *   GICv2 GICC    MmInitSystem(1); M5b)
     *   fw_cfg      — Phase-0 tripwire channel, QEMU virt only
     * Every address comes from firmware (ACPI), never from a constant: on a
     * physical board a QEMU MMIO address is ordinary RAM, so mapping one as
     * Device and writing to it corrupts memory with no fault to point at it.
     * Non-fatal on failure, mirroring the C: a UP boot without them loses
     * only the affected feature, and the HAL reports IPI loss loudly. */
    if MempMapDevicePage(crate::arm64_debug::uart_base() as u64) == 0u8 {
        ERR(b"Could not map the console UART page\n\0".as_ptr() as PCHAR);
    }
    let (Gicd, Gicc) = Arm64GicBases;
    let (Gicr, GicrLength) = Arm64GicRedistributors;
    if Gicd != 0 && Gicc != 0 {
        if MempMapDevicePage(Gicd) == 0u8 {
            ERR(b"Could not map GICD page (SMP IPIs disabled)\n\0".as_ptr() as PCHAR);
        }
        if MempMapDevicePage(Gicc) == 0u8 {
            ERR(b"Could not map GICC page (SMP IPIs disabled)\n\0".as_ptr() as PCHAR);
        }
    } else if Gicd != 0 && Gicr != 0 {
        /* GICv3: an AP wakes and programs its own redistributor before
         * MmMapIoSpace exists. Each frame's registers live in the first page of
         * each of its 64 KiB pages (RD_base, SGI_base), so map those. */
        let mut Offset: ULONG64 = 0;
        while Offset < GicrLength {
            if MempMapDevicePage(Gicr + Offset) == 0u8 {
                ERR(b"Could not map a GICR page (SMP IPIs disabled)\n\0".as_ptr() as PCHAR);
                break;
            }
            Offset += 0x1_0000;
        }
    } else {
        ERR(b"No GIC in ACPI MADT (SMP IPIs disabled)\n\0".as_ptr() as PCHAR);
    }
    if Arm64FwCfgBase != 0 && MempMapDevicePage(Arm64FwCfgBase) == 0u8 {
        ERR(b"Could not map fw_cfg page (tripwire channel disabled)\n\0".as_ptr() as PCHAR);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrSetupMachineDependent(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let mut SharedUserDataAddress: *mut core::ffi::c_void = core::ptr::null_mut();

    ERR(b">>> WinLdrSetupMachineDependent for ARM64\n\0".as_ptr() as _);

    /* Carry the firmware's own description of this board to the kernel, so no
     * kernel or HAL path has to guess an MMIO address. */
    let Xsdt = crate::arm64_debug::xsdt_base();
    let Uart = crate::arm64_debug::uart_base() as ULONG64;
    match crate::arm64_serial_acpi::gic_from_xsdt(Xsdt) {
        Some(crate::arm64_serial_acpi::Gic::V2 { gicd, gicc }) => Arm64GicBases = (gicd, gicc),
        Some(crate::arm64_serial_acpi::Gic::V3 { gicd, gicr, gicr_length }) => {
            Arm64GicBases = (gicd, 0);
            Arm64GicRedistributors = (gicr, WinLdrGicRedistributorLength(gicr, gicr_length as ULONG64));
        }
        None => {}
    }
    // QEMU virt's fw_cfg is wherever its PL011 is described, whichever UART is the
    // console (a rig may make a 16550 the console and keep QEMU's PL011).
    let QemuVirt = crate::arm64_serial_acpi::qemu_virt_from_xsdt(Xsdt);
    Arm64FwCfgBase = if QemuVirt { QEMU_VIRT_FW_CFG } else { 0 };
    // MachineType carries the console UART's register model on arm64, and whether
    // fw_cfg is there (arm64_serial_acpi::MACHINE_*; arc.h ARM_LOADER_BLOCK).
    (*LoaderBlock).u.Arm.MachineType = crate::arm64_debug::uart_kind()
        | if QemuVirt { crate::arm64_serial_acpi::MACHINE_QEMU_FW_CFG } else { 0 };
    (*LoaderBlock).u.Arm.AcpiTablePhysicalAddress = Xsdt;
    (*LoaderBlock).u.Arm.Arm64UartBase = Uart;
    (*LoaderBlock).u.Arm.Arm64GicDistributorBase = Arm64GicBases.0;
    (*LoaderBlock).u.Arm.Arm64GicCpuInterfaceBase = Arm64GicBases.1;
    (*LoaderBlock).u.Arm.Arm64GicRedistributorBase = Arm64GicRedistributors.0;
    (*LoaderBlock).u.Arm.Arm64GicRedistributorLength = Arm64GicRedistributors.1;
    // dbgcon, not ERR: ERR is `#if DBG` and compiles to nothing in the release
    // build that ships to hardware, which is exactly where this must be legible.
    crate::frldr::dbgcon_str(b"[FLDR] arm64 platform: UART=0x");
    crate::frldr::dbgcon_hex64(Uart);
    crate::frldr::dbgcon_str(b" GICD=0x");
    crate::frldr::dbgcon_hex64(Arm64GicBases.0);
    crate::frldr::dbgcon_str(b" GICC=0x");
    crate::frldr::dbgcon_hex64(Arm64GicBases.1);
    if Arm64GicRedistributors.0 != 0 {
        crate::frldr::dbgcon_str(b" GICR=0x");
        crate::frldr::dbgcon_hex64(Arm64GicRedistributors.0);
        crate::frldr::dbgcon_str(b"+0x");
        crate::frldr::dbgcon_hex64(Arm64GicRedistributors.1);
    }
    crate::frldr::dbgcon_str(b"\r\n");

    SharedUserDataAddress = MmAllocateMemoryWithType(MM_PAGE_SIZE, LoaderStartupPcrPage);
    if SharedUserDataAddress.is_null() {
        ERR(b"Can't allocate SharedUserData page.\n\0".as_ptr() as _);
        return;
    }
    SharedUserDataPfn = (SharedUserDataAddress as ULONG_PTR) >> MM_PAGE_SHIFT;
    RtlZeroMemory(SharedUserDataAddress, MM_PAGE_SIZE);

    ERR(b">>> Allocating page tables...\n\0".as_ptr() as _);

    if MempAllocatePageTables() == 0u8 {
        ERR(b"Failed to allocate page tables\n\0".as_ptr() as _);
        return;
    }

    ERR(b">>> Mapping special pages...\n\0".as_ptr() as _);

    if WinLdrMapSpecialPages() == 0u8 {
        ERR(b"Failed to map special pages\n\0".as_ptr() as _);
        return;
    }

    ERR(b"<<< WinLdrSetupMachineDependent done\n\0".as_ptr() as _);
}

unsafe fn MempDump() {

    let mut i: u32;

    TRACE(b"ARM64 Page Table Dump: PGD at %p\n\0".as_ptr() as *const i8, Arm64Pgd);
    if Arm64Pgd.is_null() {
        return;
    }

    i = 0;
    while i < 512 {
        if (*Arm64Pgd.add(i as usize) & ARM64_PTE_VALID) != 0 {
            TRACE(
                b"  PGD[0x%03x] = %016llx (PA=%016llx)\n\0".as_ptr() as *const i8,
                i,
                *Arm64Pgd.add(i as usize),
                *Arm64Pgd.add(i as usize) & !0xFFFu64,
            );
        }
        i += 1;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Boot Loader
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/ntldr/arch/arm64/winldr.c
//  * PURPOSE:         ARM64 specific Windows Loader routines
//  * PROGRAMMERS:     ReactOS ARM64 Port
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <freeldr.h>
// #include "../../winldr.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// /* GLOBALS ***************************************************************/
//
// /* ARM64 uses 4KB pages with 4-level page tables (like x86-64) */
// #define ARM64_PAGE_SHIFT    12
// #define ARM64_PAGE_SIZE     (1 << ARM64_PAGE_SHIFT)
//
// /* Page table entry indices */
// #define ARM64_PGD_SHIFT     39
// #define ARM64_PUD_SHIFT     30
// #define ARM64_PMD_SHIFT     21
// #define ARM64_PTE_SHIFT     12
//
// #define ARM64_PTRS_PER_LEVEL    512
//
// /* Get index for each level */
// #define ARM64_PGD_INDEX(va) (((va) >> ARM64_PGD_SHIFT) & 0x1FF)
// #define ARM64_PUD_INDEX(va) (((va) >> ARM64_PUD_SHIFT) & 0x1FF)
// #define ARM64_PMD_INDEX(va) (((va) >> ARM64_PMD_SHIFT) & 0x1FF)
// #define ARM64_PTE_INDEX(va) (((va) >> ARM64_PTE_SHIFT) & 0x1FF)
//
// /* Page table entry attributes */
// #define ARM64_PTE_VALID         (1ULL << 0)
// #define ARM64_PTE_TABLE         (1ULL << 1)
// #define ARM64_PTE_PAGE          (1ULL << 1)
// #define ARM64_PTE_AF            (1ULL << 10)    /* Access Flag */
// #define ARM64_PTE_SH_INNER      (3ULL << 8)     /* Inner Shareable */
// #define ARM64_PTE_AP_RW         (0ULL << 6)     /* Read-Write */
// #define ARM64_PTE_AP_RO         (1ULL << 7)     /* Read-Only */
// #define ARM64_PTE_ATTR_NORMAL   (0ULL << 2)     /* Normal memory */
//
// /* Standard page entry for kernel memory */
// #define ARM64_KERNEL_PAGE_FLAGS (ARM64_PTE_VALID | ARM64_PTE_PAGE | ARM64_PTE_AF | \
//                                  ARM64_PTE_SH_INNER | ARM64_PTE_AP_RW | ARM64_PTE_ATTR_NORMAL)
// /* Table descriptors (L0/L1/L2) include AF and SH_INNER even though ARM64 ignores
//  * bits[11:2] for table descriptors. The kernel's recursive self-map reinterprets
//  * table descriptors as L3 page descriptors where AF IS checked — without it, the
//  * hardware raises an access flag fault when walking the self-map. */
// #define ARM64_KERNEL_TABLE_FLAGS (ARM64_PTE_VALID | ARM64_PTE_TABLE | ARM64_PTE_AF | ARM64_PTE_SH_INNER)
//
// typedef ULONG64 ARM64_PTE;
// typedef ARM64_PTE *PARM64_PTE;
//
// /* Level 0 page table (PGD) */
// PARM64_PTE Arm64Pgd;
//
// /* Shared user data page frame number */
// PFN_NUMBER SharedUserDataPfn;
//
// /* FUNCTIONS **************************************************************/
//
// static
// BOOLEAN
// MempAllocatePageTables(VOID)
// {
//     TRACE(">>> MempAllocatePageTables\n");
//
//     /* Allocate a page for the PGD (Level 0) */
//     Arm64Pgd = MmAllocateMemoryWithType(PAGE_SIZE, LoaderMemoryData);
//     if (!Arm64Pgd)
//     {
//         ERR("Failed to allocate ARM64 PGD\n");
//         return FALSE;
//     }
//
//     /* Zero the PGD */
//     RtlZeroMemory(Arm64Pgd, PAGE_SIZE);
//
//     /* Self-mapping: PGD[0x1ED] points back to PGD itself.
//      * This creates the recursive page table mapping at PTE_BASE
//      * (0xFFFFF68000000000), mirroring the amd64 PXE self-map approach.
//      *
//      * IMPORTANT: Use PAGE flags (not just TABLE flags) because the self-map
//      * entry is used as a table descriptor at L0/L1/L2 (where extra bits are
//      * ignored) AND as a page descriptor at L3 (where AF must be set to
//      * avoid an Access Flag fault). */
// #define ARM64_PXE_SELFMAP_INDEX 0x1ED
//     Arm64Pgd[ARM64_PXE_SELFMAP_INDEX] =
//         ((ULONG64)Arm64Pgd & ~0xFFFULL) | ARM64_KERNEL_PAGE_FLAGS;
//
//     TRACE(">>> ARM64 PGD allocated at %p, self-map at index 0x%x\n",
//           Arm64Pgd, ARM64_PXE_SELFMAP_INDEX);
//     return TRUE;
// }
//
// static
// PARM64_PTE
// MempGetOrCreatePageDir(PARM64_PTE ParentTable, ULONG Index)
// {
//     PARM64_PTE SubDir;
//     ARM64_PTE Entry;
//
//     if (!ParentTable)
//         return NULL;
//
//     Entry = ParentTable[Index];
//
//     if (!(Entry & ARM64_PTE_VALID))
//     {
//         /* Allocate a new table */
//         SubDir = MmAllocateMemoryWithType(PAGE_SIZE, LoaderMemoryData);
//         if (!SubDir)
//             return NULL;
//
//         RtlZeroMemory(SubDir, PAGE_SIZE);
//
//         /* Create table entry */
//         ParentTable[Index] = ((ULONG64)SubDir & ~0xFFFULL) | ARM64_KERNEL_TABLE_FLAGS;
//     }
//     else
//     {
//         /* Extract physical address from entry */
//         SubDir = (PARM64_PTE)(Entry & ~0xFFFULL);
//     }
//
//     return SubDir;
// }
//
// static
// BOOLEAN
// MempMapSinglePage(ULONG64 VirtualAddress, ULONG64 PhysicalAddress)
// {
//     PARM64_PTE Pud, Pmd, Pte;
//     ULONG Index;
//
//     /* Walk the page table hierarchy, creating tables as needed */
//     Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(VirtualAddress));
//     if (!Pud)
//         return FALSE;
//
//     Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(VirtualAddress));
//     if (!Pmd)
//         return FALSE;
//
//     Pte = MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(VirtualAddress));
//     if (!Pte)
//         return FALSE;
//
//     Index = ARM64_PTE_INDEX(VirtualAddress);
//
//     /* Check if already mapped — allow overwriting (may happen with
//      * overlapping ranges or identity + kernel dual mapping) */
//     if (Pte[Index] & ARM64_PTE_VALID)
//     {
//         Pte[Index] = (PhysicalAddress & ~0xFFFULL) | ARM64_KERNEL_PAGE_FLAGS;
//         return TRUE;
//     }
//
//     /* Create the page entry */
//     Pte[Index] = (PhysicalAddress & ~0xFFFULL) | ARM64_KERNEL_PAGE_FLAGS;
//
//     return TRUE;
// }
//
// BOOLEAN
// MempIsPageMapped(PVOID VirtualAddress)
// {
//     PARM64_PTE Pud, Pmd, Pte;
//     ULONG64 Va = (ULONG64)VirtualAddress;
//
//     if (!(Arm64Pgd[ARM64_PGD_INDEX(Va)] & ARM64_PTE_VALID))
//         return FALSE;
//
//     Pud = (PARM64_PTE)(Arm64Pgd[ARM64_PGD_INDEX(Va)] & ~0xFFFULL);
//     if (!(Pud[ARM64_PUD_INDEX(Va)] & ARM64_PTE_VALID))
//         return FALSE;
//
//     Pmd = (PARM64_PTE)(Pud[ARM64_PUD_INDEX(Va)] & ~0xFFFULL);
//     if (!(Pmd[ARM64_PMD_INDEX(Va)] & ARM64_PTE_VALID))
//         return FALSE;
//
//     Pte = (PARM64_PTE)(Pmd[ARM64_PMD_INDEX(Va)] & ~0xFFFULL);
//     if (!(Pte[ARM64_PTE_INDEX(Va)] & ARM64_PTE_VALID))
//         return FALSE;
//
//     return TRUE;
// }
//
// static
// PFN_NUMBER
// MempMapRangeOfPages(ULONG64 VirtualAddress, ULONG64 PhysicalAddress, PFN_NUMBER cPages)
// {
//     PFN_NUMBER i;
//
//     for (i = 0; i < cPages; i++)
//     {
//         if (!MempMapSinglePage(VirtualAddress, PhysicalAddress))
//         {
//             ERR("Failed to map page %ld from %p to %p\n",
//                 i, (PVOID)VirtualAddress, (PVOID)PhysicalAddress);
//             return i;
//         }
//         VirtualAddress += PAGE_SIZE;
//         PhysicalAddress += PAGE_SIZE;
//     }
//     return i;
// }
//
// BOOLEAN
// MempSetupPaging(IN PFN_NUMBER StartPage,
//                 IN PFN_NUMBER NumberOfPages,
//                 IN BOOLEAN KernelMapping)
// {
//     TRACE(">>> MempSetupPaging(0x%lx, %ld, KernelMapping=%d)\n",
//           StartPage, NumberOfPages, KernelMapping);
//
//     /* Identity mapping */
//     if (MempMapRangeOfPages(StartPage * PAGE_SIZE,
//                             StartPage * PAGE_SIZE,
//                             NumberOfPages) != NumberOfPages)
//     {
//         ERR("Failed to identity map pages %ld, %ld\n", StartPage, NumberOfPages);
//         return FALSE;
//     }
//
//     /* Kernel mapping: map the same physical pages at KSEG0_BASE offset */
//     if (KernelMapping)
//     {
//         if (MempMapRangeOfPages(StartPage * PAGE_SIZE + KSEG0_BASE,
//                                 StartPage * PAGE_SIZE,
//                                 NumberOfPages) != NumberOfPages)
//         {
//             ERR("Failed to kernel-map pages\n");
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// VOID
// MempUnmapPage(PFN_NUMBER Page)
// {
//     /* TODO: Implement page unmapping */
//     TRACE(">>> MempUnmapPage(%ld)\n", Page);
// }
//
// VOID
// WinLdrSetProcessorContext(
//     _In_ USHORT OperatingSystemVersion)
// {
//     ULONG64 tcr, mair;
//     ULONG64 ttbr_val = (ULONG64)Arm64Pgd;
//
//     ERR(">>> WinLdrSetProcessorContext for ARM64\n");
//     ERR("  PGD at physical %p\n", Arm64Pgd);
//
//     /* Mask all interrupts (IRQ, FIQ, SError, Debug) via DAIF */
//     __asm__ __volatile__("msr daifset, #0xF");
//
//     /* Configure MAIR_EL1: memory attribute indirection register
//      * Attr0 = 0xFF (Normal WB-WA), Attr1 = 0x00 (Device-nGnRnE),
//      * Attr2 = 0x44 (Normal NC) */
//     mair = 0x00000000004400FFULL;
//     __asm__ __volatile__("msr mair_el1, %0" :: "r"(mair));
//
//     /* Configure TCR_EL1: translation control register
//      * T0SZ=16, T1SZ=16 (48-bit VA), TG0=4KB, TG1=4KB,
//      * Inner/Outer WB-WA cacheable, Inner Shareable */
//     tcr = (16ULL << 0)  |  /* T0SZ = 16 */
//           (16ULL << 16) |  /* T1SZ = 16 */
//           (0ULL  << 14) |  /* TG0 = 4KB (0b00) */
//           (2ULL  << 30) |  /* TG1 = 4KB (0b10) */
//           (1ULL  << 8)  |  /* IRGN0 = WB-WA (0b01) */
//           (1ULL  << 10) |  /* ORGN0 = WB-WA (0b01) */
//           (3ULL  << 12) |  /* SH0 = Inner Shareable (0b11) */
//           (1ULL  << 24) |  /* IRGN1 = WB-WA (0b01) */
//           (1ULL  << 26) |  /* ORGN1 = WB-WA (0b01) */
//           (3ULL  << 28) |  /* SH1 = Inner Shareable (0b11) */
//           (2ULL  << 32);   /* IPS = 44-bit PA (0b010) */
//     __asm__ __volatile__("msr tcr_el1, %0" :: "r"(tcr));
//
//     /* Set TTBR0 and TTBR1 to our PGD (same table for both).
//      * TTBR0 handles lower-half VAs (identity mapping at 0x40000000+).
//      * TTBR1 handles upper-half VAs (kernel at KSEG0_BASE 0xFFFFF800+). */
//     __asm__ __volatile__("msr ttbr0_el1, %0" :: "r"(ttbr_val));
//     __asm__ __volatile__("msr ttbr1_el1, %0" :: "r"(ttbr_val));
//
//     /* Full TLB invalidation + barriers */
//     __asm__ __volatile__(
//         "isb\n\t"
//         "tlbi vmalle1\n\t"
//         "dsb sy\n\t"
//         "isb"
//         ::: "memory"
//     );
//
//     ERR("<<< WinLdrSetProcessorContext done: TTBR0=TTBR1=%p TCR=0x%llx MAIR=0x%llx\n",
//           Arm64Pgd, tcr, mair);
// }
//
// static
// BOOLEAN
// WinLdrMapSpecialPages(VOID)
// {
//     PARM64_PTE Pud, Pmd;
//
//     /* Map KI_USER_SHARED_DATA */
//     if (!MempMapSinglePage(KI_USER_SHARED_DATA, SharedUserDataPfn * PAGE_SIZE))
//     {
//         ERR("Could not map KI_USER_SHARED_DATA\n");
//         return FALSE;
//     }
//
//     /* Pre-create page table hierarchy for HAL VA space (4MB at top of address space).
//      * The HAL will fill in actual PTEs later, but the upper levels must exist. */
//     Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(MM_HAL_VA_START));
//     Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(MM_HAL_VA_START));
//     MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(MM_HAL_VA_START));
//     MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(MM_HAL_VA_START + 2 * 1024 * 1024));
//
//     return TRUE;
// }
//
// VOID
// WinLdrSetupMachineDependent(PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PVOID SharedUserDataAddress = NULL;
//
//     ERR(">>> WinLdrSetupMachineDependent for ARM64\n");
//
//     /* Set machine type - ARM64 doesn't use ISA/EISA, just set a placeholder */
//     LoaderBlock->u.Arm.MachineType = 0;
//
//     /* Allocate 1 page for SharedUserData */
//     SharedUserDataAddress = MmAllocateMemoryWithType(MM_PAGE_SIZE, LoaderStartupPcrPage);
//     if (SharedUserDataAddress == NULL)
//     {
//         ERR("Can't allocate SharedUserData page.\n");
//         return;
//     }
//     SharedUserDataPfn = (ULONG_PTR)SharedUserDataAddress >> MM_PAGE_SHIFT;
//     RtlZeroMemory(SharedUserDataAddress, MM_PAGE_SIZE);
//
//     ERR(">>> Allocating page tables...\n");
//
//     /* Allocate page tables */
//     if (MempAllocatePageTables() == FALSE)
//     {
//         ERR("Failed to allocate page tables\n");
//         return;
//     }
//
//     ERR(">>> Mapping special pages...\n");
//
//     /* Map special pages (KI_USER_SHARED_DATA, HAL VA space) */
//     if (!WinLdrMapSpecialPages())
//     {
//         ERR("Failed to map special pages\n");
//         return;
//     }
//
//     ERR("<<< WinLdrSetupMachineDependent done\n");
// }
//
// VOID
// MempDump(VOID)
// {
//     ULONG i;
//
//     TRACE("ARM64 Page Table Dump: PGD at %p\n", Arm64Pgd);
//     if (!Arm64Pgd)
//         return;
//
//     for (i = 0; i < 512; i++)
//     {
//         if (Arm64Pgd[i] & ARM64_PTE_VALID)
//         {
//             TRACE("  PGD[0x%03x] = %016llx (PA=%016llx)\n",
//                   i, Arm64Pgd[i], Arm64Pgd[i] & ~0xFFFULL);
//         }
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
