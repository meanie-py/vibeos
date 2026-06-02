//! Wave SEC-1 — pure byte-builder for `sdk/lib/rtl/security.c`'s
//! `RtlpSetSecurityObject` / `RtlQuerySecurityObject` body. The
//! cabi shims do the Rtl{Get,Set}*SecurityDescriptor + heap dance;
//! this module just exports the constants both need.
//!
//! Faithful to ReactOS security.c (Eric Kohl).

use super::sd;

/// `sizeof(SECURITY_DESCRIPTOR_RELATIVE)` — 20-byte header (matches
/// `core_rs::sd::SIZEOF_SECURITY_DESCRIPTOR_RELATIVE`).
pub const SDR_HEADER_SIZE: u32 = sd::SIZEOF_SECURITY_DESCRIPTOR_RELATIVE as u32;

/// `ROUND_UP(n, 4)` — SD components are ULONG-aligned in the
/// self-relative wire format. Pulled out for unit-test visibility.
#[inline]
pub fn round_up_4(n: u32) -> u32 {
    (n + 3) & !3
}

/// Total byte size of a self-relative SD with the given component
/// sizes. Faithful to security.c `ulNewSdSize = sizeof(SDR) +
/// ROUND_UP(owner) + ROUND_UP(group) + ROUND_UP(dacl) + ROUND_UP(sacl)`.
#[inline]
pub fn sd_total_size(owner_size: u32, group_size: u32, dacl_size: u32, sacl_size: u32) -> u32 {
    SDR_HEADER_SIZE
        + round_up_4(owner_size)
        + round_up_4(group_size)
        + round_up_4(dacl_size)
        + round_up_4(sacl_size)
}
