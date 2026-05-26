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

// Phase 3 (smss.exe) + Phase 4 (basesrv.dll) translations sit under
// src/smss/ and src/basesrv/ respectively — not yet `pub mod`d in.
// Rustconv batched 7/9 smss + 5/7 basesrv files on 2026-05-25 (~$0.92
// API spend, see feedback_rustconv_main_repo_leak for the
// main-repo-leak gotcha). Integration tail: ~100 per-site cast/anon-
// union fixups + the >1000 LOC files (sminit.c 2581 / pagefile.c 1150
// / dosdev.c 1110 / vdm.c 1459) still want rustconv translate
// (per feedback_rustconv_translate_vs_batch). Wire them in once
// per-file cargo check ×3 is green; until then keep them silent.
//
// `csrss_main_only`: when set, csrss.exe consumes ONLY the `_main`
// shim from `csrss.rs`. The rest of the modules below are gated OUT
// so they don't produce a duplicate static copy of csrsrv's globals
// (notably `_CsrApiPort`) in csrss.exe. csrss.exe instead resolves
// `CsrServerInitialization` (and friends) via PE imports from
// csrsrv.dll, which means the *real* csrsrv.dll instance of the data
// statics gets written by `NtCreatePort` and read back by
// `CsrQueryApiPort` from any other DLL (winsrv, basesrv, …).
#[cfg(not(feature = "csrss_main_only"))]
pub mod smss;
#[cfg(not(feature = "csrss_main_only"))]
pub mod basesrv;

// Phase 1 — SM/CSR LPC protocol contract (translated 2026-05-16).
// csrlib = client CSR API + capture-buffer marshalling (lands in
// ntdll.dll — cohabitation flip deferred, see audit §csrlib). smlib =
// SM client (consumed by csrsrv.dll + smss.exe; cohabitation flips in
// their phases). Modules carry the #[no_mangle] protocol symbols.
#[cfg(not(feature = "csrss_main_only"))]
pub mod csrlib;
#[cfg(not(feature = "csrss_main_only"))]
pub mod smlib;

// Phase 2 — csrsrv.dll server runtime (translated 2026-05-16).
// 35 csrsrv.spec exports; cohabited into csrsrv.dll (the 7 .c
// commented out in subsystems/csr/csrsrv/CMakeLists.txt).
#[cfg(not(feature = "csrss_main_only"))]
pub mod csrsrv;

// Standalone `#![no_std]` staticlib → needs its own panic handler +
// eh_personality (same role as kdcom-rust/hal-rust). USER-MODE: must
// NOT use privileged `hlt`/`wfe` (csrss.exe runs in ring 3) — abort
// the process via the Native API instead.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let banner = b"\n!!! [SMSS-CSRSS] RUST PANIC HIT !!!\n\0";
        ntdll_sys::nt::DbgPrint(banner.as_ptr() as *const i8);
        if let Some(loc) = info.location() {
            // Filenames in core are NUL-terminated UTF-8 string literals;
            // location().file() returns a &str whose underlying bytes are
            // typically a static C-string. Use the slice's ptr directly +
            // print line/column separately.
            let mut buf = [0u8; 256];
            let s = loc.file().as_bytes();
            let n = s.len().min(buf.len() - 1);
            buf[..n].copy_from_slice(&s[..n]);
            buf[n] = 0;
            let fmt = b"PANIC LOC: %s:%u:%u\n\0";
            ntdll_sys::nt::DbgPrint(
                fmt.as_ptr() as *const i8,
                buf.as_ptr() as *const i8,
                loc.line(),
                loc.column(),
            );
        }
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
