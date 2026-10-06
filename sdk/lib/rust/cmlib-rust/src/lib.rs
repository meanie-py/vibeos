//! cmlib-rust — Rust translation of the kernel Configuration Manager hive
//! library (`sdk/lib/cmlib`), built as a cohabitation staticlib
//! (`libcmlib_rust.a`). The kernel `cmlib` cmake target REMOVE_ITEMs each
//! flipped `.c` from its SOURCE and links this archive, so the
//! `#[no_mangle] extern "C"` symbols here (CMAPI = cdecl) take over for the
//! kernel build. The bootloader (`blcmlib`) and host (`cmlibhost`) cmlib
//! variants keep the C until those links are flipped too.
//!
//! Each cmlib `.c` becomes a module here; symbols match the C ABI exactly.
//! For the KERNEL cmlib, `CMAPI` is `#define CMAPI NTAPI` (cmlib.h:218) →
//! stdcall on i386, so every `#[no_mangle]` fn here is `extern "system"`
//! (emits `_Name@N`, matching the C objects' `_CmCreateRootNode@8` etc. —
//! verified via nm). Do NOT use `extern "C"`: that would emit an undecorated
//! `Name` symbol and break the i386 cohab link. (The empty `#define CMAPI` in
//! psdk/cfgmgr32.h is the unrelated USER-MODE cfgmgr32 API — different ABI.)
//! Hive/cell types come from `ntkrnl_sys::ddk` (bindgen of cmlib's own headers).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// Standalone `#![no_std]` cohab staticlib (same as cnt-rust-lib / hal-rust):
// it needs its own panic handler + eh_personality. When linked into ntoskrnl
// alongside kernel-rust these duplicate that runtime's copies and are absorbed
// by `--allow-multiple-definition` (kept on the kernel link flags; panic=abort
// never unwinds so the bodies never run). Kept minimal — no memcpy/memset here
// (kernel-rust + the C shims provide them; extra defs risk the lld
// "X was replaced" MODULE-relink hazard noted in cnt-rust-lib).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack));
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

// No-op shims for the cmlib.h debug macros (ASSERT / PAGED_CODE). In C these are
// NDEBUG no-ops with no symbol; a few translated modules still reference them via
// local `extern "C"` declarations, so provide the _ASSERT / _PAGED_CODE symbols
// here to keep the crate self-contained on both the kernel and loader links.
// (glue.rs also has crate-local inline ASSERT(bool)/PAGED_CODE() for the modules
// that import them directly; those emit no symbol, so there is no clash.)
#[unsafe(no_mangle)]
pub extern "C" fn ASSERT(_cond: i32) {}
#[unsafe(no_mangle)]
pub extern "C" fn PAGED_CODE() {}

mod glue;
mod hivesum;
mod cmse;
mod cminit;
mod hivebin;
mod cmkeydel;
mod cmname;
// Phase 2 (deepseek-v4-pro mechanical, banked 2026-06-07): being driven to
// cargo-check green via deepdrive before correctness review.
mod cmvalue;
mod hivecell;
mod hivelock;
mod hivewrt;
mod cmheal;
mod hiveinit;
mod cmcheck;
mod cmindex;
