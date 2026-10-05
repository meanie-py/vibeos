//! panic-common — the one shared panic body for every vibeos Rust artifact.
//!
//! A Rust panic carries the two things a C bugcheck structurally cannot: the
//! panic *message* and the exact source `file:line`. Today ~168 crates throw
//! both away with `loop {}`. This crate surfaces everything, then halts in a
//! controlled, per-world way:
//!
//! 1. Format `*** RUST PANIC in <crate> at <file>:<line>: <message>` into a
//!    static buffer (no allocation, works at any IRQL).
//! 2. Emit it to every sink that can't fail: debugcon 0xe9 (kernel x86),
//!    then `DbgPrint` (resolves to ntoskrnl's or ntdll's export at link).
//! 3. Append a raw-PC backtrace via `RtlWalkFrameChain` (symbolize offline
//!    with `qemu/addr2sym.sh`; works on Rust frames now that
//!    `requires-uwtable=true` ships `.pdata` — field manual §9.35).
//! 4. Halt: `KeBugCheckEx(RUST_PANIC)` (feature `kernel`),
//!    `NtTerminateProcess` (feature `user`), or a halt loop (no feature).
//!
//! Nested-panic safe: a panic raised while reporting skips straight to the
//! halt path after a raw marker — the reporter must never recurse (the
//! "crash reporter that can crash is worse than useless" rule, brief L4).
//!
//! Adopting crates keep a two-line `#[panic_handler]` forwarder (the lang
//! item is per-crate-graph; only this body is shared):
//!
//! ```ignore
//! #[cfg(not(test))]
//! #[panic_handler]
//! fn panic(info: &core::panic::PanicInfo) -> ! {
//!     panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
//! }
//! ```

#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU32, Ordering};

/// Custom bugcheck / process exit code for a Rust panic: "RST" + 0xE0 OEM bit.
/// KeBugCheckEx params: P1 = &message (NUL-terminated), P2 = buffer capacity,
/// P3 = 0 (reserved), P4 = source line.
pub const RUST_PANIC: u32 = 0xE052_5354;

const MSG_CAP: usize = 1024;
#[cfg(not(feature = "raw"))]
const MAX_FRAMES: usize = 24;

// Feature "raw" = zero externs: some final artifacts (SCSIPORT miniports)
// link no ntoskrnl import lib, so even DbgPrint would be undefined.
#[cfg(not(feature = "raw"))]
extern "C" {
    /// ntoskrnl.spec:50 / ntdll.spec:47 — `@ varargs DbgPrint(str)`.
    fn DbgPrint(fmt: *const u8, ...) -> u32;
}

#[cfg(not(feature = "raw"))]
extern "system" {
    /// ntoskrnl.spec:1342 / ntdll.spec:1262 — `@ stdcall RtlWalkFrameChain(ptr long long)`.
    fn RtlWalkFrameChain(callers: *mut *mut core::ffi::c_void, count: u32, flags: u32) -> u32;
}

#[cfg(feature = "kernel")]
extern "system" {
    fn KeBugCheckEx(code: u32, p1: usize, p2: usize, p3: usize, p4: usize) -> !;
}

#[cfg(feature = "user")]
extern "system" {
    fn NtTerminateProcess(handle: isize, status: u32) -> i32;
}

/// Panic-in-progress state. 0 = idle; each nested entry increments, so a
/// panic raised *while reporting* (level 2) skips the reporter and goes
/// straight to the halt, and a panic raised *while halting* (level 3+) spins.
static PANIC_LEVEL: AtomicU32 = AtomicU32::new(0);

/// The formatted report. Static (not stack) so the bugcheck screen / a
/// debugger / the minidump writer can find the message after the fact via
/// the KeBugCheckEx P1 pointer. Guarded by PANIC_LEVEL: written only by the
/// first panicking thread. A concurrent panic on another CPU halts raw
/// rather than corrupting the first report.
static mut PANIC_MSG: [u8; MSG_CAP] = [0; MSG_CAP];

/// Byte-at-a-time emergency sink that cannot fail: QEMU debugcon (port 0xe9)
/// in kernel mode on x86. Elsewhere a no-op — DbgPrint is the sink there.
#[inline(always)]
fn raw_emit(bytes: &[u8]) {
    #[cfg(all(
        any(feature = "kernel", feature = "raw"),
        any(target_arch = "x86", target_arch = "x86_64")
    ))]
    for &b in bytes {
        unsafe { core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b) };
    }
    #[cfg(not(all(
        any(feature = "kernel", feature = "raw"),
        any(target_arch = "x86", target_arch = "x86_64")
    )))]
    let _ = bytes;
}

/// Bounded, non-panicking writer over the static buffer. Truncates silently
/// (never errors — an Err from write! would panic inside the panic handler).
struct Cursor {
    len: usize,
}

impl Cursor {
    fn buf(&mut self) -> &'static mut [u8; MSG_CAP] {
        // SAFETY: PANIC_LEVEL==1 guarantees a single writer (see callers).
        unsafe { &mut *core::ptr::addr_of_mut!(PANIC_MSG) }
    }
}

impl Write for Cursor {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let start = self.len;
        // Reserve one byte so the buffer is always NUL-terminated for %s/P1.
        let room = (MSG_CAP - 1).saturating_sub(start);
        let take = s.len().min(room);
        self.buf()[start..start + take].copy_from_slice(&s.as_bytes()[..take]);
        self.len = start + take;
        Ok(())
    }
}

/// `_Unwind_Resume` — the Itanium/DWARF unwinder's "keep unwinding" entry point.
///
/// stable-toolchain Axis B: on the builtin triples (amd64/arm64) we link the
/// PRECOMPILED `core`/`alloc` rlibs that rust-lang CI ships, and those are built
/// `panic = "unwind"`. Their *non-generic* (already-codegen'd) functions — e.g.
/// `alloc::ffi::c_str::CString::into_string` — therefore carry landing pads that
/// call `_Unwind_Resume`, so lld demands the symbol from any module that pulls in
/// `alloc`. `-Z build-std` used to hide this by rebuilding `alloc` itself with
/// `panic=abort`; with a precompiled `alloc` we must supply it.
///
/// It can never actually run: every vibeos crate compiles `panic = "abort"`
/// (workspace `[profile.release]`), so a Rust panic goes straight to the
/// `#[panic_handler]` above and bugchecks — no frame ever unwinds.
///
/// So this is a REACHABILITY ASSERTION, not an unwinder. Reaching it means a Rust
/// frame is unwinding inside a kernel that has no Rust unwinder; say so loudly and
/// halt, rather than returning into a half-unwound stack.
///
/// It is a FALLBACK, never a definition of `_Unwind_Resume` itself. A C++ module
/// links the real unwinder (libunwind) too, and C++ exceptions need it. When this
/// was `#[no_mangle] _Unwind_Resume`, link order picked between the two: in rapps
/// lld loaded this stub AS libc++abi's `_Unwind_Resume`, so a C++ exception
/// unwinding through a cleanup frame would have panicked. Instead the `.drectve`
/// below tells lld to use this body only if `_Unwind_Resume` is still undefined at
/// the end of the link. Both items live in the crate root, the same object as
/// `panic_common`, so any module whose panic handler is linked gets the fallback.
#[no_mangle]
pub extern "C" fn vibeos_unwind_resume_fallback(_exception: *mut core::ffi::c_void) -> ! {
    panic!("_Unwind_Resume reached: a Rust frame is unwinding, but every crate is panic=abort");
}

// The directive is raw assembly, not a `#[used]` static: rustc lists every used
// static in its `symbols.o`, and lld discards `.drectve` sections, so a static
// there is an undefined symbol at every link. i386 COFF prefixes C names with `_`.
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".section .drectve,\"yni\"",
    ".ascii \" -alternatename:__Unwind_Resume=_vibeos_unwind_resume_fallback \"",
    ".text",
);
#[cfg(not(target_arch = "x86"))]
core::arch::global_asm!(
    ".section .drectve,\"yni\"",
    ".ascii \" -alternatename:_Unwind_Resume=vibeos_unwind_resume_fallback \"",
    ".text",
);

/// The shared panic body. `krate` should be `env!("CARGO_PKG_NAME")`.
pub fn panic_common(krate: &'static str, info: &PanicInfo) -> ! {
    let level = PANIC_LEVEL.fetch_add(1, Ordering::SeqCst);
    if level == 0 {
        report(krate, info);
    } else if level == 1 {
        // Nested (reporter panicked) or concurrent (second CPU): no
        // formatting, no walking — mark it and go straight to the halt.
        raw_emit(b"\n!! nested/concurrent RUST PANIC -- skipping reporter\n");
    } else {
        // Third entry: even the halt path is panicking. Spin dead — never
        // ping-pong between panic and KeBugCheckEx.
        loop {
            core::hint::spin_loop();
        }
    }
    halt(info)
}

fn report(krate: &'static str, info: &PanicInfo) {
    let mut c = Cursor { len: 0 };

    let _ = write!(c, "\n*** RUST PANIC in {krate}");
    if let Some(loc) = info.location() {
        let _ = write!(c, " at {}:{}:{}", loc.file(), loc.line(), loc.column());
    }
    let _ = write!(c, "\n*** {}\n", info.message());

    // Report the message itself before attempting the stack walk: the walk
    // is the riskiest step (§9.35 fault-amplification class), and if it dies
    // the message is already on the wire.
    #[allow(unused_variables)]
    let msg_end = c.len;
    flush(&mut c, 0);

    #[cfg(not(feature = "raw"))]
    {
        let mut frames = [core::ptr::null_mut(); MAX_FRAMES];
        // SAFETY: exported by ntoskrnl and ntdll on every arch; bounds-checked
        // walk over the current stack, safe at panic time.
        let n = unsafe { RtlWalkFrameChain(frames.as_mut_ptr(), MAX_FRAMES as u32, 0) };
        let _ = write!(c, "*** backtrace ({n} frames, addr2sym.sh to symbolize):\n");
        for &pc in frames.iter().take(n as usize) {
            let _ = write!(c, "***   {:#018x}\n", pc as usize);
        }
        flush(&mut c, msg_end);
    }
}

/// Emit buffer[from..len] to debugcon, then the whole NUL-terminated buffer
/// (or the tail) to DbgPrint as a preformatted %s.
fn flush(c: &mut Cursor, from: usize) {
    let len = c.len;
    let buf = c.buf();
    buf[len] = 0;
    raw_emit(&buf[from..len]);
    // SAFETY: buf is NUL-terminated; "%s" consumes exactly one pointer.
    #[cfg(not(feature = "raw"))]
    unsafe {
        DbgPrint(b"%s\0".as_ptr(), buf[from..].as_ptr())
    };
}

fn halt(info: &PanicInfo) -> ! {
    #[cfg(feature = "kernel")]
    {
        let line = info.location().map_or(0, |l| l.line() as usize);
        // SAFETY: KeBugCheckEx never returns; P1 references the static
        // report buffer, which stays valid for the bugcheck consumer.
        unsafe {
            KeBugCheckEx(
                RUST_PANIC,
                core::ptr::addr_of!(PANIC_MSG) as usize,
                MSG_CAP,
                0,
                line,
            )
        }
    }
    #[cfg(not(feature = "kernel"))]
    {
        let _ = info;
        #[cfg(feature = "user")]
        // SAFETY: -1 = NtCurrentProcess; does not return for the caller's
        // own process. Fall through to the spin if it somehow does.
        unsafe {
            NtTerminateProcess(-1, RUST_PANIC);
        }
        loop {
            core::hint::spin_loop();
        }
    }
}
