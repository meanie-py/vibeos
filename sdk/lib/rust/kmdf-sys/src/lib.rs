#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_parens)]
#![allow(clippy::all)]

//! kmdf-sys — KMDF client-binding FFI substrate for the `cdrom` driver
//! translation (the fleet's only KMDF driver).
//!
//! NOT a framework reimplementation. The KMDF runtime (`Wdf01000.sys`), the
//! loader (`wdfldr.sys`), and the `WdfDriverEntry` bind stub all build and boot
//! in-tree (`vendor/wdf/`). This crate is only the *client-side* glue
//! the C compiler synthesizes from `<wdf.h>`:
//!
//! Each `Wdf*` API in the WDF headers is a `FORCEINLINE` function whose body
//! dispatches through a load-time-bound table:
//! ```c
//! FORCEINLINE NTSTATUS WdfDeviceCreate(...) {
//!     return ((PFN_WDFDEVICECREATE) WdfFunctions[WdfDeviceCreateTableIndex])
//!            (WdfDriverGlobals, ...);
//! }
//! ```
//! `WdfFunctions` (`const WDFFUNC *`) and `WdfDriverGlobals`
//! (`PWDF_DRIVER_GLOBALS`) are defined by the C `wdfdriverentry.c` stub and
//! populated by `WdfVersionBind` when `FxDriverEntry` runs (before the client
//! `DriverEntry`). `dispatch.rs` is a mechanical 1:1 Rust translation of those
//! FORCEINLINE bodies; `types.rs` is the bindgen of the WDF 1.17 surface.
//!
//! Regenerate:
//!   bash tools/rustconv/gen_driver_types.sh kmdf          # -> src/types_*.rs
//!   python3 tools/rustconv/gen_kmdf_dispatch.py           # -> src/dispatch.rs

pub mod types;
pub mod dispatch;
pub mod helpers;

pub use dispatch::*;
pub use helpers::*;
pub use types::*;
