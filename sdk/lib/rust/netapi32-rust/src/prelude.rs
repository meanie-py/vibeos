//! netapi32-rust prelude — the import surface for the 14 c2rust-anchored modules.
//!
//! netapi32.dll has TWO type universes, exactly as the C does:
//!   * this one — the `<lm.h>` LAN Manager master header (USER_INFO_*/SHARE_INFO_*/
//!     SERVER_INFO_*/WKSTA_INFO_*/AT_INFO/... + nb30/ntsam/iphlpapi), used by the 14
//!     c2rust-anchored translation units;
//!   * `prelude_rpc` / `prelude_browser` — the WIDL `*_c.h` MIDL client stubs, used by the
//!     7 RPC-stub tail units. Those headers RE-DECLARE the same wire structs, so the two
//!     sets cannot coexist in one bindgen pass (see gen_driver_types.sh netapi32-rpc).
//!
//! Everything that is NOT a type glob (wine debug channel, CRT/winsock externs, skipped
//! consts, enum bare-name aliases) is shared via `prelude_common`.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::prelude_common::*;
pub use crate::types::*;

// These 40 names are defined BOTH in prelude_common (hand-transcribed with a header citation)
// and by bindgen in the `types` island. Before the prelude was split they were local `pub const`
// items here, which shadow a glob import; now that they arrive via a glob too, glob-vs-glob is
// ambiguous (E0659). An EXPLICIT import outranks any glob, so re-export them to restore exactly
// the previous resolution. (The new MIDL islands have zero overlap — this list is `types`-only.)
pub use crate::prelude_common::{
    FALSE,
    TRUE,
    FD_SET,
    NRC_ACTSES,
    NRC_BADDR,
    NRC_BRIDGE,
    NRC_BUFLEN,
    NRC_CANCEL,
    NRC_CANOCCR,
    NRC_CMDCAN,
    NRC_CMDTMO,
    NRC_DUPENV,
    NRC_DUPNAME,
    NRC_ENVNOTDEF,
    NRC_GOODRET,
    NRC_IFBUSY,
    NRC_ILLCMD,
    NRC_ILLNN,
    NRC_INCOMP,
    NRC_INUSE,
    NRC_INVADDRESS,
    NRC_INVDDID,
    NRC_LOCKFAIL,
    NRC_LOCTFUL,
    NRC_MAXAPPS,
    NRC_NAMCONF,
    NRC_NAMERR,
    NRC_NAMTFUL,
    NRC_NOCALL,
    NRC_NORES,
    NRC_NORESOURCES,
    NRC_NOSAPS,
    NRC_NOWILD,
    NRC_OPENERR,
    NRC_OSRESNOTAV,
    NRC_PENDING,
    NRC_REMTFUL,
    NRC_SABORT,
    NRC_SCLOSED,
    NRC_SNUMOUT,
    NRC_SYSTEM,
    NRC_TOOMANY,
};
