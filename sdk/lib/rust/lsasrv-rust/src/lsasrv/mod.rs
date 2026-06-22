//! Translated lsasrv.dll bodies (cohab; namespaced by C source dir).
pub mod database;
pub mod policy;
pub mod session;
pub mod notify;
pub mod srm;
pub mod lsasrv;
// pub mod authport;  // STAGED WIP — translated + substrate ready + most fixes done,
// but the deeply-nested LSA_API_MSG anon-union access (.__bindgen_anon_1 ×3 → per-API-msg
// union → Request/Reply.Status) needs careful field-by-field mapping vs the bindgen layout
// (c2rust's nesting depth ≠ bindgen's). Correctness-critical (wrong union field = corruption);
// finish unhurried. ~29 errors remain (8 nested-Status, 13 E0308, 6 cross-file fns, 1 fn-ptr).
// pub mod service;  // STAGED — needs the winsvc substrate (SERVICE_* types +
// StartServiceCtrlDispatcherW/SetServiceStatus/… externs); next focused step.
pub mod dssetup;
pub mod privileges;
pub mod security;
pub mod registry;
// pub mod lookup;  // STAGED WIP — translated, integration pending (needs SAM substrate; see plan doc)
pub mod utils;
