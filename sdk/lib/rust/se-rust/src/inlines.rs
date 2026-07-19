//! Rust wrappers for `se_x.h` FORCEINLINE / NTAPI helper functions whose
//! bodies live in `ntkrnl_sys::inline::se` (and one tier1 entry,
//! `SeDeleteClientSecurity`).
//!
//! Each wrapper is a `#[no_mangle] #[inline(never)]` extern-C entry that
//! launders pointer arguments through `opaque_ptr` (defeats LLVM's
//! captures(none) attribute deduction) and delegates to the corresponding
//! `ntkrnl_sys::inline::*` body.
//!
//! 10 functions total — every Se*/Sep* helper that was in `shim_exports.rs`.
//!
//! Pilot — folded back from the standalone `se-inlines` rlib 2026-05-09.
//! The captures(none) firewall today is workspace-wide `lto = false` +
//! `codegen-units = 256`, so the rlib boundary that originally protected
//! these wrappers is no longer load-bearing.

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::constants::STATUS_ACCESS_VIOLATION;
use ntkrnl_sys::opaque_ptr;

// --- Security descriptor accessors (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetGroupFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PSID {
    let descriptor = opaque_ptr(descriptor);
    ntkrnl_sys::inline::se::SepGetGroupFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetOwnerFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PSID {
    let descriptor = opaque_ptr(descriptor);
    ntkrnl_sys::inline::se::SepGetOwnerFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetDaclFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PACL {
    let descriptor = opaque_ptr(descriptor);
    ntkrnl_sys::inline::se::SepGetDaclFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetSaclFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PACL {
    let descriptor = opaque_ptr(descriptor);
    ntkrnl_sys::inline::se::SepGetSaclFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

// --- Token locks (was NTAPI, now cdecl for Rust-only linkage) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepAcquireTokenLockExclusive(token: PVOID) {
    let token = opaque_ptr(token);
    ntkrnl_sys::inline::se::SepAcquireTokenLockExclusive(token as PTOKEN)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepAcquireTokenLockShared(token: PVOID) {
    let token = opaque_ptr(token);
    KeEnterCriticalRegion();
    if token.is_null() {
        #[cfg(target_arch = "aarch64")]
        {
            // The ARM64 low page is mapped, so the natural TOKEN field load at
            // NULL does not raise the access violation expected by callers.
            // Fault through an architecturally invalid address before any
            // resource state is touched, preserving the C helper's contract.
            let invalid_address = usize::MAX;
            core::arch::asm!(
                "ldrb wzr, [{address}]",
                address = in(reg) invalid_address,
                options(nostack, readonly),
            );

            // The load above cannot return, but retain a loud fallback if an
            // emulator ever violates that architectural guarantee.
            ntkrnl_sys::ex::ExRaiseStatus(STATUS_ACCESS_VIOLATION);
        }
    }
    ExAcquireResourceSharedLite((*(token as PTOKEN)).TokenLock, 1u8);
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepReleaseTokenLock(token: PVOID) {
    let token = opaque_ptr(token);
    ntkrnl_sys::inline::se::SepReleaseTokenLock(token as PTOKEN)
}

// --- SeQuerySubjectContextToken (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SeQuerySubjectContextToken(
    ctx: *const _SECURITY_SUBJECT_CONTEXT,
) -> PVOID {
    ntkrnl_sys::inline::se::SeQuerySubjectContextToken(ctx)
}

// --- SeLengthSid (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SeLengthSid(sid: PSID) -> ULONG {
    let sid = opaque_ptr(sid);
    ntkrnl_sys::inline::se::SeLengthSid(sid)
}

// --- SeDeleteClientSecurity (NTAPI, lives in inline::tier1 in ntkrnl-sys) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn SeDeleteClientSecurity(
    ctx: *mut _SECURITY_CLIENT_CONTEXT,
) {
    let ctx = opaque_ptr(ctx);
    ntkrnl_sys::inline::tier1::SeDeleteClientSecurity(ctx)
}
