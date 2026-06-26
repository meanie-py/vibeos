#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]
#![allow(dangerous_implicit_autorefs)]

//! usbport.sys — USB host-controller PORT framework. Faithful C→Rust cohab (rustconv opus;
//! 11 files c2rust-anchored, 3 (roothub/urb/device) pure-LLM where c2rust AST was unsupported).
//! The miniport substrate ehci/ohci/uhci register WITH; owns URB queue, endpoint/transfer
//! state, PnP FDO/PDO, roothub PDO, USB2 bandwidth scheduler. Exports USBPORT_GetHciMn /
//! USBPORT_RegisterUSBPortDriver (usbport.spec). NOT boot-loaded in smoke -> link-clean x3 bar.

pub mod types;
pub mod glue;
pub mod debug;
pub mod device;
pub mod endpoint;
pub mod iface;
pub mod ioctl;
pub mod iso;
pub mod power;
pub mod queue;
pub mod roothub;
pub mod trfsplit;
pub mod urb;
pub mod usb2;
pub mod usbport;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
