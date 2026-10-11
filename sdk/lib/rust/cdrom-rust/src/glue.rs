//! C-macro shims: cdrom's c2rust output declares several `<ntddk.h>` annotation
//! *macros* as `extern "C"` fns (they have no real symbols). Provide `#[no_mangle]`
//! no-op definitions so the per-module extern decls resolve at link. These are
//! debug/annotation macros — no-ops in free builds. (Memory-freeing macros like
//! FREE_POOL and the variadic TracePrint are handled at the cohab/link stage, not
//! here, since they need real bodies / a C variadic shim.)
#![allow(non_snake_case, dead_code, unused_variables)]
