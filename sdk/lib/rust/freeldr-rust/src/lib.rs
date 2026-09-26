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

// cmboot.c (ntoskrnl/config) is the shared kernel+bootloader CM boot-init unit.
// It now lives in the standalone `cmboot-rust` crate so the kernel and the
// loaders link the SAME translation. Pull its #[no_mangle] CmpFind*/CmpFindDrivers
// /CmpSortDriverList symbols into the loader archives that need them (uefildr via
// `uefi`, rosload via `rosload`); `cmboot_shims` supplies the loader-local
// InitSafeBootMode/CMTRACE/IS_NULL_TERMINATED that cmboot.c used to define
// inline. The default (BIOS freeldr.sys) archive does not link cmboot.
#[cfg(any(feature = "uefi", feature = "rosload"))]
extern crate cmboot_rust;
#[cfg(any(feature = "uefi", feature = "rosload"))]
mod cmboot_shims;

// UEFI escape-helper asm (uefiasm.S) flipped to Rust global_asm! — self-gated
// to feature="uefi" inside the file.
mod uefiasm;
// Remaining loader asm (misc.S / linux.S / i386trap.S) flipped to global_asm!,
// per-file cfg-gated inside.
mod loader_asm;

// Modules added as rustconv writes each translated file.
pub mod ffi;
pub mod frldr;
pub mod peloader;

// freeldr.c — top-level entry-point translation unit. Contains BootMain
// (called from arch/*/multiboot.S asm), LoadRosload, LaunchSecondStage-
// Loader, FrLdr{GetBootPath,GetBootDrive,GetBootPartition} accessors,
// plus the deliberate FPU-free CRT-shadow stubs (pow/log/log10/wctomb/
// mbtowc/iswctype). Cohabitation works because libfreeldr_rust.a is
// linked into freeldr_common BEFORE libcntpr in both pcat.cmake
// (target_link_libraries freeldr_pe freeldr_common cportlib libcntpr)
// and uefi.cmake (target_link_libraries uefildr uefifreeldr_common
// cportlib blcmlib blrtl libcntpr) — the Rust stubs satisfy the refs
// first, libcntpr's real pow/log/wctomb members never get pulled.
//
// Gated OFF for `rosload`: rosload.exe is a SEPARATE PE binary and
// imports FrLdr* / BootMain accessors from freeldr.sys via the importlib.
// Duplicating FrLdrBootPath here would give rosload its own data slot
// distinct from freeldr.sys's (per feedback_static_lib_shared_globals_
// per_consumer.md) — wrong, since both must observe the same boot path.
#[cfg(not(feature = "rosload"))]
pub mod freeldr;

// arch/arm64/debug.c — ARM64 UEFI PL011 UART + Rs232Port stubs.
// Active on arm64 UEFI builds; rs232.rs is gated out on arm64 (x86
// port-IO model doesn't apply) so no Rs232Port* symbol collision.
#[cfg(target_arch = "aarch64")]
pub mod arm64_debug;
#[cfg(target_arch = "aarch64")]
mod arm64_serial_acpi;

// 🟢 sweep — cache unit (lib/cache/{blocklist,cache}.c).
// Gated off for `rosload` — rosload uses CacheRead* indirectly via Arc/FS
// imports from freeldr.sys.
#[cfg(not(feature = "rosload"))]
pub mod blocklist;
#[cfg(not(feature = "rosload"))]
pub mod cache;

// 🟢 sweep — inifile unit (lib/inifile/{ini_init,inifile,parse}.c).
// Gated off for `rosload` — Ini* are exported by freeldr.sys (freeldr.spec)
// and imported via add_importlibs(rosload freeldr).
#[cfg(not(feature = "rosload"))]
pub mod ini_init;
#[cfg(not(feature = "rosload"))]
pub mod inifile;
#[cfg(not(feature = "rosload"))]
pub mod parse;

// 🟢 sweep — mm unit (lib/mm/{meminit,mm,heap}.c).
// Gated off for `rosload` — MmAllocateMemoryWithType/FrLdrHeap* exported
// by freeldr.sys; rosload imports.
#[cfg(not(feature = "rosload"))]
pub mod meminit;
#[cfg(not(feature = "rosload"))]
pub mod mm;
#[cfg(not(feature = "rosload"))]
pub mod heap;

// 🟢 sweep — lib-pure tail.
// rs232.rs is x86-only: uses cportlib's CpInitialize/CpPutByte which
// drive 8250-family UARTs at I/O ports (BaseArray = [0,0x3F8,0x2F8,...]).
// arm64 has no x86 port I/O and uses the PL011 MMIO UART via arm64_debug.
// Gated off for `rosload` — Rs232Port* and serial init owned by freeldr.sys.
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "rosload")))]
pub mod rs232;
// libsupp.rs — RtlAssert / RtlAssertImpl. rosload DEFINES its own copy
// (no spec export, called locally) so keep it on for rosload too.
pub mod libsupp;
// arcsupp/arcname/arcemul/archwsup/settings/debug — ARC path + machine
// helpers; all exported by freeldr.sys, gated off for rosload.
#[cfg(not(feature = "rosload"))]
pub mod arcsupp;
#[cfg(not(feature = "rosload"))]
pub mod arcname;
#[cfg(not(feature = "rosload"))]
pub mod arcemul;
#[cfg(not(feature = "rosload"))]
pub mod archwsup;
#[cfg(not(feature = "rosload"))]
pub mod settings;
#[cfg(not(feature = "rosload"))]
pub mod debug;

// 🟢 sweep — fs unit (lib/fs/{fs,iso,fat}.c; ntfs/ext/btrfs pruned).
// Gated off for `rosload` — FsOpenFile and friends exported by freeldr.sys.
#[cfg(not(feature = "rosload"))]
pub mod fs;
#[cfg(not(feature = "rosload"))]
pub mod iso;
#[cfg(not(feature = "rosload"))]
pub mod fat;

// 🟢 sweep — disk unit (disk/{partition,ramdisk}.c, ntldr/conversion.c).
// conversion.rs is DEFINED in rosload (used by setupldr/winldr); keep on
// for rosload. partition/ramdisk exported by freeldr.sys, gated off.
#[cfg(not(feature = "rosload"))]
pub mod partition;
#[cfg(not(feature = "rosload"))]
pub mod ramdisk;
pub mod conversion;

// 🟢 sweep — pc-hw unit (arch/i386/pc/{machpc,pcvideo,pcdisk,pcmem,pcvesa,pchw}.c).
// Gated off for `rosload` — Mach* dispatch + MachVtbl + ChainLoadBiosBootSector
// owned by freeldr.sys (exported in spec); rosload uses via importlib.
#[cfg(not(feature = "rosload"))]
pub mod machpc;
#[cfg(not(feature = "rosload"))]
pub mod pcvideo;
#[cfg(not(feature = "rosload"))]
pub mod pcdisk;
#[cfg(not(feature = "rosload"))]
pub mod pcmem;
#[cfg(not(feature = "rosload"))]
pub mod pcvesa;
#[cfg(not(feature = "rosload"))]
pub mod pchw;
// pc-hw sweep continued — pcbeep/pccons/pcrtc (BIOS-only, not built for uefi).
#[cfg(all(not(feature = "uefi"), not(feature = "rosload")))]
pub mod pcbeep;
#[cfg(all(not(feature = "uefi"), not(feature = "rosload")))]
pub mod pccons;
#[cfg(all(not(feature = "uefi"), not(feature = "rosload")))]
pub mod pcrtc;
// i386-hw — hwapm (BIOS-only), i386idt (i386 only; lives in both pcat
// and uefi i386 builds, so always built for i386).
#[cfg(all(not(feature = "uefi"), not(feature = "rosload")))]
pub mod hwapm;
#[cfg(all(target_arch = "x86", not(feature = "rosload")))]
pub mod i386idt;

// 🟢 sweep — i386-hw unit (arch/i386/{hwdisk,hwpci,hwacpi}.c, arch/drivers/hwide.c).
#[cfg(not(feature = "rosload"))]
pub mod hwdisk;
#[cfg(not(feature = "rosload"))]
pub mod hwpci;
#[cfg(not(feature = "rosload"))]
pub mod hwacpi;
#[cfg(not(feature = "rosload"))]
pub mod hwide;

// 🟢 sweep — uefi-hw unit (arch/uefi/{uefidisk,uefimem,uefivid,ueficon,uefihw}.c).
// Gated off for `rosload` (UEFI hardware, not relevant; also avoid pulling
// UEFI type deps into the BIOS-or-rosload archive variants).
#[cfg(not(feature = "rosload"))]
pub mod uefidisk;
#[cfg(not(feature = "rosload"))]
pub mod uefimem;
#[cfg(not(feature = "rosload"))]
pub mod uefivid;
#[cfg(not(feature = "rosload"))]
pub mod ueficon;
#[cfg(not(feature = "rosload"))]
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

// Boot menu / OS loader dispatch — present in both uefildr.efi and rosload.exe.
// freeldr.sys (BIOS) does NOT have these (the BIOS path hands off to rosload).
#[cfg(any(feature = "uefi", feature = "rosload"))]
pub mod options;
#[cfg(any(feature = "uefi", feature = "rosload"))]
pub mod oslist;
#[cfg(any(feature = "uefi", feature = "rosload"))]
pub mod bootmgr;
#[cfg(any(feature = "uefi", feature = "rosload"))]
pub mod custom;
// miscboot.c lives in rosload.cmake; activated via the `rosload` feature
// (or the legacy `miscboot-rust-entry` opt-in). Per AGENTS, never delete .rs.
#[cfg(any(feature = "rosload", feature = "miscboot-rust-entry"))]
pub mod miscboot;
// PC-link fallback for vidfb.c's UEFI-only FbCons*/GlobalSystemTable
// (ueficon/uefivid extern-consume them; their C providers are UEFI-only
// so freeldr_pe/BIOS has no provider). BIOS-only — on the `uefi`
// archive vidfb.c/uefildr.c own those symbols (dup otherwise).
#[cfg(all(not(feature = "uefi"), not(feature = "rosload")))]
pub mod vidfb_pc_fallback;
// arch/vidfb.c (linear framebuffer driver) + arch/vgafont.c (8x16 bitmap).
// UEFI-only: only the UEFI link unit owns the FbCons*/VidFb* symbols
// (vidfb_pc_fallback.rs owns them under `not(feature="uefi")`).
#[cfg(feature = "uefi")]
pub mod vgafont;
#[cfg(feature = "uefi")]
pub mod vidfb;

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
// headless = arch-neutral loader logic — DEFINED in rosload (rosload.cmake
// owns the C copies; flipping = these Rust modules take over). Always on.
pub mod setupldr;
pub mod inffile;
pub mod headless;
// drivemap is BIOS-only (x86 BIOS Int13 drive remapping); UEFI's stubs.c
// provides DriveMapGetBiosDriveNumber returning 0, so gate the BIOS impl
// out under feature="uefi" to avoid the dup #[no_mangle] collision.
// rosload also owns drivemap (rosload.cmake has the .c i386-only).
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
pub mod drivemap;
// linuxboot — BIOS-only (pcat.cmake; uefi.cmake doesn't list it). x86 only.
// Also present in rosload.cmake (loaded by rosload's bootmgr → LoadAndBootLinux).
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
pub mod linuxboot;
// i386bug — FrLdrBugCheck/i386PrintException — exported via freeldr.spec
// (rosload imports). Gate off for rosload.
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "rosload")))]
pub mod i386bug;
// pxe — BIOS network-boot init; owned by freeldr.sys (PxeInit() called
// from hwdisk.c). Not in rosload.cmake — gate off for rosload.
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "rosload")))]
pub mod pxe;

// Tiny printf/puts shims for rosload — tui.rs (which owns them on the
// freeldr.sys/uefildr archives) is gated off for rosload, but rosload's
// translated winldr/meminit/noui call sites still need a printf symbol.
// Forwards to TuiPrintf (imported from freeldr.sys via the importlib).
#[cfg(feature = "rosload")]
pub mod rosload_shims;

// arch/i386/halstub.c → Rust. HAL dispatch-table stubs, rosload.exe only
// (i386). FASTCALL/stdcall fn-ptrs require x86 ABI strings, so double-gate.
#[cfg(all(target_arch = "x86", feature = "rosload"))]
pub mod halstub;

// UI unit (ui/tui/tuimenu/minitui/noui/video + ui_keystone). UI dispatch
// is owned by freeldr.sys (UiVtbl, UiDisplayMenu, TuiPrintf exported in
// freeldr.spec); rosload imports via add_importlibs.
#[cfg(not(feature = "rosload"))]
pub mod ui_keystone;
#[cfg(not(feature = "rosload"))]
pub mod ui;
#[cfg(not(feature = "rosload"))]
pub mod tui;
#[cfg(not(feature = "rosload"))]
pub mod tuimenu;
#[cfg(not(feature = "rosload"))]
pub mod minitui;
#[cfg(not(feature = "rosload"))]
pub mod noui;
#[cfg(not(feature = "rosload"))]
pub mod video;


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"FREELDR-RUST-PANIC\n" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
    }
    // arm64 had a SILENT panic handler (loop {wfe}) which made every Rust
    // panic look like an opaque boot hang. Emit the marker + file:line over
    // PL011 (QEMU virt UART DR @ 0x09000000) so panics are diagnosable.
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let putc = |c: u8| crate::frldr::dbgcon(c);
        for &b in b"\n!!FREELDR-RUST-PANIC@" { putc(b); }
        if let Some(loc) = info.location() {
            for &b in loc.file().as_bytes() { putc(b); }
            putc(b':');
            let mut line = loc.line();
            let mut buf = [0u8; 10];
            let mut n = 0usize;
            if line == 0 { putc(b'0'); }
            while line > 0 { buf[n] = b'0' + (line % 10) as u8; line /= 10; n += 1; }
            while n > 0 { n -= 1; putc(buf[n]); }
        }
        putc(b'\n');
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
