//! Translated lsasrv.dll bodies (cohab; namespaced by C source dir).
pub mod database;
pub mod policy;
pub mod session;
pub mod notify;
pub mod srm;
pub mod lsasrv;
pub mod authport;
pub mod service;
// pub mod authpackage;  // STAGED — translated ($0.16, 23 fns, 2209 LOC, head→prelude,
// parse-artifact fixed) but BLOCKED on the SAM substrate (samsrv S0): authpackage.c's
// LsapLogonUser/LsapAddSamGroups bridge into SAM — needs SAMPR_HANDLE/SAMPR_ULONG_ARRAY/
// SAMPR_PSID_ARRAY/SAMPR_SID_INFORMATION (widl sam_c.h) + Samr{OpenDomain,GetAliasMembership,
// CloseHandle}/SamIFree_SAMPR_ULONG_ARRAY externs + SAM masks (SAM_SERVER_CONNECT/
// LOOKUP_DOMAIN, DOMAIN_GET_ALIAS_MEMBERSHIP, ACCOUNT_VIEW) + LSA-token-info types
// (_LSA_TOKEN_INFORMATION_V1/SECURITY_LOGON_TYPE, ntsecpkg.h) + _MSV1_0_INTERACTIVE_LOGON.
// Build samsrv-sys (S0) first, then un-stage + extern the lookup.c SID globals.
pub mod dssetup;
pub mod privileges;
pub mod security;
pub mod registry;
// pub mod lookup;  // STAGED WIP — translated, integration pending (needs SAM substrate; see plan doc)
pub mod utils;
