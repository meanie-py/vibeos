//! config subsystem (registry CM) — translated from C via per-function rustconv pipeline.
//!
//! cmdata.c remains in C — it's pure globals (no functions for tree-sitter
//! to extract). To-be hand-translated as `pub static mut` declarations later.

pub mod cmalloc;
pub mod cmapi;
pub mod cmboot;
pub mod cmconfig;
pub mod cmcontrl;
pub mod cmdelay;
pub mod cmhook;
pub mod cmhvlist;
pub mod cminit;
pub mod cmkcbncb;
pub mod cmlazy;
pub mod cmmapvw;
pub mod cmnotify;
pub mod cmparse;
pub mod cmquota;
pub mod cmse;
pub mod cmsecach;
pub mod cmsysini;
pub mod cmvalche;
pub mod cmwraprs;
pub mod ntapi;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod i386;
