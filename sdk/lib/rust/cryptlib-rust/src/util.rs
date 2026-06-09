//! util.rs — byte-reversal utility.
//!
//! In the original C, byteReverse is conditionally compiled:
//!   #if !defined(_MSC_VER) && (__BYTE_ORDER__ == __ORDER_BIG_ENDIAN__)
//!   ... real byte-swap body ...
//!   #endif
//! All our targets (x86, amd64, arm64) are LITTLE-ENDIAN. Therefore byteReverse
//! is a no-op on all supported architectures. We provide a #[no_mangle] symbol
//! so that any extern decls in sibling modules resolve (none remain after
//! cleanup, but kept for safety during integration).
//!
//! FIX applied: util.rs originally called `_byteswap_ulong` from an extern
//! block — this is undefined in no_std + LE context. Replaced with no-op.
//! See triplet-review-findings.md "byteReverse note".

#[no_mangle]
#[inline]
pub unsafe extern "C" fn byteReverse(_buf: *mut u8, _longs: u32) {
    // All our targets are little-endian; byteReverse is a compile-time no-op.
    // The C source conditionally compiles this away on LE (see #ifdef guard).
}
