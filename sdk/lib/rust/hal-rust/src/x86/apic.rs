//! APIC HAL — LAPIC/IOAPIC programming, IRQL via TPR, software-interrupt
//! delivery via APIC self-IPI.  Translated from `hal/halx86/apic/apic.c`.
//!
//! Amd64-only (apic.cmake is amd64-only in the main HAL).  All `#ifndef
//! _M_AMD64` paths in the C source are dead on amd64 and are dropped
//! here — IRQL maintenance uses CR8 directly, the APC/dispatch software
//! traps live in apic/apictrap.S, and Kf{Raise,Lower}Irql / KeGetCurrentIrql
//! / KeRaiseIrqlTo* are provided by ntoskrnl on amd64.
//!
//! Companion files: apic_halinit.rs (boot phases), apictimer.rs (profile
//! timer), rtctimer.rs (system clock), tsc.rs (calibration).

#![cfg(target_arch = "x86_64")]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, CLOCK_LEVEL, DISPATCH_LEVEL, KAFFINITY, KIRQL, PKAFFINITY, PKIRQL, PKTRAP_FRAME,
    UCHAR, ULONG, ULONG_PTR, _KINTERRUPT_MODE_LevelSensitive as LEVEL_SENSITIVE, _KIPCR,
};

// apicp.h vector constants — apicp.h is HAL-private, not in bindgen.
const APC_VECTOR: ULONG = 0x1F;
const DISPATCH_VECTOR: ULONG = 0x2F;
const CLOCK_IPI_VECTOR: ULONG = 0xD2;

use crate::x86::pcr::KeGetPcr;

// ---------------------------------------------------------------------------
// APIC / IOAPIC register addresses (apicp.h, _M_AMD64 branch).
// ---------------------------------------------------------------------------

const APIC_BASE: usize = 0xFFFFFFFFFFFE0000u64 as usize;
const IOAPIC_BASE: usize = 0xFFFFFFFFFFFE1000u64 as usize;

// APIC register offsets — APIC_REGISTER enum from apicp.h.
const APIC_ID: ULONG = 0x0020;
const APIC_VER: ULONG = 0x0030;
const APIC_TPR: ULONG = 0x0080;
const APIC_PPR: ULONG = 0x00A0;
const APIC_EOI: ULONG = 0x00B0;
const APIC_LDR: ULONG = 0x00D0;
const APIC_DFR: ULONG = 0x00E0;
const APIC_SIVR: ULONG = 0x00F0;
const APIC_IRR: ULONG = 0x0200;
const APIC_ICR0: ULONG = 0x0300;
const APIC_ICR1: ULONG = 0x0310;
const APIC_TMRLVTR: ULONG = 0x0320;
const APIC_THRMLVTR: ULONG = 0x0330;
const APIC_PCLVTR: ULONG = 0x0340;
const APIC_LINT0: ULONG = 0x0350;
const APIC_LINT1: ULONG = 0x0360;
const APIC_ERRLVTR: ULONG = 0x0370;
const APIC_EXT0LVTR: ULONG = 0x0500;
const APIC_EXT1LVTR: ULONG = 0x0510;
const APIC_EXT2LVTR: ULONG = 0x0520;
const APIC_EXT3LVTR: ULONG = 0x0530;

// IOAPIC offsets.
const IOAPIC_IOREGSEL: usize = 0x00;
const IOAPIC_IOWIN: usize = 0x10;
const IOAPIC_REDTBL: UCHAR = 0x10;

// MSR + physical addresses.
const MSR_APIC_BASE: u32 = 0x0000001B;
const IOAPIC_PHYS_BASE: u64 = 0xFEC00000;

// Vector + IRQL constants (apicp.h, _M_AMD64 branch).
const APIC_MAX_IRQ: UCHAR = 24;
const APIC_FREE_VECTOR: UCHAR = 0xFF;
const APIC_RESERVED_VECTOR: UCHAR = 0xFE;
const APIC_SPURIOUS_VECTOR: UCHAR = 0xDF;
const APIC_NMI_VECTOR: UCHAR = 0xFF;
const APIC_ERROR_VECTOR: UCHAR = 0xE2;
const APIC_CLOCK_VECTOR: ULONG = 0xD1;
const APIC_CLOCK_INDEX: UCHAR = 8;

// CMCI_LEVEL = 5 on amd64 (sdk/include/ndk/amd64/ketypes.h).
const CMCI_LEVEL: KIRQL = 5;

// PRCB_BUILD_DEBUG / PRCB_BUILD_UNIPROCESSOR — not used here.

// EFLAGS bits.
const EFLAGS_INTERRUPT_MASK: ULONG_PTR = 1 << 9;

// IDT type codes from halp.h.
const IDT_INTERNAL: UCHAR = 17;

// APIC enum values (apicp.h).
const APIC_MT_FIXED: u32 = 0;
const APIC_MT_LOWEST_PRIORITY: u32 = 1;
const APIC_MT_NMI: u32 = 4;
const APIC_MT_EXT_INT: u32 = 7;

const APIC_TGM_EDGE: u32 = 0;
const APIC_TGM_LEVEL: u32 = 1;

const APIC_DM_PHYSICAL: u32 = 0;
const APIC_DM_LOGICAL: u32 = 1;

const APIC_DSH_SELF: u32 = 1;

const APIC_DF_FLAT: ULONG = 0xFFFFFFFF;

// IrqlToTpr / TprToIrql (apicp.h amd64 branch — linear TPR mapping).
#[inline(always)]
fn irql_to_tpr(irql: KIRQL) -> ULONG {
    (irql as ULONG) << 4
}
#[inline(always)]
fn tpr_to_irql(tpr: ULONG) -> KIRQL {
    (tpr >> 4) as KIRQL
}
#[inline(always)]
fn irql_to_soft_vector(irql: KIRQL) -> UCHAR {
    (((irql as ULONG) << 4) | 0xf) as UCHAR
}
#[inline(always)]
fn apic_logical_id(cpu: ULONG) -> UCHAR {
    1u8 << cpu
}

// HalpDefaultInterruptAffinity — global from another HAL .c file.
extern "C" {
    static HalpDefaultInterruptAffinity: KAFFINITY;
}

// ---------------------------------------------------------------------------
// MMIO + intrinsics — clang `_disable`, `__readeflags`, etc. have no
// linkable symbol; replicate inline.
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
    write_register_ulong(APIC_BASE + reg as usize, value)
}

#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!("pushfq", "pop {0}", out(reg) f, options(nomem, preserves_flags));
    f
}

#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!("push {0}", "popfq", in(reg) f, options(nomem));
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

#[inline(always)]
unsafe fn read_msr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") lo,
        out("edx") hi,
        options(nomem, nostack, preserves_flags),
    );
    ((hi as u64) << 32) | (lo as u64)
}

#[inline(always)]
unsafe fn write_msr(msr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") lo,
        in("edx") hi,
        options(nomem, nostack, preserves_flags),
    );
}

#[inline(always)]
unsafe fn read_cr8() -> u64 {
    let v: u64;
    core::arch::asm!("mov {0}, cr8", out(reg) v, options(nomem, nostack, preserves_flags));
    v
}

#[inline(always)]
unsafe fn write_cr8(v: u64) {
    core::arch::asm!("mov cr8, {0}", in(reg) v, options(nomem, nostack, preserves_flags));
}

// ---------------------------------------------------------------------------
// IDT helpers — KeRegisterInterruptHandler is FORCEINLINE in
// `internal/amd64/ke.h`.  Same shape as `usage.rs` and `tsc.rs`.
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn vector_to_idt_entry(vector: ULONG) -> usize {
    let dispatch = &raw const ntkrnl_sys::hal_ddk::HalPrivateDispatchTable;
    let vec_to_idt = (*dispatch).HalVectorToIDTEntry.unwrap_unchecked();
    vec_to_idt(vector) as usize
}

#[inline(always)]
unsafe fn register_interrupt_handler(vector: ULONG, handler: *mut core::ffi::c_void) {
    let pcr = KeGetPcr() as *mut _KIPCR;
    let entry = vector_to_idt_entry(vector);
    let idt = (*pcr).IdtBase.add(entry);
    let inner = &mut (*idt).__bindgen_anon_1;
    let address = handler as usize;
    inner.OffsetLow = (address & 0xffff) as u16;
    inner.OffsetMiddle = ((address >> 16) & 0xffff) as u16;
    inner.OffsetHigh = (address >> 32) as u32;
    const KGDT64_R0_CODE: u16 = 0x0010;
    inner.Selector = KGDT64_R0_CODE;
    inner.set_IstIndex(0);
    inner.set_Type(0x0e);
    inner.set_Dpl(0);
    inner.set_Present(1);
    inner.set_Reserved0(0);
    inner.Reserved1 = 0;
}

// ---------------------------------------------------------------------------
// Externs — pulled in from sibling HAL files.
// ---------------------------------------------------------------------------

unsafe extern "C" {
    fn HalpInitializeLegacyPICs();
    fn HalpRegisterVector(IdtType: UCHAR, BusVector: ULONG, SystemVector: ULONG, Irql: KIRQL);
    fn HalpClockInterrupt();
    fn HalpClockIpi();
    // Spurious LAPIC ISR (apicp.h: __cdecl).
    fn ApicSpuriousService();
}

// ---------------------------------------------------------------------------
// Globals — exported, written by HalpAllocateSystemInterrupt + ApicInit*.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut ApicVersion: ULONG = 0;

#[unsafe(no_mangle)]
pub static mut HalpVectorToIndex: [UCHAR; 256] = [APIC_FREE_VECTOR; 256];

// ---------------------------------------------------------------------------
// IOAPIC redirection-entry register layout (apicp.h _IOAPIC_REDIRECTION_REGISTER).
// 64-bit value stored as two 32-bit halves; we manipulate via the 64-bit
// scalar and pack/unpack the bitfield positions inline.
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
#[repr(C)]
struct IoApicRedir {
    long: u64,
}

impl IoApicRedir {
    fn new() -> Self { Self { long: 0 } }
    fn long0(self) -> ULONG { self.long as ULONG }
    fn long1(self) -> ULONG { (self.long >> 32) as ULONG }
    fn from_halves(lo: ULONG, hi: ULONG) -> Self {
        Self { long: (lo as u64) | ((hi as u64) << 32) }
    }
    fn vector(self) -> UCHAR { (self.long & 0xFF) as UCHAR }
    fn mask(self) -> bool { (self.long >> 16) & 1 != 0 }
    fn trigger_mode(self) -> u32 { ((self.long >> 15) & 1) as u32 }
    fn set_field(&mut self, value: u64, shift: u32, width: u32) {
        let mask = ((1u64 << width) - 1) << shift;
        self.long = (self.long & !mask) | ((value << shift) & mask);
    }
    fn set_vector(&mut self, v: UCHAR) { self.set_field(v as u64, 0, 8); }
    fn set_message_type(&mut self, v: u32) { self.set_field(v as u64, 8, 3); }
    fn set_destination_mode(&mut self, v: u32) { self.set_field(v as u64, 11, 1); }
    fn set_polarity(&mut self, v: u32) { self.set_field(v as u64, 13, 1); }
    fn set_remote_irr(&mut self, v: u32) { self.set_field(v as u64, 14, 1); }
    fn set_trigger_mode(&mut self, v: u32) { self.set_field(v as u64, 15, 1); }
    fn set_mask(&mut self, v: u32) { self.set_field(v as u64, 16, 1); }
    fn set_destination(&mut self, v: u32) { self.set_field((v as u64) & 0xFF, 56, 8); }
}

// ---------------------------------------------------------------------------
// IOAPIC register access.
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn io_apic_read(register: UCHAR) -> ULONG {
    write_register_ulong(IOAPIC_BASE + IOAPIC_IOREGSEL, register as ULONG);
    read_register_ulong(IOAPIC_BASE + IOAPIC_IOWIN)
}

#[inline(always)]
unsafe fn io_apic_write(register: UCHAR, value: ULONG) {
    write_register_ulong(IOAPIC_BASE + IOAPIC_IOREGSEL, register as ULONG);
    write_register_ulong(IOAPIC_BASE + IOAPIC_IOWIN, value);
}

#[inline(always)]
unsafe fn apic_write_io_redirection_entry(index: UCHAR, redir: IoApicRedir) {
    io_apic_write(IOAPIC_REDTBL + 2 * index, redir.long0());
    io_apic_write(IOAPIC_REDTBL + 2 * index + 1, redir.long1());
}

#[inline(always)]
unsafe fn apic_read_io_redirection_entry(index: UCHAR) -> IoApicRedir {
    let lo = io_apic_read(IOAPIC_REDTBL + 2 * index);
    let hi = io_apic_read(IOAPIC_REDTBL + 2 * index + 1);
    IoApicRedir::from_halves(lo, hi)
}

// ---------------------------------------------------------------------------
// APIC ICR self-IPI for software interrupt delivery.
// ---------------------------------------------------------------------------

#[inline]
unsafe fn apic_request_self_interrupt(vector: UCHAR, trigger_mode: UCHAR) {
    // Pre-compute the IRR register and bit for this vector.
    let vector_high = vector as ULONG / 32;
    let vector_low = vector as ULONG % 32;
    let irr = APIC_IRR + 0x10 * vector_high;
    let irr_bit = 1u32 << vector_low;

    // Build the ICR value: vector + MT_Fixed + trigger_mode + DSH_Self.
    let icr: u64 = (vector as u64)
        | ((APIC_MT_FIXED as u64) << 8)
        | ((trigger_mode as u64) << 15)
        | ((APIC_DSH_SELF as u64) << 18);
    let icr_lo = icr as ULONG;
    let icr_hi = (icr >> 32) as ULONG;

    let flags = read_eflags();
    cli();

    // Wait for the APIC to be idle (DeliveryStatus bit at bit 12 of ICR0).
    while (apic_read(APIC_ICR0) & (1 << 12)) != 0 {}

    // Write high dword first, then low dword sends the IPI.
    apic_write(APIC_ICR1, icr_hi);
    apic_write(APIC_ICR0, icr_lo);

    // Wait until the request is visible in IRR (stays asserted until we
    // re-enable interrupts and the CPU services it).
    while (apic_read(irr) & irr_bit) == 0 {
        yield_processor();
    }

    if (flags & EFLAGS_INTERRUPT_MASK) != 0 {
        sti();
    }
}

#[inline(always)]
unsafe fn apic_send_eoi() {
    apic_write(APIC_EOI, 0);
}

#[inline(always)]
unsafe fn apic_get_processor_irql() -> KIRQL {
    tpr_to_irql(apic_read(APIC_PPR))
}

#[inline(always)]
unsafe fn apic_get_current_irql() -> KIRQL {
    read_cr8() as KIRQL
}

#[inline(always)]
unsafe fn apic_set_irql(irql: KIRQL) {
    write_cr8(irql as u64);
}

// On amd64, ApicLowerIrql == ApicSetIrql (no APIC_LAZY_IRQL on amd64).
#[inline(always)]
unsafe fn apic_lower_irql(irql: KIRQL) {
    write_cr8(irql as u64);
}

// ---------------------------------------------------------------------------
// HalpIrqToVector / HalpVectorToIrql / HalpVectorToIrq — exported to other
// HAL files (pic_pic.rs, usage.rs, etc.). All FASTCALL on the C side.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpIrqToVector(Irq: UCHAR) -> UCHAR {
    let lo = io_apic_read(IOAPIC_REDTBL + 2 * Irq);
    (lo & 0xFF) as UCHAR
}

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpVectorToIrql(Vector: UCHAR) -> KIRQL {
    tpr_to_irql(Vector as ULONG)
}

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpVectorToIrq(Vector: UCHAR) -> UCHAR {
    HalpVectorToIndex[Vector as usize]
}

// ---------------------------------------------------------------------------
// HalpSendEOI — NTAPI thunk over apic_send_eoi.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpSendEOI() {
    apic_send_eoi();
}

// ---------------------------------------------------------------------------
// ApicInitializeLocalApic — per-CPU LAPIC bring-up.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ApicInitializeLocalApic(Cpu: ULONG) {
    // Enable the APIC (MSR_APIC_BASE bit 11).
    let mut base = read_msr(MSR_APIC_BASE);
    base |= 1 << 11; // Enable
    if Cpu == 0 {
        base |= 1 << 8; // BootStrapCPUCore
    } else {
        base &= !(1u64 << 8);
    }
    write_msr(MSR_APIC_BASE, base);

    // Spurious vector + SoftwareEnable.
    let mut sivr = apic_read(APIC_SIVR);
    sivr = (sivr & !0xFF) | (APIC_SPURIOUS_VECTOR as ULONG);
    sivr |= 1 << 8; // SoftwareEnable
    sivr &= !(1u32 << 9); // FocusCPUCoreChecking = 0
    apic_write(APIC_SIVR, sivr);

    if Cpu == 0 {
        ApicVersion = apic_read(APIC_VER);
    }

    apic_write(APIC_DFR, APIC_DF_FLAT);
    apic_write(APIC_LDR, (apic_logical_id(Cpu) as ULONG) << 24);

    // Spurious ISR.
    register_interrupt_handler(APIC_SPURIOUS_VECTOR as ULONG, ApicSpuriousService as _);

    // Template LVT — vector=APIC_FREE_VECTOR, MT=Fixed, mask=1, edge.
    let lvt_masked: ULONG = (APIC_FREE_VECTOR as ULONG)
        | ((APIC_MT_FIXED as ULONG) << 8)
        | (1 << 16); // Mask

    apic_write(APIC_TMRLVTR, lvt_masked);
    apic_write(APIC_THRMLVTR, lvt_masked);
    apic_write(APIC_PCLVTR, lvt_masked);
    apic_write(APIC_EXT0LVTR, lvt_masked);
    apic_write(APIC_EXT1LVTR, lvt_masked);
    apic_write(APIC_EXT2LVTR, lvt_masked);
    apic_write(APIC_EXT3LVTR, lvt_masked);

    // LINT0 — ExtInt, masked.
    let lint0: ULONG = (APIC_SPURIOUS_VECTOR as ULONG)
        | ((APIC_MT_EXT_INT as ULONG) << 8)
        | (1 << 16);
    apic_write(APIC_LINT0, lint0);

    // LINT1 — NMI, level-triggered, unmasked.
    let lint1: ULONG = (APIC_NMI_VECTOR as ULONG)
        | ((APIC_MT_NMI as ULONG) << 8)
        | ((APIC_TGM_LEVEL as ULONG) << 15);
    apic_write(APIC_LINT1, lint1);

    // Error LVT — Fixed, masked at template (mask still set).
    let err_lvt: ULONG = (APIC_ERROR_VECTOR as ULONG)
        | ((APIC_MT_FIXED as ULONG) << 8)
        | (1 << 16);
    apic_write(APIC_ERRLVTR, err_lvt);

    // Set IRQL from PCR.
    apic_set_irql((*KeGetPcr()).Irql);
}

// ---------------------------------------------------------------------------
// HalpAllocateSystemInterrupt — set up an IOAPIC redir entry for `Irq`,
// dispatching to `Vector`.  Masked.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpAllocateSystemInterrupt(Irq: UCHAR, Vector: UCHAR) -> UCHAR {
    let mut redir = IoApicRedir::new();
    redir.set_vector(Vector);
    redir.set_message_type(APIC_MT_LOWEST_PRIORITY);
    redir.set_destination_mode(APIC_DM_LOGICAL);
    redir.set_polarity(0);
    redir.set_remote_irr(0);
    redir.set_trigger_mode(APIC_TGM_EDGE);
    redir.set_mask(1);
    redir.set_destination((apic_read(APIC_ID) >> 24) as u32);
    apic_write_io_redirection_entry(Irq, redir);

    HalpVectorToIndex[Vector as usize] = Irq;
    Vector
}

// ---------------------------------------------------------------------------
// HalpGetRootInterruptVector — find/allocate a vector for an IRQ and report
// its IRQL + affinity.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetRootInterruptVector(
    BusInterruptLevel: ULONG,
    _BusInterruptVector: ULONG,
    OutIrql: PKIRQL,
    OutAffinity: PKAFFINITY,
) -> ULONG {
    let mut vector = HalpIrqToVector(BusInterruptLevel as UCHAR);

    if vector != APIC_FREE_VECTOR {
        // Already allocated — return its IRQL.
        core::ptr::write_volatile(OutIrql, HalpVectorToIrql(vector));
    } else {
        // Search for a free slot in the allowed IRQL range.
        let mut found = false;
        'outer: for offset in 0u32..15 {
            let mut irql = (CLOCK_LEVEL as KIRQL).wrapping_sub(1);
            while irql >= CMCI_LEVEL {
                let cand = (irql_to_tpr(irql) + offset) as UCHAR;
                if HalpVectorToIrq(cand) == APIC_FREE_VECTOR {
                    vector = HalpAllocateSystemInterrupt(BusInterruptLevel as UCHAR, cand);
                    core::ptr::write_volatile(OutIrql, irql);
                    found = true;
                    break 'outer;
                }
                if irql == 0 { break; }
                irql -= 1;
            }
        }
        if !found {
            core::ptr::write_volatile(OutAffinity, 0);
            core::ptr::write_volatile(OutIrql, 0);
            return 0;
        }
    }

    core::ptr::write_volatile(OutAffinity, HalpDefaultInterruptAffinity);
    vector as ULONG
}

// ---------------------------------------------------------------------------
// ApicInitializeIOApic — map the IOAPIC MMIO page, mask all redir entries,
// init HalpVectorToIndex, then unmask the timer slot.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ApicInitializeIOApic() {
    use ntkrnl_sys::hal_ddk::PHARDWARE_PTE;

    // HalAddressToPte = MiAddressToPte (FORCEINLINE in halp.h →
    // internal/amd64/mm.h).  Inline the amd64 PTE address math.
    const PTE_BASE_AMD64: usize = 0xFFFFF68000000000;
    let offset = ((IOAPIC_BASE as u64) >> 9) & (0xFFFFFFFFFu64 << 3);
    let pte: PHARDWARE_PTE = (PTE_BASE_AMD64 + offset as usize) as PHARDWARE_PTE;

    (*pte).set_PageFrameNumber(IOAPIC_PHYS_BASE / 4096);
    (*pte).set_Valid(1);
    (*pte).set_Write(1);
    (*pte).set_Owner(1);
    (*pte).set_CacheDisable(1);
    (*pte).set_Global(1);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    let mut template = IoApicRedir::new();
    template.set_vector(APIC_FREE_VECTOR);
    template.set_message_type(APIC_MT_FIXED);
    template.set_destination_mode(APIC_DM_PHYSICAL);
    template.set_trigger_mode(APIC_TGM_EDGE);
    template.set_mask(1);
    let dest = (apic_read(APIC_ID) >> 24) as u32;
    template.set_destination(dest);

    for index in 0..APIC_MAX_IRQ {
        apic_write_io_redirection_entry(index, template);
    }
    for v in 0..=255usize {
        HalpVectorToIndex[v] = APIC_FREE_VECTOR;
    }

    // Timer slot, level-triggered, masked.
    let mut clock = IoApicRedir::new();
    clock.set_vector(APIC_CLOCK_VECTOR as UCHAR);
    clock.set_message_type(APIC_MT_FIXED);
    clock.set_destination_mode(APIC_DM_PHYSICAL);
    clock.set_trigger_mode(APIC_TGM_LEVEL);
    clock.set_mask(1);
    clock.set_destination(dest);
    apic_write_io_redirection_entry(APIC_CLOCK_INDEX, clock);
}

// ---------------------------------------------------------------------------
// HalpInitializePICs — wire up legacy PIC, then init IOAPIC, reserve
// system vectors, and install clock IDT entries.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitializePICs(EnableInterrupts: BOOLEAN) {
    let mut eflags = read_eflags();
    cli();

    HalpInitializeLegacyPICs();
    ApicInitializeIOApic();

    // Reserve fixed system vectors.
    HalpVectorToIndex[APC_VECTOR as usize] = APIC_RESERVED_VECTOR;
    HalpVectorToIndex[DISPATCH_VECTOR as usize] = APIC_RESERVED_VECTOR;
    HalpVectorToIndex[APIC_CLOCK_VECTOR as usize] = 8;
    HalpVectorToIndex[CLOCK_IPI_VECTOR as usize] = APIC_RESERVED_VECTOR;
    HalpVectorToIndex[APIC_SPURIOUS_VECTOR as usize] = APIC_RESERVED_VECTOR;

    register_interrupt_handler(APIC_CLOCK_VECTOR, HalpClockInterrupt as _);
    register_interrupt_handler(CLOCK_IPI_VECTOR, HalpClockIpi as _);
    // amd64: APC + dispatch handlers live in apic/apictrap.S, not here.

    HalpRegisterVector(IDT_INTERNAL, 0, APC_VECTOR, 1 /* APC_LEVEL */);
    HalpRegisterVector(IDT_INTERNAL, 0, DISPATCH_VECTOR, DISPATCH_LEVEL as KIRQL);

    if EnableInterrupts != 0 {
        eflags |= EFLAGS_INTERRUPT_MASK;
    }
    write_eflags(eflags);
}

// ---------------------------------------------------------------------------
// Software-interrupt thunks — request/clear via APIC self-IPI.  FASTCALL.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalRequestSoftwareInterrupt(Irql: KIRQL) {
    apic_request_self_interrupt(irql_to_soft_vector(Irql), APIC_TGM_EDGE as UCHAR);
}

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalClearSoftwareInterrupt(_Irql: KIRQL) {
    // Nothing to do.
}

// ---------------------------------------------------------------------------
// HalEnableSystemInterrupt / HalDisableSystemInterrupt — flip the Mask bit
// on the IOAPIC redirection entry for a vector.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalEnableSystemInterrupt(
    Vector: ULONG,
    _Irql: KIRQL,
    InterruptMode: u32,
) -> BOOLEAN {
    let index = HalpVectorToIndex[Vector as usize];
    if index == APIC_FREE_VECTOR {
        return 0; // FALSE
    }

    let mut redir = apic_read_io_redirection_entry(index);
    if !redir.mask() {
        // Already enabled — confirm vector matches.
        return if redir.vector() == Vector as UCHAR { 1 } else { 0 };
    }

    redir.set_vector(Vector as UCHAR);
    redir.set_message_type(APIC_MT_FIXED);
    redir.set_destination_mode(APIC_DM_PHYSICAL);
    redir.set_destination((apic_read(APIC_ID) >> 24) as u32);
    let trigger = if InterruptMode == LEVEL_SENSITIVE {
        APIC_TGM_LEVEL
    } else {
        APIC_TGM_EDGE
    };
    redir.set_trigger_mode(trigger);
    redir.set_mask(0);

    apic_write_io_redirection_entry(index, redir);
    1 // TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalDisableSystemInterrupt(Vector: ULONG, _Irql: KIRQL) {
    let index = HalpVectorToIndex[Vector as usize];

    // Mask just the lower dword (Mask bit at position 16).
    let mut lo = io_apic_read(IOAPIC_REDTBL + 2 * index);
    lo |= 1 << 16;
    io_apic_write(IOAPIC_REDTBL + 2 * index, lo);
}

// ---------------------------------------------------------------------------
// HalBeginSystemInterrupt — entry from the trap handler.  On amd64 there's
// no APIC_LAZY_IRQL path, so this is just "save IRQL, raise, enable INTs".
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalBeginSystemInterrupt(
    Irql: KIRQL,
    _Vector: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    let current = apic_get_current_irql();
    core::ptr::write_volatile(OldIrql, current);
    apic_set_irql(Irql);
    sti();
    1 // TRUE
}

// ---------------------------------------------------------------------------
// HalEndSystemInterrupt — EOI + restore IRQL.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalEndSystemInterrupt(OldIrql: KIRQL, _TrapFrame: PKTRAP_FRAME) {
    apic_send_eoi();
    apic_lower_irql(OldIrql);
}
