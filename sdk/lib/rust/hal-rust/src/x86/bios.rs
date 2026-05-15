//! i386 V8086-mode BIOS access — translated from
//! `hal/halx86/generic/bios.c`.
//!
//! i386-only (the C file is in generic.cmake's `if(ARCH STREQUAL
//! "i386")` block; amd64 uses the fast486 path in `x86bios.rs`).
//!
//! The mechanism: map the low 1 MB 1:1, borrow a TSS with IOPM space,
//! install custom #UD/#GP handlers, `_setjmp` a return context, drop to
//! V86 via `HalpExitToV86` (asm), let the real-mode stub run the BIOS
//! `INT`, and bounce back through a #UD BOP that `longjmp`s home.
//!
//! ## ABI
//! - `Halp*` NTAPI fns → stdcall on i386 → `extern "system"`.
//! - V86 opcode/push helpers + `HalpTrap0DHandler` are `FASTCALL`
//!   (`__fastcall` on i386) → `extern "fastcall"`.
//! - `HalpTrap06` is bare `__cdecl` (registered as the int-6 handler
//!   address) → `extern "C"`.
//! - `HalpExitToV86` / `KiEoiHelper` are fastcall (`@Name@4`).
//! - `_setjmp` / `longjmp` resolve to the linked `setjmp` vcruntime lib
//!   (`__setjmp` / `_longjmp`).

#![cfg(target_arch = "x86")]
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, PVOID, ULONG, ULONG_PTR, USHORT, PKTRAP_FRAME, PHAL_BIOS_FRAME,
    _KTSS, _KiIoAccessMap, _KGDTENTRY, _HARDWARE_PTE_X86, HalPrivateDispatchTable,
};
use crate::x86::pcr::KeGetPcr;

// ---------------------------------------------------------------------------
// Constants (sdk/include/ndk/i386/asm.h / ketypes.h / mmtypes.h).
// ---------------------------------------------------------------------------

const PAGE_SIZE: ULONG = 0x1000;
const PAGE_SHIFT: u32 = 12;
const RPL_MASK: ULONG = 0x0003;
const KGDT_R3_DATA: ULONG = 0x20;
const KGDT_R0_PCR: ULONG = 0x30;
const KGDT_TSS: usize = 0x28;
const I386_TSS: ULONG = 0x9;
const EFLAGS_IOPL: ULONG = 0x3000;
const EFLAGS_V86_MASK: ULONG = 0x20000;
const CR0_AM: ULONG = 0x40000;

const IOPM_SIZE: usize = 8192;
const IOPM_FULL_SIZE: usize = 8196;

// i386 non-PAE self-map bases (cf. ntkrnl-sys inline mm: PTE_BASE 0xC0000000).
const PTE_BASE: usize = 0xC000_0000;
const PDE_BASE: usize = 0xC030_0000;

const EXCEPTION_CHAIN_END: ULONG = !0;

// ---------------------------------------------------------------------------
// jmp_buf — i386 vcruntime: _JBTYPE int, _JBLEN 16 → 64 bytes.
// ---------------------------------------------------------------------------

#[repr(C)]
struct JmpBuf {
    _raw: [u32; 16],
}

unsafe extern "C" {
    // vcruntime i386: PUBLIC __setjmp / longjmp.c __cdecl longjmp.
    #[link_name = "_setjmp"]
    fn c_setjmp(buf: *mut JmpBuf) -> i32;
    fn longjmp(buf: *mut JmpBuf, val: i32) -> !;
    fn DbgPrint(fmt: *const u8, ...) -> u32;

    // Asm entry points from generic/v86.S + trap.S.
    fn HalpRealModeStart(); // __cdecl label
    static HalpRealModeEnd: u8; // extern PVOID — label, take &
    fn HalpTrap0D(); // trap.S TRAP_ENTRY stub
}

unsafe extern "system" {
    fn DbgBreakPoint();
    fn HalpFlushTLB();
}

// KiDumpTrapFrame is an internal ntoskrnl symbol — NOT in ntoskrnl.spec,
// so it can't be imported into hal.dll.  It's only used on the fatal
// "Trap0D outside V86" path (NMI during V8086), which halts anyway.
// Inline a minimal salient-field dump instead of importing it.
#[inline(always)]
unsafe fn dump_trap_frame(tf: PKTRAP_FRAME) {
    DbgPrint(
        b"HAL: TrapFrame Eip=%08X SegCs=%04X EFlags=%08X HwEsp=%08X HwSs=%04X\n\0".as_ptr(),
        (*tf).Eip, (*tf).SegCs, (*tf).EFlags, (*tf).HardwareEsp, (*tf).HardwareSegSs,
    );
}

unsafe extern "fastcall" {
    fn HalpExitToV86(TrapFrame: PKTRAP_FRAME);
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME);
}

// HalpNMIInProgress lives in nmi.rs (Rust HAL); referenced as the C global.
unsafe extern "C" {
    static HalpNMIInProgress: u8;
}

// ---------------------------------------------------------------------------
// Globals — names match the C symbols (consumers/v86.S reference them).
// ---------------------------------------------------------------------------

#[no_mangle]
pub static mut HalpSavedPfn: ULONG = 0;
#[no_mangle]
pub static mut HalpSavedPte: _HARDWARE_PTE_X86 =
    unsafe { core::mem::transmute::<u32, _HARDWARE_PTE_X86>(0) };
#[no_mangle]
pub static mut HalpGpfHandler: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut HalpBopHandler: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut HalpSavedEsp0: ULONG = 0;
#[no_mangle]
pub static mut HalpSavedTss: USHORT = 0;
#[no_mangle]
pub static mut HalpSavedIopmBase: USHORT = 0;
#[no_mangle]
pub static mut HalpSavedIoMap: *mut USHORT = core::ptr::null_mut();
#[no_mangle]
pub static mut HalpSavedIoMapData: [[USHORT; 2]; IOPM_SIZE / 2] = [[0; 2]; IOPM_SIZE / 2];
#[no_mangle]
pub static mut HalpSavedIoMapEntries: ULONG = 0;
#[no_mangle]
pub static mut HalpSavedEsp: ULONG_PTR = 0;

static mut HalpSavedContext: JmpBuf = JmpBuf { _raw: [0; 16] };

// ---------------------------------------------------------------------------
// FORCEINLINE replicas.
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn HalAddressToPte(addr: usize) -> *mut _HARDWARE_PTE_X86 {
    (PTE_BASE + ((addr >> 12) << 2)) as *mut _HARDWARE_PTE_X86
}

#[inline(always)]
unsafe fn HalAddressToPde(addr: usize) -> *mut _HARDWARE_PTE_X86 {
    (PDE_BASE + ((addr >> 22) << 2)) as *mut _HARDWARE_PTE_X86
}

#[inline(always)]
unsafe fn Ke386GetTr() -> USHORT {
    let tr: USHORT;
    core::arch::asm!("str {0:x}", out(reg) tr, options(nomem, nostack, preserves_flags));
    tr
}

#[inline(always)]
unsafe fn Ke386SetTr(sel: USHORT) {
    core::arch::asm!("ltr {0:x}", in(reg) sel, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn Ke386SetEs(sel: ULONG) {
    core::arch::asm!("mov es, {0:e}", in(reg) sel, options(nomem, nostack, preserves_flags));
}
#[inline(always)]
unsafe fn Ke386SetDs(sel: ULONG) {
    core::arch::asm!("mov ds, {0:e}", in(reg) sel, options(nomem, nostack, preserves_flags));
}
#[inline(always)]
unsafe fn Ke386SetFs(sel: ULONG) {
    core::arch::asm!("mov fs, {0:e}", in(reg) sel, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_cr0() -> ULONG {
    let v: ULONG;
    core::arch::asm!("mov {0:e}, cr0", out(reg) v, options(nomem, nostack, preserves_flags));
    v
}
#[inline(always)]
unsafe fn write_cr0(v: ULONG) {
    core::arch::asm!("mov cr0, {0:e}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline(always)]
unsafe fn read_eflags() -> ULONG {
    let f: ULONG;
    core::arch::asm!("pushfd", "pop {0:e}", out(reg) f, options(nomem, preserves_flags));
    f
}
#[inline(always)]
unsafe fn write_eflags(f: ULONG) {
    core::arch::asm!("push {0:e}", "popfd", in(reg) f, options(nomem));
}
#[inline(always)]
unsafe fn _disable() {
    core::arch::asm!("cli", options(nomem, nostack));
}

// KiComputeIopmOffset(1) = FIELD_OFFSET(KTSS, IoMaps[0].IoMap).
#[inline(always)]
fn ki_compute_iopm_offset_1() -> USHORT {
    (core::mem::offset_of!(_KTSS, IoMaps) + core::mem::offset_of!(_KiIoAccessMap, IoMap)) as USHORT
}

#[inline(always)]
unsafe fn hal_vector_to_idt_entry(vector: ULONG) -> usize {
    let d = &raw const HalPrivateDispatchTable;
    ((*d).HalVectorToIDTEntry.unwrap_unchecked())(vector) as usize
}

// KeRegisterInterruptHandler / KeQueryInterruptHandler — FORCEINLINE in
// internal/i386/ke.h.  i386 _KIDTENTRY { Offset, Selector, Access,
// ExtendedOffset }; only Offset/ExtendedOffset carry the handler addr.
#[inline(always)]
unsafe fn KeRegisterInterruptHandler(vector: ULONG, handler: PVOID) {
    let pcr = KeGetPcr();
    let entry = hal_vector_to_idt_entry(vector);
    let idt = (*pcr).IDT.add(entry);
    let addr = handler as usize;
    (*idt).ExtendedOffset = (addr >> 16) as USHORT;
    (*idt).Offset = addr as USHORT;
}

#[inline(always)]
unsafe fn KeQueryInterruptHandler(vector: ULONG) -> PVOID {
    let pcr = KeGetPcr();
    let entry = hal_vector_to_idt_entry(vector);
    let idt = (*pcr).IDT.add(entry);
    ((((*idt).ExtendedOffset as usize) << 16) & 0xFFFF_0000
        | ((*idt).Offset as usize) & 0xFFFF) as PVOID
}

// KiEnterTrap — i386 trap_x.h:404.  Our build is DBG; replicate the
// KiFillTrapFrameDebug dance (same as pic_pic.rs::KiEnterInterruptTrap).
#[inline(always)]
unsafe fn KiEnterTrap(TrapFrame: PKTRAP_FRAME) {
    let pcr = KeGetPcr();
    (*pcr).__bindgen_anon_1.NtTib.ExceptionList = EXCEPTION_CHAIN_END as usize as *mut _;
    (*TrapFrame).Dr7 = 0;
    (*TrapFrame).DbgArgPointer = (*TrapFrame).Edx;
    (*TrapFrame).DbgArgMark = 0xBADB_0D00;
    (*TrapFrame).DbgEip = (*TrapFrame).Eip;
    (*TrapFrame).DbgEbp = (*TrapFrame).Ebp;
    (*TrapFrame).PreviousPreviousMode = !0;
}

// ---------------------------------------------------------------------------
// V86 opcode handlers (FASTCALL).
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "fastcall" fn HalpOpcodeInvalid(BiosFrame: PHAL_BIOS_FRAME) -> BOOLEAN {
    let inst = ((*BiosFrame).CsBase + (*BiosFrame).Eip) as *const u8;
    DbgPrint(
        b"HAL: An invalid V86 opcode was encountered at address %X:%X\nOpcode: %02X %02X %02X %02X %02X %02X %02X %02X %02X %02X\n\0".as_ptr(),
        (*BiosFrame).SegCs, (*BiosFrame).Eip,
        *inst.add(0) as u32, *inst.add(1) as u32, *inst.add(2) as u32, *inst.add(3) as u32,
        *inst.add(4) as u32, *inst.add(5) as u32, *inst.add(6) as u32, *inst.add(7) as u32,
        *inst.add(8) as u32, *inst.add(9) as u32,
    );
    DbgBreakPoint();
    0
}

#[no_mangle]
pub unsafe extern "fastcall" fn HalpPushInt(
    BiosFrame: PHAL_BIOS_FRAME,
    Interrupt: ULONG,
) -> BOOLEAN {
    // Stack = (PUSHORT)(SsBase + (Esp & 0xFFFF))
    let mut stack = ((*BiosFrame).SsBase + ((*BiosFrame).Esp & 0xFFFF)) as *mut USHORT;

    stack = stack.offset(-1);
    *stack = ((*BiosFrame).EFlags & 0xFFFF) as USHORT;
    stack = stack.offset(-1);
    *stack = ((*BiosFrame).SegCs & 0xFFFF) as USHORT;
    stack = stack.offset(-1);
    *stack = ((*BiosFrame).Eip & 0xFFFF) as USHORT;

    // CS:IP from the real-mode IVT entry.
    let eip = *((Interrupt * 4) as *const ULONG);
    (*BiosFrame).Eip = eip & 0xFFFF;
    (*BiosFrame).SegCs = eip >> 16;

    (*BiosFrame).Esp = (stack as ULONG_PTR) & 0xFFFF;

    (*BiosFrame).CsBase = (*BiosFrame).SegCs << 4;
    (*BiosFrame).CsLimit = 0xFFFF;
    (*BiosFrame).CsFlags = 0;
    1
}

#[no_mangle]
pub unsafe extern "fastcall" fn HalpOpcodeINTnn(BiosFrame: PHAL_BIOS_FRAME) -> BOOLEAN {
    (*BiosFrame).SsBase = (*BiosFrame).SegSs << 4;
    (*BiosFrame).SsLimit = 0xFFFF;
    (*BiosFrame).SsFlags = 0;

    (*BiosFrame).Eip += 1;
    if (*BiosFrame).Eip > (*BiosFrame).CsLimit {
        return 0;
    }

    let interrupt = *(((*BiosFrame).CsBase + (*BiosFrame).Eip) as *const u8) as ULONG;

    (*BiosFrame).Eip += 1;
    if HalpPushInt(BiosFrame, interrupt) != 0 {
        let tf = (*BiosFrame).TrapFrame;
        (*tf).HardwareSegSs = (*BiosFrame).SegSs;
        (*tf).HardwareEsp = (*BiosFrame).Esp;
        (*tf).SegCs = (*BiosFrame).SegCs;
        (*tf).EFlags = (*BiosFrame).EFlags;
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "fastcall" fn HalpDispatchV86Opcode(TrapFrame: PKTRAP_FRAME) -> BOOLEAN {
    let mut bf: ntkrnl_sys::hal_ddk::HAL_BIOS_FRAME = core::mem::zeroed();
    bf.TrapFrame = TrapFrame;
    bf.SegSs = (*TrapFrame).HardwareSegSs;
    bf.Esp = (*TrapFrame).HardwareEsp;
    bf.EFlags = (*TrapFrame).EFlags;
    bf.SegCs = (*TrapFrame).SegCs;
    bf.Eip = (*TrapFrame).Eip;
    bf.Prefix = 0;

    bf.CsBase = bf.SegCs << 4;
    bf.CsLimit = 0xFFFF;
    bf.CsFlags = 0;

    if bf.Eip > bf.CsLimit {
        return 0;
    }

    let instruction = *((bf.CsBase + bf.Eip) as *const u8);
    if instruction != 0xCD {
        HalpOpcodeInvalid(&mut bf);
        return 0;
    }

    if HalpOpcodeINTnn(&mut bf) != 0 {
        (*TrapFrame).Eip = bf.Eip;
        return 1;
    }
    0
}

// ---------------------------------------------------------------------------
// V86 trap handlers.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "fastcall" fn HalpTrap0DHandler(TrapFrame: PKTRAP_FRAME) -> ! {
    KiEnterTrap(TrapFrame);

    if (*TrapFrame).EFlags & EFLAGS_V86_MASK != 0 {
        HalpDispatchV86Opcode(TrapFrame);
        KiEoiHelper(TrapFrame);
        // KiEoiHelper is noreturn in practice (iret); satisfy the `!`.
        loop {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }

    DbgPrint(b"HAL: Trap0D while not in V86 mode\n\0".as_ptr());
    dump_trap_frame(TrapFrame);
    loop {
        core::arch::asm!("hlt", options(nomem, nostack));
    }
}

#[no_mangle]
pub unsafe extern "C" fn HalpTrap06() -> ! {
    Ke386SetEs(KGDT_R3_DATA | RPL_MASK);
    Ke386SetDs(KGDT_R3_DATA | RPL_MASK);
    Ke386SetFs(KGDT_R0_PCR);

    (*(*KeGetPcr()).TSS).Esp0 = HalpSavedEsp0;

    longjmp(&raw mut HalpSavedContext, 1);
}

// ---------------------------------------------------------------------------
// V8086 enter.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpBiosCall() {
    // Must be volatile so it doesn't get optimized away across longjmp.
    let mut v86: ntkrnl_sys::hal_ddk::KTRAP_FRAME = core::mem::zeroed();

    if c_setjmp(&raw mut HalpSavedContext) != 0 {
        // Returned from V86 via HalpTrap06's longjmp.
        return;
    }

    // Kill alignment faults.
    write_cr0(read_cr0() & !CR0_AM);

    // New ring-0 stack = just below the V86 trap frame.
    let fx_save_area: usize = 528; // sizeof(FX_SAVE_AREA) — 512 + 16 header
    (*(*KeGetPcr()).TSS).Esp0 =
        (&raw mut v86 as usize - 0x20 - fx_save_area) as ULONG;

    let stack_offset =
        (&raw const HalpRealModeEnd as usize) - 4 - (HalpRealModeStart as usize);
    let code_offset = (HalpRealModeStart as usize) & 0xFFF;

    core::ptr::write_volatile(&raw mut v86.V86Es, 0);
    core::ptr::write_volatile(&raw mut v86.V86Ds, 0);
    core::ptr::write_volatile(&raw mut v86.V86Gs, 0);
    core::ptr::write_volatile(&raw mut v86.V86Fs, 0);
    core::ptr::write_volatile(&raw mut v86.HardwareSegSs, 0x2000);
    core::ptr::write_volatile(&raw mut v86.HardwareEsp, (stack_offset + code_offset) as ULONG);
    core::ptr::write_volatile(
        &raw mut v86.EFlags,
        read_eflags() | EFLAGS_V86_MASK | EFLAGS_IOPL,
    );
    core::ptr::write_volatile(&raw mut v86.SegCs, 0x2000);
    core::ptr::write_volatile(&raw mut v86.Eip, code_offset as ULONG);

    HalpExitToV86(&raw mut v86);
}

// ---------------------------------------------------------------------------
// TSS borrow / return.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpBorrowTss() {
    let tss = Ke386GetTr();
    let gdt = (*KeGetPcr()).GDT;
    let tss_gdt = gdt.add(tss as usize / core::mem::size_of::<_KGDTENTRY>());

    let tss_limit = (*tss_gdt).LimitLow as usize
        | (((*tss_gdt).HighWord.Bits.LimitHi() as usize) << 16);

    if tss_limit > IOPM_SIZE {
        HalpSavedTss = 0;
        return;
    }

    let real_gdt = gdt.add(KGDT_TSS / core::mem::size_of::<_KGDTENTRY>());
    let base = (*real_gdt).BaseLow as usize
        | (((*real_gdt).HighWord.Bytes.BaseMid as usize) << 16)
        | (((*real_gdt).HighWord.Bytes.BaseHi as usize) << 24);
    let tss_base = base as *mut _KTSS;

    (*KeGetPcr()).TSS = tss_base;

    (*real_gdt).HighWord.Bits.set_Type(I386_TSS);
    (*real_gdt).HighWord.Bits.set_Pres(1);
    (*real_gdt).HighWord.Bits.set_Dpl(0);

    Ke386SetTr(KGDT_TSS as USHORT);
    HalpSavedTss = tss;
}

#[no_mangle]
pub unsafe extern "system" fn HalpReturnTss() {
    let gdt = (*KeGetPcr()).GDT;
    let tss_gdt = gdt.add(HalpSavedTss as usize / core::mem::size_of::<_KGDTENTRY>());
    let base = (*tss_gdt).BaseLow as usize
        | (((*tss_gdt).HighWord.Bytes.BaseMid as usize) << 16)
        | (((*tss_gdt).HighWord.Bytes.BaseHi as usize) << 24);
    let tss_base = base as *mut _KTSS;

    (*KeGetPcr()).TSS = tss_base;

    (*tss_gdt).HighWord.Bits.set_Type(I386_TSS);
    (*tss_gdt).HighWord.Bits.set_Pres(1);
    (*tss_gdt).HighWord.Bits.set_Dpl(0);

    Ke386SetTr(HalpSavedTss);
}

// ---------------------------------------------------------------------------
// IOPM store / clear / restore.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpStoreAndClearIopm() {
    let mut entry = HalpSavedIoMap;
    let mut j: usize = 0;
    let mut i: usize = 0;

    while i < IOPM_SIZE / 2 {
        if *entry != 0xFFFF {
            HalpSavedIoMapData[j][0] = i as USHORT;
            HalpSavedIoMapData[j][1] = *entry;
            j += 1;
        }
        *entry = 0;
        entry = entry.add(1);
        i += 1;
    }

    // i is now IOPM_SIZE/2; continue terminating up to IOPM_FULL_SIZE/2.
    while i < IOPM_FULL_SIZE / 2 {
        *entry = 0xFFFF;
        entry = entry.add(1);
        i += 1;
    }

    HalpSavedIoMapEntries = j as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn HalpRestoreIopm() {
    let mut i = HalpSavedIoMapEntries;
    core::ptr::write_bytes(HalpSavedIoMap as *mut u8, 0xFF, IOPM_FULL_SIZE);
    while i != 0 {
        i -= 1;
        let idx = HalpSavedIoMapData[i as usize][0] as usize;
        *HalpSavedIoMap.add(idx) = HalpSavedIoMapData[i as usize][1];
    }
}

// ---------------------------------------------------------------------------
// Low-1MB real-mode memory map / unmap.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpMapRealModeMemory() {
    // PDE for the lowest meg.
    let pde = HalAddressToPde(0);
    HalpSavedPfn = (*pde).PageFrameNumber();
    HalpSavedPte = *pde;

    (*pde).set_Valid(1);
    (*pde).set_Write(1);
    (*pde).set_Owner(1);
    (*pde).set_PageFrameNumber((*HalAddressToPde(0xFFC0_0000)).PageFrameNumber());

    HalpFlushTLB();

    // Identity-map the first meg.
    let mut i: usize = 0;
    while i < 0x100000 {
        let pte = HalAddressToPte(i);
        (*pte).set_PageFrameNumber((i >> PAGE_SHIFT) as ULONG);
        (*pte).set_Valid(1);
        (*pte).set_Write(1);
        (*pte).set_Owner(1);
        i += PAGE_SIZE as usize;
    }

    // Map our real-mode V86 code into the 0x20000 region.
    let mut pte = HalAddressToPte(0x20000);
    let mut v86pte = HalAddressToPte(HalpRealModeStart as usize);
    let end = HalAddressToPte(&raw const HalpRealModeEnd as usize);
    loop {
        (*pte).set_PageFrameNumber((*v86pte).PageFrameNumber());
        pte = pte.add(1);
        v86pte = v86pte.add(1);
        if v86pte > end {
            break;
        }
    }

    HalpFlushTLB();
}

#[no_mangle]
pub unsafe extern "system" fn HalpUnmapRealModeMemory() {
    let mut i: usize = 0;
    while i < 0x100000 {
        let pte = HalAddressToPte(i);
        (*pte).set_Valid(0);
        (*pte).set_Write(0);
        (*pte).set_Owner(0);
        (*pte).set_PageFrameNumber(0);
        i += PAGE_SIZE as usize;
    }

    let pde = HalAddressToPde(0);
    *pde = HalpSavedPte;
    (*pde).set_PageFrameNumber(HalpSavedPfn);

    HalpFlushTLB();
}

// ---------------------------------------------------------------------------
// Trap-handler swap + IOPM/TSS setup/teardown.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpSwitchToRealModeTrapHandlers() {
    HalpGpfHandler = KeQueryInterruptHandler(13);
    HalpBopHandler = KeQueryInterruptHandler(6);
    KeRegisterInterruptHandler(13, HalpTrap0D as PVOID);
    KeRegisterInterruptHandler(6, HalpTrap06 as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn HalpSetupRealModeIoPermissionsAndTask() {
    HalpBorrowTss();
    HalpSavedIoMap =
        core::ptr::addr_of_mut!((*(*KeGetPcr()).TSS).IoMaps[0].IoMap) as *mut USHORT;
    HalpStoreAndClearIopm();
    HalpSavedIopmBase = (*(*KeGetPcr()).TSS).IoMapBase;
    (*(*KeGetPcr()).TSS).IoMapBase = ki_compute_iopm_offset_1();
    HalpSavedEsp0 = (*(*KeGetPcr()).TSS).Esp0;
}

#[no_mangle]
pub unsafe extern "system" fn HalpRestoreTrapHandlers() {
    if HalpNMIInProgress == 0 {
        KeRegisterInterruptHandler(13, HalpGpfHandler);
    }
    KeRegisterInterruptHandler(6, HalpBopHandler);
}

#[no_mangle]
pub unsafe extern "system" fn HalpRestoreIoPermissionsAndTask() {
    (*(*KeGetPcr()).TSS).Esp0 = HalpSavedEsp0;
    HalpRestoreIopm();
    (*(*KeGetPcr()).TSS).IoMapBase = HalpSavedIopmBase;
    if HalpSavedTss != 0 {
        HalpReturnTss();
    }
}

// ---------------------------------------------------------------------------
// Public entry point.
// ---------------------------------------------------------------------------

// minihal provides a stub HalpBiosDisplayReset (minihal_stubs.rs); the
// real V86 implementation is only in the full HAL build.
#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpBiosDisplayReset() -> BOOLEAN {
    let flags = read_eflags();
    _disable();

    HalpMapRealModeMemory();

    // P5 cmpxchg8b errata: the first 7 IDT entries can be write-protected.
    // Unprotect the IDT page so we can install our int-6 handler.
    let idt_pte = HalAddressToPte((*KeGetPcr()).IDT as usize);
    let restore_wp = (*idt_pte).Write() != 0;
    (*idt_pte).set_Write(1);

    HalpSwitchToRealModeTrapHandlers();
    HalpSetupRealModeIoPermissionsAndTask();

    HalpBiosCall();

    HalpRestoreTrapHandlers();

    (*idt_pte).set_Write(if restore_wp { 1 } else { 0 });

    HalpRestoreIoPermissionsAndTask();
    HalpUnmapRealModeMemory();

    write_eflags(flags);
    1
}
