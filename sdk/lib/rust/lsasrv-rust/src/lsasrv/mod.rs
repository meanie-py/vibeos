//! Translated lsasrv.dll bodies (cohab; namespaced by C source dir).
pub mod database;
pub mod policy;
pub mod session;
pub mod notify;
// pub mod service;  // STAGED — needs the winsvc substrate (SERVICE_* types +
// StartServiceCtrlDispatcherW/SetServiceStatus/… externs); next focused step.
pub mod dssetup;
pub mod privileges;
pub mod security;
pub mod registry;
// pub mod lookup;  // STAGED WIP — translated, integration pending (needs SAM substrate; see plan doc)
pub mod utils;
