//! classpnp-rust — drivers/storage/class/classpnp → Rust (DeepSeek+deepdrive).
#![cfg_attr(not(test), no_std)]
#![feature(c_variadic)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut)]

pub mod types;
pub mod glue;
pub mod shims;
pub mod data;
pub mod guid;
pub mod globals;

pub mod autorun;
pub mod class;
pub mod classwmi;
pub mod clntirp;
pub mod create;
pub mod debug;
pub mod dictlib;
pub mod dispatch;
pub mod history;
pub mod lock;
pub mod obsolete;
pub mod power;
pub mod retry;
pub mod srblib;
pub mod utils;
pub mod xferpkt;
