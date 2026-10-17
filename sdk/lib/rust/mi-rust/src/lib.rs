//! mi — the Management Infrastructure client (mi.dll), written in Rust over wbemprox.
//!
//! Not a translation: Wine has no mi.dll, and neither does ReactOS. The ABI is OMI's MI.h
//! (`sdk/include/psdk/mi.h`), whose function-table layouts were measured against Windows
//! 11's mi.dll; the backend is this machine's WMI through IWbemLocator/IWbemServices.
//!
//! What works: application, local sessions, QueryInstances (WQL), EnumerateInstances,
//! GetInstance, TestConnection, sync and async (with manual acknowledgement) delivery,
//! instances with their full element API, and operation/destination options. What
//! completes with MI_RESULT_NOT_SUPPORTED: create/modify/delete, Invoke, associations,
//! subscriptions, classes, serializers, hosted providers and remote destinations.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

extern crate alloc;

pub mod types; // bindgen island, per arch (gen_driver_types.sh mi)

#[macro_use]
mod sys;
mod application;
mod heap;
mod instance;
mod operation;
mod options;
mod session;
mod value;
mod wstr;

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: heap::ProcessHeap = heap::ProcessHeap;

#[no_mangle]
pub unsafe extern "system" fn DllMain(_inst: *mut core::ffi::c_void, _reason: u32, _reserved: *mut core::ffi::c_void) -> i32 {
    1
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
