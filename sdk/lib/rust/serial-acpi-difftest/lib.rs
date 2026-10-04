//! Compiles the loader's ACPI console-UART parser for the host so its tests
//! (captured Pi 4B DBG2, x86 SPCR 16550 I/O-port cases) run under the gate.
#![allow(dead_code)]

#[path = "../freeldr-rust/src/arm64_serial_acpi.rs"]
mod serial_acpi;
