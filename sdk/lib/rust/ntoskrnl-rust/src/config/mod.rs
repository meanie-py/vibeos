//! config subsystem (registry CM) — translated from C via per-function rustconv pipeline.
//!
//! cmdata.rs (Phase A): verbatim translation of cmdata.c — globals + tables.
//! Phase B (planned) splits CmControlVector by subsystem ownership; see
//! docs/engineering/cmdata-chopping-plan.md.

pub mod cmalloc;
pub mod cmapi;
// cmboot.c moved to the shared `cmboot-rust` crate (it is "shared by both the
// kernel and the bootloader" per its own header) — linked into the kernel via
// kernel-rust's `extern crate cmboot_rust` and into the loaders via freeldr-rust.
// pub mod cmboot;  -> sdk/lib/rust/cmboot-rust
pub mod cmconfig;
pub mod cmcontrl;
pub mod cmdata;
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

// Folded back 2026-05-09 from per-subsystem rlibs (pilot validated
// the captures(none) firewall is sufficient).
pub mod inlines;
