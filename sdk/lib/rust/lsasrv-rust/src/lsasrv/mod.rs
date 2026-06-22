//! Translated lsasrv.dll bodies (cohab; namespaced by C source dir).
pub mod database;
pub mod policy;
pub mod session;
pub mod notify;
pub mod srm;
pub mod lsasrv;
pub mod authport;
pub mod service;
// pub mod authpackage;  // STAGED — samsrv S0 substrate landed (samsrv-sys + prelude SAM/
// LSA-token types), integration driven 199→92 errors (all "cannot find type/value"
// resolved; union nesting fixed: c2rust anon_0/2/3 → bindgen anon_1, CallAuth/Lookup +2
// union levels, _LUID .__bindgen_anon_1 → named LowPart/HighPart). REMAINING ~92, all
// mechanical: (a) ~10 fn externs — Lsar{OpenPolicy,OpenAccount,Close,EnumeratePrivilegesAccount}
// (lsarpc.c, __stdcall, sigs in lsa_s.h), Lsap{AppendRidToSid,CreateTokenSd} (lsasrv.h),
// EqualSid, SamIConnect, + use siblings LsaIFree_LSAPR_PRIVILEGE_SET (lsasrv.rs)/
// LsapSetLogonSessionData (session.rs); (b) shadow renames — global `static mut PackageId`
// (47,595,640) vs locals, and locals TokenUser/TokenPrimaryGroup shadow ntdll enum consts;
// (c) ~18 derefs ((*p).field), ~33 E0308 casts, ~8 binops, ~5 fn-ptr casts. Un-stage + finish.
pub mod dssetup;
pub mod privileges;
pub mod security;
pub mod registry;
// pub mod lookup;  // STAGED WIP — translated, integration pending (needs SAM substrate; see plan doc)
pub mod utils;
