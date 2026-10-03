//! rtl-misc: the genuinely-open rtl/ leaves (res / memstream / registry) not
//! covered by lane-C's rtl-diag / rtl-bootdata / rtl-rangelist / rtl-rxact sweep.
//! Banked Opus translations, DeepSeek-ground, reviewed against the C, smoked.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod shared;

#[cfg(feature = "cabi")]
pub mod res;
#[cfg(feature = "cabi")]
pub mod memstream;
#[cfg(feature = "cabi")]
pub mod registry;
#[cfg(feature = "cabi")]
pub mod memres;
