// PROJECT:     ReactOS HAL (Rust port) — arm64 SMP
// PURPOSE:     arm64 AP bring-up via PSCI CPU_ON (smp-bringup-quest.md M5a).
//
// GREENFIELD: ReactOS never had arm64 SMP; there is no upstream to port. This
// file supplies the HAL half of the arm64 AP-start mechanism the kernel half
// (`ke/arm64/mproc.rs` KeStartAllProcessors) drives, exactly mirroring the
// amd64 contract: KeStartAllProcessors publishes the AP's boot params into the
// shared LoaderBlock (Prcb/Thread/Process/KernelStack) and calls
// HalStartNextProcessor; the AP reuses KiSystemStartup (already CPU-number
// aware — `if Cpu == 0` gates all global init) and zeroes LoaderBlock->Prcb as
// the alive handshake.
//
// The mechanism differs from amd64 (INIT-SIPI-SIPI + <1MB real-mode
// trampoline) because arm64 uses PSCI CPU_ON (a firmware call, full 64-bit
// physical entry, no low-memory constraint) and starts the AP in AArch64 with
// the MMU OFF (PSCI spec). So the trampoline's job is narrow: enable the MMU
// with the BSP's live translation state, install SP/PCR/x0, and branch into
// the high-half kernel VA at KiSystemStartup.
//
// THE ONE SUBTLETY the staged design (arm64-ap/AP-ENTRY-DESIGN.md) under-
// specified: the instruction *immediately after* `msr sctlr_el1` (MMU on) is
// fetched at the trampoline's LOW physical PC, which the kernel's high-half
// TTBR1 does not cover — so without a low mapping the AP takes an instruction
// abort with no vectors installed (silent hang). We therefore build a
// TEMPORARY identity map (TTBR0) covering the trampoline page for PC-continuity
// across the enable, the arm64 analogue of amd64's "identity map is gone"
// temp-PTE. Deterministic; does not gamble on whatever the BSP's post-boot
// TTBR0 happens to hold.
//
// Gating: only compiled for the SMP HAL (`feature = "smp"`, i.e. the HAL that
// links with ntkrnlmp.exe). The UP HAL keeps the FALSE stub in halstubs.rs.

use ntkrnl_sys::ddk;
use core::sync::atomic::{AtomicU32, Ordering};

// ---------------------------------------------------------------------------
// PSCI (ARM DEN0022). QEMU `virt` booted EL1-only (no `virtualization=on`)
// services PSCI via SMC; its default FADT sets PSCI_COMPLIANT=1, USE_HVC=0.
// SMC64 CPU_ON = 0xC400_0003. TODO(M5a-hardening): consume the FADT
// ARM_BOOT_ARCH PSCI_USE_HVC bit instead of assuming SMC (NOTES.md §1).
// ---------------------------------------------------------------------------
const PSCI_CPU_ON_SMC64: u64 = 0xC400_0003;
const PSCI_SUCCESS: i64 = 0;

const TRUE: ddk::BOOLEAN = 1;
const FALSE: ddk::BOOLEAN = 0;

// arm64 PTE/table encodings — verbatim from freeldr (winldr_arm64.rs), so the
// temp idmap uses the same Normal-WB attribute (MAIR Attr0=0xFF at AttrIndx 0)
// the kernel already runs with.
const PTE_VALID: u64 = 1 << 0;
const PTE_TABLE: u64 = 1 << 1; // table descriptor (L0/L1/L2); cleared => block
const PTE_AF: u64 = 1 << 10;
const PTE_SH_INNER: u64 = 3 << 8;
const PTE_AP_RW: u64 = 0 << 6;
const PTE_ATTR_NORMAL: u64 = 0 << 2;
const TABLE_FLAGS: u64 = PTE_VALID | PTE_TABLE | PTE_AF | PTE_SH_INNER;
// 1GB block descriptor at L1 (PUD): bit1=0 (block), Normal-WB, RW, AF, inner-sh.
const BLOCK_FLAGS: u64 = PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_AP_RW | PTE_ATTR_NORMAL;

const PGD_SHIFT: u32 = 39;
const PUD_SHIFT: u32 = 30;
const ONE_GB: u64 = 1 << 30;
const PAGE_SIZE: usize = 0x1000;

#[inline]
fn pgd_index(va: u64) -> usize { ((va >> PGD_SHIFT) & 0x1FF) as usize }
#[inline]
fn pud_index(va: u64) -> usize { ((va >> PUD_SHIFT) & 0x1FF) as usize }

// ---------------------------------------------------------------------------
// The AP trampoline. PC-relative (`adr`) only until the MMU is on, so it runs
// correctly from whatever physical page we copy it to. The 8-quad parameter
// block is part of the blob and is patched in the copy by HalStartNextProcessor.
// ---------------------------------------------------------------------------
core::arch::global_asm!(
    // Plain .text — the target is aarch64 PE-COFF, whose assembler rejects the
    // ELF `%progbits`/section-flag syntax. The blob is copied out at runtime, so
    // its section placement is irrelevant beyond being executable code.
    ".text",
    ".balign 16",
    ".global KiArm64ApTrampolineStart",
    ".global KiArm64ApTrampolineParams",
    ".global KiArm64ApTrampolineEnd",
    "KiArm64ApTrampolineStart:",
    // x0 = PSCI context_id (unused; params are read from the blob). MMU off.
    "    adr   x19, KiArm64ApTrampolineParams", // PC-relative → this copy's params
    "    ldr   x1, [x19, #0]",                   // ttbr0 (temp idmap PA)
    "    msr   ttbr0_el1, x1",
    "    ldr   x1, [x19, #8]",                   // ttbr1 (kernel high map)
    "    msr   ttbr1_el1, x1",
    "    ldr   x1, [x19, #16]",                  // tcr
    "    msr   tcr_el1, x1",
    "    ldr   x1, [x19, #24]",                  // mair
    "    msr   mair_el1, x1",
    "    dsb   sy",
    "    isb",
    "    ldr   x1, [x19, #32]",                  // sctlr with M=1
    "    msr   sctlr_el1, x1",
    "    isb",                                    // MMU on; low PC identity-mapped
    "    ldr   x2, [x19, #40]",                  // sp (kernel VA)
    "    mov   sp, x2",
    "    ldr   x0, [x19, #48]",                  // x0 = LoaderBlock (VA)
    "    ldr   x1, [x19, #56]",                  // entry = KiSystemStartup (VA)
    "    br    x1",                               // into high-half kernel; no return
    "    brk   #0",                               // unreachable guard
    ".balign 8",
    "KiArm64ApTrampolineParams:",
    "    .quad 0",  // +0  ttbr0
    "    .quad 0",  // +8  ttbr1
    "    .quad 0",  // +16 tcr
    "    .quad 0",  // +24 mair
    "    .quad 0",  // +32 sctlr (M=1)
    "    .quad 0",  // +40 sp
    "    .quad 0",  // +48 x0 / LoaderBlock
    "    .quad 0",  // +56 entry / KiSystemStartup
    "KiArm64ApTrampolineEnd:",
);

unsafe extern "C" {
    static KiArm64ApTrampolineStart: u8;
    static KiArm64ApTrampolineParams: u8;
    static KiArm64ApTrampolineEnd: u8;
    fn DbgPrint(Format: *const u8, ...) -> u32;
}
// NOTE: the AP entry (KiSystemStartup) and SP are delivered by the kernel in
// ProcessorState.ContextFrame.Pc/Sp — the HAL does NOT reference the kernel
// symbol directly (that would leave the standalone HAL link with an undefined
// KiSystemStartup). Mirrors the amd64 contract (kernel seeds Rip, HAL reads it).

// Serial diagnostics — bring-up is greenfield; a silent FALSE here otherwise
// surfaces only as "AP never signaled" from the kernel side.
macro_rules! dbgp {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        DbgPrint($fmt.as_ptr() $(, $arg)*)
    };
}

// Parameter-block field offsets (bytes from KiArm64ApTrampolineParams).
const P_TTBR0: usize = 0;
const P_TTBR1: usize = 8;
const P_TCR: usize = 16;
const P_MAIR: usize = 24;
const P_SCTLR: usize = 32;
const P_SP: usize = 40;
const P_X0: usize = 48;
const P_ENTRY: usize = 56;

// Next AP's MPIDR affinity to start. QEMU `virt` flat topology: Aff0 = index,
// higher affinities 0. Starts at 1 (CPU0 = BSP already running). PSCI returns
// INVALID_PARAMETERS for a nonexistent CPU, which KeStartAllProcessors reads as
// "no more CPUs" — so we auto-discover the count without a MADT parser.
static NEXT_MPIDR: AtomicU32 = AtomicU32::new(1);

#[inline(always)]
unsafe fn read_ttbr1() -> u64 { let v: u64; core::arch::asm!("mrs {}, ttbr1_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_tcr() -> u64 { let v: u64; core::arch::asm!("mrs {}, tcr_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_mair() -> u64 { let v: u64; core::arch::asm!("mrs {}, mair_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_sctlr() -> u64 { let v: u64; core::arch::asm!("mrs {}, sctlr_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_ttbr0() -> u64 { let v: u64; core::arch::asm!("mrs {}, ttbr0_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_ctr() -> u64 { let v: u64; core::arch::asm!("mrs {}, ctr_el0", out(reg) v, options(nomem, nostack)); v }

#[inline]
unsafe fn phys_of(va: *mut u8) -> u64 {
    ddk::MmGetPhysicalAddress(va as ddk::PVOID).QuadPart as u64
}

#[inline]
unsafe fn alloc_page() -> *mut u8 {
    // A single zeroed contiguous physical page. HighestAcceptableAddress = all
    // RAM (MmAllocateContiguousMemory clamps to MmHighestPhysicalPage).
    let mut high: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    high.QuadPart = i64::MAX;
    let p = ddk::MmAllocateContiguousMemory(PAGE_SIZE as ddk::SIZE_T, high) as *mut u8;
    if !p.is_null() {
        core::ptr::write_bytes(p, 0, PAGE_SIZE);
    }
    p
}

/// Clean the D-cache to PoC and invalidate the I-cache to PoU over
/// `[va, va+len)`, so an AP fetching this memory by physical address with its
/// MMU/caches off sees the patched instructions and data.
unsafe fn cache_sync(va: *mut u8, len: usize) {
    let ctr = read_ctr();
    let dline = 4usize << ((ctr >> 16) & 0xF); // CTR_EL0.DminLine (words → bytes)
    let iline = 4usize << (ctr & 0xF);          // CTR_EL0.IminLine
    let base = va as usize;
    let mut p = base & !(dline - 1);
    while p < base + len {
        core::arch::asm!("dc cvac, {}", in(reg) p, options(nostack, preserves_flags));
        p += dline;
    }
    core::arch::asm!("dsb ish", options(nostack, preserves_flags));
    let mut p = base & !(iline - 1);
    while p < base + len {
        core::arch::asm!("ic ivau, {}", in(reg) p, options(nostack, preserves_flags));
        p += iline;
    }
    core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));
}

/// Build a temporary identity map (TTBR0) covering `tramp_pa` with a single
/// 1GB Normal-WB block. Returns the L0 (PGD) physical address, or 0 on OOM.
/// FALLBACK ONLY — HalStartNextProcessor currently reuses the BSP's live TTBR0
/// (correct device attributes for the UART). Kept for the case where the BSP's
/// TTBR0 no longer identity-maps at KeStartAllProcessors time; a real use would
/// also need a Device-attr block for the 0x0*000000 MMIO the AP touches early.
#[allow(dead_code)]
unsafe fn build_trampoline_idmap(tramp_pa: u64) -> u64 {
    let l0 = alloc_page();
    let l1 = alloc_page();
    if l0.is_null() || l1.is_null() {
        return 0;
    }
    let l0_pa = phys_of(l0);
    let l1_pa = phys_of(l1);
    // L0[pgd] -> L1 table.
    *(l0 as *mut u64).add(pgd_index(tramp_pa)) = (l1_pa & !0xFFFu64) | TABLE_FLAGS;
    // L1[pud] = 1GB identity block covering tramp_pa.
    *(l1 as *mut u64).add(pud_index(tramp_pa)) = (tramp_pa & !(ONE_GB - 1)) | BLOCK_FLAGS;
    // The AP's table walker reads these by PA; make them visible past our D-cache.
    cache_sync(l0, PAGE_SIZE);
    cache_sync(l1, PAGE_SIZE);
    l0_pa
}

#[inline]
unsafe fn psci_cpu_on(target_mpidr: u64, entry_pa: u64, context_id: u64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "smc #0",
        inout("x0") PSCI_CPU_ON_SMC64 => result,
        in("x1") target_mpidr,
        in("x2") entry_pa,
        in("x3") context_id,
        out("x4") _, out("x5") _, out("x6") _, out("x7") _,
        out("x8") _, out("x9") _, out("x10") _, out("x11") _,
        out("x12") _, out("x13") _, out("x14") _, out("x15") _,
        out("x16") _, out("x17") _,
        options(nostack),
    );
    result
}

// ---------------------------------------------------------------------------
// The real HalStartNextProcessor (SMP HAL). Returns TRUE if PSCI accepted the
// CPU_ON for the next candidate AP; the kernel then waits on the LoaderBlock
// handshake. FALSE means "no more processors / could not start" — the kernel
// stops the enumeration loop.
// ---------------------------------------------------------------------------
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartNextProcessor(
    LoaderBlock: ddk::PLOADER_PARAMETER_BLOCK,
    ProcessorState: ddk::PKPROCESSOR_STATE,
) -> ddk::BOOLEAN {
    if ProcessorState.is_null() {
        return FALSE;
    }
    // Entry (KiSystemStartup VA) + SP are seeded by the kernel's mproc.rs.
    let entry_va = (*ProcessorState).ContextFrame.Pc;
    let sp_va = (*ProcessorState).ContextFrame.Sp;
    let target = NEXT_MPIDR.fetch_add(1, Ordering::SeqCst) as u64;

    let start = core::ptr::addr_of!(KiArm64ApTrampolineStart) as *const u8;
    let end = core::ptr::addr_of!(KiArm64ApTrampolineEnd) as *const u8;
    let params = core::ptr::addr_of!(KiArm64ApTrampolineParams) as *const u8;
    let blob_len = end as usize - start as usize;
    let params_off = params as usize - start as usize;

    // Scratch physical page the AP executes from with the MMU off.
    let tramp = alloc_page();
    if tramp.is_null() {
        dbgp!(b"[SMP] arm64: trampoline page alloc failed\n\0");
        return FALSE;
    }
    let tramp_pa = phys_of(tramp);
    core::ptr::copy_nonoverlapping(start, tramp, blob_len);

    // TTBR0 for PC-continuity across the MMU-enable: reuse the BSP's LIVE TTBR0.
    // At KeStartAllProcessors time (Phase 1) this is the early low identity map,
    // which maps BOTH the trampoline RAM page (Normal) AND the PL011 UART at
    // 0x09000000 (Device) with CORRECT per-region attributes — which early
    // KiSystemStartup writes before it installs its own vectors. A hand-rolled
    // idmap (build_trampoline_idmap, kept below as a fallback) covers the
    // trampoline page but would map the UART as Normal memory (wrong for MMIO),
    // so the live TTBR0 is the correct choice. If a future boot shows the BSP's
    // TTBR0 no longer identity-maps here, switch to a full-attribute idmap.
    let _ = tramp_pa;
    let ttbr0 = read_ttbr0();

    // Patch the copied param block with the AP's boot state.
    let pp = tramp.add(params_off) as *mut u64;
    let sctlr_on = read_sctlr() | 1; // SCTLR_EL1.M = 1
    *pp.add(P_TTBR0 / 8) = ttbr0;
    *pp.add(P_TTBR1 / 8) = read_ttbr1();
    *pp.add(P_TCR / 8) = read_tcr();
    *pp.add(P_MAIR / 8) = read_mair();
    *pp.add(P_SCTLR / 8) = sctlr_on;
    *pp.add(P_SP / 8) = sp_va;
    *pp.add(P_X0 / 8) = LoaderBlock as u64;
    *pp.add(P_ENTRY / 8) = entry_va;

    // Make the copied code + patched params visible to the AP's cold fetch
    // (whole page — the blob is well under 4KB).
    cache_sync(tramp, PAGE_SIZE);

    dbgp!(
        b"[SMP] arm64: PSCI CPU_ON mpidr=%p entry_pa=%p (tramp_va=%p ttbr0=%p entry=%p)\n\0",
        target as ddk::PVOID,
        tramp_pa as ddk::PVOID,
        tramp as ddk::PVOID,
        ttbr0 as ddk::PVOID,
        entry_va as ddk::PVOID
    );

    let r = psci_cpu_on(target, tramp_pa, 0);
    if r == PSCI_SUCCESS {
        TRUE
    } else {
        dbgp!(b"[SMP] arm64: PSCI CPU_ON returned %d (no more CPUs / error)\n\0", r as i32);
        FALSE
    }
}
