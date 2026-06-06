//! Hand-written translation of the private types/constants from
//! `drivers/base/beep/beep.c` (the `_BEEP_DEVICE_EXTENSION` struct) plus the
//! ntddbeep.h driver-facing surface (BEEP_SET_PARAMETERS / IOCTL_BEEP_SET /
//! FILE_DEVICE_BEEP) which is not in the ntkrnl-sys bindgen. Layout must match
//! the C exactly — DEVICE_EXTENSION is the IoCreateDevice extension blob.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{FAST_MUTEX, KTIMER};

// typedef struct _BEEP_SET_PARAMETERS { ULONG Frequency; ULONG Duration; }
// (ntddbeep.h)
#[repr(C)]
pub struct BEEP_SET_PARAMETERS {
    pub Frequency: ULONG,
    pub Duration: ULONG,
}
pub type PBEEP_SET_PARAMETERS = *mut BEEP_SET_PARAMETERS;

// typedef struct _BEEP_DEVICE_EXTENSION { ... } DEVICE_EXTENSION; (beep.c)
#[repr(C)]
pub struct DEVICE_EXTENSION {
    pub ReferenceCount: LONG,
    pub Mutex: FAST_MUTEX,
    pub Timer: KTIMER,
    pub TimerActive: LONG,
    pub SectionHandle: PVOID,
}
pub type PDEVICE_EXTENSION = *mut DEVICE_EXTENSION;

// ntddbeep.h: FILE_DEVICE_BEEP = 0x00000001;
//   IOCTL_BEEP_SET = CTL_CODE(FILE_DEVICE_BEEP, 0, METHOD_BUFFERED,
//   FILE_ANY_ACCESS) = (1<<16)|(0<<14)|(0<<2)|0 = 0x00010000.
pub const FILE_DEVICE_BEEP: u32 = 0x0000_0001;
pub const IOCTL_BEEP_SET: u32 = 0x0001_0000;
