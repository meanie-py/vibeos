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
