//! hal-rust — Rust translation of the ReactOS x86 HAL.
//!
//! Per `docs/engineering/hal-translation-audit.md` (Strategic update
//! 2026-05-10): only x86 (i386 + amd64) is in scope for Rust translation.
//! arm64 keeps its C stubs in `hal/halarm64/`. This crate is therefore
//! built only for x86_64 and i686 targets.
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

pub mod acpi;
pub mod arbiter;
pub mod bus;
pub mod irql;
pub mod kdpci;
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
    }
}
