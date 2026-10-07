//! hal-rust — Rust translation of selected ReactOS HAL files.
//!
//! The main translation surface is x86 (i386 + amd64). ARM64 also builds a
//! small translated stub surface for the current QEMU HAL.
//!
//! Module layout — portable submodules + arch-specific backends:
//!
//! ```text
//! ├── irql/      ├── arbiter/      ├── acpi/      ── arch-PORTABLE
//! ├── kdpci/     ├── bus/          ├── pnpdd/        (no cfg, pure logic;
//! │                                                   take backends as traits)
//! ├── x86/       ── x86-shared backends (PIC/APIC/DMA/CMOS/timer/PCI/BIOS/etc.)
//! ├── i386/      ── i386-only paths (v86 trampoline, fastcall ABI plumbing)
//! └── amd64/     ── amd64-only paths
//! ```
//!
//! Files are translated from the C tree (`hal/halx86/{generic,legacy,
//! pic,acpi,apic}/`) and slotted into the appropriate target submodule
//! — NOT into a flat mirror of the C directory layout.
//!
//! Each translated function is `#[no_mangle] pub unsafe extern "..."` to
//! preserve the HAL ABI defined by `hal/hal.spec`. Once a function is
//! translated, the corresponding C source is removed from
//! `hal/halx86/{generic,legacy,...}.cmake` so there's no symbol clash.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// ============================================================================
// arch-portable submodules (no cfg, pure logic)
// ============================================================================

// Full-HAL PnP/ACPI/bus-arbiter/kd-pci subsystems — NOT in the C
// `mini_hal` MINI_HAL_SOURCE set; FreeLDR's loader HAL has none of them.
// Gating out of `--features minihal` keeps their kernel deps
// (Io*/Po*/Obf*/raw-dylib) out of libhal_rust_mini.a.
// feedback_freeldr_clean_build_regression
#[cfg(not(feature = "minihal"))]
pub mod acpi;
#[cfg(not(feature = "minihal"))]
pub mod arbiter;
#[cfg(not(feature = "minihal"))]
pub mod bus;
pub mod irql;
pub mod message;
#[cfg(not(feature = "minihal"))]
pub mod kdpci;
#[cfg(not(feature = "minihal"))]
pub mod pnpdd;

// ============================================================================
// arch-specific backends (cfg-gated)
// ============================================================================

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86;

#[cfg(target_arch = "x86")]
pub mod i386;

#[cfg(target_arch = "x86_64")]
pub mod amd64;

#[cfg(target_arch = "aarch64")]
pub mod arm64;

// ============================================================================
// Panic handler — HAL is a separate DLL from ntoskrnl, needs its own.
// ============================================================================

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Output a panic marker via debugcon port 0xe9 for QEMU diagnostics.
    // Same pattern as kernel-rust's panic_handler.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"HAL-PANIC:" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
        if let Some(loc) = info.location() {
            for &b in loc.file().as_bytes() {
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
            }
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b':');
            // Emit decimal line number
            let mut line = loc.line();
            let mut buf = [0u8; 10];
            let mut i = buf.len();
            if line == 0 {
                i -= 1;
                buf[i] = b'0';
            } else {
                while line > 0 {
                    i -= 1;
                    buf[i] = b'0' + (line % 10) as u8;
                    line /= 10;
                }
            }
            for &b in &buf[i..] {
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
            }
        }
        core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'\n');
    }
    let _ = info;
    loop {
        // halt loop — never returns
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack));
        }
    }
}

// Some `core` / `compiler_builtins` objects emit `.eh_frame` references to
// `rust_eh_personality`.  Panic = abort means unwinding never runs, but we
// still need the symbol so the linker can resolve the .eh_frame entries.
// Same stub kernel-rust uses (kernel-rust/src/lib.rs).
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
