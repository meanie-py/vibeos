#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

//! acpica-sys — focused ACPICA FFI substrate for the ReactOS `acpi` driver glue
//! translation (FFI-to-blob: Intel's ACPICA reference impl stays a C static lib;
//! only the ~10K-LOC ReactOS glue becomes Rust).
//!
//! bindgen-generated (generate_bindings.sh) from acpica/include/acpi.h with
//! ACPI_USE_LOCAL_CACHE / ACPI_USE_SYSTEM_INTTYPES, allowlisted to the
//! (ACPI_.*|Acpi.*|AE_.*) surface:
//!   - the public ACPICA types (ACPI_STATUS, ACPI_HANDLE, ACPI_OBJECT,
//!     ACPI_TABLE_*, ACPI_BUFFER, ACPI_DEVICE_INFO, ACPI_RESOURCE, GPE/event …),
//!   - the outbound `Acpi*` API fns the glue calls (AcpiInitializeSubsystem,
//!     AcpiGetHandle, AcpiEvaluateObject, AcpiWalkNamespace, AcpiGetTable, …),
//!   - the AcpiOs* OSL callback typedefs (ACPI_OSD_HANDLER etc.) so the Rust
//!     `#[no_mangle]` OSL impls (acpi-rust, osl.c successor) match.
//!
//! Per-arch (pointer widths / ACPI table packing differ).
//! Regenerate with: bash sdk/lib/rust/acpica-sys/generate_bindings.sh

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
