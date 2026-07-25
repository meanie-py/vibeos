use crate::prelude::*;
use core::ffi::c_void;

// file-scope preamble HAND-WRITTEN: DeepSeek returned empty content for this unit
// (finish_reason=stop, ~1K output tokens — reasoning emitted, no code). Not a budget
// problem (the 64K cap was nowhere near hit); the model simply declines a nearly-empty
// file scope. Same mode as the dnsapi/dhcpcsvc/netapi32 preambles, all hand-assembled.
// The C file scope here is just includes + WINE_DEFAULT_DEBUG_CHANNEL, so there is
// nothing to carry over beyond the debug channel the prelude already supplies.


// ---- DisableMediaSense (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn DisableMediaSense(pHandle: *mut HANDLE, pOverLapped: *mut OVERLAPPED) -> u32 {
    // UNIMPLEMENTED
    0u32
}

// ---- RestoreMediaSense (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn RestoreMediaSense(
    _pOverlapped: *mut OVERLAPPED,
    _lpdwEnableCount: LPDWORD,
) -> u32 {
    0u32
}
