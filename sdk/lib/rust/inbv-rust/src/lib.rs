//! inbv-rust — boot video (Inbv*/bootanim), extracted from ntoskrnl-rust (crate-split
//! Phase 1b). #[no_mangle] Inbv* symbols link via `extern crate inbv_rust`.
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_unsafe)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(static_mut_refs)]

pub mod global_barrier;
pub mod bootanim;
pub mod inbv;
pub mod inbvport;
