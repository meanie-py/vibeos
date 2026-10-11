#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut, unused_unsafe)]
#![allow(clippy::all)]

//! cdrom.sys — Rust translation of the CD-ROM class driver
//! (drivers/storage/class/cdrom). staticlib -> libcdrom_rust.a, cohab-linked into
//! the cmake-built cdrom.sys with the translated .c commented out.
//!
//! KMDF: every Wdf* call routes through the kmdf-sys substrate (the WdfFunctions[]
//! dispatch table populated by WdfVersionBind at load — see kmdf-sys). The C
//! wdfdriverentry stub (FxDriverEntry) stays the linker entry point and binds the
//! framework before invoking this crate's DriverEntry.

// NT_ASSERT is a checked-build assertion (wdm.h): its operand is never evaluated in a free
// build. It was an extern `fn`, so every call evaluated its argument out of line (cachesup.rs
// documents a fault that caused). `debug_assert!` type-checks it and evaluates nothing here.
#[macro_export]
macro_rules! NT_ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// UNREFERENCED_PARAMETER is a macro in the DDK headers; an out-of-line no-op fn is an
// optimization barrier that can uncloak an ABI mismatch.
#[allow(unused_macros)]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// `TracePrint!(level, flag, fmt, ...)` — WPP tracing. In the retail C build this is
// `#define TracePrint!(x)` (storswtr.h:124): it expands to NOTHING and its arguments are
// never evaluated. The translation rendered it as a no-op C-variadic *fn*, which stable
// Rust cannot define (`c_variadic`, rust-lang#44930).
//
// It is a MACRO and not a C trampoline on purpose. A trampoline is an OPAQUE extern call,
// so LLVM can no longer fold the arguments of all ~960 call sites — which (a) emits a dead
// call + argument evaluation per site, and (b) uncloaked a `ud2` cascade that BUGCHECK'd
// i386 (0x1E / 0xC000001D at a `ud2` right after `calll _TracePrint`; see the field manual,
// "a NO-OP debug-print fn is a CLOAK"). The macro restores exactly the pre-existing codegen
// AND is the faithful translation of the C macro. Arguments are still evaluated-then-dropped,
// matching what the old no-op fn did.
// NOTE: no argument is type-bound. The 11 extern decls this replaced did NOT agree with each
// other on their parameter types — some modules declared the level/flag as `u32` while call
// sites in aacs.rs pass `i32` constants, and the format pointer is `*const i8` in some modules
// and `*const u8` in others. Pinning any of them to one type here would reject call sites that
// compile today. (Each module's decl only had to be self-consistent, because nothing ever
// checked them against a single definition — itself a small warning about these decls.)
// Arguments are consumed BY VALUE (`let _ = $arg`), not by reference: some call sites pass a
// field of a `#[repr(packed)]` struct, and `&packed.field` is E0793. By-value is also exactly
// what the old `TracePrint(..)` fn call did — it copied each argument in.
macro_rules! TracePrint {
    ($($arg:expr),* $(,)?) => {{ $( let _ = $arg; )* }};
}

pub mod types;
pub mod glue;
pub mod ddk_shims;

pub mod aacs;
pub mod autorun;
pub mod cdrom;
pub mod common;
pub mod init;
pub mod ioctl;
pub mod mmc;
pub mod pnppower;
pub mod scratch;
pub mod sense;
pub mod zpodd;

/// Write `<prefix><num>\0` as a UTF-16 string into `dst` (capacity `cch` u16 units).
///
/// Replaces the C `RtlStringCchPrintfW(dst, cch, L"...%d", num)` calls used to build
/// the device name (`\Device\CdRom%d`) and well-known symlink (`\DosDevices\CdRom%d`).
/// In this cohab build `RtlStringCchPrintfW` is only a no-op stub (ntstrsafe is an
/// inline header with no exported symbol), so those calls silently produced a
/// zero-length name — which left the CD-ROM device UNNAMED (ARC `\Device\CdRom0`
/// lookup → BUGCHECK 0x6B) and made `WdfDeviceCreateSymbolicLink` reject the empty
/// name with STATUS_INVALID_DEVICE_REQUEST. Formatting directly here avoids both the
/// broken stub and any C-variadic ABI risk. Returns STATUS_SUCCESS or
/// STATUS_BUFFER_TOO_SMALL.
pub(crate) unsafe fn write_wide_cdrom_name(
    dst: *mut u16,
    cch: usize,
    prefix: &[u8],
    num: i32,
) -> i32 {
    let mut digits = [0u8; 12];
    let mut n: u32 = if num < 0 { 0 } else { num as u32 };
    let mut dlen = 0usize;
    if n == 0 {
        digits[0] = b'0';
        dlen = 1;
    } else {
        while n > 0 {
            digits[dlen] = b'0' + (n % 10) as u8;
            n /= 10;
            dlen += 1;
        }
    }
    // prefix + digits + NUL must fit
    if prefix.len() + dlen + 1 > cch {
        return 0xC0000023u32 as i32; // STATUS_BUFFER_TOO_SMALL
    }
    let mut i = 0usize;
    for &b in prefix {
        *dst.add(i) = b as u16;
        i += 1;
    }
    // digits were accumulated least-significant first
    let mut j = dlen;
    while j > 0 {
        j -= 1;
        *dst.add(i) = digits[j] as u16;
        i += 1;
    }
    *dst.add(i) = 0;
    0 // STATUS_SUCCESS
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
