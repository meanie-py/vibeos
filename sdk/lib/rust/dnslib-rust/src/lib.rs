//! dnslib-rust — Rust translation of sdk/lib/dnslib. DeepSeek mechanical
//! translate, deepdrive-ground to compile. DNS message/address helpers.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

pub mod dnslib {
    pub mod types;
    pub mod addr;
    pub mod dnsaddr;
    pub mod flatbuf;
    pub mod hostent;
    pub mod memory;
    pub mod sablob;
    pub mod straddr;
    pub mod string;
}
