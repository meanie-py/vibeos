//! kdgdb-rust — Rust translation of the ReactOS `kdgdb` KD transport (the
//! GDB-remote-protocol stub that, under `_WINKD_`+`GDB`, builds `kdcom.dll`).
//!
//! Translated from `drivers/base/kdgdb/*.c` (one C file → one module). The
//! DLL satisfies the 8-symbol `kdgdb.spec` ABI (`KdD0Transition`,
//! `KdD3Transition`, `KdDebuggerInitialize0/1`, `KdReceivePacket`,
//! `KdRestore`, `KdSave`, `KdSendPacket`) — same kddll contract as kdcom.
//! Each export is `#[no_mangle] pub unsafe extern "system"` so i386 stdcall
//! decoration (`_Name@N`) matches the spec's `@ stdcall` entries.
//!
//! Compiled as `crate-type = ["staticlib"]` → `libkdgdb_rust.a`. The default
//! llvm-mingw build forces `_WINKD_=FALSE` (config.cmake), so kdgdb is never
//! wired into the cmake build; this crate is validated standalone via
//! `cargo build -p kdgdb-rust` for i386 + amd64 (kdgdb has no arm64 path).
//!
//! Module map (per the `kdgdb` rustconv Target, src_subdir = src):
//!   gdb_input.c   → gdb_input     gdb_receive.c → gdb_receive
//!   gdb_send.c    → gdb_send      kdcom.c       → kdcom (serial transport)
//!   kdpacket.c    → kdpacket      utils.c       → utils
//!   i386_sup.c    → i386_sup (cfg x86)   amd64_sup.c → amd64_sup (cfg x86_64)

// TODO(kdgdb full GDB-protocol debugging): kdgdb-rust links into a real
// kdcom.dll and its KdDebuggerInitialize0/1 run at boot (proven by swapping it
// in as kdcom.dll — see kdgdb-cargo). But the GDB-remote-protocol core
// (KdSendPacket/KdReceivePacket → gdb_input/gdb_send) only runs when the kernel
// is in _WINKD_ KD-protocol mode. On llvm-mingw, config.cmake forces
// _WINKD_=FALSE, and -D_WINKD_ is a C define that never reaches Rust — so
// ntoskrnl-rust's kd64/kd/kdinit were translated frozen to the KDBG (non-WINKD)
// path. Exercising kdgdb end-to-end (gdb-multiarch over qemu/serial.sock, NOT
// QEMU's host gdbstub) requires teaching ntoskrnl-rust the _WINKD_ cfg path —
// a separate, larger effort. See memory project_kdgdb_translation (SPIKE).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

// kdgdb.h `#define KDDBGPRINT(...)` is a no-op when KDDEBUG is undefined (the
// default). rustconv wrongly externed it as a varargs C fn; model it as a
// no-op macro that swallows its arguments so call sites compile away.
#[macro_export]
macro_rules! KDDBGPRINT {
    ($($arg:tt)*) => {{}};
}

// kdgdb.h FORCEINLINE helpers that live in no .c file (so were never
// translated). Hand-ported faithfully from the header.
pub mod kdgdb_h;

pub mod gdb_input;
pub mod gdb_receive;
pub mod gdb_send;
pub mod kdcom;
pub mod kdpacket;
pub mod utils;

#[cfg(target_arch = "x86")]
pub mod i386_sup;
#[cfg(target_arch = "x86_64")]
pub mod amd64_sup;

// kdgdb-rust is built as a standalone `#![no_std]` staticlib, so it needs its
// own panic handler. Halt loop with a debugcon (0xe9) marker, mirroring
// kdcom-rust / hal-rust / kernel-rust.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"KDGDB-PANIC\n" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
    }
    let _ = info;
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        core::hint::spin_loop();
    }
}

// `core` / `compiler_builtins` emit `.eh_frame` refs to `rust_eh_personality`.
// Panic = abort never unwinds, but the symbol must resolve. Same stub as
// kdcom-rust / hal-rust / kernel-rust.
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
