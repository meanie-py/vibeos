//! ntdll-rust — Rust translation of `ntdll.dll` (and its `ntdll_vista.dll`
//! companion).
//!
//! Per-file cohabitation: each translated `.c` is commented out in its
//! CMakeLists and the `#[no_mangle]` symbol here takes over, linked in
//! via a cmake-driven `cargo build` → `.a` with
//! `--allow-multiple-definition` during the swap-in window (mirrors the
//! smss-csrss-rust / csrsrv pattern and the sdk/lib/rtl cargo→.a flip).
//! The `.c` files stay on disk per the never-delete rule.
//!
//! FFI surface is `ntdll-sys` (the user-mode Native API + Ldr/Rtl/Dbg).
//! NOT a standalone binary — an `rlib`/`staticlib` linked into the
//! cmake-built native PEs. See
//! docs/engineering/ntdll-translation-audit.md.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// nt_0600/ — the ntdll_vista C subdir. Always compiled, but its submodules
// are split by audience (see nt_0600/mod.rs):
//   * dllmain (DllMain) is `vista`-gated — it is ntdll_vista.dll's ENTRY
//     point and must not be pulled into ntdll.dll.
//   * ldr (LdrpHeap) is UNgated — although the C lives in the nt_0600 subdir,
//     the loader-heap global is referenced by the MAIN ntdll.dll loader
//     (ldrinit.c/ldrutils.c), which historically resolved it from
//     ntdll_vista_static linked into ntdll.dll. So it must exist in BOTH the
//     default (ntdll.dll) and vista (ntdll_vista.dll) builds.
pub mod nt_0600;

// rtl/ — main ntdll.dll rtl-glue translations (the rtl_um OBJECT lib:
// uilist.c / version.c / libsupp.c). Always compiled — these land in
// ntdll.dll proper.
pub mod rtl;

// etw/ — main ntdll.dll Event Tracing API (the etwtrace static lib:
// etw/trace.c). Always compiled — lands in ntdll.dll proper.
pub mod etw;

// dbg/ — main ntdll.dll debug-UI native wrappers (dbg/dbgui.c). Always
// compiled — lands in ntdll.dll proper.
pub mod dbg;

// ldr/ — main ntdll.dll loader translations (ldr/verifier.c, ldrapi.c,
// ldrinit.c, ldrpe.c, ldrutils.c). Lands in ntdll.dll proper. Distinct from
// nt_0600::ldr. NOT compiled into the `vista` build: ntdll_vista.dll imports
// the loader API (e.g. LdrDisableThreadCalloutsForDll) from ntdll.dll via its
// import lib. If the loader def were in ntdll_vista's staticlib, the nt_0600
// DllMain reference would pull it in statically and cascade the whole loader +
// its C callees (LdrProcessRelocationBlockLongLong, find_entry_*, … — not
// linked into ntdll_vista). Gating keeps ntdll_vista a thin companion.
#[cfg(not(feature = "vista"))]
pub mod ldr;

// dispatch/ — user-mode NT dispatchers (dispatch/dispatch.c). ARM64-ONLY: on
// i386/amd64 the dispatcher set is pure asm (dispatch/<arch>/dispatch.S) and the
// .c is not compiled, so defining these #[no_mangle] symbols there would collide
// with the .S. On arm64 the .S still owns KiUserExceptionDispatcher/KiUserApc
// Dispatcher/LdrInitializeThunk; this provides only KiRaiseUserExceptionDispatcher
// + KiUserCallbackDispatcher. Lands in ntdll.dll proper.
#[cfg(target_arch = "aarch64")]
pub mod dispatch;

// setjmp/ — x64 `longjmp` C glue (vcruntime/amd64/longjmp.c). amd64 ntdll.dll
// is the only consumer of setjmp/longjmp (i386/arm64 never reference it — the
// cmake libsetjmp.a is dead there; ntdll_vista.dll doesn't link it). The
// register save/restore stays in pre-expanded asm; this drives RtlUnwind.
// Replaces the cmake libsetjmp.a reuse on amd64.
#[cfg(all(target_arch = "x86_64", not(feature = "vista")))]
pub mod setjmp;

// Standalone `#![no_std]` staticlib → needs its own panic handler +
// eh_personality (same role as smss-csrss-rust/kdcom-rust/hal-rust).
// USER-MODE: must NOT use privileged `hlt`/`wfe` (ntdll runs in ring 3).
// Report over DbgPrint then spin (the kernel observer / debugger sees
// the banner; a real abort would need the Native API but spin keeps the
// failure visible without privileged faults).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let banner = b"\n!!! [NTDLL] RUST PANIC HIT !!!\n\0";
        ntdll_sys::nt::DbgPrint(banner.as_ptr() as *const i8);
        if let Some(loc) = info.location() {
            let mut buf = [0u8; 256];
            let s = loc.file().as_bytes();
            let n = s.len().min(buf.len() - 1);
            buf[..n].copy_from_slice(&s[..n]);
            buf[n] = 0;
            let fmt = b"PANIC LOC: %s:%u:%u\n\0";
            ntdll_sys::nt::DbgPrint(
                fmt.as_ptr() as *const i8,
                buf.as_ptr() as *const i8,
                loc.line(),
                loc.column(),
            );
        }
    }
    loop {
        core::hint::spin_loop();
    }
}

// `core`/`compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`; panic=abort never unwinds but the symbol must
// resolve. Same stub as smss-csrss-rust/kdcom-rust/hal-rust/kernel-rust.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
