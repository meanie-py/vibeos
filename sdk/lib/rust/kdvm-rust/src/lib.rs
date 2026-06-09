//! kdvm-rust — Rust translation of the VirtualBox KD transport
//! (`drivers/base/kdvm`: kdvm.c + kdvbox.c, ships as `kdvbox.dll`). Sibling of
//! kdgdb-rust / kdcom-serial-rust; satisfies the same 8-symbol kddll spec.
//!
//! _WINKD_-only, never built on the default llvm-mingw build; validated
//! standalone via `cargo build -p kdvm-rust` (i386 + amd64).
//!
//! Module map (per the `kdvm` rustconv Target, src_subdir = src):
//!   kdvm.c → kdvm    kdvbox.c → kdvbox
//! kdvbox_asm.S's single `KdVmExchange` (VirtualBox backdoor port I/O at
//! 0x5659) is rendered below as a cfg-gated inline-asm fn instead of a
//! separate .S — i386 fastcall (`@KdVmExchange@8`), amd64 sysv (`KdVmExchange`).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

pub mod kdvm_h;
pub mod kdvm;
pub mod kdvbox;

// kdvbox_asm.S → Rust. The VBox backdoor: write the FIRST argument
// (PhysicalAddress) to I/O port 0x5659. The original asm:
//   i386 (@KdVmExchange@8, FASTCALL): xchg eax,ecx; mov edx,0x5659; out dx,eax
//   amd64 (KdVmExchange):             xchg rax,rcx; mov edx,0x5659; out dx,eax
// i.e. it moves arg1 into (e/r)ax and OUTs the low 32 bits, returning VOID.
// Signature mirrors kdvm.h:  VOID FASTCALL KdVmExchange(ULONG_PTR, SIZE_T).
// On i386 FASTCALL passes arg1 in ecx, arg2 in edx → extern "fastcall".
// On amd64 NTAPI is the SysV/Win64 cdecl → extern "C" (arg1 in rcx).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KdVmExchange(
    physical_address: usize,
    _buffer_size: usize,
) {
    core::arch::asm!(
        "mov edx, 0x5659",
        "out dx, eax",
        in("eax") physical_address as u32,
        out("edx") _,
        options(nostack, preserves_flags),
    );
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KdVmExchange(
    physical_address: usize,
    _buffer_size: usize,
) {
    core::arch::asm!(
        "mov edx, 0x5659",
        "out dx, eax",
        in("eax") physical_address as u32,
        out("edx") _,
        options(nostack, preserves_flags),
    );
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"KDVM-PANIC\n" {
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

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
