//! Import surface for the RPC-stub tail units that talk to the srvsvc / wkssvc / atsvc /
//! dssetup / netlogon MIDL client interfaces (misc, netlogon, schedule, srvsvc, wksta_new,
//! dssetup).
//!
//! These modules mirror C files that include `netapi32.h` + their `*_c.h` — and `netapi32.h`
//! pointedly does NOT include `<lm.h>`, only the narrow lmaccess/lmapibuf/lmerr trio, so the
//! WIDL stubs are free to supply the wire structs themselves. Importing this prelude instead
//! of `crate::prelude` reproduces that exact scope: same non-type surface, MIDL types instead
//! of LAN Manager ones. (377 type names collide between the two islands — AT_INFO,
//! CONNECTION_INFO_*, SERVER_INFO_*, FILE_INFO_*, ... — which is why they stay separate
//! rather than being merged behind one glob.)
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::prelude_common::*;
pub use crate::types_rpc::*;

// The MIDL client-stub prototypes (Netr*/Dsr*/Netpr*). The `netapi32-rpc` generator profile
// sets BLOCKLIST_FUNCS=1, so `types_rpc` is wire STRUCTS only; `rpc_stubs` is the matching
// FUNCTION half from the same bindgen pass. The symbols live in the WIDL-generated `*_c.c`,
// which cmake still compiles as C — a genuine cross-language import, not translation surface.
pub use crate::rpc_stubs::*;

// TRUE/FALSE reach the island indirectly (each `types*` re-exports `ntdll_sys::nt::*`), so they
// collide with prelude_common's own definitions as glob-vs-glob. Explicit import wins — same
// reasoning as the `types`-island list in prelude.rs.
pub use crate::prelude_common::{FALSE, TRUE};

// ---- intra-crate helpers shared ACROSS the tail modules. The C compiled these TUs together
// and declared the helpers in netapi32.h; in Rust each module is its own scope. These live in
// the types_rpc island (misc.rs / wksta_new.rs), so they re-export here rather than in the
// island-agnostic prelude_common.
pub use crate::netapi32::misc::{NetpAllocAnsiStrFromWStr, NetpAllocWStrFromAnsiStr};
// The MIDL allocator pair (rpcndr.h maps the lowercase spelling onto these) — netlogon.c frees
// RPC-allocated arrays with MIDL_user_free.
pub use crate::netapi32::wksta_new::{MIDL_user_allocate, MIDL_user_free};

// ---- cast-#define typedefs bindgen cannot see. LMSTR is a MACRO, not a typedef, so it never
// reaches the island:
//   lmcons.h:65  #define LMSTR   LPWSTR      (the UNICODE branch — the build passes -DUNICODE)
//   lmcons.h:68  #define LMSTR   LPSTR       (the ANSI branch, not taken)
pub type LMSTR = LPWSTR;

// ---- enum bare-name aliases. bindgen prefixes every enum member with its type name
// (`_NETSETUP_JOIN_STATUS_NetSetupUnknownStatus`); the C names the member bare. Alias the ones
// the tail actually uses back to their bare spelling so the translated bodies resolve.
pub use crate::types_rpc::{
    // lmjoin.h — NETSETUP_JOIN_STATUS
    _NETSETUP_JOIN_STATUS_NetSetupUnknownStatus as NetSetupUnknownStatus,
    // dssetup_c.h — DSROLE_PRIMARY_DOMAIN_INFO_LEVEL
    _DSROLE_PRIMARY_DOMAIN_INFO_LEVEL_DsRoleOperationState as DsRoleOperationState,
    _DSROLE_PRIMARY_DOMAIN_INFO_LEVEL_DsRolePrimaryDomainInfoBasic as DsRolePrimaryDomainInfoBasic,
    // ntsecapi.h — POLICY_NOTIFICATION_INFORMATION_CLASS
    _POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyDnsDomainInformation as PolicyNotifyDnsDomainInformation,
};
