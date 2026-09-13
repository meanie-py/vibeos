//! PortCls resource lists: Rust-owned storage, interface, allocation and lifetime.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
extern crate panic_common;
macro_rules! kernel_assert {
    ($condition:expr) => {
        if !$condition {
            ntkrnl_sys::ddk::RtlAssert(
                concat!(stringify!($condition), "\0").as_ptr() as _,
                concat!(file!(), "\0").as_ptr() as _,
                line!(),
                core::ptr::null_mut(),
            );
        }
    };
}
macro_rules! PC_ASSERT_IRQL_EQUAL {
    ($level:expr) => {
        kernel_assert!((ntkrnl_sys::ke::KeGetCurrentIrql() as i32) == ($level as i32));
    };
}
pub mod resource;
pub mod resource_factory;
pub mod resource_owner;
pub mod resource_support;
pub mod types;
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
