//! hidparser internal types (parser.h) — bindgen-generated per-arch by
//! generate_parser_h.sh. Tag-named structs + their typedef aliases
//! (HID_PARSER_CONTEXT, HID_COLLECTION=__HID_COLLECTION__, HID_REPORT=_HID_REPORT,
//! GLOBAL_ITEM_STATE, LOCAL_ITEM_STATE, USAGE_VALUE=usage_value, HID_REPORT_ITEM,
//! SHORT_ITEM/LONG_ITEM/MAIN_ITEM_DATA/ITEM_PREFIX). Public HIDP_* come from
//! hid_sys (re-exported here via the generated `use hid_sys::*`).
#[cfg(target_arch = "x86")]
#[path = "parser_h_i386.rs"]
mod inner;
#[cfg(target_arch = "x86_64")]
#[path = "parser_h_amd64.rs"]
mod inner;
#[cfg(target_arch = "aarch64")]
#[path = "parser_h_arm64.rs"]
mod inner;
pub use inner::*;
