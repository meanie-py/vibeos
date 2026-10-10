#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// arbiter.a — Rust translation of the PnP resource arbiter library
// (sdk/lib/drivers/arbiter, upstream's #9484 engine: arbiter.c entry.c handler.c ordering.c
// range.c transaction.c). crate-type=staticlib → libarbiter_rust.a, linked into ntoskrnl +
// pcix.sys in place of the C `arbiter` lib (kept on disk). Own panic_handler/eh_personality
// absorbed by --allow-multiple-definition on the kernel link (same as cmlib-rust).

pub mod prelude;
pub mod types;

pub mod arbiter;
pub mod entry;
pub mod handler;
pub mod ordering;
pub mod range;
pub mod transaction;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// arbiter.rs calls KeGetCurrentIrql OUT-OF-LINE. On i386 the kernel exports a real
// `_KeGetCurrentIrql@0` symbol so this resolves; on amd64/arm64 it's a FORCEINLINE
// (read cr8 / PCR.Irql) that the kernel materializes only in ntoskrnl-rust/
// shim_exports.rs. But arbiter-rust is a SEPARATE staticlib that links BEFORE
// libkernel_rust.a in the kernel's single-pass rlib order — so arbiter's undefined
// ref cannot back-resolve to the kernel's (earlier, unpulled) definition. arbiter
// therefore needs its OWN amd64 definition, kept here.
//
// NOTE: ntoskrnl-rust's shim_exports.rs defines the same amd64 symbol, and both
// objects are in the kernel archives (llvm-nm, 2026-09-26). Both bodies are the
// same `mov cr8` read, so either winning is correct. Deleting this one breaks the
// amd64 link (KeGetCurrentIrql undefined for arbiter's own callers). arm64
// resolves via the kernel's exported helper, so no shim needed.
// CC = ntkrnl-sys bindgen_i386 decl class (callers import THAT decl; a mismatched
// shim def in the same crate folds every such call to ud2 — the 2026-07-18 i386
// driver-fleet bomb class).
// nolint: duplicate-symbol (same `mov cr8` body as ntoskrnl's; see NOTE above)
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}
