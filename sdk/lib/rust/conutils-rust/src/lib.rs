//! conutils-rust — Rust translation of sdk/lib/conutils. DeepSeek mechanical
//! translate, deepdrive-ground. Console stream/output/screen/pager utilities.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

pub mod conutils {
    pub mod outstream;
    pub mod pager;
    pub mod screen;
    pub mod stream;
    pub mod utils;
}
