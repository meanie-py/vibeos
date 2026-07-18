//! Import surface for `browser.rs` (the I_Browserr* MIDL client).
//!
//! `browser_c.h` needs its own island — it re-declares `_SERVER_INFO_100`, which `srvsvc_c.h`
//! also declares, so it cannot share the `types_rpc` pass. browser.c mirrors this by being the
//! only tail file that pulls `<lmbrowsr.h>`/`<lmserver.h>` alongside its stub header.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::prelude_common::*;
pub use crate::types_browser::*;

// The I_Browserr* MIDL client-stub prototypes — the function half of the `netapi32-browser`
// bindgen pass (its profile sets BLOCKLIST_FUNCS=1). Defined by the C `browser_c.c` stub.
pub use crate::browser_stubs::*;

// TRUE/FALSE reach the island indirectly (each `types*` re-exports `ntdll_sys::nt::*`), so they
// collide with prelude_common's own definitions as glob-vs-glob. Explicit import wins — same
// reasoning as the `types`-island list in prelude.rs.
pub use crate::prelude_common::{FALSE, TRUE};
