//! cdfs-rust — drivers/filesystems/cdfs → Rust (DeepSeek+deepdrive).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

pub mod allocsup;
pub mod cachesup;
pub mod cddata;
pub mod cdinit;
pub mod cleanup;
pub mod close;
pub mod create;
pub mod devctrl;
pub mod deviosup;
pub mod dirctrl;
pub mod dup.rs;
pub mod fileinfo;
pub mod fsctrl;
pub mod fspdisp;
pub mod lockctrl;
pub mod namesup;
pub mod pathsup;
pub mod pnp;
pub mod prefxsup;
pub mod read;
pub mod resrcsup;
pub mod shutdown;
pub mod strucsup;
pub mod verfysup;
pub mod volinfo;
pub mod workque;
pub mod write;
