//! APIC HAL — LAPIC/IOAPIC programming, IRQL via TPR, software-interrupt
//! delivery via APIC self-IPI.  Translated from `hal/halx86/apic/apic.c`.
//!
//! Dual-arch (i386-apic-hal lane): compiled whenever `hal_apic` is set —
//! amd64 always, or i386 with the `apic` feature.  The two arches diverge in
//! their IRQL model, which is `#[cfg(target_arch = ...)]`-forked in place:
//!   - **amd64**: IRQL lives in CR8 (linear `irql<<4` TPR map); the APC/dispatch
//!     software traps live in apic/apictrap.S; Kf{Raise,Lower}Irql /
//!     KeGetCurrentIrql / KeRaiseIrqlTo* come from ntoskrnl.
//!   - **i386**: 32 IRQLs compress onto 16 TPR classes via the HalpIRQLtoTPR /
//!     HalVectorToIRQL tables; IRQL is kept lazily in KPCR.Irql (APIC_LAZY_IRQL)
//!     and only flushed to the TPR when an interrupt would actually fire; the
//!     HAL owns the APC(0x3D)/DISPATCH(0x41) software vectors + their handlers,
//!     and exports the Kf/Ke IRQL primitives itself.  All from the C source's
//!     `#ifndef _M_AMD64` / APIC_LAZY_IRQL branches.
//!
//! Companion files: apic_halinit.rs (boot phases), apictimer.rs (profile
//! timer), rtctimer.rs (system clock), tsc.rs (calibration).

#![allow(
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    unused_unsafe,
    unused_imports,
    unused_variables,
    unused_mut
)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, KAFFINITY, KIRQL, PKAFFINITY, PKIRQL, PKTRAP_FRAME, UCHAR, ULONG, ULONG_PTR,
    _KINTERRUPT_MODE_LevelSensitive as LEVEL_SENSITIVE, _KIPCR,
};

// DISPATCH_LEVEL / CLOCK_LEVEL differ per arch (16-level amd64 vs 32-level
// i386), so define them locally rather than importing the bindgen #define —
// the loop range in HalpGetRootInterruptVector depends on the right value.
#[cfg(target_arch = "x86_64")]
const DISPATCH_LEVEL: KIRQL = 2;
#[cfg(target_arch = "x86_64")]
const CLOCK_LEVEL: KIRQL = 13;
#[cfg(target_arch = "x86")]
const DISPATCH_LEVEL: KIRQL = 2;
#[cfg(target_arch = "x86")]
const CLOCK_LEVEL: KIRQL = 28; // CLOCK2_LEVEL on i386

// apicp.h vector constants — apicp.h is HAL-private, not in bindgen.
// APC/DISPATCH software-interrupt vectors differ per arch (apicp.h).
#[cfg(target_arch = "x86_64")]
const APC_VECTOR: ULONG = 0x1F;
#[cfg(target_arch = "x86_64")]
const DISPATCH_VECTOR: ULONG = 0x2F;
#[cfg(target_arch = "x86")]
const APC_VECTOR: ULONG = 0x3D;
#[cfg(target_arch = "x86")]
const DISPATCH_VECTOR: ULONG = 0x41;
const CLOCK_IPI_VECTOR: ULONG = 0xD2;

use crate::x86::pcr::KeGetPcr;

// ---------------------------------------------------------------------------
// APIC / IOAPIC register addresses (apicp.h, _M_AMD64 branch).
// ---------------------------------------------------------------------------

// LAPIC/IOAPIC MMIO window (apicp.h): high-canonical on amd64, the i386
// non-PAE fixed HAL window 0xFFFE0000/0xFFFE1000 on x86.
#[cfg(target_arch = "x86_64")]
const APIC_BASE: usize = 0xFFFFFFFFFFFE0000u64 as usize;
#[cfg(target_arch = "x86_64")]
const IOAPIC_BASE: usize = 0xFFFFFFFFFFFE1000u64 as usize;
#[cfg(target_arch = "x86")]
const APIC_BASE: usize = 0xFFFE0000;
#[cfg(target_arch = "x86")]
const IOAPIC_BASE: usize = 0xFFFE1000;

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
// Spurious/error vectors differ per arch (apicp.h).
#[cfg(target_arch = "x86_64")]
const APIC_SPURIOUS_VECTOR: UCHAR = 0xDF;
#[cfg(target_arch = "x86_64")]
const APIC_ERROR_VECTOR: UCHAR = 0xE2;
#[cfg(target_arch = "x86")]
const APIC_SPURIOUS_VECTOR: UCHAR = 0x1F;
#[cfg(target_arch = "x86")]
const APIC_ERROR_VECTOR: UCHAR = 0xE3;
const APIC_NMI_VECTOR: UCHAR = 0xFF;
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

// ---------------------------------------------------------------------------
// IRQL model selector.
//   amd64: 16 IRQLs map linearly onto the 16 TPR priority classes; the current
//          IRQL lives in CR8.  No lazy path.
//   i386 : 32 IRQLs compress non-uniformly onto 16 TPR classes via lookup
//          tables, and APIC_LAZY_IRQL keeps the soft IRQL in KPCR.Irql, only
//          writing the (uncached MMIO) TPR when an interrupt actually needs it.
// ---------------------------------------------------------------------------

// APIC_LAZY_IRQL is `#define`d for !_M_AMD64 in the C.  Kept as a const bool so
// the eager path (write the TPR on every IRQL change) stays compiled and can be
// flipped for bisection — but lazy is the default (and avoids reading the TPR
// MMIO before the LAPIC is even mapped).  amd64 never consults it (CR8 path).
#[cfg(target_arch = "x86")]
const APIC_LAZY_IRQL: bool = true;

// i386 IRQL↔TPR compression tables (apic.c:31-88, _M_IX86 branch).
#[cfg(target_arch = "x86")]
static HalpIRQLtoTPR: [UCHAR; 32] = [
    0x00, // 0  PASSIVE_LEVEL
    0x3d, // 1  APC_LEVEL
    0x41, // 2  DISPATCH_LEVEL
    0x41, // 3
    0x51, // 4
    0x61, // 5
    0x71, // 6
    0x81, // 7
    0x91, // 8
    0xa1, // 9
    0xb1, // 10
    0xb1, // 11
    0xb1, // 12
    0xb1, // 13
    0xb1, // 14
    0xb1, // 15 DEVICE IRQL
    0xb1, // 16
    0xb1, // 17
    0xb1, // 18
    0xb1, // 19
    0xb1, // 20
    0xb1, // 21
    0xb1, // 22
    0xb1, // 23
    0xb1, // 24
    0xb1, // 25
    0xb1, // 26
    0xc1, // 27 PROFILE_LEVEL
    0xd1, // 28 CLOCK2_LEVEL
    0xe1, // 29 IPI_LEVEL
    0xef, // 30 POWER_LEVEL
    0xff, // 31 HIGH_LEVEL
];

#[cfg(target_arch = "x86")]
static HalVectorToIRQL: [KIRQL; 16] = [
    0,    // 00 PASSIVE_LEVEL
    0xff, // 10
    0xff, // 20
    1,    // 3D APC_LEVEL
    2,    // 41 DISPATCH_LEVEL
    4,    // 50
    5,    // 60
    6,    // 70
    7,    // 80 DEVICE IRQL
    8,    // 90
    9,    // A0
    10,   // B0
    27,   // C1 PROFILE_LEVEL
    28,   // D1 CLOCK2_LEVEL
    29,   // E1 IPI_LEVEL / EF POWER_LEVEL
    31,   // FF HIGH_LEVEL
];

// IrqlToTpr / TprToIrql / IrqlToSoftVector — apicp.h.  amd64 = linear; i386 =
// table lookups (IrqlToSoftVector == IrqlToTpr on i386, per apicp.h:56).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn irql_to_tpr(irql: KIRQL) -> ULONG {
    (irql as ULONG) << 4
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn tpr_to_irql(tpr: ULONG) -> KIRQL {
    (tpr >> 4) as KIRQL
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn irql_to_soft_vector(irql: KIRQL) -> UCHAR {
    (((irql as ULONG) << 4) | 0xf) as UCHAR
}

#[cfg(target_arch = "x86")]
#[inline(always)]
fn irql_to_tpr(irql: KIRQL) -> ULONG {
    HalpIRQLtoTPR[irql as usize] as ULONG
}
#[cfg(target_arch = "x86")]
#[inline(always)]
fn tpr_to_irql(tpr: ULONG) -> KIRQL {
    HalVectorToIRQL[(tpr >> 4) as usize]
}
#[cfg(target_arch = "x86")]
#[inline(always)]
fn irql_to_soft_vector(irql: KIRQL) -> UCHAR {
    HalpIRQLtoTPR[irql as usize]
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

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!("pushfq", "pop {0}", out(reg) f, options(nomem, preserves_flags));
    f
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!("push {0}", "popfq", in(reg) f, options(nomem));
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!("pushfd", "pop {0:e}", out(reg) f, options(nomem, preserves_flags));
    f
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!("push {0:e}", "popfd", in(reg) f, options(nomem));
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

// CR8 is amd64-only (there is no CR8 on i386 — the i386 IRQL model uses the
// soft KPCR.Irql byte + lazy TPR instead).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_cr8() -> u64 {
    let v: u64;
    core::arch::asm!("mov {0}, cr8", out(reg) v, options(nomem, nostack, preserves_flags));
    v
}

#[cfg(target_arch = "x86_64")]
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

// amd64: build a full 16-byte IDT gate (Selector KGDT64_R0_CODE + 64-bit split
// offset + type/dpl/present).  This mirrors the amd64 KeRegisterInterruptHandler.
#[cfg(target_arch = "x86_64")]
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

// i386: the kernel pre-creates every IDT entry with the correct selector +
// 32-bit-interrupt-gate access; KeRegisterInterruptHandler only patches the
// 32-bit handler offset, split across the 8-byte KIDTENTRY's Offset (low 16)
// and ExtendedOffset (high 16).  (internal/i386/ke.h KeRegisterInterruptHandler.)
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn register_interrupt_handler(vector: ULONG, handler: *mut core::ffi::c_void) {
    let pcr = KeGetPcr() as *mut _KIPCR;
    let entry = vector_to_idt_entry(vector) as usize;
    let e = &mut *(*pcr).IDT.add(entry);
    let address = handler as usize;
    e.Offset = (address & 0xffff) as u16;
    e.ExtendedOffset = ((address >> 16) & 0xffff) as u16;
}

// ---------------------------------------------------------------------------
// Externs — pulled in from sibling HAL files.
// ---------------------------------------------------------------------------

// NTAPI HAL helpers — stdcall on i386 (`_Name@N`), Microsoft-x64 on amd64;
// `extern "system"` decorates correctly on both (defined in pic.rs / usage.rs).
unsafe extern "system" {
    fn HalpInitializeLegacyPICs();
    fn HalpRegisterVector(IdtType: UCHAR, BusVector: ULONG, SystemVector: ULONG, Irql: KIRQL);
}

// Trap-stub symbols from apic/apictrap.S — address-taken only (installed into
// the IDT), never called with args, so the ABI is irrelevant; only the symbol
// name matters (`_HalpClockInterrupt` on i386, `HalpClockInterrupt` on amd64 —
// both what `extern "C"` produces).
unsafe extern "C" {
    fn HalpClockInterrupt();
    fn HalpClockIpi();
    // Spurious LAPIC ISR (apicp.h: __cdecl).
    fn ApicSpuriousService();
    // i386 APC/DISPATCH software-interrupt trap stubs (apictrap.S _M_IX86 half).
    #[cfg(target_arch = "x86")]
    fn HalpApcInterrupt();
    #[cfg(target_arch = "x86")]
    fn HalpDispatchInterrupt();
    // i386 kernel-IPI (0xE1) trap stub (apictrap.S; handler in rtctimer.rs).
    // amd64 must NOT wire this: its kernel owns IDT[0xE1] (KiIpiInterrupt).
    #[cfg(target_arch = "x86")]
    fn HalpIpiInterrupt();
    // Serial flavor banner (i386): variadic cdecl, resolved via the ntoskrnl
    // import lib — proves in every boot log that the APIC HAL is live.
    #[cfg(target_arch = "x86")]
    fn DbgPrint(fmt: *const u8, ...) -> u32;
}

// ---------------------------------------------------------------------------
// i386-only kernel helpers + trap-frame inlines for the APC/dispatch handlers
// (amd64 handles APC/dispatch entirely in apictrap.S — none of this compiles).
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
const APC_LEVEL: KIRQL = 1;
#[cfg(target_arch = "x86")]
use crate::x86::buildtype::SYNCH_LEVEL; // 2 on the UP HAL, 27 on the SMP HAL (i386)
#[cfg(target_arch = "x86")]
const EFLAGS_V86_MASK: ULONG = 0x0002_0000;
#[cfg(target_arch = "x86")]
const KERNEL_MODE: i8 = 0;
#[cfg(target_arch = "x86")]
const USER_MODE: i8 = 1;
#[cfg(target_arch = "x86")]
const EXCEPTION_CHAIN_END: usize = !0;

// KiDeliverApc / KiDispatchInterrupt are NTAPI (stdcall) exports; KiEoiHelper is
// FASTCALL and DECLSPEC_NORETURN.  Same declarations pic_pic.rs uses.
#[cfg(target_arch = "x86")]
unsafe extern "system" {
    fn KiDeliverApc(PreviousMode: i8, ExceptionFrame: *mut core::ffi::c_void, TrapFrame: PKTRAP_FRAME);
    fn KiDispatchInterrupt();
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
}

// KiUserTrap / KiEnterInterruptTrap are FORCEINLINE in the C (trap_x.h) — never
// exported — so replicate them.  KiEnterInterruptTrap must stamp the DBG
// trap-frame fields (our kernel is a DBG build), else KiExitTrapDebugChecks
// bugchecks on the magic mismatch.  Verbatim from pic_pic.rs / timer.rs.
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn KiUserTrap(TrapFrame: PKTRAP_FRAME) -> BOOLEAN {
    if ((*TrapFrame).SegCs & 1) != 0 { 1 } else { 0 }
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn KiEnterInterruptTrap(TrapFrame: PKTRAP_FRAME) {
    let pcr = KeGetPcr();
    (*pcr).__bindgen_anon_1.NtTib.ExceptionList = EXCEPTION_CHAIN_END as *mut _;
    (*TrapFrame).Dr7 = 0;
    (*TrapFrame).DbgArgPointer = (*TrapFrame).Edx;
    (*TrapFrame).DbgArgMark = 0xBADB_0D00;
    (*TrapFrame).DbgEip = (*TrapFrame).Eip;
    (*TrapFrame).DbgEbp = (*TrapFrame).Ebp;
    (*TrapFrame).PreviousPreviousMode = !0;
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

// amd64: IRQL is CR8, directly.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn apic_get_current_irql() -> KIRQL {
    read_cr8() as KIRQL
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn apic_set_irql(irql: KIRQL) {
    write_cr8(irql as u64);
}
// On amd64, ApicLowerIrql == ApicSetIrql (no APIC_LAZY_IRQL on amd64).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn apic_lower_irql(irql: KIRQL) {
    write_cr8(irql as u64);
}

// i386: the lazy model (apic.c:205-254).  Raising only writes the soft
// KPCR.Irql byte; the hard TPR is deferred and written on the lowering path
// (or in HalBeginSystemInterrupt) only when it must actually take effect.
// KPCR.IRR is repurposed here as "the hard IRQL currently latched in the TPR".
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn apic_get_current_irql() -> KIRQL {
    if APIC_LAZY_IRQL {
        (*KeGetPcr()).Irql
    } else {
        tpr_to_irql(apic_read(APIC_TPR))
    }
}
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn apic_set_irql(irql: KIRQL) {
    if APIC_LAZY_IRQL {
        (*KeGetPcr()).Irql = irql;
    } else {
        apic_write(APIC_TPR, irql_to_tpr(irql));
    }
}
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn apic_lower_irql(irql: KIRQL) {
    if APIC_LAZY_IRQL {
        let pcr = KeGetPcr();
        (*pcr).Irql = irql;
        // Only push the hard TPR back down if it is currently latched higher.
        if (irql as ULONG) < (*pcr).IRR {
            (*pcr).IRR = irql as ULONG;
            apic_write(APIC_TPR, irql_to_tpr(irql));
        }
    } else {
        apic_write(APIC_TPR, irql_to_tpr(irql));
    }
}

// ApicRaiseIrql == ApicSetIrql on both arches (apic.c:233).
#[inline(always)]
unsafe fn apic_raise_irql(irql: KIRQL) {
    apic_set_irql(irql);
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
    // i386 lazy model: seed the hard-IRQL shadow (KPCR.IRR) to match (apic.c:361-364).
    #[cfg(target_arch = "x86")]
    if APIC_LAZY_IRQL {
        (*KeGetPcr()).IRR = (*KeGetPcr()).Irql as ULONG;
    }
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
// Message-signaled interrupts (crate::message). An MSI is a write to the
// local APIC's address window that names a vector directly, so a message
// vector is just an IDT vector with no IOAPIC pin behind it: claimed in
// HalpVectorToIndex as APIC_RESERVED_VECTOR (the marker the software vectors
// already use, which also routes the i386 lazy-IRQL replay to an edge
// self-IPI, the right shape for a message), and remembered here so the
// enable/disable paths leave the IOAPIC alone.
// ---------------------------------------------------------------------------

static mut MessageVectors: [bool; 256] = [false; 256];

fn is_message_vector(vector: ULONG) -> bool {
    vector < 256 && unsafe { MessageVectors[vector as usize] }
}

/// Every free vector in the device IRQL band, in the order
/// HalpGetRootInterruptVector hands them out (highest IRQL first).
unsafe fn free_device_vectors(mut f: impl FnMut(UCHAR, KIRQL) -> bool) {
    for offset in 0u32..15 {
        let mut irql = (CLOCK_LEVEL as KIRQL).wrapping_sub(1);
        while irql >= CMCI_LEVEL {
            let cand = (irql_to_tpr(irql) + offset) as UCHAR;
            if HalpVectorToIrq(cand) == APIC_FREE_VECTOR && !f(cand, irql) {
                return;
            }
            if irql == 0 {
                break;
            }
            irql -= 1;
        }
    }
}

pub(crate) unsafe fn message_capacity() -> u32 {
    let mut n = 0;
    free_device_vectors(|_, _| {
        n += 1;
        true
    });
    n
}

pub(crate) unsafe fn allocate_message_vector() -> Option<(u32, KIRQL, KAFFINITY)> {
    let mut got = None;
    free_device_vectors(|vector, irql| {
        got = Some((vector, irql));
        false
    });
    let (vector, irql) = got?;
    HalpVectorToIndex[vector as usize] = APIC_RESERVED_VECTOR;
    MessageVectors[vector as usize] = true;
    Some((vector as u32, irql, HalpDefaultInterruptAffinity))
}

/// Address 0xFEE0_0000 with the destination APIC ID in bits 19:12 (physical
/// mode, no redirection hint); data = the vector, fixed delivery, edge. The
/// destination is this processor, as HalEnableSystemInterrupt does for a line.
pub(crate) unsafe fn message_target(vector: u32) -> Option<(u64, u32)> {
    if !is_message_vector(vector) {
        return None;
    }
    let dest = (apic_read(APIC_ID) >> 24) as u64;
    Some((0xFEE0_0000 | (dest << 12), vector))
}

// ---------------------------------------------------------------------------
// ApicInitializeIOApic — map the IOAPIC MMIO page, mask all redir entries,
// init HalpVectorToIndex, then unmask the timer slot.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ApicInitializeIOApic() {
    // amd64 has one HARDWARE_PTE; i386 bindgen names the non-PAE layout
    // _HARDWARE_PTE_X86 (there is also a PAE variant), so alias per arch.
    #[cfg(target_arch = "x86_64")]
    use ntkrnl_sys::hal_ddk::PHARDWARE_PTE as HwPte;
    #[cfg(target_arch = "x86")]
    use ntkrnl_sys::hal_ddk::PHARDWARE_PTE_X86 as HwPte;

    // HalAddressToPte = MiAddressToPte (FORCEINLINE in halp.h → internal/<arch>/mm.h).
    // amd64: 4-level self-map, 8-byte PTEs, PTE_BASE 0xFFFFF68000000000.
    #[cfg(target_arch = "x86_64")]
    let pte: HwPte = {
        const PTE_BASE_AMD64: usize = 0xFFFFF68000000000;
        let offset = ((IOAPIC_BASE as u64) >> 9) & (0xFFFFFFFFFu64 << 3);
        (PTE_BASE_AMD64 + offset as usize) as HwPte
    };
    // i386 non-PAE: 2-level, 4-byte PTEs, PTE_BASE 0xC0000000.
    // MiAddressToPte(VA) = PTE_BASE + (VA >> 12) * 4.
    #[cfg(target_arch = "x86")]
    let pte: HwPte = {
        const PTE_BASE_I386: usize = 0xC0000000;
        (PTE_BASE_I386 + ((IOAPIC_BASE >> 12) << 2)) as HwPte
    };

    // PFN is 64-bit on amd64, 32-bit (ULONG) on i386 — cast to fit the setter.
    #[cfg(target_arch = "x86_64")]
    (*pte).set_PageFrameNumber(IOAPIC_PHYS_BASE / 4096);
    #[cfg(target_arch = "x86")]
    (*pte).set_PageFrameNumber((IOAPIC_PHYS_BASE / 4096) as u32);
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

    // Flavor banner — proves in the boot log that the i386 APIC HAL is live
    // (LAPIC/IOAPIC/TPR, lazy IRQL, RTC system clock).  i386 only; amd64 boot
    // path is unchanged.
    #[cfg(target_arch = "x86")]
    DbgPrint(b"[HAL] i386 APIC flavor active: LAPIC/IOAPIC/TPR, lazy-IRQL, RTC clock\n\0".as_ptr());

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
    // amd64: APC + dispatch handlers live in apic/apictrap.S.  i386: the HAL owns
    // the APC(0x3D)/DISPATCH(0x41) software vectors, so install their trap stubs
    // (apic.c:538-541).
    #[cfg(target_arch = "x86")]
    {
        register_interrupt_handler(APC_VECTOR, HalpApcInterrupt as _);
        register_interrupt_handler(DISPATCH_VECTOR, HalpDispatchInterrupt as _);
        // Kernel IPI receive (0xE1 -> KiIpiServiceRoutine): i386-only — the
        // amd64 kernel wires IDT[0xE1] itself (KiIpiInterrupt, ke/amd64).
        // Harmless on UP (nothing ever sends 0xE1); the AP inherits this via
        // the live-IDT copy in KeStartAllProcessors (ke/i386/mproc.rs).
        const APIC_IPI_VECTOR: ULONG = 0xE1;
        HalpVectorToIndex[APIC_IPI_VECTOR as usize] = APIC_RESERVED_VECTOR;
        register_interrupt_handler(APIC_IPI_VECTOR, HalpIpiInterrupt as _);
    }

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
    if is_message_vector(Vector) {
        return 1; // TRUE: nothing to unmask, the device's table entry is the gate
    }
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
    if is_message_vector(Vector) {
        return;
    }
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
    Vector: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    let current = apic_get_current_irql();

    // i386 lazy path (apic.c:753-793): a raise never touches the hard TPR, so an
    // interrupt can fire while the soft IRQL is already >= its level.  Catch that:
    // flush the real TPR, record it in the IRR shadow, EOI, and re-request the
    // interrupt via self-IPI so it re-fires once the IRQL drops — then report it
    // as spurious (FALSE).
    #[cfg(target_arch = "x86")]
    if APIC_LAZY_IRQL && current >= Irql {
        let pcr = KeGetPcr();
        apic_write(APIC_TPR, irql_to_tpr(current));
        (*pcr).IRR = current as ULONG;
        apic_send_eoi();
        let index = HalpVectorToIndex[Vector as usize];
        if (index as ULONG) < APIC_MAX_IRQ as ULONG {
            let redir = apic_read_io_redirection_entry(index);
            apic_request_self_interrupt(Vector as UCHAR, redir.trigger_mode() as UCHAR);
        } else {
            // Reserved (software) vector — edge-triggered self-IPI.
            apic_request_self_interrupt(Vector as UCHAR, APIC_TGM_EDGE as UCHAR);
        }
        return 0; // FALSE — pretend spurious
    }

    core::ptr::write_volatile(OldIrql, current);
    apic_raise_irql(Irql);
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

// ===========================================================================
// i386 software-interrupt trap handlers + IRQL primitives (apic.c:556-644,
// 823-887, all `#ifndef _M_AMD64`).  amd64 delivers APC/dispatch entirely in
// apictrap.S and gets the Kf/Ke IRQL functions from ntoskrnl, so none of this
// compiles there.  The trap stubs in apic/apictrap.S (`_HalpApcInterrupt` etc.)
// build the full KTRAP_FRAME then tail-call these `@..Handler@4` fastcall fns.
// ===========================================================================

// HalpApcInterruptHandler — apic.c:556-603.  DECLSPEC_NORETURN FASTCALL.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) -> ! {
    KiEnterInterruptTrap(TrapFrame);

    let mut old_irql: KIRQL = 0;
    if APIC_LAZY_IRQL {
        if HalBeginSystemInterrupt(APC_LEVEL, APC_VECTOR, &raw mut old_irql) == 0 {
            // "Spurious": HalBeginSystemInterrupt re-queued it; just exit.
            KiEoiHelper(TrapFrame);
        }
    } else {
        old_irql = apic_get_current_irql();
    }

    apic_raise_irql(APC_LEVEL);
    apic_send_eoi();

    // Kernel or user APC?
    let mode = if KiUserTrap(TrapFrame) != 0 || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
        USER_MODE
    } else {
        KERNEL_MODE
    };

    sti();
    KiDeliverApc(mode, core::ptr::null_mut(), TrapFrame);
    cli();

    apic_lower_irql(old_irql);
    KiEoiHelper(TrapFrame);
}

// HalpDispatchInterruptHandler — apic.c:605-644.  DECLSPEC_NORETURN FASTCALL.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpDispatchInterruptHandler(TrapFrame: PKTRAP_FRAME) -> ! {
    KiEnterInterruptTrap(TrapFrame);

    let mut old_irql: KIRQL = 0;
    if APIC_LAZY_IRQL {
        if HalBeginSystemInterrupt(DISPATCH_LEVEL, DISPATCH_VECTOR, &raw mut old_irql) == 0 {
            KiEoiHelper(TrapFrame);
        }
    } else {
        old_irql = apic_get_current_irql();
    }

    apic_raise_irql(DISPATCH_LEVEL);
    apic_send_eoi();

    sti();
    KiDispatchInterrupt();
    cli();

    apic_lower_irql(old_irql);
    KiEoiHelper(TrapFrame);
}

// KeGetCurrentIrql — apic.c:824-830.  NTAPI.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeGetCurrentIrql() -> KIRQL {
    apic_get_current_irql()
}

// KfLowerIrql — apic.c:832-847.  FASTCALL.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn KfLowerIrql(OldIrql: KIRQL) {
    apic_lower_irql(OldIrql);
}

// KfRaiseIrql — apic.c:849-871.  FASTCALL.  Returns the previous IRQL.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    let old_irql = apic_get_current_irql();
    apic_raise_irql(NewIrql);
    old_irql
}

// KeRaiseIrqlToDpcLevel — apic.c:873-878.  NTAPI.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeRaiseIrqlToDpcLevel() -> KIRQL {
    KfRaiseIrql(DISPATCH_LEVEL)
}

// KeRaiseIrqlToSynchLevel — apic.c:880-885.  NTAPI.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeRaiseIrqlToSynchLevel() -> KIRQL {
    KfRaiseIrql(SYNCH_LEVEL)
}
