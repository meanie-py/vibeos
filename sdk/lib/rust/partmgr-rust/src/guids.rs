// @generated — GUID data symbols instantiated by the kept-C guid.c (DEFINE_GUID).
// guid.c stays in the cmake build (PCH_SKIP_SOURCE) and #includes wdmguid.h /
// ntdddisk.h / initguid.h, which define these symbols; the Rust just declares them.
#![allow(non_upper_case_globals)]
use ntkrnl_sys::ddk::GUID;
extern "C" {
    pub static GUID_DEVINTERFACE_DISK: GUID;
    pub static GUID_DEVINTERFACE_PARTITION: GUID;
    pub static GUID_DEVINTERFACE_VOLUME: GUID;
    // GUID_IO_DISK_LAYOUT_CHANGE comes from the bindgen substrate (crate::types, via
    // the allowlisted ioevent.h) — declaring it here too is an E0659 glob ambiguity.
}
