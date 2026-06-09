#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

//! usb-sys — focused USB DDK FFI substrate for the usbd.sys translation.
//!
//! bindgen-generated (generate_bindings.sh) from usbdi.h (→ usb.h/usb100.h) +
//! usbdlib.h, allowlisted to the USB/USBD/URB type surface. These types live in
//! the USB DDK headers, NOT in ntoskrnl.h, so they are absent from ntkrnl-sys;
//! usbd-rust imports the specific USB types it needs from here and the kernel
//! types from ntkrnl-sys. Per-arch (URB layout / pointer widths differ).
//!
//! Regenerate with: bash sdk/lib/rust/usb-sys/generate_bindings.sh

#[cfg(target_arch = "aarch64")]
mod bindgen_arm64;
#[cfg(target_arch = "aarch64")]
pub use bindgen_arm64::*;

#[cfg(target_arch = "x86_64")]
mod bindgen_amd64;
#[cfg(target_arch = "x86_64")]
pub use bindgen_amd64::*;

#[cfg(target_arch = "x86")]
mod bindgen_i386;
#[cfg(target_arch = "x86")]
pub use bindgen_i386::*;
