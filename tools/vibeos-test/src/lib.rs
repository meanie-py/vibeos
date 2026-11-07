//! Shared native test mechanisms behind the existing vibeos-cli front door.
pub use vibeos_build::{Error, Result};
pub mod admission;
#[cfg(test)]
mod admission_controls;
pub mod backend;
pub mod callback;
pub mod characterize;
pub mod checkpoint;
pub mod config;
pub mod conformance;
pub mod conformance_run;
pub mod debugcon;
pub mod farm;
pub mod ladder;
pub mod ladder_watch;
pub mod launch_policy;
pub mod lifecycle;
pub mod media;
pub mod payload;
pub mod portlease;
pub mod protocol;
pub mod publication;
pub mod screenshot;
pub mod smoke;
pub mod smoke_checks;
pub mod smoke_evidence;
pub mod smoke_fixtures;
pub mod smoke_run;
pub mod verdict;
pub mod vm;
pub mod wine_failures;

pub mod bench;
pub mod bench_boot;
pub mod bench_helpers;
pub mod bench_lease_commands;
pub mod bench_readonly;
pub mod bench_stage;

pub mod bench_services_control;

pub mod diagnostics;
pub mod pc_sample;
pub mod record_replay;
pub mod replay_sample;
pub mod rr_cli;
pub mod rr_commands;

pub mod cli_report;
pub mod tooling;
