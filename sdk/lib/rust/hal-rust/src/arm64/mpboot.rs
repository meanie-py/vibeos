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
// PSCI (ARM DEN0022). The conduit is the firmware's to state, in the ACPI FADT
// ARM_BOOT_ARCH flags: QEMU `virt` (no EL3) says PSCI_USE_HVC; a Raspberry Pi 4
// (TF-A at EL3) says SMC. Using the wrong one is not an error return: `smc` with
// no EL3 is UNDEFINED (it corrupted the BSP on QEMU, ESR EC=0), and `hvc` on a
// board whose EL2 nobody owns jumps into whatever vectors firmware left behind.
// So the conduit, and the set of cores to start, come from ACPI (FADT + MADT
// GICC entries) - never from what QEMU happens to do.
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
const PTE_ATTR_NORMAL: u64 = 0 << 2; // MAIR AttrIndx 0 = 0xFF Normal-WB
const PTE_ATTR_DEVICE: u64 = 1 << 2; // MAIR AttrIndx 1 = 0x00 Device-nGnRnE
const TABLE_FLAGS: u64 = PTE_VALID | PTE_TABLE | PTE_AF | PTE_SH_INNER;
// 1GB block descriptor at L1 (PUD): bit1=0 (block), Normal-WB, RW, AF, inner-sh.
const BLOCK_FLAGS: u64 = PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_AP_RW | PTE_ATTR_NORMAL;
// Device 1GB block (for the debug PL011): no shareability (ignored for Device),
// Device-nGnRnE so the trampoline's post-MMU UART traces reach it. The UART's
// address is the loader's (ACPI DBG2/SPCR), never QEMU's 0x09000000, which is
// plain RAM on a Pi - and these stores run with the MMU and caches off.
const BLOCK_FLAGS_DEVICE: u64 = PTE_VALID | PTE_AF | PTE_AP_RW | PTE_ATTR_DEVICE;

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
    // x9 = the debug PL011's PA, or 0 for no traces. Each trace byte waits for
    // room in the TX FIFO (UARTFR.TXFF, bit 5): a real PL011 drops what does
    // not fit. 'A' (MMU off) proves the AP executed the trampoline at all.
    "    adr   x19, KiArm64ApTrampolineParams", // PC-relative → this copy's params
    "    ldr   x9, [x19, #64]",                  // uart PA (0 = none)
    "    mov   w10, #0x41",                        // 'A'
    "    bl    2f",
    // PSCI firmware starts a core at NS-EL2 whenever EL2 exists (TF-A on a
    // Raspberry Pi 4: SCR_EL3.HCE is set because UEFI itself ran at EL2), while
    // QEMU virt starts it at EL1. The kernel runs at EL1, and at EL2 the EL1
    // MMU programming below would be inert, so drop exactly as the loader does
    // for the boot CPU (freeldr-rust winldr_arm64.rs Arm64DropToEl1). 'E' says
    // this path ran.
    "    mrs   x10, CurrentEL",
    "    cmp   x10, #0x8",                        // EL2 (CurrentEL = EL << 2)
    "    b.ne  5f",
    "    mov   w10, #0x45",                        // 'E'
    "    bl    2f",
    "    mov   x10, #(1 << 31)",                   // HCR_EL2.RW = AArch64 EL1, TGE = 0
    "    msr   hcr_el2, x10",
    "    mov   x10, #3",                           // EL1PCTEN | EL1PCEN
    "    msr   cnthctl_el2, x10",
    "    msr   cntvoff_el2, xzr",
    "    msr   vttbr_el2, xzr",
    "    mrs   x10, midr_el1",
    "    msr   vpidr_el2, x10",
    "    mrs   x10, mpidr_el1",
    "    msr   vmpidr_el2, x10",
    "    mov   x10, #0x33ff",                      // CPTR_EL2 RES1, TFP clear
    "    msr   cptr_el2, x10",
    "    mov   x10, #0x3c5",                       // EL1h, DAIF masked
    "    msr   spsr_el2, x10",
    "    adr   x10, 5f",
    "    msr   elr_el2, x10",
    "    isb",
    "    eret",
    "5:",
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
    // 'B' (MMU on): the UART is now reached through the TTBR0 device block.
    "    mov   w10, #0x42",                        // 'B'
    "    bl    2f",
    "    ldr   x2, [x19, #40]",                  // sp (kernel VA)
    "    mov   sp, x2",
    "    ldr   x0, [x19, #48]",                  // x0 = LoaderBlock (VA)
    "    ldr   x1, [x19, #56]",                  // entry = KiSystemStartup (VA)
    // Dump x1 (the br target) as 16 hex nibbles, so a wrong entry value is
    // visible on serial. Scratch: x10-x15. x0, x1, x9, x19 kept.
    "    mov   x12, #60",
    "1:",
    "    lsr   x13, x1, x12",
    "    and   x13, x13, #0xF",
    "    add   x14, x13, #0x30",                  // '0'+n
    "    add   x15, x13, #0x37",                  // 'A'+(n-10)
    "    cmp   x13, #10",
    "    csel  x10, x14, x15, lt",
    "    bl    2f",
    "    subs  x12, x12, #4",
    "    b.ge  1b",
    "    mov   w10, #0x0A",                        // newline
    "    bl    2f",
    "    mov   w10, #0x43",                        // 'C' (about to branch)
    "    bl    2f",
    "    br    x1",                               // into high-half kernel; no return
    "    brk   #0",                               // unreachable guard
    // putc: w10 -> PL011 at x9 (no-op when x9 == 0). Clobbers w11, x30 only.
    "2:",
    "    cbz   x9, 4f",
    "3:",
    "    ldr   w11, [x9, #0x18]",                 // UARTFR
    "    tbnz  w11, #5, 3b",                      // TXFF: wait for room
    "    str   w10, [x9]",
    "4:",
    "    ret",
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
    "    .quad 0",  // +64 debug PL011 PA (0 = no traces)
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
const P_UART: usize = 64;

// The platform, as the firmware describes it in ACPI: which PSCI conduit to use
// and which cores exist. Read once, on the first HalStartNextProcessor, from the
// XSDT the loader hands over (LoaderBlock->u.Arm.AcpiTablePhysicalAddress).
const MAX_APS: usize = 64;
struct Platform {
    ready: bool,
    hvc: bool,
    /// MPIDR affinity (Aff3..Aff0 as MPIDR_EL1 lays them out) of every enabled
    /// GICC entry except the boot CPU, in MADT order.
    ap_mpidrs: [u64; MAX_APS],
    ap_count: usize,
}
static mut PLATFORM: Platform = Platform { ready: false, hvc: false, ap_mpidrs: [0; MAX_APS], ap_count: 0 };
/// Index into PLATFORM.ap_mpidrs of the next AP to start.
static NEXT_AP: AtomicU32 = AtomicU32::new(0);

const MPIDR_AFFINITY_MASK: u64 = 0xFF_00FF_FFFF;
const FADT_ARM_BOOT_ARCH: usize = 129; // ACPI 5.1+ FADT, u16
const ARM_BOOT_PSCI_COMPLIANT: u16 = 1 << 0;
const ARM_BOOT_PSCI_USE_HVC: u16 = 1 << 1;
const MADT_RECORDS: usize = 44; // header (36) + LocalApicAddress + Flags
const MADT_TYPE_GICC: u8 = 11;
const GICC_FLAGS: usize = 12; // u32, bit 0 = enabled
const GICC_MPIDR: usize = 68; // u64
const GICC_MIN_LENGTH: usize = 76;

use super::acpi::{for_each_table, rd};

/// Fill PLATFORM from the firmware's FADT and MADT. Returns false (and says
/// why) when the firmware did not describe PSCI or any secondary core: this
/// boot then stays uniprocessor instead of guessing a conduit or an MPIDR.
unsafe fn discover_platform(loader_block: ddk::PLOADER_PARAMETER_BLOCK) -> bool {
    let p = &mut *core::ptr::addr_of_mut!(PLATFORM);
    if p.ready {
        return p.ap_count != 0;
    }
    p.ready = true;
    let xsdt_pa = if loader_block.is_null() { 0 } else { (*loader_block).u.Arm.AcpiTablePhysicalAddress };
    if xsdt_pa == 0 {
        dbgp!(b"[SMP] arm64: no ACPI XSDT from the loader - staying uniprocessor\n\0");
        return false;
    }
    let bsp: u64;
    core::arch::asm!("mrs {}, mpidr_el1", out(reg) bsp, options(nomem, nostack));
    let bsp = bsp & MPIDR_AFFINITY_MASK;
    let mut boot_arch: Option<u16> = None;
    let mut enabled = 0usize;
    let walked = for_each_table(xsdt_pa, |sig, t, len| {
        if sig == b"FACP" && len >= FADT_ARM_BOOT_ARCH + 2 {
            boot_arch = Some(rd::<u16>(t, FADT_ARM_BOOT_ARCH));
        } else if sig == b"APIC" {
            let mut off = MADT_RECORDS;
            while off + 2 <= len {
                let ty = rd::<u8>(t, off);
                let rl = rd::<u8>(t, off + 1) as usize;
                if rl < 2 || off + rl > len {
                    break;
                }
                if ty == MADT_TYPE_GICC && rl >= GICC_MIN_LENGTH && rd::<u32>(t, off + GICC_FLAGS) & 1 != 0 {
                    enabled += 1;
                    let mpidr = rd::<u64>(t, off + GICC_MPIDR) & MPIDR_AFFINITY_MASK;
                    if mpidr != bsp && p.ap_count < MAX_APS {
                        p.ap_mpidrs[p.ap_count] = mpidr;
                        p.ap_count += 1;
                    }
                }
                off += rl;
            }
        }
    });
    let flags = boot_arch.unwrap_or(0);
    dbgp!(
        b"[SMP] arm64: ACPI walked=%u ARM_BOOT_ARCH=0x%x enabled cores=%u (APs=%u, BSP mpidr=%p)\n\0",
        walked as u32, flags as u32, enabled as u32, p.ap_count as u32, bsp as ddk::PVOID
    );
    if flags & ARM_BOOT_PSCI_COMPLIANT == 0 {
        dbgp!(b"[SMP] arm64: firmware does not declare PSCI - staying uniprocessor\n\0");
        p.ap_count = 0;
        return false;
    }
    p.hvc = flags & ARM_BOOT_PSCI_USE_HVC != 0;
    dbgp!(b"[SMP] arm64: PSCI conduit = %s\n\0", if p.hvc { b"HVC\0".as_ptr() } else { b"SMC\0".as_ptr() });
    p.ap_count != 0
}

#[inline(always)]
unsafe fn read_ttbr1() -> u64 { let v: u64; core::arch::asm!("mrs {}, ttbr1_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_tcr() -> u64 { let v: u64; core::arch::asm!("mrs {}, tcr_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_mair() -> u64 { let v: u64; core::arch::asm!("mrs {}, mair_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
unsafe fn read_sctlr() -> u64 { let v: u64; core::arch::asm!("mrs {}, sctlr_el1", out(reg) v, options(nomem, nostack)); v }
#[inline(always)]
#[allow(dead_code)] // superseded by build_trampoline_idmap; kept for diagnostics
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

/// Build a temporary identity map (TTBR0) for the AP's MMU-enable window:
/// a Normal-WB 1GB block covering `tramp_pa` (so the low PC + param reads work
/// after `msr sctlr_el1`) plus a Device 1GB block covering the PL011 UART (so
/// the trampoline's post-MMU diagnostic writes reach hardware). Returns the L0
/// (PGD) physical address, or 0 on OOM.
///
/// This is REQUIRED, not a fallback: at KeStartAllProcessors time the BSP's live
/// TTBR0 no longer identity-maps the contiguous scratch page (it lands hundreds
/// of MB into RAM), so reusing it faults the AP's first post-MMU instruction
/// fetch (observed: AP prints 'A' then dies before 'B'). QEMU virt's 1GB RAM
/// keeps every PA < 0x8000_0000, so both blocks share PGD[0] with a single L1.
unsafe fn build_trampoline_idmap(tramp_pa: u64, uart_pa: u64) -> u64 {
    let l0 = alloc_page();
    let l1 = alloc_page();
    if l0.is_null() || l1.is_null() {
        return 0;
    }
    let l0_pa = phys_of(l0);
    let l1_pa = phys_of(l1);
    // L0[pgd] -> L1 table. Both tramp_pa and the UART are < 512GB => PGD[0].
    *(l0 as *mut u64).add(pgd_index(tramp_pa)) = (l1_pa & !0xFFFu64) | TABLE_FLAGS;
    // L1[pud(tramp)] = Normal 1GB identity block covering the trampoline page.
    *(l1 as *mut u64).add(pud_index(tramp_pa)) = (tramp_pa & !(ONE_GB - 1)) | BLOCK_FLAGS;
    // L1[pud(uart)] = Device 1GB identity block covering the PL011. The caller
    // passes 0 unless the UART shares the trampoline's L0 slot and not its 1GB.
    if uart_pa != 0 {
        *(l1 as *mut u64).add(pud_index(uart_pa)) = (uart_pa & !(ONE_GB - 1)) | BLOCK_FLAGS_DEVICE;
    }
    // The AP's table walker reads these by PA; make them visible past our D-cache.
    cache_sync(l0, PAGE_SIZE);
    cache_sync(l1, PAGE_SIZE);
    l0_pa
}

#[inline]
unsafe fn psci_cpu_on(hvc: bool, target_mpidr: u64, entry_pa: u64, context_id: u64) -> i64 {
    let result: i64;
    macro_rules! call {
        ($insn:literal) => {
            core::arch::asm!(
                $insn,
                inout("x0") PSCI_CPU_ON_SMC64 => result,
                inout("x1") target_mpidr => _,
                inout("x2") entry_pa => _,
                inout("x3") context_id => _,
                out("x4") _, out("x5") _, out("x6") _, out("x7") _,
                out("x8") _, out("x9") _, out("x10") _, out("x11") _,
                out("x12") _, out("x13") _, out("x14") _, out("x15") _,
                out("x16") _, out("x17") _,
                options(nostack),
            )
        };
    }
    if hvc {
        call!("hvc #0");
    } else {
        call!("smc #0");
    }
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
    if !discover_platform(LoaderBlock) {
        return FALSE;
    }
    let platform = &*core::ptr::addr_of!(PLATFORM);
    let index = NEXT_AP.fetch_add(1, Ordering::SeqCst) as usize;
    if index >= platform.ap_count {
        return FALSE; // every core the MADT lists has been started
    }
    let target = platform.ap_mpidrs[index];

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

    // TTBR0 for PC-continuity across the MMU-enable. The BSP's LIVE TTBR0 does
    // NOT identity-map the contiguous scratch page here (it lands hundreds of MB
    // into RAM), so the AP faulted its first post-MMU fetch — 'A' printed, 'B'
    // never did. Build a dedicated idmap: Normal block for the trampoline page +
    // Device block for the PL011 UART (so post-MMU traces reach hardware).
    // Trace through the loader's debug UART only when the temp idmap can give
    // it a Device block of its own (same L0 slot, different 1GB).
    let uart = if LoaderBlock.is_null() { 0 } else { (*LoaderBlock).u.Arm.Arm64UartBase };
    let uart = if uart != 0
        && pgd_index(uart) == pgd_index(tramp_pa)
        && pud_index(uart) != pud_index(tramp_pa)
    {
        uart
    } else {
        0
    };
    let ttbr0 = build_trampoline_idmap(tramp_pa, uart);
    if ttbr0 == 0 {
        dbgp!(b"[SMP] arm64: trampoline idmap alloc failed\n\0");
        return FALSE;
    }

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
    *pp.add(P_UART / 8) = uart;

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

    let r = psci_cpu_on(platform.hvc, target, tramp_pa, 0);
    if r == PSCI_SUCCESS {
        // BSP survived the PSCI call and firmware accepted CPU_ON. If this prints but the
        // AP's 'A' never does, the AP was started at the wrong PA / never fetched
        // our trampoline; if this DOESN'T print, the SMC itself wedged the BSP.
        dbgp!(b"[SMP] arm64: BSP post-PSCI SUCCESS, AP should be launching\n\0");
        TRUE
    } else {
        dbgp!(b"[SMP] arm64: PSCI CPU_ON returned %d (no more CPUs / error)\n\0", r as i32);
        FALSE
    }
}
