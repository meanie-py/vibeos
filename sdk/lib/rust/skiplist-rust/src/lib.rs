//! skiplist-rust — Rust translation of sdk/lib/skiplist/skiplist.c.
//! The C lib is built as `skiplist16` with `-DSKIPLIST_LEVELS=16`.
//! DeepSeek mechanical translation, deepdrive-ground to compile.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod skiplist;
