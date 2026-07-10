//! kdcom-serial-rust — Rust translation of the classic NS16550 serial KD
//! transport (`drivers/base/kdcom`: kdcom.c + kddll.c + kdserial.c). Sibling
//! of kdgdb-rust; satisfies the same 8-symbol kddll spec (`KdD0Transition`,
//! `KdD3Transition`, `KdDebuggerInitialize0/1`, `KdReceivePacket`, `KdRestore`,
//! `KdSave`, `KdSendPacket`) over a COM port via cportlib (= cport-rust).
//!
//! Distinct from `kdcom-rust` (which is the kdrosdbg stub that ships as
//! kdcom.dll on the default _WINKD_=FALSE build). This serial transport only
//! builds under _WINKD_=TRUE, never on the default llvm-mingw build, so the
//! crate is validated standalone via `cargo build -p kdcom-serial-rust`
//! (i386 + amd64; no arm64 in the C).
//!
//! Module map (per the `kdcom-serial` rustconv Target, src_subdir = src):
//!   kdcom.c → kdcom    kddll.c → kddll    kdserial.c → kdserial

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

#[macro_use]
pub mod kddll_h;
pub mod kdcom;
pub mod kddll;
pub mod kdserial;

// SharedUserData is the C macro `((KUSER_SHARED_DATA*)KI_USER_SHARED_DATA)` — a
// fixed kernel VA, NOT a real symbol. ntkrnl_sys::globals' extern is satisfied
// by a kd64 shim absent from a standalone transport DLL, so define the fixed
// address here and `use crate::SharedUserData` in the modules (shadows the
// ntkrnl_sys glob). i386 kernel = 0xFFDF0000; amd64/arm64 = 0xFFFFF78000000000.
#[cfg(target_arch = "x86")]
pub const SharedUserData: *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA =
    0xFFDF_0000usize as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const SharedUserData: *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA =
    0xFFFF_F780_0000_0000usize as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;

// Under `embedded` this crate is an rlib inside the kernel graph, which
// already owns the lang item + personality — only the standalone kdcom.dll
// staticlib build defines them here.
#[cfg(not(feature = "embedded"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(feature = "embedded"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
