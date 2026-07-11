//! AP real-mode startup trampoline for SMP bring-up.
//!
//! An application processor woken by a SIPI begins executing in 16-bit REAL
//! MODE at physical `vector << 12`. This module supplies the code that runs
//! there and lifts the AP into protected mode (i386) / long mode (amd64) with
//! the exact `KPROCESSOR_STATE` the kernel prepared, then jumps to
//! `KiSystemStartup`.
//!
//! ---------------------------------------------------------------------------
//! WHY A BYTE ARRAY (the .code16 caveat):
//! ---------------------------------------------------------------------------
//! This repo's Clang assembler CANNOT assemble `.code16` (see AGENTS.md "16-bit
//! assembly"): real-mode startup code normally lives in a *separate GAS-built
//! file* (boot sectors, frldr16, MVDM). `global_asm!` here would be compiled
//! for the crate's 32/64-bit target and produce wrong-width encodings for the
//! real-mode prologue. So — following the established workaround family — the
//! trampoline is shipped as a RAW BYTE ARRAY, `include_bytes!`'d from a flat
//! binary produced by the SYSTEM GNU as/ld (NOT clang): `aptramp64.S` (amd64)
//! and `aptramp32.S` (i386) are the sources of truth, assembled+linked by
//! `regen-trampoline.sh`; both are real, disassembly-verified machine code as
//! of 2026-07-10 (NOTES.md's "trampoline bytes owed" item is CLOSED — see
//! VERIFY.md for the i386 side's byte-by-byte walk).
//!
//! ---------------------------------------------------------------------------
//! TRAMPOLINE PAGE (integrator must provide — see smp_processor.rs externs):
//! ---------------------------------------------------------------------------
//!   * One page, PHYSICAL address < 0x100000 (1MB) and page-aligned, so the
//!     SIPI vector `phys >> 12` is a valid 8-bit startup vector.
//!   * IDENTITY-MAPPED in the kernel CR3 (linear == physical for that page):
//!     the moment the trampoline turns on paging, execution continues at the
//!     SAME linear address, so that page must translate to itself. In ReactOS
//!     this is the `HalpLowStub` page (reserved during HAL init via
//!     HalpMapPhysicalMemory64 / a boot-time low reservation). This fork
//!     dropped x86bios so no low stub is reserved today — that reservation is
//!     part of the integration work (NOTES.md).
//!
//! The BSP writes a PARAMETER BLOCK into the page (right after the code) with
//! the AP's target CR0/CR3/CR4, GDTR/IDTR, selectors and entry point; the
//! real-mode code reads it via a fixed page offset.

#![allow(non_snake_case)]

use ntkrnl_sys::hal_ddk::PKPROCESSOR_STATE;

// Fixed offset within the trampoline page where the parameter block lives.
// The real-mode code references it as an absolute offset from the page base
// (CS:PARAM_BLOCK_OFFSET in real mode, then linear stub_pa+offset once in PM).
pub const PARAM_BLOCK_OFFSET: usize = 0x800; // 2KB in — clear of the code

/// Parameter block the real-mode/PM trampoline consumes. `#[repr(C, packed)]`
/// with EXPLICIT padding — aptramp64.S hard-codes these byte offsets (see its
/// header comment), so DO NOT reorder without updating both. The GDTR/IDTR
/// limit+base pairs are packed CONTIGUOUS so lgdt/lidt consume them in place
/// (the original draft had 6 pad bytes between limit and base, which would
/// have loaded a garbage descriptor-table base).
#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct ApTrampolineParams {
    // --- control registers ---
    pub cr0: u64,        // 0x00
    pub cr3: u64,        // 0x08
    pub cr4: u64,        // 0x10
    // --- descriptor tables: pseudo-descriptor form, limit+base CONTIGUOUS ---
    pub gdtr_limit: u16, // 0x18
    pub gdtr_base: u64,  // 0x1A
    pub _pad0: [u8; 6],  // 0x22
    pub idtr_limit: u16, // 0x28
    pub idtr_base: u64,  // 0x2A
    pub _pad1: [u8; 6],  // 0x32
    // --- selectors ---
    pub seg_cs: u16,     // 0x38
    pub seg_ds: u16,     // 0x3A
    pub seg_es: u16,     // 0x3C
    pub seg_ss: u16,     // 0x3E
    pub seg_fs: u16,     // 0x40
    pub seg_gs: u16,     // 0x42
    pub tr: u16,         // 0x44
    pub _pad2: u16,      // 0x46
    // --- entry ---
    pub entry: u64,      // 0x48  KiSystemStartup (Eip / Rip)
    pub stack: u64,      // 0x50  ContextFrame.Esp / Rsp
    pub eflags: u64,     // 0x58
    pub arg0: u64,       // 0x60  KiSystemStartup's first arg (LoaderBlock)
}

// Compile-time proof the layout matches aptramp64.S's hard-coded offsets.
const _: () = {
    assert!(core::mem::offset_of!(ApTrampolineParams, gdtr_limit) == 0x18);
    assert!(core::mem::offset_of!(ApTrampolineParams, gdtr_base) == 0x1A);
    assert!(core::mem::offset_of!(ApTrampolineParams, idtr_limit) == 0x28);
    assert!(core::mem::offset_of!(ApTrampolineParams, idtr_base) == 0x2A);
    assert!(core::mem::offset_of!(ApTrampolineParams, seg_cs) == 0x38);
    assert!(core::mem::offset_of!(ApTrampolineParams, tr) == 0x44);
    assert!(core::mem::offset_of!(ApTrampolineParams, entry) == 0x48);
    assert!(core::mem::offset_of!(ApTrampolineParams, stack) == 0x50);
    assert!(core::mem::offset_of!(ApTrampolineParams, eflags) == 0x58);
    assert!(core::mem::offset_of!(ApTrampolineParams, arg0) == 0x60);
    assert!(core::mem::size_of::<ApTrampolineParams>() == 0x68);
};

// ===========================================================================
// install() — copy the arch trampoline blob into the low stub page and patch
// its parameter block from *ProcessorState.
//
//   stub_va : kernel VA of the identity-mapped low page (write here)
//   stub_pa : its physical address (used only for assertions / logging)
// ===========================================================================
pub unsafe fn install(stub_va: *mut u8, _stub_pa: u64, state: PKPROCESSOR_STATE) {
    // 1. Copy the code blob to the head of the page.
    let blob = trampoline_blob();
    core::ptr::copy_nonoverlapping(blob.as_ptr(), stub_va, blob.len());

    // 2. Build the parameter block from KPROCESSOR_STATE.
    //
    // NOTE the i386 vs amd64 KPROCESSOR_STATE field ORDER differs:
    //   i386 : { ContextFrame, SpecialRegisters }   (ndk/i386/ketypes.h)
    //   amd64: { SpecialRegisters, ContextFrame }    (ndk/amd64/ketypes.h)
    // We access through the bindgen `_KPROCESSOR_STATE` so the compiler picks
    // the right layout per target; the reads below are written arch-generically.
    let mut p = ApTrampolineParams::default();
    fill_params_from_state(&mut p, state);

    // 3. Deposit it at the fixed offset.
    let param_ptr = stub_va.add(PARAM_BLOCK_OFFSET) as *mut ApTrampolineParams;
    core::ptr::write_volatile(param_ptr, p);

    // 4. Ensure the writes are globally visible before the AP starts fetching.
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

// Pull the fields out of the (arch-specific) KPROCESSOR_STATE.
//
// The bindgen structs expose SpecialRegisters.{Cr0,Cr3,Cr4,Gdtr,Idtr,Tr} and
// ContextFrame.{SegCs,SegDs,SegEs,SegSs,SegFs,SegGs, Eip/Rip, Esp/Rsp, EFlags}.
// KDESCRIPTOR is { Pad:u16, Limit:u16, Base:usize } on i386 and
// { Pad[3]:u16, Limit:u16, Base:u64 } on amd64 — read Limit/Base by field.
#[cfg(target_arch = "x86")]
unsafe fn fill_params_from_state(p: &mut ApTrampolineParams, state: PKPROCESSOR_STATE) {
    let sr = &(*state).SpecialRegisters;
    let cf = &(*state).ContextFrame;
    p.cr0 = sr.Cr0 as u64;
    p.cr3 = sr.Cr3 as u64;
    p.cr4 = sr.Cr4 as u64;
    p.gdtr_limit = sr.Gdtr.Limit;
    p.gdtr_base = sr.Gdtr.Base as u64;
    p.idtr_limit = sr.Idtr.Limit;
    p.idtr_base = sr.Idtr.Base as u64;
    p.tr = sr.Tr;
    p.seg_cs = cf.SegCs as u16;
    p.seg_ds = cf.SegDs as u16;
    p.seg_es = cf.SegEs as u16;
    p.seg_ss = cf.SegSs as u16;
    p.seg_fs = cf.SegFs as u16;
    p.seg_gs = cf.SegGs as u16;
    p.entry = cf.Eip as u64;
    p.stack = cf.Esp as u64;
    p.eflags = cf.EFlags as u64;
    // arg0 (LoaderBlock): i386 KiSystemStartup is __stdcall, so there is no
    // register carrying it the way amd64's Rcx does (see the x86_64 arm
    // below) — mproc.c instead pre-builds the arg directly into the AP's
    // kernel stack (AP_SETUP_STACK: {ReturnAddr=NULL, KxLoaderBlock} at
    // cf.Esp/cf.Esp+4 — see aptramp32.S's header "i386 KiSystemStartup arg
    // contract"). We read it back out of that same pre-built image here so
    // p.arg0 is populated for struct/ABI parity with amd64. This is PURELY
    // documentation-level parity: aptramp32.S does NOT consume p.arg0 — it
    // loads ESP = p.stack verbatim and far-returns, so KiSystemStartup sees
    // LoaderBlock at [esp+4] from the stack image itself, not from this field.
    let ap_stack = cf.Esp as *const u32; // { ReturnAddr, KxLoaderBlock }
    p.arg0 = *ap_stack.add(1) as u64;
}

#[cfg(target_arch = "x86_64")]
unsafe fn fill_params_from_state(p: &mut ApTrampolineParams, state: PKPROCESSOR_STATE) {
    let sr = &(*state).SpecialRegisters;
    let cf = &(*state).ContextFrame;
    p.cr0 = sr.Cr0;
    p.cr3 = sr.Cr3;
    p.cr4 = sr.Cr4;
    p.gdtr_limit = sr.Gdtr.Limit;
    p.gdtr_base = sr.Gdtr.Base as u64;
    p.idtr_limit = sr.Idtr.Limit;
    p.idtr_base = sr.Idtr.Base as u64;
    p.tr = sr.Tr;
    p.seg_cs = cf.SegCs;
    p.seg_ds = cf.SegDs;
    p.seg_es = cf.SegEs;
    p.seg_ss = cf.SegSs;
    p.seg_fs = cf.SegFs;
    p.seg_gs = cf.SegGs;
    p.entry = cf.Rip;
    p.stack = cf.Rsp;
    p.eflags = cf.EFlags as u64;
    // Windows x64 ABI: KiSystemStartup's LoaderBlock rides in ContextFrame.Rcx
    // (set by KeStartAllProcessors — ke/amd64/mproc.rs). The trampoline loads
    // it into rcx just before the lretq.
    p.arg0 = cf.Rcx;
}

#[cfg(target_arch = "x86")]
fn trampoline_blob() -> &'static [u8] {
    TRAMPOLINE_I386
}
#[cfg(target_arch = "x86_64")]
fn trampoline_blob() -> &'static [u8] {
    TRAMPOLINE_AMD64
}

// ===========================================================================
// i386 trampoline — REAL machine code (workstream B, this file's sibling
// aptramp32.S is the source of truth; assembled by regen-trampoline.sh with
// the system GNU as/ld → flat binary at origin 0, matching aptramp64.S's
// recipe). Real mode -> protected mode with a flat temp GDT -> load kernel
// CR3/CR4 -> enable paging via kernel CR0 (no long-mode leg — i386 stays in
// 32-bit protected mode throughout) -> install the real per-AP GDT/IDT ->
// clear the AP's own TSS busy bit + ltr -> reload data segments -> load ESP
// from the ALREADY-BUILT AP_SETUP_STACK stack image (mproc.c) -> far-return
// into KiSystemStartup. See aptramp32.S's header for the full byte-offset
// contract and the verified i386 KiSystemStartup arg contract (stdcall,
// LoaderBlock pre-pushed onto the AP stack by mproc.c — NOT passed in a
// register, unlike amd64's rcx). 212 bytes as of the 2026-07-10 build
// (sha256 cbbe3fd0…); regen + re-check-in together whenever aptramp32.S
// changes. Disassembly-verified in both 16-bit real-mode and 32-bit
// protected-mode objdump views — see VERIFY.md.
// ===========================================================================
static TRAMPOLINE_I386: &[u8] = include_bytes!("aptramp32.bin");

// ===========================================================================
// amd64 trampoline — assembly LISTING (assemble to bytes).
//
// Real mode -> protected mode -> enable PAE + long mode -> 64-bit entry.
// The AP's CR3 (PARAM.cr3) is a valid long-mode PML4 that identity-maps the
// stub page AND maps kernel space (it's __readcr3() from the BSP, so it already
// does). Sequence:
//
//   [BITS 16]
//   _start:
//       cli
//       mov ax, cs : mov ds, ax
//       lgdt [tmp_gdtr - _start]     ; GDT with a 64-bit code descriptor
//       mov eax, cr0 : or eax,1 : mov cr0, eax     ; PE
//       jmp 0x08:pm32                 ; 32-bit code seg
//   [BITS 32]
//   pm32:
//       mov ax,0x10 : mov ds,ax : es,ax : ss,ax
//       mov eax, [PARAM+off(cr4)]     ; kernel CR4 (has PAE=1)
//       mov cr4, eax
//       mov eax, [PARAM+off(cr3_lo)]  ; PML4 physical (< 4GB assumed for stub)
//       mov cr3, eax
//       mov ecx, 0xC0000080           ; IA32_EFER
//       rdmsr : or eax, 1<<8 : wrmsr  ; EFER.LME
//       mov eax, [PARAM+off(cr0)]     ; kernel CR0 (PG|PE) -> long mode active
//       mov cr0, eax
//       jmp 0x18:lm64                 ; 64-bit code seg in tmp GDT
//   [BITS 64]
//   lm64:
//       lgdt [PARAM+off(gdtr_limit)]  ; real per-AP GDT (10-byte pseudo-desc)
//       lidt [PARAM+off(idtr_limit)]
//       mov rsp, [PARAM+off(stack)]
//       ; reload data segs (mostly ignored in long mode, but SS/FS/GS matter)
//       mov ax, [PARAM+off(seg_ds)] : mov ds,ax ... ss,ax
//       ; GS base for the PCR is set later by KiSystemStartup via MSR; skip here
//       mov ax, [PARAM+off(tr)] : ltr ax
//       push qword [PARAM+off(seg_cs)]
//       push qword [PARAM+off(entry)]
//       ; rcx/first-arg: LoaderBlock is already on the AP stack per mproc; amd64
//       ; passes arg0 in rcx — integrator must confirm the amd64 AP stack/rcx
//       ; contract (amd64 KeStartAllProcessors is UNIMPLEMENTED today, NOTES.md).
//       retfq                         ; -> KiSystemStartup in long mode
//
// REAL machine code: aptramp64.S assembled by regen-trampoline.sh (system
// GNU as/ld → flat binary at origin 0). The .S in this directory is the
// source of truth; its header documents the byte-offset contract the
// ApTrampolineParams const-asserts above pin down. 274 bytes as of the
// 2026-07-10 build (sha256 8eb91bc8…); regen + re-check-in together.
// ===========================================================================
static TRAMPOLINE_AMD64: &[u8] = include_bytes!("aptramp64.bin");
