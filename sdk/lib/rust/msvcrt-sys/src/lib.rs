//! `msvcrt-sys` — FFI view of msvcrt.dll's **internal state structs**.
//!
//! The Rust bodies we link into msvcrt.dll (`msvcrt-crt-rust`) mostly need nothing but
//! their arguments. Two families are different: they must reach *inside* msvcrt's own
//! per-thread and per-locale state.
//!
//! * **`thread_data_t`** — `strtok`/`wcstok`/`_mbstok` keep their saved pointer there;
//!   `rand`/`srand` keep the seed there.
//! * **`threadlocinfo`** — and this is the important one — carries `pctype`, `pclmap`
//!   and `pcumap`: *the locale's own* character-class table and lower/upper mapping
//!   tables.
//!
//! ## Why `pclmap`/`pcumap` are the whole point
//!
//! The differential harness pins the "C" locale, which has always been recorded as a
//! blind spot forbidding locale-aware flips (§9.60): a function like `toupper` would
//! report MATCH against the C locale and then be wrong in CP1252, where `é`→`É`.
//!
//! That is true of a *locale-independent* implementation — one that hardcodes
//! `c >= 'a' && c <= 'z'`. It is **not** true of a faithful one. msvcrt's `_toupper_l`
//! is, for `c < 256`, exactly `locinfo->pcumap[c]`. A Rust body that reads *the same
//! table* inherits the C's locale-correctness **by construction**: the locale-dependence
//! lives in the data, `setlocale` repopulates it, and both bodies follow it.
//!
//! So the blind spot forbids *hardcoding ASCII*. It does not forbid *reading the real
//! table*. That distinction is what this crate exists to enable.
//!
//! ## Layout is machine-generated and double-guarded
//!
//! These structs are **not** stable by inspection — `pctype` and `mb_cur_max` migrate to
//! the front of `threadlocinfo` at `_MSVCR_VER >= 140`, and `pcumap` sits at offset 208
//! on i386 but 336 on amd64/arm64. A hand-rolled numeric offset here is precisely the
//! bug class that cost this repo the `KTHREAD.Teb` hunt and the amd64
//! `TEB.GdiBatchCount`-at-`0x74` read that killed explorer (field manual §9.41), and
//! which `tools/scan_magic_offsets.py` exists to forbid.
//!
//! So: bindgen, `offset_of!`, never arithmetic — and the offsets are asserted on **two**
//! sides by **two** compilers on every build:
//!
//! | guardian | compiler | catches |
//! |---|---|---|
//! | `layout_<arch>.rs` (here) | rustc | bindgen regenerated with wrong flags |
//! | `dll/win32/msvcrt/rust_layout_assert.c` | clang, *msvcrt's real flags*, ×3 arches | the C headers moved under us |
//!
//! Both are pinned to the same literals, emitted together by `generate_bindings.py`.
//! Neither side can drift without a loud build failure.

#![no_std]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]

#[cfg(target_arch = "x86")]
mod bindgen_i386;
#[cfg(target_arch = "x86")]
pub use bindgen_i386::*;

#[cfg(target_arch = "x86_64")]
mod bindgen_amd64;
#[cfg(target_arch = "x86_64")]
pub use bindgen_amd64::*;

#[cfg(target_arch = "aarch64")]
mod bindgen_arm64;
#[cfg(target_arch = "aarch64")]
pub use bindgen_arm64::*;

// GUARDIAN 1 of 2 — rustc asserts every offset we rely on. See the module docs.
#[cfg(target_arch = "x86")]
mod layout_i386;
#[cfg(target_arch = "x86_64")]
mod layout_amd64;
#[cfg(target_arch = "aarch64")]
mod layout_arm64;

/// MSVC's locale categories — the index into `locinfo->lc_handle[]` / `lc_id[]`.
///
/// Spelled out here rather than pulled from bindgen because `<locale.h>`'s `LC_CTYPE` is a
/// plain `#define` that bindgen's `--allowlist-var` will not emit under our tight allowlist.
///
/// **These are MSVC's numbers, not the host's.** glibc numbers the same names completely
/// differently (`LC_CTYPE=0` … `LC_ALL=6` … `LC_IDENTIFICATION=12`), and `lc_handle[]` is
/// only 6 entries long — so a host `LC_ALL` would index one PAST the end.
pub const LC_ALL: usize = 0;
pub const LC_COLLATE: usize = 1;
pub const LC_CTYPE: usize = 2;
pub const LC_MONETARY: usize = 3;
pub const LC_NUMERIC: usize = 4;
pub const LC_TIME: usize = 5;

/// Safe-ish accessor for the current thread's locale info.
///
/// msvcrt guarantees `get_locinfo()` never returns NULL (locale.c installs safe defaults
/// before any CRT entry point can run), but we are crossing an FFI boundary into C, so
/// this returns a raw pointer and callers stay `unsafe`. It exists to give every call
/// site one obvious spelling.
///
/// # Safety
/// Must be called on a thread with initialised CRT per-thread data — i.e. from inside a
/// real msvcrt entry point, which is the only place our code runs.
#[inline]
pub unsafe fn locinfo() -> *mut threadlocaleinfostruct {
    get_locinfo()
}
