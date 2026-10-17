//! i386 8259 PIC backend — translated from `hal/halx86/pic/pic.c`.
//!
//! Implements IRQL primitives (KeGetCurrentIrql / KfRaiseIrql / KfLowerIrql)
//! plus PIC initialization, software-interrupt request/clear/dispatch, and
//! the per-IRQ Dismiss handlers used by the trap stubs in `pic.S`.

#![cfg(target_arch = "x86")]
#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe, unused_imports, unused_variables, unused_mut)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, KAFFINITY, KIRQL, PVOID, UCHAR, ULONG, USHORT,
    PKAFFINITY, PKIRQL, PKPCR, PKTRAP_FRAME,
    EISA_ELCR, I8259_ISR, I8259_OCW2, I8259_OCW3, PIC_MASK,
    PHAL_DISMISS_INTERRUPT, PHAL_SW_INTERRUPT_HANDLER, PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY,
    _KINTERRUPT_MODE as KINTERRUPT_MODE,
    _KINTERRUPT_MODE_Latched, _KINTERRUPT_MODE_LevelSensitive,
};
use core::ffi::c_void;

// ntoskrnl-exported kernel helpers — declared as plain externs (no
// raw-dylib).  The linker resolves these via libntoskrnl.a's import-lib
// trampolines (e.g. `_KeBugCheck@4`, `@KiEoiHelper@4`), which in turn
// dispatch through hal.dll's IAT to the undecorated exports.  Per
// ntoskrnl.spec all `Ke/KiBugCheck*`/`KiDeliverApc`/`KiDispatchInterrupt`
// are NTAPI/stdcall on i386; KiEoiHelper is fastcall.  Using `extern
// "system"` (= stdcall on i386) generates the right `_Name@N` decoration.
unsafe extern "system" {
    fn KeBugCheck(BugCheckCode: ULONG);
    fn KeBugCheckEx(
        BugCheckCode: ULONG,
        P1: usize,
        P2: usize,
        P3: usize,
        P4: usize,
    );
    fn KiDeliverApc(
        PreviousMode: i8,
        ExceptionFrame: PVOID,
        TrapFrame: PKTRAP_FRAME,
    );
    fn KiDispatchInterrupt();
}
unsafe extern "fastcall" {
    // Noreturn: trap-exit iret trampoline. Must match timer.rs / bios.rs.
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
}

// `KiUserTrap` and `KiEnterInterruptTrap` are FORCEINLINE in
// ntoskrnl/include/internal/i386/trap_x.h:18,370 — never exported by
// ntoskrnl.spec.  We inline them here.
//
// trap_x.h:18  — `return !!(TrapFrame->SegCs & MODE_MASK);`  MODE_MASK = 1 (RPL).
#[inline(always)]
unsafe fn KiUserTrap(TrapFrame: PKTRAP_FRAME) -> BOOLEAN {
    if ((*TrapFrame).SegCs & 1) != 0 { 1 } else { 0 }
}

// trap_x.h:370.  The kernel build is DBG (compile log shows `-DDBG=1`), so we
// must replicate `KiFillTrapFrameDebug` — without it, `KiExitTrapDebugChecks`
// fires `Exiting a trap without restoring previous mode` and __debugbreak()s,
// which leaks back as a corrupt trap frame and a 0x6B PROCESS1_INITIALIZATION_FAILED.
//
//   ExceptionList = KeGetPcr()->NtTib.ExceptionList;        (assert only)
//   KeGetPcr()->NtTib.ExceptionList = EXCEPTION_CHAIN_END;  // -1
//   TrapFrame->Dr7 = 0;
//   if (KiUserTrap(...) || EFLAGS & V86) { /* debug-reg dance, skipped */ }
//   KiFillTrapFrameDebug(TrapFrame);   // DBG-only: sets DbgArg{Mark,Pointer},
//                                      // DbgEip, DbgEbp, PreviousPreviousMode=-1
const EXCEPTION_CHAIN_END: ULONG = !0;
#[inline(always)]
unsafe fn KiEnterInterruptTrap(TrapFrame: PKTRAP_FRAME) {
    let Pcr = KeGetPcr();
    (*Pcr).__bindgen_anon_1.NtTib.ExceptionList = EXCEPTION_CHAIN_END as usize as *mut _;
    (*TrapFrame).Dr7 = 0;
    // KiFillTrapFrameDebug — DBG-only in the C kernel, but our build is DBG.
    (*TrapFrame).DbgArgPointer = (*TrapFrame).Edx;
    (*TrapFrame).DbgArgMark = 0xBADB_0D00;
    (*TrapFrame).DbgEip = (*TrapFrame).Eip;
    (*TrapFrame).DbgEbp = (*TrapFrame).Ebp;
    (*TrapFrame).PreviousPreviousMode = !0;  // (ULONG)-1
}

// IRQL levels (from sdk/include/ndk/ketypes.h, FORCEINLINE in irql.h).
// These are #defines in C — bindgen doesn't expose them.
pub const PASSIVE_LEVEL: KIRQL  = 0;
pub const APC_LEVEL: KIRQL      = 1;
pub const DISPATCH_LEVEL: KIRQL = 2;
pub const CMCI_LEVEL: KIRQL     = 5;
pub const CLOCK1_LEVEL: KIRQL   = 28;
pub const CLOCK2_LEVEL: KIRQL   = 28;
pub const IPI_LEVEL: KIRQL      = 29;
pub const POWER_LEVEL: KIRQL    = 30;
pub const PROFILE_LEVEL: KIRQL  = 27;
pub use crate::x86::buildtype::SYNCH_LEVEL; // 2 on the UP HAL, 27 on the SMP HAL (i386)
pub const HIGH_LEVEL: KIRQL     = 31;

// EFLAGS bits.
const EFLAGS_INTERRUPT_MASK: ULONG = 0x200;
const EFLAGS_V86_MASK: ULONG       = 0x20000;

// 8259 PIC port-mapped I/O.
const PIC1_CONTROL_PORT: USHORT = 0x20;
const PIC1_DATA_PORT:    USHORT = 0x21;
const PIC2_CONTROL_PORT: USHORT = 0xA0;
const PIC2_DATA_PORT:    USHORT = 0xA1;
const EISA_ELCR_MASTER:  USHORT = 0x4D0;
const EISA_ELCR_SLAVE:   USHORT = 0x4D1;

// IRQ wiring constants.
const PIC_TIMER_IRQ:   ULONG = 0;
const PIC_CASCADE_IRQ: ULONG = 2;
const PRIMARY_VECTOR_BASE: ULONG = 0x30;

// IDT_INTERNAL is a HalpRegisterVector usage flag, halx86/include/halp.h:61 =
// 0x11 (IDT_REGISTERED|0x10). 2 is IDT_LATCHED, which left the cascade vector
// unregistered in HalpIDTUsageFlags.
const IDT_INTERNAL: ULONG = 0x11; // halp.h: IDT_INTERNAL 0x11, as the other hal copies; was 2
// _KGDT_R0_CODE selector value (0x08).
const KGDT_R0_CODE: USHORT = 0x08;

// I8259_READ_REQUEST enum values from halhw.h.
const ReadIdr: UCHAR = 2;
const ReadIsr: UCHAR = 3;

// PIC bugcheck codes used by KfRaiseIrql / KfLowerIrql in DBG builds.
const IRQL_NOT_GREATER_OR_EQUAL: ULONG = 0x0000_0009;
const IRQL_NOT_LESS_OR_EQUAL:    ULONG = 0x0000_000A;

// Tables defined in irql.rs (HAL component) — read-only at runtime.
// KiI8259MaskTable is ULONG[32] (C irql.c / irql.rs). A USHORT[32] extern here
// indexed at HALF the real stride: KiI8259MaskTable[16] read the low word of
// entry 8, so the level-dismiss pend path never masked IRQ11 at its own IRQL —
// a held PCI INTx line then EOI'd+refired forever (livelock in
// _HalpDismissIrqLevel; first level-triggered PIC interrupt ever exercised,
// 2026-07-10 AHCI CD bring-up).
unsafe extern "C" {
    static KiI8259MaskTable: [ULONG; 32];
    static FindHigherIrqlMask: [ULONG; 32];
}

// HalpDefaultInterruptAffinity is defined in hal/halx86/pic/processor.c
// (i386 legacy PIC backend) — stays C-side for now.
unsafe extern "C" {
    static mut HalpDefaultInterruptAffinity: KAFFINITY;
}

// `LONG HalpEisaELCR;` — pic.c:214.  Inside C `#ifndef _MINIHAL_` (pic.c:22).
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub static mut HalpEisaELCR: ULONG = 0;

// ============================================================================
// HalpHardwareInterrupt[0-15] — 16 per-IRQ "delayed" software-int stubs.
// pic.c:164-179 expands the `HalpDelayedHardwareInterrupt(N)` macro 16 times,
// each emits `int 0x30+N` inline.  Each stub is __cdecl (extern "C"), called
// indirectly via SWInterruptHandlerTable.
// ============================================================================
// C `HalpDelayedHardwareInterrupt(N)` expansions (pic.c:148-163) are inside
// `#ifndef _MINIHAL_` — gate every generated stub out of the minihal build.
macro_rules! hw_int_stub {
    ($name:ident, $vec:literal) => {
        #[cfg(not(feature = "minihal"))]
        #[no_mangle]
        pub unsafe extern "C" fn $name() {
            core::arch::asm!(concat!("int ", stringify!($vec)), options(nostack));
        }
    };
}
hw_int_stub!(HalpHardwareInterrupt0,  0x30);
hw_int_stub!(HalpHardwareInterrupt1,  0x31);
hw_int_stub!(HalpHardwareInterrupt2,  0x32);
hw_int_stub!(HalpHardwareInterrupt3,  0x33);
hw_int_stub!(HalpHardwareInterrupt4,  0x34);
hw_int_stub!(HalpHardwareInterrupt5,  0x35);
hw_int_stub!(HalpHardwareInterrupt6,  0x36);
hw_int_stub!(HalpHardwareInterrupt7,  0x37);
hw_int_stub!(HalpHardwareInterrupt8,  0x38);
hw_int_stub!(HalpHardwareInterrupt9,  0x39);
hw_int_stub!(HalpHardwareInterrupt10, 0x3A);
hw_int_stub!(HalpHardwareInterrupt11, 0x3B);
hw_int_stub!(HalpHardwareInterrupt12, 0x3C);
hw_int_stub!(HalpHardwareInterrupt13, 0x3D);
hw_int_stub!(HalpHardwareInterrupt14, 0x3E);
hw_int_stub!(HalpHardwareInterrupt15, 0x3F);

// ============================================================================
// File-scope tables — translated from pic.c:55-214.
// ============================================================================

// SWInterruptLookUpTable — pic.c:122.  Minimum IRQL needed to process pending
// SW interrupts in each IRR slot.  Inside C `#ifndef _MINIHAL_` (pic.c:22).
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub static mut SWInterruptLookUpTable: [KIRQL; 8] = [
    PASSIVE_LEVEL,   // IRR 0
    PASSIVE_LEVEL,   // IRR 1
    APC_LEVEL,       // IRR 2
    APC_LEVEL,       // IRR 3
    DISPATCH_LEVEL,  // IRR 4
    DISPATCH_LEVEL,  // IRR 5
    DISPATCH_LEVEL,  // IRR 6
    DISPATCH_LEVEL,  // IRR 7
];

// SWInterruptHandlerTable — pic.c:182.  Per-IRR-slot pending-int dispatcher.
// First 4 slots are SW interrupts; slots 4..20 are HW IRQ stubs.
//
// Cast from `unsafe extern "C" fn()` to `Option<unsafe extern "C" fn()>` is
// done via `Some(fn)` — the macro above generates the right ABI.
// Inside C `#ifndef _MINIHAL_` (pic.c:22).
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub static mut SWInterruptHandlerTable: [PHAL_SW_INTERRUPT_HANDLER; 20] = [
    Some(unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn()>(KiUnexpectedInterrupt as *const ()) }),
    Some(HalpApcInterrupt),
    Some(HalpDispatchInterrupt),
    Some(unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn()>(KiUnexpectedInterrupt as *const ()) }),
    Some(HalpHardwareInterrupt0),
    Some(HalpHardwareInterrupt1),
    Some(HalpHardwareInterrupt2),
    Some(HalpHardwareInterrupt3),
    Some(HalpHardwareInterrupt4),
    Some(HalpHardwareInterrupt5),
    Some(HalpHardwareInterrupt6),
    Some(HalpHardwareInterrupt7),
    Some(HalpHardwareInterrupt8),
    Some(HalpHardwareInterrupt9),
    Some(HalpHardwareInterrupt10),
    Some(HalpHardwareInterrupt11),
    Some(HalpHardwareInterrupt12),
    Some(HalpHardwareInterrupt13),
    Some(HalpHardwareInterrupt14),
    Some(HalpHardwareInterrupt15),
];

// SWInterruptHandlerTable2 — pic.c:207.  Pending-SW-int handlers reentered
// from inside the trap-frame path (fastcall, take TrapFrame ptr).
// Inside C `#ifndef _MINIHAL_` (pic.c:22).
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub static mut SWInterruptHandlerTable2: [PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY; 3] = [
    Some(unsafe {
        core::mem::transmute::<*const (), unsafe extern "fastcall" fn(PKTRAP_FRAME)>(
            KiUnexpectedInterrupt as *const (),
        )
    }),
    Some(HalpApcInterrupt2ndEntry),
    Some(HalpDispatchInterrupt2ndEntry),
];

// HalpSpecialDismissTable — pic.c:55.  Per-IRQ initial Dismiss routine (edge).
// HalpInitializePICs may overwrite specific slots with the level variants if
// the ELCR shows an IRQ is level-triggered.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub static mut HalpSpecialDismissTable: [PHAL_DISMISS_INTERRUPT; 16] = [
    Some(HalpDismissIrqGeneric),  // 0
    Some(HalpDismissIrqGeneric),  // 1
    Some(HalpDismissIrqGeneric),  // 2
    Some(HalpDismissIrqGeneric),  // 3
    Some(HalpDismissIrqGeneric),  // 4
    Some(HalpDismissIrqGeneric),  // 5
    Some(HalpDismissIrqGeneric),  // 6
    Some(HalpDismissIrq07),       // 7
    Some(HalpDismissIrqGeneric),  // 8
    Some(HalpDismissIrqGeneric),  // 9
    Some(HalpDismissIrqGeneric),  // 10
    Some(HalpDismissIrqGeneric),  // 11
    Some(HalpDismissIrqGeneric),  // 12
    Some(HalpDismissIrq13),       // 13
    Some(HalpDismissIrqGeneric),  // 14
    Some(HalpDismissIrq15),       // 15
];

// HalpSpecialDismissLevelTable — pic.c:87.  Same shape but with the *Level
// variants for level-triggered IRQs.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub static HalpSpecialDismissLevelTable: [PHAL_DISMISS_INTERRUPT; 16] = [
    Some(HalpDismissIrqLevel),       // 0
    Some(HalpDismissIrqLevel),       // 1
    Some(HalpDismissIrqLevel),       // 2
    Some(HalpDismissIrqLevel),       // 3
    Some(HalpDismissIrqLevel),       // 4
    Some(HalpDismissIrqLevel),       // 5
    Some(HalpDismissIrqLevel),       // 6
    Some(HalpDismissIrq07Level),     // 7
    Some(HalpDismissIrqLevel),       // 8
    Some(HalpDismissIrqLevel),       // 9
    Some(HalpDismissIrqLevel),       // 10
    Some(HalpDismissIrqLevel),       // 11
    Some(HalpDismissIrqLevel),       // 12
    Some(HalpDismissIrq13Level),     // 13
    Some(HalpDismissIrqLevel),       // 14
    Some(HalpDismissIrq15Level),     // 15
];

// Dismiss handlers are defined later in this file with `extern "system"`
// (= stdcall on i386).  But PHAL_DISMISS_INTERRUPT is
// `Option<unsafe extern "stdcall" fn ...>` and Rust treats stdcall/system as
// different *types* even though the ABI is identical, so the `Some(fn)`
// constructor would mismatch.  Switch the definitions below to extern
// "stdcall" so the names line up — no transmute needed.

// KiUnexpectedInterrupt[0-31] are ntoskrnl exports (KiUnexpectedInterruptN);
// the table-entry casts above use the bare `KiUnexpectedInterrupt` symbol
// the same way the original C code did.  Per ntoskrnl.spec it's `cdecl
// -arch=i386,arm`, so use `extern "C"` (cdecl on i386 — generates `_Name`
// import which the libntoskrnl.a trampoline resolves).
unsafe extern "C" {
    fn KiUnexpectedInterrupt();
}

// HalpApcInterrupt2ndEntry / HalpDispatchInterrupt2ndEntry are defined
// further down in this file with `extern "fastcall"`; the static
// initializers above name them directly — no extern decl needed.

// `KeGetPcr` is FORCEINLINE in C (reads fs:[0x1C]); inline asm replica.
#[inline(always)]
unsafe fn KeGetPcr() -> PKPCR {
    let pcr: PKPCR;
    core::arch::asm!("mov {}, fs:[0x1C]", out(reg) pcr, options(nomem, preserves_flags));
    pcr
}

// MSVC intrinsics — clang lowers these to inline asm at C-side use; on the
// Rust side there's no linkable symbol, so we provide our own inline-asm
// replicas with the same names.  All `#[inline(always)]` so the call sites
// emit the asm directly.
#[inline(always)]
unsafe fn __readeflags() -> ULONG {
    let f: u32;
    core::arch::asm!("pushfd", "pop {:e}", out(reg) f, options(nomem, preserves_flags));
    f
}

#[inline(always)]
unsafe fn __writeeflags(eflags: ULONG) {
    core::arch::asm!("push {:e}", "popfd", in(reg) eflags, options(nomem));
}

#[inline(always)]
unsafe fn __inbyte(port: USHORT) -> UCHAR {
    let v: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") v, options(nostack, preserves_flags));
    v
}

#[inline(always)]
unsafe fn __outbyte(port: USHORT, value: UCHAR) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
}

#[inline(always)]
unsafe fn _disable() {
    core::arch::asm!("cli", options(nomem, nostack));
}

#[inline(always)]
unsafe fn _enable() {
    core::arch::asm!("sti", options(nomem, nostack));
}

// BitScanReverse(Index*, Mask) — sets *Index to bit position of MSB and
// returns nonzero if Mask != 0.  Models the MSVC intrinsic.
#[inline(always)]
unsafe fn BitScanReverse(index: *mut ULONG, mask: ULONG) -> UCHAR {
    if mask == 0 { return 0; }
    let result: u32;
    core::arch::asm!("bsr {0:e}, {1:e}", out(reg) result, in(reg) mask, options(nomem, nostack, preserves_flags));
    *index = result;
    1
}

// HAL registration helper — lives in pic_halinit.rs.
unsafe extern "system" {
    fn HalpRegisterVector(Flags: UCHAR, BusVector: ULONG, SystemVector: ULONG, Irql: KIRQL);
    fn HalpInitializeLegacyPICs();
}

// HalpEndSoftwareInterrupt: defined in pic.S as `_HalpEndSoftwareInterrupt@8`
// (NTAPI/stdcall) — it's a stdcall trampoline that calls HalpEndSoftwareInterrupt2
// (fastcall) and conditionally tail-jumps into the returned handler ptr.
unsafe extern "stdcall" {
    fn HalpEndSoftwareInterrupt(OldIrql: KIRQL, TrapFrame: PKTRAP_FRAME);
}

// SW interrupt handlers and Dismiss-IRQ functions exposed by pic.S +
// other files — pulled in via spec linkage.  These are PHAL_SW_INTERRUPT_HANDLER
// = Option<unsafe extern "C" fn()>; declare with matching ABI so they can go
// straight into the SWInterruptHandlerTable.
unsafe extern "C" {
    fn HalpApcInterrupt();
    fn HalpDispatchInterrupt();
    fn HalpHardwareInterruptLevel();
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpInitializePICs(EnableInterrupts: BOOLEAN) {

    let mut eflags: ULONG;
    let mut elcr: EISA_ELCR = core::mem::zeroed();
    let i: ULONG;
    let j: ULONG;
    let elcr_found: BOOLEAN;

    eflags = unsafe { __readeflags() };
    unsafe { _disable() };

    unsafe { HalpInitializeLegacyPICs() };

    elcr.Bits = (unsafe { __inbyte(0x4D1) } as USHORT) << 8 | unsafe { __inbyte(0x4D0) } as USHORT;

    #[cfg(not(target_feature = "pc98"))]
    {
        elcr_found = ((elcr.__bindgen_anon_1.Master.Irq0Level() == 0)
            && (elcr.__bindgen_anon_1.Master.Irq1Level() == 0)
            && (elcr.__bindgen_anon_1.Master.Irq2Level() == 0)
            && (elcr.__bindgen_anon_1.Slave.Irq8Level() == 0)
            && (elcr.__bindgen_anon_1.Slave.Irq13Level() == 0)) as BOOLEAN;
    }

    #[cfg(target_feature = "pc98")]
    {
        if elcr.Bits == 0xFFFF {
            elcr.__bindgen_anon_1.Master.set_Irq0Level(0);
            elcr.__bindgen_anon_1.Master.set_Irq1Level(0);
            elcr.__bindgen_anon_1.Master.set_Irq7Level(0);
            elcr.__bindgen_anon_1.Slave.set_Irq8Level(0);
        }
        elcr_found = 1;
    }

    if elcr_found != 0 {
        HalpEisaELCR = elcr.Bits as ULONG;

        let mut i: ULONG = 1;
        let mut j: ULONG = 0;
        while j < 16 {
            if (HalpEisaELCR as ULONG & i) != 0 {
                SWInterruptHandlerTable[(j + 4) as usize] = Some(HalpHardwareInterruptLevel);

                HalpSpecialDismissTable[j as usize] = HalpSpecialDismissLevelTable[j as usize];
            }
            i <<= 1;
            j += 1;
        }
    }

    unsafe {
        HalpRegisterVector(
            IDT_INTERNAL as UCHAR,
            PRIMARY_VECTOR_BASE + PIC_CASCADE_IRQ,
            PRIMARY_VECTOR_BASE + PIC_CASCADE_IRQ,
            HIGH_LEVEL,
        )
    };

    if EnableInterrupts != 0 {
        eflags |= EFLAGS_INTERRUPT_MASK;
    }
    unsafe { __writeeflags(eflags) };
}

// HalpIrqToVector — pic.c:256-261.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpIrqToVector(Irq: UCHAR) -> UCHAR {
    (PRIMARY_VECTOR_BASE + Irq as ULONG) as UCHAR
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpIrqToVector(Irq: UCHAR) -> UCHAR {
    (PRIMARY_VECTOR_BASE + Irq as ULONG) as UCHAR
}

// HalpVectorToIrq — pic.c:263-267.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpVectorToIrq(Vector: UCHAR) -> UCHAR {
    Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR)
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpVectorToIrq(Vector: UCHAR) -> UCHAR {
    Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR)
}

// HalpVectorToIrql — pic.c:269-279.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpVectorToIrql(Vector: u8) -> KIRQL {
    PROFILE_LEVEL.wrapping_sub(Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR) as KIRQL)
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpVectorToIrql(Vector: u8) -> KIRQL {
    PROFILE_LEVEL.wrapping_sub(Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR) as KIRQL)
}

// KeRaiseIrqlToDpcLevel — pic.c:292-300.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn KeRaiseIrqlToDpcLevel() -> KIRQL {
    
    let pcr = unsafe { KeGetPcr() };
    let current_irql: KIRQL;

    current_irql = unsafe { (*pcr).Irql };
    unsafe { (*pcr).Irql = DISPATCH_LEVEL as KIRQL };

    current_irql
}

// KeRaiseIrqlToSynchLevel — pic.c:318-347.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn KeRaiseIrqlToSynchLevel() -> KIRQL {
    
    let pcr = unsafe { KeGetPcr() };
    let current_irql: KIRQL;

    unsafe {
        current_irql = (*pcr).Irql;
        (*pcr).Irql = SYNCH_LEVEL as KIRQL;
    }

    #[cfg(debug_assertions)]
    unsafe {
        if current_irql > SYNCH_LEVEL as KIRQL {
            KeBugCheckEx(
                ntkrnl_sys::bugcodes::IRQL_NOT_GREATER_OR_EQUAL,
                current_irql as usize,
                SYNCH_LEVEL as usize,
                0,
                1,
            );
        }
    }

    current_irql
}

// HalRequestSoftwareInterrupt — pic.c:434-457.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalRequestSoftwareInterrupt(Irql: KIRQL) {
    

    let eflags = __readeflags();
    _disable();

    let pcr = KeGetPcr();
    (*pcr).IRR |= 1 << Irql;

    let pending_irql = SWInterruptLookUpTable[((*pcr).IRR & 3) as usize];
    if pending_irql > (*pcr).Irql {
        if let Some(handler) = SWInterruptHandlerTable[pending_irql as usize] {
            handler();
        }
    }

    __writeeflags(eflags);
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalRequestSoftwareInterrupt(Irql: KIRQL) {
    

    let eflags = __readeflags();
    _disable();

    let pcr = KeGetPcr();
    (*pcr).IRR |= 1 << Irql;

    let pending_irql = SWInterruptLookUpTable[((*pcr).IRR & 3) as usize];
    if pending_irql > (*pcr).Irql {
        if let Some(handler) = SWInterruptHandlerTable[pending_irql as usize] {
            handler();
        }
    }

    __writeeflags(eflags);
}

// HalClearSoftwareInterrupt — pic.c:460-465.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalClearSoftwareInterrupt(Irql: KIRQL) {
    unsafe {
        (*KeGetPcr()).IRR &= !(1 << Irql);
    }
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalClearSoftwareInterrupt(Irql: KIRQL) {
    unsafe {
        (*KeGetPcr()).IRR &= !(1 << Irql);
    }
}

// HalpEndSoftwareInterrupt2 — pic.c:468-523.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpEndSoftwareInterrupt2(
    OldIrql: KIRQL,
    TrapFrame: PKTRAP_FRAME,
) -> PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY {
    let _ = TrapFrame;

    let Pcr = unsafe { KeGetPcr() };
    let mut PendingIrql: ULONG = 0;
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG;
    let mut Mask: PIC_MASK = unsafe { core::mem::zeroed() };

    unsafe { (*Pcr).Irql = OldIrql };

    loop {
        PendingIrqlMask = unsafe { (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize] };
        if PendingIrqlMask == 0 {
            return None;
        }

        if unsafe { (*Pcr).IrrActive & 0xFFFFFFF0 != 0 } {
            return None;
        }

        unsafe { BitScanReverse(&raw mut PendingIrql, PendingIrqlMask) };
        if PendingIrql > DISPATCH_LEVEL as ULONG {
            Mask.Both = unsafe { ((*Pcr).IDR & 0xFFFF) as USHORT };
            unsafe {
                write_port_uchar(PIC1_DATA_PORT as *mut u8, Mask.__bindgen_anon_1.Master);
                write_port_uchar(PIC2_DATA_PORT as *mut u8, Mask.__bindgen_anon_1.Slave);
            }

            PendingIrqMask = 1 << PendingIrql;
            unsafe {
                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;
            }

            unsafe {
                let handler = SWInterruptHandlerTable[PendingIrql as usize];
                if let Some(f) = handler {
                    f();
                }
            }

            unsafe { (*Pcr).IrrActive ^= PendingIrqMask };
        } else {
            return unsafe { SWInterruptHandlerTable2[PendingIrql as usize] };
        }
    }
}

// Below: dismiss-IRQ machinery + interrupt dispatch — all inside C
// `#ifndef _MINIHAL_` (pic.c:22-1168).  Private helpers gated too so the
// minihal build doesn't emit unused-fn churn.
#[cfg(not(feature = "minihal"))]
#[inline(always)]
unsafe fn write_port_uchar(port: *mut u8, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port as u16,
            in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

// _HalpDismissIrqGeneric — pic.c:525-579.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
unsafe fn _HalpDismissIrqGeneric(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    

    let Pcr = KeGetPcr();

    let CurrentIrql = (*Pcr).Irql;

    if Irql > CurrentIrql {
        (*Pcr).Irql = Irql;
        *OldIrql = CurrentIrql;

        let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
        Ocw2.Bits = (0);
        Ocw2.__bindgen_anon_1.set_EoiMode(3);

        if Irq >= 8 {
            #[cfg(not(target_feature = "pc98"))]
            {
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);
                __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
            }

            #[cfg(target_feature = "pc98")]
            {
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);

                let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
                Ocw3.Bits = (0);
                Ocw3.__bindgen_anon_1.set_Sbo(1);
                Ocw3.__bindgen_anon_1.set_ReadRequest(3);
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits);

                let mut Isr: I8259_ISR = core::mem::zeroed();
                Isr.Bits = (__inbyte(PIC2_CONTROL_PORT as u16));

                if Isr.Bits == 0 {
                    __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
                }
            }
        } else {
            __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | (Irq & 0xFF) as u8);
        }

        _enable();
        return 1;
    }

    (*Pcr).IRR |= 1 << (Irq + 4);

    let mut Mask: PIC_MASK = core::mem::zeroed();
    Mask.Both = ((KiI8259MaskTable[CurrentIrql as usize] | (*Pcr).IDR) & 0xFFFF) as USHORT;
    __outbyte(PIC1_DATA_PORT as u16, Mask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, Mask.__bindgen_anon_1.Slave);

    0
}

// HalpDismissIrqGeneric — pic.c:581-589.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrqGeneric(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe { _HalpDismissIrqGeneric(Irql, Irq, OldIrql) }
}

// HalpDismissIrq15 — pic.c:591-624.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq15(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {

    let mut ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut ocw2: I8259_OCW2 = core::mem::zeroed();
    let mut isr: I8259_ISR;

    ocw3.Bits = (0);
    ocw3.__bindgen_anon_1.set_Sbo(1);
    ocw3.__bindgen_anon_1.set_ReadRequest(3);
    __outbyte(PIC2_CONTROL_PORT as u16, ocw3.Bits);

    isr = core::mem::zeroed();
    isr.Bits = (__inbyte(PIC2_CONTROL_PORT as u16));

    if isr.__bindgen_anon_1.Irq7() == 0 {
        ocw2.Bits = (0);
        ocw2.__bindgen_anon_1.set_EoiMode(1);
        __outbyte(PIC1_CONTROL_PORT as u16, ocw2.Bits | PIC_CASCADE_IRQ as u8);

        return 0;
    }

    _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
}

// HalpDismissIrq13 — pic.c:626-638.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq13(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    
    
    unsafe {
        __outbyte(0xF0, 0);
        _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
    }
}

// HalpDismissIrq08 — companion to the pic.c:642 HalpDismissIrq07 family;
// part of the dismiss machinery inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq08(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {

    const CPU_IO_o_FPU_BUSY_LATCH: u16 = 0xF0;

    unsafe {
        __outbyte(CPU_IO_o_FPU_BUSY_LATCH, 0);
        _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
    }
}

// HalpDismissIrq07 — pic.c:640-666.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq07(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {


    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.Bits = (0);
    Ocw3.__bindgen_anon_1.set_Sbo(1);
    Ocw3.__bindgen_anon_1.set_ReadRequest(ReadIsr as u8);
    __outbyte(PIC1_CONTROL_PORT as u16, Ocw3.Bits);

    Isr.Bits = (__inbyte(PIC1_CONTROL_PORT as u16));

    if Isr.__bindgen_anon_1.Irq7() == 0 {
        return 0;
    }

    _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
}

// _HalpDismissIrqLevel — pic.c:668-722.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
unsafe fn _HalpDismissIrqLevel(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {

    let Pcr = KeGetPcr();
    
    let mut Mask: PIC_MASK = core::mem::zeroed();
    Mask.Both = ((KiI8259MaskTable[Irql as usize] as ULONG | (*Pcr).IDR) & 0xFFFF) as USHORT;
    __outbyte(PIC1_DATA_PORT as u16, Mask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, Mask.__bindgen_anon_1.Slave);
    
    (*Pcr).IRR |= 1 << (Irq + 4);
    
    let CurrentIrql = (*Pcr).Irql;
    
    let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
    Ocw2.__bindgen_anon_1.set_EoiMode(3);
    
    if Irq >= 8 {
        #[cfg(target_feature = "pc98")]
        {
            let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
            let Isr: I8259_ISR;
            
            __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);
            
            Ocw3.__bindgen_anon_1.set_Sbo(1);
            Ocw3.__bindgen_anon_1.set_ReadRequest(3);
            __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits);
            
            Isr.Bits = __inbyte(PIC2_CONTROL_PORT as u16);
            
            if Isr.Bits == 0 {
                __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
            }
        }
        
        #[cfg(not(target_feature = "pc98"))]
        {
            __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);
            __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
        }
    } else {
        __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | (Irq & 0xFF) as u8);
    }
    
    if Irql > CurrentIrql {
        (*Pcr).Irql = Irql;
        *OldIrql = CurrentIrql;
        
        _enable();
        return 1;
    }
    
    0
}

// HalpDismissIrqLevel — pic.c:724-732.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrqLevel(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe { _HalpDismissIrqLevel(Irql, Irq, OldIrql) }
}

// HalpDismissIrq15Level — pic.c:734-767.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq15Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {

    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.Bits = (0);
    Ocw3.__bindgen_anon_1.set_Sbo(1);
    Ocw3.__bindgen_anon_1.set_ReadRequest(0x3);

    unsafe { __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits) };

    Isr.Bits = (unsafe { __inbyte(PIC2_CONTROL_PORT as u16) });

    if Isr.__bindgen_anon_1.Irq7() == 0 {
        Ocw2.Bits = (0);
        Ocw2.__bindgen_anon_1.set_EoiMode(0x1);
        unsafe { __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8) };

        return 0;
    }

    unsafe { _HalpDismissIrqLevel(Irql, Irq, OldIrql) }
}

// HalpDismissIrq13Level — pic.c:769-781.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq13Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {


    unsafe {
        __outbyte(0xF0, 0);
        _HalpDismissIrqLevel(Irql, Irq, OldIrql)
    }
}

// HalpDismissIrq08Level — companion to pic.c:785 HalpDismissIrq07Level
// family; part of the dismiss machinery inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq08Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {

    const CPU_IO_o_FPU_BUSY_LATCH: u16 = 0xF0;

    unsafe {
        __outbyte(CPU_IO_o_FPU_BUSY_LATCH, 0);
    }

    unsafe { _HalpDismissIrqLevel(Irql, Irq, OldIrql) }
}

// HalpDismissIrq07Level — pic.c:783-808.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "stdcall" fn HalpDismissIrq07Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {

    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.Bits = (0);
    Ocw3.__bindgen_anon_1.set_Sbo(1);
    Ocw3.__bindgen_anon_1.set_ReadRequest(3);

    unsafe {
        __outbyte(PIC1_CONTROL_PORT as u16, Ocw3.Bits);
    }

    unsafe {
        Isr.Bits = (__inbyte(PIC1_CONTROL_PORT as u16));
    }

    if Isr.__bindgen_anon_1.Irq7() == 0 {
        return 0;
    }

    _HalpDismissIrqLevel(Irql, Irq, OldIrql)
}

// HalpHardwareInterruptLevel2 — pic.c:810-838.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "C" fn HalpHardwareInterruptLevel2() -> PHAL_SW_INTERRUPT_HANDLER {

    let pcr: PKPCR = KeGetPcr();
    let mut pending_irql: ULONG = 0;

    let pending_irql_mask = (*pcr).IRR & FindHigherIrqlMask[(*pcr).Irql as usize];
    if pending_irql_mask != 0 {
        if (*pcr).IrrActive & 0xFFFFFFF0 != 0 {
            return None;
        }

        BitScanReverse(&raw mut pending_irql, pending_irql_mask);

        (*pcr).IRR ^= 1 << pending_irql;

        return SWInterruptHandlerTable[pending_irql as usize];
    }

    None
}

// HalEnableSystemInterrupt — pic.c:840-881.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalEnableSystemInterrupt(
    Vector: ULONG,
    Irql: KIRQL,
    InterruptMode: KINTERRUPT_MODE,
) -> BOOLEAN {



    let Irq: ULONG;
    let Pcr: PKPCR = KeGetPcr();
    let mut PicMask: PIC_MASK = core::mem::zeroed();

    Irq = Vector.wrapping_sub(PRIMARY_VECTOR_BASE) as ULONG;
    if Irq >= CLOCK2_LEVEL as ULONG {
        return 0;
    }

    if InterruptMode == _KINTERRUPT_MODE_LevelSensitive {
        SWInterruptHandlerTable[(Irq + 4) as usize] = Some(HalpHardwareInterruptLevel);

        HalpSpecialDismissTable[Irq as usize] = HalpSpecialDismissLevelTable[Irq as usize];
    }

    _disable();

    (*Pcr).IDR &= !(1 << Irq);

    PicMask.Both = ((KiI8259MaskTable[(*Pcr).Irql as usize] as ULONG | (*Pcr).IDR) & 0xFFFF) as USHORT;
    __outbyte(PIC1_DATA_PORT as u16, PicMask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, PicMask.__bindgen_anon_1.Slave);

    _enable();
    1
}

// HalDisableSystemInterrupt — pic.c:883-926.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalDisableSystemInterrupt(
    Vector: ULONG,
    Irql: KIRQL,
) {

    let mut irq_mask: ULONG;
    let mut pic_mask: PIC_MASK;

    if (Vector as ULONG) == PRIMARY_VECTOR_BASE + PIC_TIMER_IRQ {
        __outbyte(0xE9, b'!');
        __outbyte(0xE9, b'P');
        __outbyte(0xE9, b'I');
        __outbyte(0xE9, b'T');
        __outbyte(0xE9, b'!');
    }

    irq_mask = 1u32 << Vector.wrapping_sub(PRIMARY_VECTOR_BASE) as ULONG;

    _disable();

    (*KeGetPcr()).IDR |= irq_mask;

    pic_mask = core::mem::zeroed();
    pic_mask.__bindgen_anon_1.Master = __inbyte(PIC1_DATA_PORT as u16);
    pic_mask.__bindgen_anon_1.Slave = __inbyte(PIC2_DATA_PORT as u16);

    pic_mask.Both |= irq_mask as USHORT;

    __outbyte(PIC1_DATA_PORT as u16, pic_mask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, pic_mask.__bindgen_anon_1.Slave);

    _enable();
}

// HalBeginSystemInterrupt — pic.c:928-943.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalBeginSystemInterrupt(
    Irql: KIRQL,
    Vector: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    let Irq: ULONG;

    Irq = Vector.wrapping_sub(PRIMARY_VECTOR_BASE) as ULONG;
    unsafe {
        HalpSpecialDismissTable[Irq as usize].unwrap()(Irql, Irq, OldIrql)
    }
}

// HalEndSystemInterrupt2 — pic.c:945-1005.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalEndSystemInterrupt2(
    OldIrql: KIRQL,
    TrapFrame: PKTRAP_FRAME,
) -> PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY {
    let _ = TrapFrame;



    let mut PendingIrql: ULONG = 0;
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG = 0;
    let Pcr: PKPCR = KeGetPcr();
    let mut Mask: PIC_MASK = core::mem::zeroed();

    (*Pcr).Irql = OldIrql;

    PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        if ((*Pcr).IrrActive & 0xFFFFFFF0) != 0 {
            return None;
        }

        loop {
            BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
            if PendingIrql > DISPATCH_LEVEL as ULONG {
                Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
                __outbyte(PIC1_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Master);
                __outbyte(PIC2_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Slave);

                PendingIrqMask = 1 << PendingIrql;
                if ((*Pcr).IrrActive & PendingIrqMask) != 0 {
                    return None;
                }

                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;

                (SWInterruptHandlerTable[PendingIrql as usize].unwrap())();

                (*Pcr).IrrActive ^= PendingIrqMask;

                PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[(*Pcr).Irql as usize];
                if PendingIrqlMask == 0 {
                    break;
                }
            } else {
                return SWInterruptHandlerTable2[PendingIrql as usize];
            }
        }
    }

    None
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalEndSystemInterrupt2(
    OldIrql: KIRQL,
    TrapFrame: PKTRAP_FRAME,
) -> PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY {
    let _ = TrapFrame;



    let mut PendingIrql: ULONG = 0;
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG = 0;
    let Pcr: PKPCR = KeGetPcr();
    let mut Mask: PIC_MASK = core::mem::zeroed();

    (*Pcr).Irql = OldIrql;

    PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        if ((*Pcr).IrrActive & 0xFFFFFFF0) != 0 {
            return None;
        }

        loop {
            BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
            if PendingIrql > DISPATCH_LEVEL as ULONG {
                Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
                __outbyte(PIC1_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Master);
                __outbyte(PIC2_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Slave);

                PendingIrqMask = 1 << PendingIrql;
                if ((*Pcr).IrrActive & PendingIrqMask) != 0 {
                    return None;
                }

                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;

                (SWInterruptHandlerTable[PendingIrql as usize].unwrap())();

                (*Pcr).IrrActive ^= PendingIrqMask;

                PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[(*Pcr).Irql as usize];
                if PendingIrqlMask == 0 {
                    break;
                }
            } else {
                return SWInterruptHandlerTable2[PendingIrql as usize];
            }
        }
    }

    core::ptr::null_mut()
}

// _HalpApcInterruptHandler — pic.c:1007-1034 (calls KiDeliverApc/KiEoiHelper).
// Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn _HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) {


    let Pcr: PKPCR = KeGetPcr();
    
    let CurrentIrql: KIRQL = (*Pcr).Irql;
    (*Pcr).Irql = APC_LEVEL as KIRQL;
    
    (*Pcr).IRR &= !(1 << APC_LEVEL);
    
    _enable();
    KiDeliverApc(
        (if KiUserTrap(TrapFrame) != 0 || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 { 1i8 } else { 0i8 }),
        core::ptr::null_mut(),
        TrapFrame,
    );

    _disable();
    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);

    KiEoiHelper(TrapFrame);
}

// HalpApcInterrupt2ndEntry — pic.c:1036-1043.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpApcInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {
    unsafe {
        _HalpApcInterruptHandler(TrapFrame);
    }
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpApcInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {
    unsafe {
        _HalpApcInterruptHandler(TrapFrame);
    }
}

// HalpApcInterruptHandler — pic.c:1045-1061.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) {


    (*TrapFrame).EFlags = __readeflags();
    (*TrapFrame).SegCs = KGDT_R0_CODE as u32;
    (*TrapFrame).Eip = (*TrapFrame).Eax;

    KiEnterInterruptTrap(TrapFrame);

    _HalpApcInterruptHandler(TrapFrame);
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) {


    (*TrapFrame).EFlags = __readeflags() as u32;
    (*TrapFrame).SegCs = KGDT_R0_CODE as u32;
    (*TrapFrame).Eip = (*TrapFrame).Eax as u32;

    KiEnterInterruptTrap(TrapFrame);

    _HalpApcInterruptHandler(TrapFrame);
}

// _HalpDispatchInterruptHandler — pic.c:1063-1082 (calls KiDispatchInterrupt).
// Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
unsafe fn _HalpDispatchInterruptHandler() -> KIRQL {

    let Pcr = KeGetPcr();

    let CurrentIrql = (*Pcr).Irql;
    (*Pcr).Irql = DISPATCH_LEVEL as KIRQL;

    (*Pcr).IRR &= !(1 << DISPATCH_LEVEL);

    _enable();
    KiDispatchInterrupt();
    _disable();

    CurrentIrql
}

// HalpDispatchInterrupt2ndEntry — pic.c:1085-1101 (calls KiEoiHelper).
// Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpDispatchInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {

    let CurrentIrql = _HalpDispatchInterruptHandler();

    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);

    KiEoiHelper(TrapFrame);
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpDispatchInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {

    let CurrentIrql = _HalpDispatchInterruptHandler();

    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);

    KiEoiHelper(TrapFrame);
}

// HalpDispatchInterrupt2 — pic.c:1103-1139.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "C" fn HalpDispatchInterrupt2() -> PHAL_SW_INTERRUPT_HANDLER {


    let Pcr = KeGetPcr();

    let OldIrql = _HalpDispatchInterruptHandler();

    (*Pcr).Irql = OldIrql;

    let PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        let mut PendingIrql: ULONG = 0;
        BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
        if PendingIrql > DISPATCH_LEVEL as ULONG {
            let mut Mask: PIC_MASK = core::mem::zeroed();
            Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
            __outbyte(PIC1_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Master);
            __outbyte(PIC2_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Slave);

            (*Pcr).IRR ^= 1 << PendingIrql;
        }

        return SWInterruptHandlerTable[PendingIrql as usize];
    }

    None
}

// HalpGetRootInterruptVector — pic.c:1141-1168.  Inside C `#ifndef _MINIHAL_`.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpGetRootInterruptVector(
    BusInterruptLevel: ULONG,
    BusInterruptVector: ULONG,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    let _ = BusInterruptVector;

    if BusInterruptLevel > 23 {
        return 0;
    }

    #[cfg(target_arch = "x86")]
    let system_vector = unsafe {
        HalpIrqToVector(BusInterruptLevel as UCHAR)
    };

    #[cfg(target_arch = "x86_64")]
    let system_vector = unsafe {
        HalpIrqToVector(BusInterruptLevel as UCHAR)
    };

    #[cfg(target_arch = "x86")]
    unsafe {
        *Irql = HalpVectorToIrql(system_vector);
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        *Irql = HalpVectorToIrql(system_vector);
    }


    unsafe {
        core::ptr::write_unaligned(Affinity, HalpDefaultInterruptAffinity);
    }

    system_vector as ULONG
}

// ============================================================================
// KeGetCurrentIrql / KfRaiseIrql / KfLowerIrql — the ONLY three symbols pic.c
// provides in BOTH faces.  Full bodies are inside C `#ifndef _MINIHAL_`
// (pic.c:281-287, 349-376, 378-432); the C `#else /* _MINIHAL_ */` block
// (pic.c:1170-1192) provides trivial stubs.  Cfg-split per Rule 2.
// ============================================================================

/// Returns the current IRQL by reading PCR.Irql.  pic.c:281-287 (non-minihal).
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> KIRQL {
    unsafe { (*KeGetPcr()).Irql }
}

/// Minihal stub — pic.c:1172-1177 `return PASSIVE_LEVEL;` (NTAPI).
#[cfg(feature = "minihal")]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> KIRQL {
    PASSIVE_LEVEL as KIRQL
}

/// Raises the IRQL to NewIrql; returns the previous IRQL.  pic.c:349-376
/// (non-minihal).  Caller must ensure NewIrql >= CurrentIrql (DBG bugcheck).
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "fastcall" fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    let Pcr = unsafe { KeGetPcr() };
    let CurrentIrql = unsafe { (*Pcr).Irql };
    unsafe { (*Pcr).Irql = NewIrql };
    CurrentIrql
}

/// Minihal stub — pic.c:1186-1192 `return NewIrql;` (FASTCALL).
#[cfg(feature = "minihal")]
#[no_mangle]
pub unsafe extern "fastcall" fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    NewIrql
}

/// Minihal stub — pic.c:1179-1184 empty body (FASTCALL).
#[cfg(feature = "minihal")]
#[no_mangle]
pub unsafe extern "fastcall" fn KfLowerIrql(_OldIrql: KIRQL) {
}

/// Lowers the IRQL to OldIrql and dispatches any pending software interrupts
/// whose level is above OldIrql.  pic.c:378-432 (non-minihal).
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "fastcall" fn KfLowerIrql(OldIrql: KIRQL) {
    let Pcr = unsafe { KeGetPcr() };

    // Save EFlags and disable interrupts for the IRR/PIC update.
    let EFlags = unsafe { __readeflags() };
    unsafe { _disable() };

    // Drop the IRQL.
    unsafe { (*Pcr).Irql = OldIrql };

    // Are any pending software interrupts higher than the new IRQL?
    let IRR = unsafe { (*Pcr).IRR };
    let PendingIrqlMask = IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        let mut PendingIrql: ULONG = 0;
        unsafe { BitScanReverse(&mut PendingIrql, PendingIrqlMask) };

        // If the pending IRQL is above DISPATCH_LEVEL we need to update the
        // hardware PIC mask too (mask off everything at-or-below).
        if PendingIrql > DISPATCH_LEVEL as ULONG {
            let mut Mask: PIC_MASK = unsafe { core::mem::zeroed() };
            Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
            unsafe { __outbyte(PIC1_DATA_PORT, Mask.__bindgen_anon_1.Master) };
            unsafe { __outbyte(PIC2_DATA_PORT, Mask.__bindgen_anon_1.Slave) };

            // Clear the IRR bit we're about to dispatch.
            unsafe { (*Pcr).IRR ^= 1 << PendingIrql };
        }

        // Dispatch the pending software interrupt.  The handler tail-jumps
        // back into the SW interrupt context.
        if let Some(handler) = SWInterruptHandlerTable[PendingIrql as usize] {
            unsafe { handler() };
        }
    }

    // Restore interrupt state.
    unsafe { __writeeflags(EFlags) };
}

// ---------------------------------------------------------------------------
// The APIC flavor's interrupt-line exports (apic.rs). The 8259 flavor routes
// no interrupt through an I/O APIC: 0 inputs keeps acpi.sys in the PIC model
// (no `\_PIC(1)`), so PCI INTx stays on the firmware's Interrupt Line, which
// is exactly what the PICs are wired to.
// ---------------------------------------------------------------------------

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosIoApicInputs() -> ULONG {
    0
}

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosSetInterruptLine(_Gsi: ULONG, _LevelSensitive: BOOLEAN, _ActiveLow: BOOLEAN) -> BOOLEAN {
    0
}
