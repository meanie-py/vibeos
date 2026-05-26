//! freeldr-rust — Rust translation of FreeLDR static components.
//!
//! Compiled as `crate-type = ["staticlib"]` → `libfreeldr_rust.a`,
//! linked into the cmake-built freeldr lib (freeldr.sys/uefildr.efi/
//! rosload.exe) with the matching `.c` commented out in
//! boot/freeldr/freeldr/CMakeLists.txt — hal-rust cohabitation, not a
//! DLL cargo-link (FreeLDR is a pre-kernel freestanding binary, no
//! ntoskrnl/CRT; FFI is to freeldr-internal C: MmAllocateMemoryWithType,
//! RtlImageNtHeader, TRACE/ERR, etc.).
//!
//! Crown jewel: `peloader.c` — the PE loader that loads ntoskrnl.
//! Once Rust, the entire kernel-mode load path is Rust end-to-end.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

// Modules added as rustconv writes each translated file.
pub mod ffi;
pub mod frldr;
pub mod peloader;

// 🟢 sweep — cache unit (lib/cache/{blocklist,cache}.c).
pub mod blocklist;
pub mod cache;

// 🟢 sweep — inifile unit (lib/inifile/{ini_init,inifile,parse}.c).
pub mod ini_init;
pub mod inifile;
pub mod parse;

// 🟢 sweep — mm unit (lib/mm/{meminit,mm,heap}.c).
pub mod meminit;
pub mod mm;
pub mod heap;

// 🟢 sweep — lib-pure tail.
pub mod rs232;
pub mod libsupp;
pub mod arcsupp;
pub mod arcname;
pub mod arcemul;
pub mod archwsup;
pub mod settings;
pub mod debug;

// 🟢 sweep — fs unit (lib/fs/{fs,iso,fat}.c; ntfs/ext/btrfs pruned).
pub mod fs;
pub mod iso;
pub mod fat;

// 🟢 sweep — disk unit (disk/{partition,ramdisk}.c, ntldr/conversion.c).
pub mod partition;
pub mod ramdisk;
pub mod conversion;

// 🟢 sweep — pc-hw unit (arch/i386/pc/{machpc,pcvideo,pcdisk,pcmem,pcvesa,pchw}.c).
pub mod machpc;
pub mod pcvideo;
pub mod pcdisk;
pub mod pcmem;
pub mod pcvesa;
pub mod pchw;
// pc-hw sweep continued — pcbeep/pccons/pcrtc (BIOS-only, not built for uefi).
#[cfg(not(feature = "uefi"))]
pub mod pcbeep;
#[cfg(not(feature = "uefi"))]
pub mod pccons;
#[cfg(not(feature = "uefi"))]
pub mod pcrtc;
// i386-hw — hwapm (BIOS-only), i386idt (i386 only; lives in both pcat
// and uefi i386 builds, so always built for i386).
#[cfg(not(feature = "uefi"))]
pub mod hwapm;
#[cfg(target_arch = "x86")]
pub mod i386idt;

// 🟢 sweep — i386-hw unit (arch/i386/{hwdisk,hwpci,hwacpi}.c, arch/drivers/hwide.c).
pub mod hwdisk;
pub mod hwpci;
pub mod hwacpi;
pub mod hwide;

// 🟢 sweep — uefi-hw unit (arch/uefi/{uefidisk,uefimem,uefivid,ueficon,uefihw}.c).
pub mod uefidisk;
pub mod uefimem;
pub mod uefivid;
pub mod ueficon;
pub mod uefihw;
// uefi-only translation unit — UEFI MACHVTBL fill + stubs + utils + entry.
// uefisetup/uefi_stubs/uefiutil are .a-bound (cohabitation under feature
// "uefi"); uefildr.rs is the .efi binary-entry translation gated behind
// `uefildr-rust-entry` so C uefildr.c retains symbol ownership for the
// uefildr.exe link until a future cohabitation flip.
#[cfg(feature = "uefi")]
pub mod uefisetup;
#[cfg(feature = "uefi")]
pub mod uefi_stubs;
#[cfg(feature = "uefi")]
pub mod uefiutil;
#[cfg(feature = "uefi")]
pub mod uefildr;
// PC-link fallback for vidfb.c's UEFI-only FbCons*/GlobalSystemTable
// (ueficon/uefivid extern-consume them; their C providers are UEFI-only
// so freeldr_pe/BIOS has no provider). BIOS-only — on the `uefi`
// archive vidfb.c/uefildr.c own those symbols (dup otherwise).
#[cfg(not(feature = "uefi"))]
pub mod vidfb_pc_fallback;

// 🟢 sweep — winldr kernel-handoff unit (ntldr/{winldr,registry,wlregistry,
// wlmemory,ntldropts}.c + ntldr/arch/<a>/winldr.c + arch/i386/ntoskrnl.c).
// arch winldr.c are per-arch in C (CMakeLists FREELDR_NTLDR_SOURCE) → cfg-gate.
pub mod winldr;
pub mod registry;
pub mod wlregistry;
pub mod wlmemory;
pub mod ntldropts;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod ntoskrnl;
#[cfg(target_arch = "x86")]
pub mod winldr_i386;
#[cfg(target_arch = "x86_64")]
pub mod winldr_amd64;
#[cfg(target_arch = "aarch64")]
pub mod winldr_arm64;

// 🟢 sweep — setup/legacy unit (ntldr/{setupldr,inffile,headless}.c +
// arch/i386/{drivemap,i386bug}.c + lib/fs/pxe.c). setupldr/inffile/
// headless = arch-neutral loader logic; drivemap/i386bug/pxe = x86
// BIOS-era (PC-AT i386/amd64, not arm64) → cfg-gate.
pub mod setupldr;
pub mod inffile;
pub mod headless;
// drivemap is BIOS-only (x86 BIOS Int13 drive remapping); UEFI's stubs.c
// provides DriveMapGetBiosDriveNumber returning 0, so gate the BIOS impl
// out under feature="uefi" to avoid the dup #[no_mangle] collision.
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
pub mod drivemap;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod i386bug;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod pxe;

// --- ui unit WIP (subagent) — temporarily un-wired so the fs milestone
// builds/commits at green; re-enable when the ui unit converges. ---
pub mod ui_keystone;
pub mod ui;
pub mod tui;
pub mod tuimenu;
pub mod minitui;
pub mod noui;
pub mod video;


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"FREELDR-RUST-PANIC\n" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
    }
    let _ = info;
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe { core::arch::asm!("hlt"); }
        #[cfg(target_arch = "aarch64")]
        unsafe { core::arch::asm!("wfe", options(nomem, nostack)); }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
