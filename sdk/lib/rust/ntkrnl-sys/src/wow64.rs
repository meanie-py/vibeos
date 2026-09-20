//! The WoW64 combined TEB region, in one place.
//!
//! A WoW64 thread gets a single low allocation holding both TEBs: TEB64 at +0
//! and TEB32 at [`WOW64_TEB32_OFFSET`], the whole thing below 2GB so a 32-bit
//! guest pointer can reach either. `MmCreateTeb` lays the region out; everyone
//! else reads it.
//!
//! This module exists because that offset was hand-rolled in five Rust files,
//! each with its own `const` and its own `// nolint: magic-offset` — and the
//! lint firing five times on five copies of one fact is the detection event for
//! a missing accessor, not five things to annotate. Kernel-side C keeps a
//! mirror in `ntoskrnl/ke/arm64/trap.c` (`WOW64_TEB32_OFFSET`), which is the
//! only remaining copy; change one and change that one.
//!
//! i386 has no WoW64 and no `_TEB32` in its bindgen island, so the TEB32
//! accessors are gated off there. The OFFSET and the process predicate are
//! not: callers reach them from code shared with the 32-bit kernel, inside a
//! runtime `if IsWow64` that is simply never true there. Gating the whole
//! module instead is what broke the i386 build the first time.

#[cfg(not(target_arch = "x86"))]
use crate::ddk::{_EPROCESS, _TEB, _TEB32};

/// Displacement of TEB32 from TEB64 in the combined region. `sizeof(TEB)` is
/// 0x17d8, so TEB64 spans two pages and TEB32 starts on the third.
pub const WOW64_TEB32_OFFSET: usize = 0x2000;

/// The TEB32 that shares `teb`'s combined region.
///
/// Callers must already know this is a WoW64 thread — on a native thread the
/// returned pointer is just the memory past the TEB. Use [`teb32_of_current`]
/// when you want that checked.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub const unsafe fn teb32_of(teb: *const _TEB) -> *mut _TEB32 {
    (teb as *const u8).add(WOW64_TEB32_OFFSET) as *mut _TEB32 // nolint: magic-offset (WoW64 combined-region layout; this is the one definition of it)
}

/// `true` when `process` is a 32-bit guest process.
///
/// Constant `false` on the i386 kernel, which hosts no guests -- its
/// `_EPROCESS` has no `Wow64Process` field at all, so this cannot even be
/// written there, let alone be true.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn is_wow64_process(process: *const _EPROCESS) -> bool {
    !process.is_null() && !(*process).Wow64Process.is_null()
}

/// TEB32 of `teb`'s thread when the *current* process is WoW64, else `None`.
///
/// This is the form kernel code almost always wants: a path that reads a TEB
/// field the guest writes must read TEB32, and a path shared with native
/// threads must fall back to TEB64. Getting that choice wrong does not fail to
/// build — it silently reads a field nobody ever wrote (field manual §9.41).
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn teb32_of_current(teb: *const _TEB) -> Option<*mut _TEB32> {
    if teb.is_null() || !is_wow64_process(crate::ps::PsGetCurrentProcess() as *const _EPROCESS) {
        return None;
    }
    Some(teb32_of(teb))
}

/// Offset of `DeallocationStack` inside TEB32 — the BOTTOM of a guest thread's
/// 32-bit stack reservation.
///
/// `NtTib.StackLimit` is only the committed floor and moves as the stack grows,
/// so it cannot tell the kernel how far a guest stack is *allowed* to grow;
/// this is the field that can, and nothing populated it until now. The guest
/// hands the value over in its `INITIAL_TEB`'s `AllocatedStackBase` slot, which
/// wow64's `NtCreateThread` thunk parses and had simply been discarding.
#[cfg(not(target_arch = "x86"))]
pub const TEB32_DEALLOCATION_STACK: usize = 0xE0C;

#[cfg(not(target_arch = "x86"))]
const _: () = assert!(
    core::mem::offset_of!(_TEB32, DeallocationStack) == TEB32_DEALLOCATION_STACK,
    "TEB32.DeallocationStack moved; wow64.dll writes this displacement by hand"
);
