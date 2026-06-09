#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

//! hid-sys — focused HID parser DDK FFI substrate for the hidparser translation.
//!
//! bindgen-generated (generate_bindings.sh) from hidpi.h + hidpddi.h + hidusage.h,
//! allowlisted to the HIDP_/HID_/HIDD_ type surface (HIDP_CAPS, HIDP_VALUE_CAPS,
//! HIDP_BUTTON_CAPS, HIDP_DATA, HIDP_PREPARSED_DATA, HIDP_DEVICE_DESC,
//! HIDP_REPORT_IDS, HIDP_STATUS_*, …). These live in the HID parser DDK headers,
//! NOT in ntoskrnl.h, so they are absent from ntkrnl-sys; hidparser-rust imports
//! the HID types from here and the kernel types from ntkrnl-sys. Per-arch.
//!
//! Regenerate with: bash sdk/lib/rust/hid-sys/generate_bindings.sh

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

// ---------------------------------------------------------------------------
// HIDP_STATUS_* — NTSTATUS codes from the HIDP_ERROR_CODES function-like macro
// in hidpi.h:  #define HIDP_ERROR_CODES(SEV,CODE) ((NTSTATUS)(((SEV)<<28)|(0x11<<16)|(CODE)))
// bindgen cannot evaluate function-like #defines, so these are absent from the
// generated bindings. They are the single source of truth; hidparser-rust imports
// them from here rather than fabricating local copies. NTSTATUS is i32, and the
// failure codes have bit 31 set, so they are written as `u32 as i32`.
// SEV: 0x0=SUCCESS, 0x8=NULL(informational), 0xC=error.
// ---------------------------------------------------------------------------
pub const HIDP_STATUS_SUCCESS: i32 = 0x0011_0000u32 as i32;
pub const HIDP_STATUS_NULL: i32 = 0x8011_0001u32 as i32;
pub const HIDP_STATUS_INVALID_PREPARSED_DATA: i32 = 0xC011_0001u32 as i32;
pub const HIDP_STATUS_INVALID_REPORT_TYPE: i32 = 0xC011_0002u32 as i32;
pub const HIDP_STATUS_INVALID_REPORT_LENGTH: i32 = 0xC011_0003u32 as i32;
pub const HIDP_STATUS_USAGE_NOT_FOUND: i32 = 0xC011_0004u32 as i32;
pub const HIDP_STATUS_VALUE_OUT_OF_RANGE: i32 = 0xC011_0005u32 as i32;
pub const HIDP_STATUS_BAD_LOG_PHY_VALUES: i32 = 0xC011_0006u32 as i32;
pub const HIDP_STATUS_BUFFER_TOO_SMALL: i32 = 0xC011_0007u32 as i32;
pub const HIDP_STATUS_INTERNAL_ERROR: i32 = 0xC011_0008u32 as i32;
pub const HIDP_STATUS_I8042_TRANS_UNKNOWN: i32 = 0xC011_0009u32 as i32;
pub const HIDP_STATUS_INCOMPATIBLE_REPORT_ID: i32 = 0xC011_000Au32 as i32;
pub const HIDP_STATUS_NOT_VALUE_ARRAY: i32 = 0xC011_000Bu32 as i32;
pub const HIDP_STATUS_IS_VALUE_ARRAY: i32 = 0xC011_000Cu32 as i32;
pub const HIDP_STATUS_DATA_INDEX_NOT_FOUND: i32 = 0xC011_000Du32 as i32;
pub const HIDP_STATUS_DATA_INDEX_OUT_OF_RANGE: i32 = 0xC011_000Eu32 as i32;
pub const HIDP_STATUS_BUTTON_NOT_PRESSED: i32 = 0xC011_000Fu32 as i32;
pub const HIDP_STATUS_REPORT_DOES_NOT_EXIST: i32 = 0xC011_0010u32 as i32;
pub const HIDP_STATUS_NOT_IMPLEMENTED: i32 = 0xC011_0020u32 as i32;

// HIDP_REPORT_TYPE discriminants. The bindgen enum is
// _HIDP_REPORT_TYPE_HidP_Input=0 / _HidP_Output=1 / _HidP_Feature=2; expose the
// bare names hidparser-rust uses (matching hidpi.h's typedef enum).
pub const HidP_Input: u32 = 0;
pub const HidP_Output: u32 = 1;
pub const HidP_Feature: u32 = 2;
