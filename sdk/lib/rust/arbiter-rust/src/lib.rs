#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// arbiter.a — Rust translation of the PnP resource arbiter library
// (sdk/lib/drivers/arbiter/arbiter.c). crate-type=staticlib → libarbiter_rust.a,
// linked into ntoskrnl + pcix.sys in place of the C `arbiter` lib (kept on disk).
// Own panic_handler/eh_personality absorbed by --allow-multiple-definition on the
// kernel link (same as cmlib-rust). DeepSeek+c2rust translation (Batch 4).
pub mod arbiter;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// KeGetCurrentIrql is a real ntoskrnl export on i386 but a FORCEINLINE (read cr8)
// on amd64/arm64, so the kernel does not export it on amd64. arbiter.rs calls it
// out-of-line; when libarbiter_rust.a is linked into pcix.sys (which links the
// partial libntoskrnl.a static archive, not the full cargo kernel) the symbol is
// undefined on a from-scratch amd64 link. Provide the trivial cr8-read shim for
// amd64 only — same cohab-shim pattern as cdrom-rust/cdfs-rust ddk_shims. In the
// ntoskrnl link the kernel's own ke definition wins first under
// --allow-multiple-definition (this copy is harmless there). arm64 resolves via
// the kernel's exported helper, so no shim needed.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}
