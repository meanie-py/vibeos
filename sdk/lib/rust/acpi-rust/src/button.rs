//! busmgr/button.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! ACPI Button Driver (power/sleep/lid). Linux-style acpi_* glue.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*;
use core::ffi::{c_char, c_int, c_ulong, c_void};

// Kernel types (UINT8, UINT32, SIZE_T, PVOID, NTSTATUS) via the ddk glob. ACPI_*
// types come from crate::shared (acpica-sys) — ddk has no ACPI_HANDLE, no collision.
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::NonPagedPool;

use acpica_sys::{
    AcpiInstallFixedEventHandler, AcpiInstallNotifyHandler, AcpiRemoveFixedEventHandler,
    AcpiRemoveNotifyHandler, ACPI_DEVICE_NOTIFY, ACPI_EVENT_POWER_BUTTON, ACPI_EVENT_SLEEP_BUTTON,
};

// DPRINT is a no-op under the driver's NDEBUG build.
macro_rules! DPRINT {
    ($($t:tt)*) => {};
}

// ---- ACPICA tracing macros — no-ops under NDEBUG. return_VOID/return_VALUE/
//      return_ACPI_STATUS reduce to plain returns (NDEBUG build). ----
macro_rules! ACPI_FUNCTION_TRACE {
    ($($t:tt)*) => {};
}
macro_rules! ACPI_MODULE_NAME {
    ($($t:tt)*) => {};
}
macro_rules! ACPI_DEBUG_PRINT {
    ($($t:tt)*) => {};
}

// ---- ACPICA result macros (acexcep.h): SUCCESS(a)=!(a), FAILURE(a)=(a). AE_OK==0. ----
#[inline]
fn ACPI_SUCCESS(s: ACPI_STATUS) -> bool {
    s == AE_OK
}
#[inline]
fn ACPI_FAILURE(s: ACPI_STATUS) -> bool {
    s != AE_OK
}

// Pool tag 'IPCA' = ('I'<<24)|('P'<<16)|('C'<<8)|'A'.
const TAG_IPCA: u32 = u32::from_be_bytes([b'I', b'P', b'C', b'A']);

// ---- ACPI_BUTTON_* constants (acpi_drivers.h). ----
// HIDs
const ACPI_BUTTON_HID_POWER: &[u8] = b"PNP0C0C\0";
const ACPI_BUTTON_HID_POWERF: &[u8] = b"ACPI_FPB\0";
const ACPI_BUTTON_HID_SLEEP: &[u8] = b"PNP0C0E\0";
const ACPI_BUTTON_HID_SLEEPF: &[u8] = b"ACPI_FSB\0";
const ACPI_BUTTON_HID_LID: &[u8] = b"PNP0C0D\0";
// Device-name strings — FIXME(integrator): confirm exact ACPI_BUTTON_DEVICE_NAME_*
// strings from acpi_drivers.h. Faithful transcription of upstream ACPI names below.
const ACPI_BUTTON_DEVICE_NAME_POWER: &[u8] = b"Power Button (CM)\0";
const ACPI_BUTTON_DEVICE_NAME_POWERF: &[u8] = b"Power Button (FF)\0";
const ACPI_BUTTON_DEVICE_NAME_SLEEP: &[u8] = b"Sleep Button (CM)\0";
const ACPI_BUTTON_DEVICE_NAME_SLEEPF: &[u8] = b"Sleep Button (FF)\0";
const ACPI_BUTTON_DEVICE_NAME_LID: &[u8] = b"Lid Switch\0";
// Subclasses + class
const ACPI_BUTTON_SUBCLASS_POWER: &[u8] = b"power\0";
const ACPI_BUTTON_SUBCLASS_SLEEP: &[u8] = b"sleep\0";
const ACPI_BUTTON_SUBCLASS_LID: &[u8] = b"lid\0";
const ACPI_BUTTON_CLASS_BYTES: &[u8] = b"button\0";
// Types
const ACPI_BUTTON_TYPE_POWER: u8 = 0x01;
const ACPI_BUTTON_TYPE_POWERF: u8 = 0x02;
const ACPI_BUTTON_TYPE_SLEEP: u8 = 0x03;
const ACPI_BUTTON_TYPE_SLEEPF: u8 = 0x04;
const ACPI_BUTTON_TYPE_LID: u8 = 0x05;
// Notify
const ACPI_BUTTON_NOTIFY_STATUS: u32 = 0x80;
// Driver identity (used in the static acpi_driver). [c_char;80] name/class arrays.
const ACPI_BUTTON_DRIVER_NAME: &[u8] = b"ACPI Button Driver";
const ACPI_BUTTON_CLASS: &[u8] = b"button";

// ---- Sibling acpi-rust functions (extern "C"/cdecl; #[no_mangle] elsewhere). ----
extern "C" {
    fn acpi_bus_register_driver(driver: *mut acpi_driver) -> c_int;
    fn acpi_bus_unregister_driver(driver: *mut acpi_driver);
    fn acpi_bus_generate_event(device: *mut acpi_device, type_: u8, data: c_int) -> c_int;
}

// ---- CRT (libcntpr). ----
extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn sprintf(buffer: *mut c_char, format: *const c_char, ...) -> c_int;
}

// ---- file-private acpi_button (struct acpi_button). C field `type` → `type_`. ----
#[repr(C)]
struct acpi_button {
    handle: ACPI_HANDLE,
    device: *mut acpi_device,
    type_: UINT8,
    pushed: c_ulong,
}

// ---- exported globals (external linkage; main.c reads power_button/sleep_button). ----
#[no_mangle]
pub static mut power_button: *mut acpi_device = core::ptr::null_mut();
#[no_mangle]
pub static mut sleep_button: *mut acpi_device = core::ptr::null_mut();
#[no_mangle]
pub static mut lid_button: *mut acpi_device = core::ptr::null_mut();

// ---- acpi_driver_data(device) = (*device).driver_data ----
#[inline]
unsafe fn acpi_driver_data(device: *mut acpi_device) -> *mut c_void {
    (*device).driver_data
}
// ---- acpi_device_hid(d) = (*d).pnp.hardware_id (*mut c_char) ----
#[inline]
unsafe fn acpi_device_hid(d: *mut acpi_device) -> *mut c_char {
    (*d).pnp.hardware_id
}

// ---- helper: build the file-private static acpi_driver name/class byte arrays ----
const fn pad_name(s: &[u8]) -> [c_char; 80] {
    let mut out = [0 as c_char; 80];
    let mut i = 0;
    while i < s.len() && i < 80 {
        out[i] = s[i] as c_char;
        i += 1;
    }
    out
}

// ids string: "ACPI_FPB,ACPI_FSB,PNP0C0D,PNP0C0C,PNP0C0E"
static ACPI_BUTTON_IDS: &[u8] = b"ACPI_FPB,ACPI_FSB,PNP0C0D,PNP0C0C,PNP0C0E\0";

// ---- static struct acpi_driver acpi_button_driver ----
// The {0,0} list_head node is initialized by register; name/class are byte arrays,
// ids points at the c-string, ops holds {add, remove}.
static mut acpi_button_driver: acpi_driver = acpi_driver {
    node: list_head {
        next: core::ptr::null_mut(),
        prev: core::ptr::null_mut(),
    },
    name: pad_name(ACPI_BUTTON_DRIVER_NAME),
    class: pad_name(ACPI_BUTTON_CLASS),
    references: 0,
    flags: 0,
    ids: ACPI_BUTTON_IDS.as_ptr() as *mut c_char,
    ops: acpi_device_ops {
        add: Some(acpi_button_add),
        remove: Some(acpi_button_remove),
        start: None,
        suspend: None,
        resume: None,
        bind: None,
        unbind: None,
        notify: None,
        scan: None,
    },
};

/* --------------------------------------------------------------------------
                            Driver Interface
   -------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn acpi_button_notify(handle: ACPI_HANDLE, event: UINT32, data: *mut c_void) {
    let button: *mut acpi_button = data as *mut acpi_button;

    ACPI_FUNCTION_TRACE!("acpi_button_notify");

    if button.is_null() || (*button).device.is_null() {
        return;
    }

    match event {
        ACPI_BUTTON_NOTIFY_STATUS => {
            (*button).pushed = (*button).pushed.wrapping_add(1);
            acpi_bus_generate_event((*button).device, event as u8, (*button).pushed as c_int);
        }
        _ => {
            ACPI_DEBUG_PRINT!();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn acpi_button_notify_fixed(data: *mut c_void) -> ACPI_STATUS {
    let button: *mut acpi_button = data as *mut acpi_button;

    ACPI_FUNCTION_TRACE!("acpi_button_notify_fixed");

    if button.is_null() {
        return AE_BAD_PARAMETER;
    }

    acpi_button_notify(
        (*button).handle,
        ACPI_BUTTON_NOTIFY_STATUS,
        button as *mut c_void,
    );

    AE_OK
}

// `static` C fn but stored in the acpi_driver.ops dispatch table → must be
// `extern "C"` so the table's fn-pointer ABI matches (acpi_op_add = extern "C").
unsafe extern "C" fn acpi_button_add(device: *mut acpi_device) -> c_int {
    let mut result: c_int = 0;
    let mut status: ACPI_STATUS = AE_OK;
    let mut button: *mut acpi_button = core::ptr::null_mut();

    ACPI_FUNCTION_TRACE!("acpi_button_add");

    if device.is_null() {
        return -1;
    }

    button = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<acpi_button>() as usize,
        TAG_IPCA,
    ) as *mut acpi_button;
    if button.is_null() {
        return -4;
    }
    core::ptr::write_bytes(button as *mut u8, 0, core::mem::size_of::<acpi_button>());

    (*button).device = device;
    (*button).handle = (*device).handle;
    (*device).driver_data = button as *mut c_void;

    /*
     * Determine the button type (via hid), as fixed-feature buttons
     * need to be handled a bit differently than generic-space.
     */
    if strcmp(acpi_device_hid(device), ACPI_BUTTON_HID_POWER.as_ptr() as *const c_char) == 0 {
        (*button).type_ = ACPI_BUTTON_TYPE_POWER;
        sprintf(
            (*device).pnp.device_name.as_mut_ptr(),
            b"%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_DEVICE_NAME_POWER.as_ptr(),
        );
        sprintf(
            (*device).pnp.device_class.as_mut_ptr(),
            b"%s/%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_CLASS_BYTES.as_ptr(),
            ACPI_BUTTON_SUBCLASS_POWER.as_ptr(),
        );
    } else if strcmp(acpi_device_hid(device), ACPI_BUTTON_HID_POWERF.as_ptr() as *const c_char) == 0 {
        (*button).type_ = ACPI_BUTTON_TYPE_POWERF;
        sprintf(
            (*device).pnp.device_name.as_mut_ptr(),
            b"%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_DEVICE_NAME_POWERF.as_ptr(),
        );
        sprintf(
            (*device).pnp.device_class.as_mut_ptr(),
            b"%s/%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_CLASS_BYTES.as_ptr(),
            ACPI_BUTTON_SUBCLASS_POWER.as_ptr(),
        );
    } else if strcmp(acpi_device_hid(device), ACPI_BUTTON_HID_SLEEP.as_ptr() as *const c_char) == 0 {
        (*button).type_ = ACPI_BUTTON_TYPE_SLEEP;
        sprintf(
            (*device).pnp.device_name.as_mut_ptr(),
            b"%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_DEVICE_NAME_SLEEP.as_ptr(),
        );
        sprintf(
            (*device).pnp.device_class.as_mut_ptr(),
            b"%s/%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_CLASS_BYTES.as_ptr(),
            ACPI_BUTTON_SUBCLASS_SLEEP.as_ptr(),
        );
    } else if strcmp(acpi_device_hid(device), ACPI_BUTTON_HID_SLEEPF.as_ptr() as *const c_char) == 0 {
        (*button).type_ = ACPI_BUTTON_TYPE_SLEEPF;
        sprintf(
            (*device).pnp.device_name.as_mut_ptr(),
            b"%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_DEVICE_NAME_SLEEPF.as_ptr(),
        );
        sprintf(
            (*device).pnp.device_class.as_mut_ptr(),
            b"%s/%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_CLASS_BYTES.as_ptr(),
            ACPI_BUTTON_SUBCLASS_SLEEP.as_ptr(),
        );
    } else if strcmp(acpi_device_hid(device), ACPI_BUTTON_HID_LID.as_ptr() as *const c_char) == 0 {
        (*button).type_ = ACPI_BUTTON_TYPE_LID;
        sprintf(
            (*device).pnp.device_name.as_mut_ptr(),
            b"%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_DEVICE_NAME_LID.as_ptr(),
        );
        sprintf(
            (*device).pnp.device_class.as_mut_ptr(),
            b"%s/%s\0".as_ptr() as *const c_char,
            ACPI_BUTTON_CLASS_BYTES.as_ptr(),
            ACPI_BUTTON_SUBCLASS_LID.as_ptr(),
        );
    } else {
        ACPI_DEBUG_PRINT!();
        result = -15;
        // goto end
        if result != 0 {
            ExFreePoolWithTag(button as PVOID, TAG_IPCA);
        }
        return result;
    }

    /*
     * Ensure only one button of each type is used.
     */
    match (*button).type_ {
        ACPI_BUTTON_TYPE_POWER | ACPI_BUTTON_TYPE_POWERF => {
            if power_button.is_null() {
                power_button = device;
            } else {
                ExFreePoolWithTag(button as PVOID, TAG_IPCA);
                return -15;
            }
        }
        ACPI_BUTTON_TYPE_SLEEP | ACPI_BUTTON_TYPE_SLEEPF => {
            if sleep_button.is_null() {
                sleep_button = device;
            } else {
                ExFreePoolWithTag(button as PVOID, TAG_IPCA);
                return -15;
            }
        }
        ACPI_BUTTON_TYPE_LID => {
            if lid_button.is_null() {
                lid_button = device;
            } else {
                ExFreePoolWithTag(button as PVOID, TAG_IPCA);
                return -15;
            }
        }
        _ => {}
    }

    match (*button).type_ {
        ACPI_BUTTON_TYPE_POWERF => {
            status = AcpiInstallFixedEventHandler(
                ACPI_EVENT_POWER_BUTTON,
                Some(acpi_button_notify_fixed),
                button as *mut c_void,
            );
        }
        ACPI_BUTTON_TYPE_SLEEPF => {
            status = AcpiInstallFixedEventHandler(
                ACPI_EVENT_SLEEP_BUTTON,
                Some(acpi_button_notify_fixed),
                button as *mut c_void,
            );
        }
        ACPI_BUTTON_TYPE_LID => {
            // NOTE: faithful to C — passes ACPI_BUTTON_TYPE_LID (a button-type, not
            // an ACPICA event id) as the AcpiEvent arg. Preserved verbatim.
            status = AcpiInstallFixedEventHandler(
                ACPI_BUTTON_TYPE_LID as UINT32,
                Some(acpi_button_notify_fixed),
                button as *mut c_void,
            );
        }
        _ => {
            status = AcpiInstallNotifyHandler(
                (*button).handle,
                ACPI_DEVICE_NOTIFY,
                Some(acpi_button_notify),
                button as *mut c_void,
            );
        }
    }

    if ACPI_FAILURE(status) {
        ACPI_DEBUG_PRINT!();
        result = -15;
        // goto end
        if result != 0 {
            ExFreePoolWithTag(button as PVOID, TAG_IPCA);
        }
        return result;
    }

    DPRINT!();

    // end:
    if result != 0 {
        ExFreePoolWithTag(button as PVOID, TAG_IPCA);
    }

    result
}

// `static` C fn stored in acpi_driver.ops dispatch table → extern "C".
unsafe extern "C" fn acpi_button_remove(device: *mut acpi_device, type_: c_int) -> c_int {
    let mut status: ACPI_STATUS = 0;
    let button: *mut acpi_button;

    ACPI_FUNCTION_TRACE!("acpi_button_remove");

    if device.is_null() || acpi_driver_data(device).is_null() {
        return -1;
    }

    button = acpi_driver_data(device) as *mut acpi_button;

    /* Unregister for device notifications. */
    match (*button).type_ {
        ACPI_BUTTON_TYPE_POWERF => {
            status =
                AcpiRemoveFixedEventHandler(ACPI_EVENT_POWER_BUTTON, Some(acpi_button_notify_fixed));
        }
        ACPI_BUTTON_TYPE_SLEEPF => {
            status =
                AcpiRemoveFixedEventHandler(ACPI_EVENT_SLEEP_BUTTON, Some(acpi_button_notify_fixed));
        }
        ACPI_BUTTON_TYPE_LID => {
            status = AcpiRemoveFixedEventHandler(
                ACPI_BUTTON_TYPE_LID as UINT32,
                Some(acpi_button_notify_fixed),
            );
        }
        _ => {
            status = AcpiRemoveNotifyHandler(
                (*button).handle,
                ACPI_DEVICE_NOTIFY,
                Some(acpi_button_notify),
            );
        }
    }

    if ACPI_FAILURE(status) {
        ACPI_DEBUG_PRINT!();
    }

    ExFreePoolWithTag(button as PVOID, TAG_IPCA);

    0
}

#[no_mangle]
pub unsafe extern "C" fn acpi_button_init() -> c_int {
    let mut result: c_int = 0;

    ACPI_FUNCTION_TRACE!("acpi_button_init");

    result = acpi_bus_register_driver(core::ptr::addr_of_mut!(acpi_button_driver));
    if result < 0 {
        return -15;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn acpi_button_exit() {
    ACPI_FUNCTION_TRACE!("acpi_button_exit");

    acpi_bus_unregister_driver(core::ptr::addr_of_mut!(acpi_button_driver));
}
