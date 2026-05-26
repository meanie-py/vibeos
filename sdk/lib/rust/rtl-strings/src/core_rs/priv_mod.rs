//! Wave PRIV — type/const subset of `sdk/lib/rtl/priv.c`. priv.c is
//! pure syscall plumbing (every fn chains `Zw*` calls); there are no
//! pure-data decision kernels to factor out. This module just hosts
//! the constants and struct layouts so the cabi shims read cleanly.
//!
//! User-mode-dominated (the only kernel caller is `csrss/winsrv`'s
//! TUI terminal frontend invoking `RtlAdjustPrivilege`); ntoskrnl
//! doesn't reference these. Per AGENTS.md "translate + smoke" — no
//! byte-diff harness (priv.c's outputs are syscall traces, not byte
//! buffers; see priv.c follow-up memory for the alt-harness option).

#![allow(dead_code)]

// --- RTL_ACQUIRE_PRIVILEGE flags (ndk/rtltypes.h).

pub const RTL_ACQUIRE_PRIVILEGE_IMPERSONATE: u32 = 1;
pub const RTL_ACQUIRE_PRIVILEGE_PROCESS: u32 = 2;

// --- TOKEN access masks (winnt.h).

pub const TOKEN_DUPLICATE: u32 = 0x0002;
pub const TOKEN_IMPERSONATE: u32 = 0x0004;
pub const TOKEN_QUERY: u32 = 0x0008;
pub const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;

// --- Privilege attributes.

pub const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;
pub const SE_PRIVILEGE_REMOVED: u32 = 0x00000004;

// --- TOKEN_INFORMATION_CLASS subset.

pub const TOKEN_PRIVILEGES_CLASS: u32 = 3;

// --- SE well-known privilege limit (ntddk.h).
//
// SE_MIN_WELL_KNOWN_PRIVILEGE = 2; SE_MAX_WELL_KNOWN_PRIVILEGE = 35.
pub const SE_MIN_WELL_KNOWN_PRIVILEGE: u64 = 2;
pub const SE_MAX_WELL_KNOWN_PRIVILEGE: u64 = 35;
pub const SE_MAX_OLD_PRIV_BUFFER_ENTRIES: usize =
    (SE_MAX_WELL_KNOWN_PRIVILEGE - SE_MIN_WELL_KNOWN_PRIVILEGE) as usize;

// --- THREADINFOCLASS::ThreadImpersonationToken.

pub const THREAD_IMPERSONATION_TOKEN: u32 = 5;

// --- SECURITY_IMPERSONATION_LEVEL.

pub const SECURITY_DELEGATION: u32 = 3;

// --- TokenImpersonation type.

pub const TOKEN_IMPERSONATION_TYPE: u32 = 2;

// --- Pseudo-handles (winnt.h).

#[inline]
pub fn nt_current_process() -> *mut u8 {
    -1isize as usize as *mut u8
}
#[inline]
pub fn nt_current_thread() -> *mut u8 {
    -2isize as usize as *mut u8
}

// --- TEB.IsImpersonating offset (computed backwards from
// `FIELD_OFFSET(TEB, ActiveFrame) == 0x17C0` (amd64) / `0xFB0`
// (i386) — see `C_ASSERT` in ndk/peb_teb.h:505/516. Layout between
// IsImpersonating and ActiveFrame (NTDDI_WS03 baseline, no Win8+):
//   IsImpersonating:ULONG NlsCache:PVOID pShimData:PVOID
//   HeapVirtualAffinity:ULONG (+Padding7[4] on _STRUCT64)
//   CurrentTransactionHandle:PVOID ActiveFrame:PVOID
// → 4 + 8 + 8 + 4 + 4(pad) + 8 = 36 backwards from ActiveFrame on amd64
// → 4 + 4 + 4 + 4 + 0 + 4    = 20 backwards from ActiveFrame on i386
#[cfg(target_arch = "x86")]
pub const TEB_IS_IMPERSONATING_OFFSET: usize = 0xFB0 - 0x14; // = 0xF9C
#[cfg(target_arch = "x86_64")]
pub const TEB_IS_IMPERSONATING_OFFSET: usize = 0x17C0 - 0x24; // = 0x179C
#[cfg(target_arch = "aarch64")]
pub const TEB_IS_IMPERSONATING_OFFSET: usize = 0x17C0 - 0x24; // = 0x179C (matches amd64 layout)

// `read_is_impersonating()` lives in cabi/mod.rs — core_rs forbids
// `unsafe_code` per the crate-level lint.
