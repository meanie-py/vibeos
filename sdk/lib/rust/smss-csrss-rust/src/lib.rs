//! smss-csrss-rust — Rust translation of the SMSS→CSRSS user-mode
//! spine (smss.exe, csrss.exe, csrsrv.dll, basesrv.dll, …).
//!
//! Per-binary cohabitation: each translated `.c` is commented out in
//! its CMakeLists and the `#[no_mangle]` symbol here takes over. FFI
//! surface is `ntdll-sys` (the user-mode Native API + SM/CSR/BASE
//! protocol). See docs/engineering/smss-csrss-translation-audit.md.
//!
//! NOT a standalone binary — an `rlib` linked into the cmake-built
//! native PEs (smss.exe / csrss.exe / csrsrv.dll …).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// csrss.exe `_main` — clean rustconv translation after the P1a
// pipeline fixes. Linked into csrss.exe via the rtl-style cmake
// cargo→.a + --allow-multiple-definition swap-in (csrss.c kept until
// N green smokes).
pub mod csrss;

// Phase 1 — SM/CSR LPC protocol contract (translated 2026-05-16).
// csrlib = client CSR API + capture-buffer marshalling (lands in
// ntdll.dll — cohabitation flip deferred, see audit §csrlib). smlib =
// SM client (consumed by csrsrv.dll + smss.exe; cohabitation flips in
// their phases). Modules carry the #[no_mangle] protocol symbols.
pub mod csrlib;
pub mod smlib;

// Phase 2 — csrsrv.dll server runtime (translated 2026-05-16).
// 35 csrsrv.spec exports; cohabited into csrsrv.dll (the 7 .c
// commented out in subsystems/csr/csrsrv/CMakeLists.txt).
pub mod csrsrv;

// Standalone `#![no_std]` staticlib → needs its own panic handler +
// eh_personality (same role as kdcom-rust/hal-rust). USER-MODE: must
// NOT use privileged `hlt`/`wfe` (csrss.exe runs in ring 3) — abort
// the process via the Native API instead.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        // STATUS_UNSUCCESSFUL (0xC0000001). Translated code is
        // panic=abort and the C had no panics — this never executes
        // in practice, but the symbol must resolve and be sound.
        ntdll_sys::nt::NtTerminateProcess(
            ntdll_sys::nt::NtCurrentProcess(),
            0xC0000001u32 as i32,
        );
    }
    loop {
        core::hint::spin_loop();
    }
}

// `core`/`compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`; panic=abort never unwinds but the symbol
// must resolve. Same stub as kdcom-rust/hal-rust/kernel-rust.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
