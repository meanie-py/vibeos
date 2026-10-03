//! SMP processor support — real APIC implementations of the HAL
//! multiprocessor entry points, replacing the UP stubs in
//! `hal/halx86/generic/up.c` (translated to `x86/up.rs`).
//!
//! Provides:
//!   * `HalRequestIpi(KAFFINITY)`      — fixed-delivery IPI to a CPU set via LAPIC ICR.
//!   * `HalStartNextProcessor(...)`    — wake the next AP with INIT/SIPI/SIPI.
//!   * `ApicStartApplicationProcessor` — the raw INIT-SIPI-SIPI helper (smp.h).
//!   * `HalpSetupProcessorsTable`      — per-CPU PROCESSOR_IDENTITY bookkeeping.
//!   * `HalpBroadcastClockIpi`         — CLOCK_IPI to all-but-self.
//!   * `HalAllProcessorsStarted`       — MP-aware "did every AP come up?" query.
//!
//! ============================================================================
//! WIRING (integrator — DO NOT let the translator edit these):
//! ============================================================================
//!   1. Add to `sdk/lib/rust/hal-rust/src/x86/mod.rs`:
//!          #[cfg(target_arch = "x86_64")]      // see NOTES.md re i386 APIC HAL
//!          pub mod smp_processor;
//!          #[cfg(target_arch = "x86_64")]
//!          mod trampoline;                     // sibling file, used by smp_processor
//!      (or `pub(crate) mod trampoline;` — smp_processor references it).
//!
//!   2. This file DEFINES the symbols `HalRequestIpi`, `HalStartNextProcessor`,
//!      `HalpSetupProcessorsTable`, `HalpBroadcastClockIpi` — which `up.rs`
//!      ALSO defines. For an MP build you must remove those four `#[no_mangle]`
//!      fns from `up.rs` (or `#[cfg(not(feature="smp"))]`-gate them there) so
//!      the linker sees exactly one definition. `HalAllProcessorsStarted` lives
//!      in `processor.rs` today (returns TRUE) — gate/replace it the same way.
//!      Mirror the C model: `up.c` and `apic/apicsmp.c` are never linked into
//!      the same HAL flavor.
//!
//!   3. Cohab / cmake: this is a NEW Rust module with no 1:1 C source in the
//!      current tree (mainline ReactOS keeps it in `hal/halx86/apic/apicsmp.c`,
//!      which this fork never imported). No `# replaced by Rust` cmake line is
//!      needed — nothing to comment out. It links via the normal libhal_rust.a
//!      path once `mod smp_processor;` is added.
//!
//!   4. Feature gate: hal-rust has NO `smp` feature yet (see Cargo.toml). Add
//!      one and gate the MP paths, OR ship unconditionally — the code degrades
//!      safely on UP (HalRequestIpi to an empty/self-only set is a no-op,
//!      HalStartNextProcessor returns FALSE once no AP remains). See NOTES.md.
//!
//!   5. Trampoline low page: HalStartNextProcessor needs a <1MB, identity-mapped
//!      physical page for the real-mode SIPI target. The integrator must reserve
//!      it (see `trampoline.rs` header + DESIGN.md §Trampoline page). We consume
//!      it via the externs `HalpLowStub` / `HalpLowStubPhysicalAddress`.
//! ============================================================================
//!
//! Register-access style is copied from `apic.rs` (read_register_ulong /
//! apic_read / apic_write / cli / sti / read_msr). Those helpers are private
//! there, so they are re-declared here rather than imported.

#![allow(non_snake_case)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, KAFFINITY, KIRQL, PKPROCESSOR_STATE, PLOADER_PARAMETER_BLOCK, UCHAR, ULONG,
};

use core::sync::atomic::{AtomicU32, Ordering};

// The trampoline byte blobs + parameter-block layout live in the sibling
// `trampoline` module (deliverable #2).
use crate::x86::trampoline;

// ---------------------------------------------------------------------------
// LAPIC base + register offsets (apicp.h). amd64 uses the HAL's fixed
// high-canonical MMIO window; i386 uses the low identity window. On amd64 this
// is byte-identical to the private constants in apic.rs.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
const APIC_BASE: usize = 0xFFFFFFFFFFFE0000u64 as usize;
#[cfg(target_arch = "x86")]
const APIC_BASE: usize = 0xFFFE0000usize;

// APIC_REGISTER offsets (apicp.h).
const APIC_ID: ULONG = 0x0020;
const APIC_ICR0: ULONG = 0x0300; // ICR low  dword (write here SENDS)
const APIC_ICR1: ULONG = 0x0310; // ICR high dword (destination)

// APIC_MT (message / delivery type) — apicp.h _APIC_MT.
const APIC_MT_FIXED: u64 = 0;
const APIC_MT_NMI: u64 = 4;
const APIC_MT_INIT: u64 = 5;
const APIC_MT_STARTUP: u64 = 6;

// Trigger mode / level / destination-mode / shorthand bit-field VALUES.
const APIC_TGM_EDGE: u64 = 0;
const APIC_TGM_LEVEL: u64 = 1;
const APIC_DM_PHYSICAL: u64 = 0;
const APIC_DM_LOGICAL: u64 = 1;
const APIC_LEVEL_DEASSERT: u64 = 0;
const APIC_LEVEL_ASSERT: u64 = 1;
const APIC_DSH_DESTINATION: u64 = 0;
const APIC_DSH_ALL_EXCL_SELF: u64 = 3;

// ICR bit positions (APIC_INTERRUPT_COMMAND_REGISTER, apicp.h).
const ICR_VECTOR_SHIFT: u64 = 0;
const ICR_MT_SHIFT: u64 = 8;
const ICR_DM_SHIFT: u64 = 11;
const ICR_DELIVERY_STATUS_BIT: u64 = 1 << 12;
const ICR_LEVEL_SHIFT: u64 = 14;
const ICR_TGM_SHIFT: u64 = 15;
const ICR_DSH_SHIFT: u64 = 18;
const ICR_DEST_SHIFT: u64 = 56; // physical/logical APIC-ID destination

// Kernel IPI vector — apicp.h APIC_IPI_VECTOR, IRQL 14 (IPI_LEVEL).
// The amd64 IDT wires 0xE1 -> KiIpiInterrupt (ke/amd64/except.c:46). Same
// numeric value on i386 (apicp.h i386 branch).
const APIC_IPI_VECTOR: u64 = 0xE1;

// apicp.h CLOCK_IPI_VECTOR, IRQL 13 (CLOCK_LEVEL) -> HalpClockIpi.
const CLOCK_IPI_VECTOR: u64 = 0xD2;

// ---------------------------------------------------------------------------
// PROCESSOR_IDENTITY / HALP_APIC_INFO_TABLE — filled by madt.rs. We only read
// LapicId + ProcessorCount and flip ProcessorStarted. Types mirror
// hal/halx86/include/smp.h (see madt.rs for the canonical Rust definition).
// ---------------------------------------------------------------------------

// Same crate: use madt.rs's canonical definitions and halacpi.rs's low-stub
// statics directly (ACPI Phase-0 allocates + maps the 1 low page —
// halacpi.rs:1051). No externs needed.
use crate::x86::halacpi::{
    HalpApPtScratch, HalpApPtScratchPhysicalAddress, HalpLowStub, HalpLowStubPhysicalAddress,
};
use crate::x86::madt::{HalpApicInfoTable, HalpProcessorIdentity};

// Serial diagnostics for the FALSE paths — a silent FALSE here surfaces as a
// bare HAL1_INITIALIZATION_FAILED bugcheck with no clue which guard tripped.
unsafe extern "C" {
    fn DbgPrint(Format: ntkrnl_sys::hal_ddk::PCSTR, ...) -> ULONG;
}

// ---------------------------------------------------------------------------
// MMIO + intrinsics — mirror apic.rs (private there).
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn read_register_ulong(addr: usize) -> ULONG {
    core::ptr::read_volatile(addr as *const ULONG)
}
#[inline(always)]
unsafe fn write_register_ulong(addr: usize, value: ULONG) {
    core::ptr::write_volatile(addr as *mut ULONG, value);
}
#[inline(always)]
unsafe fn apic_read(reg: ULONG) -> ULONG {
    read_register_ulong(APIC_BASE + reg as usize)
}
#[inline(always)]
unsafe fn apic_write(reg: ULONG, value: ULONG) {
    write_register_ulong(APIC_BASE + reg as usize, value);
}

#[inline(always)]
unsafe fn cli() {
    core::arch::asm!("cli", options(nomem, nostack));
}
#[inline(always)]
unsafe fn sti() {
    core::arch::asm!("sti", options(nomem, nostack));
}
#[inline(always)]
unsafe fn yield_processor() {
    core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
}

// EFLAGS.IF probe so we can restore the interrupt state around an ICR write.
#[inline(always)]
unsafe fn read_eflags() -> usize {
    let f: usize;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("pushfq", "pop {0}", out(reg) f, options(nomem, preserves_flags));
    #[cfg(target_arch = "x86")]
    core::arch::asm!("pushfd", "pop {0}", out(reg) f, options(nomem, preserves_flags));
    f
}
const EFLAGS_IF: usize = 1 << 9;

// ---------------------------------------------------------------------------
// Local APIC ID of the CURRENT processor (top 8 bits of the APIC_ID reg on
// xAPIC). Used to skip self and to pick the "next" AP.
// ---------------------------------------------------------------------------
#[inline(always)]
unsafe fn current_lapic_id() -> u8 {
    (apic_read(APIC_ID) >> 24) as u8
}

// ---------------------------------------------------------------------------
// Core ICR write. Waits for the previous IPI to drain (DeliveryStatus==0),
// programs the destination (ICR1) then the command (ICR0, which sends).
// Interrupts are masked across the two writes so nothing sneaks in between the
// dword pair. `icr` is the full 64-bit APIC_INTERRUPT_COMMAND_REGISTER value.
// ---------------------------------------------------------------------------
#[inline]
unsafe fn apic_write_icr(icr: u64) {
    let flags = read_eflags();
    cli();

    // Wait for idle before touching the ICR.
    while (apic_read(APIC_ICR0) as u64 & ICR_DELIVERY_STATUS_BIT) != 0 {
        yield_processor();
    }

    apic_write(APIC_ICR1, (icr >> 32) as ULONG); // destination first
    apic_write(APIC_ICR0, icr as ULONG); // low dword sends

    if (flags & EFLAGS_IF) != 0 {
        sti();
    }
}

// Wait until the last-sent IPI has been accepted (DeliveryStatus clears).
#[inline]
unsafe fn apic_wait_delivery() {
    while (apic_read(APIC_ICR0) as u64 & ICR_DELIVERY_STATUS_BIT) != 0 {
        yield_processor();
    }
}

// ---------------------------------------------------------------------------
// Crude busy-wait delays for the INIT/SIPI timing. The Intel MP spec wants
// ~10ms after INIT and ~200us between SIPIs. We don't have a calibrated stall
// here (KeStallExecutionProcessor may not be usable this early / this is HAL),
// so use a PAUSE spin loop sized from an assumed <=5GHz core. This is a FLOOR,
// generous is fine — over-waiting only slows bring-up. See NOTES.md (owed:
// switch to HalpStallScaleFactor / KeStallExecutionProcessor once validated).
// ---------------------------------------------------------------------------
#[inline(never)]
unsafe fn crude_stall(iterations: u64) {
    let mut i = 0u64;
    while i < iterations {
        yield_processor();
        i += 1;
    }
}
const STALL_10MS: u64 = 5_000_000; // ~10ms floor spin
const STALL_200US: u64 = 100_000; // ~200us floor spin

// ===========================================================================
// HalRequestIpi — send the kernel IPI vector (0xE1) to a set of processors.
// ===========================================================================
//
// TargetProcessors is a KAFFINITY bitmask of NT processor NUMBERS (bit i == CPU
// i). We deliver via LOGICAL FLAT destination, which is exactly the addressing
// mode apic.rs already configured every LAPIC for:
//     ApicInitializeLocalApic: DFR = APIC_DF_FLAT (0xFFFFFFFF)
//                              LDR = (1 << Cpu) << 24        (ApicLogicalId)
// so the low 8 bits of the KAFFINITY map 1:1 onto the flat logical-ID bitmask
// and one ICR write reaches the whole set (up to 8 CPUs — the flat-model max).
// This is only valid while APIC ID / logical ID == processor index; see the
// ASSUMPTION note in DESIGN.md and the >8-CPU caveat in NOTES.md.
//
// UP-safe degrade: an empty target set is a no-op (matches "nothing to send").
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalRequestIpi(TargetProcessors: KAFFINITY) {
    let mask = (TargetProcessors as u64) & 0xFF;
    if mask == 0 {
        return; // no target — no-op (UP or self-excluded empty set)
    }

    let icr: u64 = (APIC_IPI_VECTOR << ICR_VECTOR_SHIFT)
        | (APIC_MT_FIXED << ICR_MT_SHIFT)
        | (APIC_DM_LOGICAL << ICR_DM_SHIFT)
        | (APIC_LEVEL_ASSERT << ICR_LEVEL_SHIFT)
        | (APIC_TGM_EDGE << ICR_TGM_SHIFT)
        | (APIC_DSH_DESTINATION << ICR_DSH_SHIFT)
        | (mask << ICR_DEST_SHIFT);

    apic_write_icr(icr);
}

// ===========================================================================
// HalpRequestIpi / HalRequestIpiSpecifyVector — smp.h helpers a full IPI stack
// wants. Provided for completeness; both funnel through the same ICR path.
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalRequestIpiSpecifyVector(TargetSet: KAFFINITY, Vector: UCHAR) {
    let mask = (TargetSet as u64) & 0xFF;
    if mask == 0 {
        return;
    }
    let icr: u64 = ((Vector as u64) << ICR_VECTOR_SHIFT)
        | (APIC_MT_FIXED << ICR_MT_SHIFT)
        | (APIC_DM_LOGICAL << ICR_DM_SHIFT)
        | (APIC_LEVEL_ASSERT << ICR_LEVEL_SHIFT)
        | (APIC_TGM_EDGE << ICR_TGM_SHIFT)
        | (mask << ICR_DEST_SHIFT);
    apic_write_icr(icr);
}

// ===========================================================================
// HalpBroadcastClockIpi — send the clock IPI to every OTHER processor. FASTCALL
// (matches up.rs / rtctimer's caller). Uses the AllExcludingSelf shorthand so
// we don't need to know the target set.
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpBroadcastClockIpi(Vector: UCHAR) {
    let icr: u64 = ((Vector as u64) << ICR_VECTOR_SHIFT)
        | (APIC_MT_FIXED << ICR_MT_SHIFT)
        | (APIC_DM_PHYSICAL << ICR_DM_SHIFT)
        | (APIC_LEVEL_ASSERT << ICR_LEVEL_SHIFT)
        | (APIC_TGM_EDGE << ICR_TGM_SHIFT)
        | (APIC_DSH_ALL_EXCL_SELF << ICR_DSH_SHIFT);
    let _ = CLOCK_IPI_VECTOR; // named for reference; caller passes the vector
    apic_write_icr(icr);
}

// ===========================================================================
// ApicStartApplicationProcessor — the raw INIT / SIPI / SIPI sequence (smp.h
// signature). NTProcessorNumber selects the target LAPIC via
// HalpProcessorIdentity[]; StartupLoc is the PHYSICAL address of the trampoline
// page (must be page-aligned and < 1MB so StartupLoc>>12 fits the SIPI vector).
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ApicStartApplicationProcessor(
    NTProcessorNumber: ULONG,
    StartupLoc: u64, // PHYSICAL_ADDRESS in C; low 32 bits are all that matter
) {
    let ident = (*(&raw const HalpProcessorIdentity)).add(NTProcessorNumber as usize);
    let lapic_id = (*ident).LapicId as u64;

    // SIPI vector = trampoline physical page number (phys >> 12). Real-mode AP
    // begins at CS = vector<<8, IP = 0  ->  linear vector<<12 == StartupLoc.
    let sipi_vector = (StartupLoc >> 12) & 0xFF;

    let dest = lapic_id << ICR_DEST_SHIFT;

    // ---- 1. INIT assert (level-triggered, assert) ----
    let init_assert: u64 = (APIC_MT_INIT << ICR_MT_SHIFT)
        | (APIC_DM_PHYSICAL << ICR_DM_SHIFT)
        | (APIC_LEVEL_ASSERT << ICR_LEVEL_SHIFT)
        | (APIC_TGM_LEVEL << ICR_TGM_SHIFT)
        | (APIC_DSH_DESTINATION << ICR_DSH_SHIFT)
        | dest;
    apic_write_icr(init_assert);
    apic_wait_delivery();

    // ---- 2. INIT deassert (level-triggered, deassert) ----
    // Classic sequence; harmless / recommended on QEMU. Some modern CPUs ignore
    // the deassert, but sending it matches the MP spec and real ReactOS.
    let init_deassert: u64 = (APIC_MT_INIT << ICR_MT_SHIFT)
        | (APIC_DM_PHYSICAL << ICR_DM_SHIFT)
        | (APIC_LEVEL_DEASSERT << ICR_LEVEL_SHIFT)
        | (APIC_TGM_LEVEL << ICR_TGM_SHIFT)
        | (APIC_DSH_DESTINATION << ICR_DSH_SHIFT)
        | dest;
    apic_write_icr(init_deassert);
    apic_wait_delivery();

    // ~10ms after INIT before the first SIPI.
    crude_stall(STALL_10MS);

    // ---- 3. SIPI x2 (edge-triggered startup) ----
    // A discrete 82489DX local APIC (version < 0x10) has no STARTUP IPI (MP Spec B.4,
    // upstream e34b04c3037); send none rather than a reserved message type.
    if (apic_read(0x0030 /* APIC_VER */) & 0xFF) < 0x10 {
        return;
    }
    let sipi: u64 = (sipi_vector << ICR_VECTOR_SHIFT)
        | (APIC_MT_STARTUP << ICR_MT_SHIFT)
        | (APIC_DM_PHYSICAL << ICR_DM_SHIFT)
        | (APIC_LEVEL_ASSERT << ICR_LEVEL_SHIFT)
        | (APIC_TGM_EDGE << ICR_TGM_SHIFT)
        | (APIC_DSH_DESTINATION << ICR_DSH_SHIFT)
        | dest;

    apic_write_icr(sipi);
    apic_wait_delivery();
    crude_stall(STALL_200US);

    apic_write_icr(sipi);
    apic_wait_delivery();
    crude_stall(STALL_200US);
}

// ===========================================================================
// HalStartNextProcessor — wake the next not-yet-started AP.
//
// KPROCESSOR_STATE contract (from ke/i386/mproc.c KeStartAllProcessors):
//   The AP must resume execution in PROTECTED mode (i386) / LONG mode (amd64)
//   with:
//     SpecialRegisters.Cr0/Cr3/Cr4       loaded (Cr0 already has PE|PG set)
//     SpecialRegisters.Gdtr / Idtr       loaded (per-AP GDT/IDT copies)
//     SpecialRegisters.Tr                = KGDT_TSS  (AP's TSS selector)
//     ContextFrame.SegCs/Ds/Es/Ss/Fs     loaded (Fs = KGDT_R0_PCR)
//     ContextFrame.Esp/Eip (amd64: Rsp/Rip)  -> stack + KiSystemStartup
//     ContextFrame.EFlags                (IF cleared)
//   The kernel stack already carries AP_SETUP_STACK { ReturnAddr, KxLoaderBlock }
//   at [Esp], so KiSystemStartup receives LoaderBlock as its first arg with no
//   extra push from us.
//
// The trampoline (trampoline.rs) is the piece that takes the AP from the SIPI
// real-mode entry to that state and jumps to Eip/Rip. HalStartNextProcessor
// copies the trampoline blob into the low stub page, patches the blob's
// parameter block from *ProcessorState, then fires INIT/SIPI/SIPI.
//
// Returns FALSE when there is no further AP to start (UP-safe: on a 1-CPU box
// ProcessorCount==1 so the very first call returns FALSE, mirroring up.rs).
// ===========================================================================
static NEXT_PROCESSOR: AtomicU32 = AtomicU32::new(1); // 0 == BSP, already up

// ---------------------------------------------------------------------------
// identity_map_stub — make VA==stub_pa valid in the current (System) address
// space via the recursive self-map. For a <1MB VA every table index but the
// PT's is 0, so on amd64 the chain is PML4[0] -> PDPT[0] -> PD[0] ->
// PT[stub_pa>>12] (pages 1-3 of the scratch run become the PDPT/PD/PT) and on
// i386 (non-PAE 2-level) it collapses to PD[0] -> PT[stub_pa>>12] (one scratch
// page is the PT). Idempotent: a second call sees the chain valid and just
// revalidates the PTE. Fails (false) only if a DIFFERENT low mapping already
// occupies the chain — never silently reuses foreign tables with a mismatched
// PA. The AP trampoline enables paging on this same kernel CR3, so the stub
// page must translate VA==PA for the instruction window around the CR0 write.
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86_64")]
const PTE_BASE: u64 = 0xFFFF_F680_0000_0000;
#[cfg(target_arch = "x86_64")]
const PDE_BASE: u64 = 0xFFFF_F6FB_4000_0000;
#[cfg(target_arch = "x86_64")]
const PPE_BASE: u64 = 0xFFFF_F6FB_7DA0_0000;
#[cfg(target_arch = "x86_64")]
const PXE_BASE: u64 = 0xFFFF_F6FB_7DBE_D000;
#[cfg(target_arch = "x86_64")]
const PTE_VALID_WRITE: u64 = 0x3; // P | RW (kernel, not global, not NX)

#[cfg(target_arch = "x86_64")]
unsafe fn identity_map_stub(_stub_va: *mut u8, stub_pa: u64) -> bool {
    // 3 contiguous scratch pages (PDPT/PD/PT), allocated <4GB at Phase 0
    // (halacpi.rs HalpApPtScratch) — the <1MB constraint only binds the SIPI
    // target page itself, and UEFI leaves no 4-page low run to share.
    let scratch_pa = (&raw const HalpApPtScratchPhysicalAddress).read().QuadPart as u64;
    let scratch_va = (&raw const HalpApPtScratch).read() as *mut u8;
    if scratch_pa == 0 || scratch_va.is_null() {
        return false; // Phase 0 couldn't allocate page-table scratch
    }
    let pdpt_pa = scratch_pa;
    let pd_pa = scratch_pa + 0x1000;
    let pt_pa = scratch_pa + 0x2000;

    // Self-map entry VAs for VA = stub_pa (all upper indices are 0).
    let pxe = PXE_BASE as *mut u64;
    let ppe = PPE_BASE as *mut u64;
    let pde = PDE_BASE as *mut u64;
    let pte = (PTE_BASE + (stub_pa >> 12) * 8) as *mut u64;

    // Walk top-down; install a scratch table at each absent level. If a level
    // is already valid we keep the existing table (some other low mapping) —
    // the final PTE write is still ours, but a valid PTE pointing at a
    // DIFFERENT frame means someone owns this VA: back off loudly.
    let mut level_pairs = [
        (pxe, pdpt_pa, 0usize),
        (ppe, pd_pa, 0x1000usize),
        (pde, pt_pa, 0x2000usize),
    ];
    for (entry, table_pa, va_off) in level_pairs.iter_mut() {
        if entry.read_volatile() & 1 == 0 {
            // Zero the scratch table through its BSP-visible mapping, then
            // plug it in. The self-map VA for the next level becomes valid
            // as soon as this entry is written (TLB: fresh entry, no stale).
            core::ptr::write_bytes(scratch_va.add(*va_off), 0, 0x1000);
            entry.write_volatile(*table_pa | PTE_VALID_WRITE);
        }
    }

    let want = stub_pa | PTE_VALID_WRITE;
    let cur = pte.read_volatile();
    // Ignore hardware-stamped Accessed/Dirty (bits 5/6) in the ours-vs-foreign
    // compare: AP1 executing through this mapping sets A (and D on its stack
    // pokes), and the exact-match check then called OUR OWN page "foreign" —
    // HalStartNextProcessor returned FALSE for every AP after the first
    // (the -smp 4 BUGCHECK 0x61 cap, quest log 2026-07-12).
    if cur & 1 != 0 && (cur & !0x60u64) != want {
        return false; // foreign mapping at our VA — refuse to clobber
    }
    pte.write_volatile(want);
    core::arch::asm!("invlpg [{}]", in(reg) stub_pa, options(nostack, preserves_flags));
    true
}

// i386 (non-PAE, 2-level) self-map: PTE_BASE=0xC0000000, PDE self-map at
// 0xC0300000 (== PTE_BASE + (PTE_BASE>>12)*4), 4-byte PTEs — same constants
// the HAL's memory.rs uses. For a <1MB stub the PD index (stub_pa>>22) is 0,
// so the whole chain is just PD[0] -> scratch PT -> PT[stub_pa>>12]. One
// scratch page (the PT for VA 0-4MB) suffices; we reuse page 0 of the same
// HalpApPtScratch run the amd64 path allocates 3 of.
#[cfg(target_arch = "x86")]
unsafe fn identity_map_stub(_stub_va: *mut u8, stub_pa: u64) -> bool {
    const PTE_BASE_I386: u32 = 0xC000_0000;
    const PDE_BASE_I386: u32 = 0xC030_0000; // PTE_BASE + (PTE_BASE>>12)*4
    const PTE_VALID_WRITE_I386: u32 = 0x3; // P | RW (kernel, not global)

    let scratch_pa = (&raw const HalpApPtScratchPhysicalAddress).read().QuadPart as u32;
    let scratch_va = (&raw const HalpApPtScratch).read() as *mut u8;
    if scratch_pa == 0 || scratch_va.is_null() {
        return false; // Phase 0 couldn't allocate page-table scratch
    }

    let stub_pa32 = stub_pa as u32; // guaranteed < 1MB by the caller
    // PD index is 0 for a <4MB VA, so the PDE is exactly PDE_BASE[0].
    let pde0 = PDE_BASE_I386 as *mut u32;
    // Self-map PTE slot for VA == stub_pa32 (reachable once PD[0] is valid).
    let pte = (PTE_BASE_I386 + (stub_pa32 >> 12) * 4) as *mut u32;

    if pde0.read_volatile() & 1 == 0 {
        // Zero the scratch PT through its BSP mapping, then plug it in as the
        // page table for VA 0-4MB. The PTE self-map slot above becomes valid
        // as soon as this write lands.
        core::ptr::write_bytes(scratch_va, 0, 0x1000);
        pde0.write_volatile(scratch_pa | PTE_VALID_WRITE_I386);
    }

    let want = stub_pa32 | PTE_VALID_WRITE_I386;
    let cur = pte.read_volatile();
    // Ignore hardware-stamped Accessed/Dirty (bits 5/6) in the ours-vs-foreign
    // compare (same rationale as the amd64 arm — AP1 running through this
    // mapping sets A/D and must not read as "foreign" on the next AP start).
    if cur & 1 != 0 && (cur & !0x60u32) != want {
        return false; // foreign mapping at our VA — refuse to clobber
    }
    pte.write_volatile(want);
    core::arch::asm!("invlpg [{}]", in(reg) stub_pa32, options(nostack, preserves_flags));
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartNextProcessor(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
    ProcessorState: PKPROCESSOR_STATE,
) -> BOOLEAN {
    let next = NEXT_PROCESSOR.load(Ordering::Relaxed);
    let count = (&raw const HalpApicInfoTable.ProcessorCount).read_volatile();

    // No more APs enumerated in the MADT -> nothing to start.
    if next >= count {
        DbgPrint(
            b"[HAL-SMP] StartNext: next=%u >= MADT ProcessorCount=%u - done/none\n\0".as_ptr()
                as ntkrnl_sys::hal_ddk::PCSTR,
            next,
            count,
        );
        return 0; // FALSE
    }

    // Low-memory trampoline pages: 4 allocated <1MB + mapped by ACPI Phase-0
    // (halacpi.rs HALP_LOW_STUB_SIZE_IN_PAGES). Page 0 = trampoline; pages
    // 1-3 = PDPT/PD/PT scratch for the identity map below. The BSP-visible
    // VA is a HalpMapPhysicalMemory64 mapping, NOT identity.
    let stub_va = (&raw const HalpLowStub).read() as *mut u8;
    let stub_pa = (&raw const HalpLowStubPhysicalAddress).read().QuadPart as u64;
    if stub_va.is_null() || stub_pa == 0 || (stub_pa & 0xFFF) != 0 || stub_pa >= 0x100000 {
        // Misconfigured low stub — cannot post a valid SIPI vector. Fail loud
        // rather than send a SIPI to a garbage page.
        DbgPrint(
            b"[HAL-SMP] StartNext: low stub bad (va=%p pa=%I64x)\n\0".as_ptr()
                as ntkrnl_sys::hal_ddk::PCSTR,
            stub_va,
            stub_pa,
        );
        return 0; // FALSE
    }

    // The trampoline enables paging on the KERNEL CR3 while executing at
    // stub_pa, so stub_pa must translate to itself in the System address
    // space for the one instruction window around the CR0 write. The boot
    // identity map is gone by Phase 1 (probed live, staging NOTES.md 14) —
    // install a VA==PA chain for page 0 through the recursive self-map,
    // using pages 1-3 as the PDPT/PD/PT. One-shot; stays mapped across all
    // AP starts (teardown is owed hardening — System has no user threads).
    if !identity_map_stub(stub_va, stub_pa) {
        DbgPrint(
            b"[HAL-SMP] StartNext: identity map refused (foreign PTE at %I64x)\n\0".as_ptr()
                as ntkrnl_sys::hal_ddk::PCSTR,
            stub_pa,
        );
        return 0; // FALSE - low VA range unexpectedly occupied; fail loud
    }

    // Stage the trampoline: copy the arch blob into the page and patch its
    // parameter block from the KPROCESSOR_STATE the kernel handed us.
    trampoline::install(stub_va, stub_pa, ProcessorState);

    // Fire INIT/SIPI/SIPI at the target LAPIC.
    ApicStartApplicationProcessor(next, stub_pa);

    // Mark it started and advance. The kernel (KeStartAllProcessors) now spins
    // on KeLoaderBlock->Prcb until the AP clears it in KiSystemStartup.
    let ident = (*(&raw const HalpProcessorIdentity)).add(next as usize);
    (*ident).ProcessorStarted = 1;
    NEXT_PROCESSOR.store(next + 1, Ordering::Relaxed);

    1 // TRUE
}

// ===========================================================================
// HalpSetupProcessorsTable — per-CPU PROCESSOR_IDENTITY finalization. Called
// (smp.h) once the given NT processor number is known-good. On UP this was a
// no-op; here we stamp the BSP self-check bit. Kept minimal — the heavy MADT
// population already happened in madt.rs::HalpParseApicTables.
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpSetupProcessorsTable(NTProcessorNumber: u32) {
    let count = (&raw const HalpApicInfoTable.ProcessorCount).read_volatile();
    if NTProcessorNumber >= count {
        return;
    }
    let ident = (*(&raw const HalpProcessorIdentity)).add(NTProcessorNumber as usize);
    // Tag the entry whose LAPIC matches the CPU currently executing as the BSP.
    if (*ident).LapicId == current_lapic_id() {
        (*ident).BSPCheck = 1;
    }
}

// ===========================================================================
// HalAllProcessorsStarted — MP-aware. TRUE once every enumerated AP has been
// handed a start request. (up/processor.rs returns TRUE unconditionally; this
// replaces it for MP builds.)
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllProcessorsStarted() -> BOOLEAN {
    let count = (&raw const HalpApicInfoTable.ProcessorCount).read_volatile();
    if NEXT_PROCESSOR.load(Ordering::Relaxed) >= count {
        1
    } else {
        0
    }
}

// ===========================================================================
// amd64-only: HalSendNMI / HalSendSoftwareInterrupt (up.rs had no-op stubs).
// ===========================================================================
#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSendNMI(TargetSet: KAFFINITY) {
    let mask = (TargetSet as u64) & 0xFF;
    if mask == 0 {
        return;
    }
    let icr: u64 = (APIC_MT_NMI << ICR_MT_SHIFT)
        | (APIC_DM_LOGICAL << ICR_DM_SHIFT)
        | (APIC_LEVEL_ASSERT << ICR_LEVEL_SHIFT)
        | (APIC_TGM_EDGE << ICR_TGM_SHIFT)
        | (mask << ICR_DEST_SHIFT);
    apic_write_icr(icr);
}

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSendSoftwareInterrupt(TargetSet: KAFFINITY, Irql: KIRQL) {
    // Software interrupt == fixed IPI at IrqlToSoftVector(Irql) = (Irql<<4)|0xF.
    let vector = (((Irql as u64) << 4) | 0xF) as UCHAR;
    HalRequestIpiSpecifyVector(TargetSet, vector);
}
