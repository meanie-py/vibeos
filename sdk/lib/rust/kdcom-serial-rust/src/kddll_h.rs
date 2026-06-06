//! Hand-ported definitions from `drivers/base/kdcom/kddll.h`.
//!
//! These live only in the header (the `KDP_STATUS` enum + the NTAPI function
//! prototypes that cross between kdcom.c / kddll.c / kdserial.c). rustconv
//! translated each `.c` in isolation, so the shared header declarations were
//! never produced. Ported faithfully here.

#![allow(non_upper_case_globals)]

use ntkrnl_sys::base::ULONG;

// ---------------------------------------------------------------------------
// KDP_STATUS — the kddll.h enum. Modelled as `type + bare consts` (mirrors the
// project's KDSTATUS convention so callers write `KDP_PACKET_RECEIVED`, never
// `KDP_STATUS::KDP_PACKET_RECEIVED`).
// ---------------------------------------------------------------------------
pub type KDP_STATUS = ULONG;

pub const KDP_PACKET_RECEIVED: KDP_STATUS = 0;
pub const KDP_PACKET_TIMEOUT: KDP_STATUS = 1;
pub const KDP_PACKET_RESEND: KDP_STATUS = 2;

// ---------------------------------------------------------------------------
// CpGetByte status codes (cportlib/cportlib.h). Not present in ntkrnl-sys;
// defined here to match cport-rust's CpGetByte return values.
// ---------------------------------------------------------------------------
pub const CP_GET_SUCCESS: u16 = 0;
pub const CP_GET_NODATA: u16 = 1;
pub const CP_GET_ERROR: u16 = 2;

// ---------------------------------------------------------------------------
// KDDEBUG is disabled by default (matching the C `#ifndef KDDEBUG` path that
// makes KDDBGPRINT a no-op). Provide a variadic-free no-op so the translated
// `KDDBGPRINT(...)` call sites compile away to nothing, exactly as in C.
// ---------------------------------------------------------------------------
#[macro_export]
macro_rules! KDDBGPRINT {
    ($($arg:tt)*) => {{
        // no-op; KDDEBUG is not defined (see kddll.h `#ifndef KDDEBUG`).
        // The C macro expands to nothing and never evaluates its arguments.
    }};
}
